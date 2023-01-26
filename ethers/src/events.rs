use std::{
    fmt::Display,
    ops::{Sub, SubAssign},
    sync::{Arc, Mutex},
    time::Duration,
};

use async_timer_rs::{hashed::Timeout, Timer};
use completeq_rs::{channel, oneshot, user_event::UserEvent};
use ethers_hardhat_rs::futures::{executor::ThreadPool, task::SpawnExt};
use ethers_providers_rs::Provider;
use ethers_types_rs::{Filter, PollLogs, TransactionReceipt, H256, U256};
use once_cell::sync::OnceCell;

/// Ether client support event types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Transaction mint event
    Transaction(H256),
    /// onchain state changes based on filter options
    Log(Filter),
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transaction(hash) => {
                write!(f, "event_tx: {}", hash)
            }
            Self::Log(filter) => {
                write!(
                    f,
                    "event_filter: {}",
                    serde_json::to_string(filter).unwrap()
                )
            }
        }
    }
}

/// Event emit arg
pub enum EventArg {
    /// Transaction mint event return value [`TransactionReceipt`]
    Transaction(TransactionReceipt),
    /// `get_ethLogs`/`eth_getFilterLogs` return value [`PollLogs`]
    Log(PollLogs),
}

#[derive(Default, Clone)]
pub struct Event;

impl UserEvent for Event {
    type Argument = EventArg;
    type ID = EventType;
}

type OnshotCompleteQ = oneshot::CompleteQ<Event>;
type ChannelCompleteQ = channel::CompleteQ<Event>;

/// Async event complete xecutor
#[derive(Clone)]
#[allow(unused)]
pub struct EventEmitter {
    onshot: OnshotCompleteQ,
    channel: ChannelCompleteQ,
    events: Arc<Mutex<Vec<EventType>>>,
    duration: Arc<Mutex<Duration>>,
}

impl EventEmitter {
    pub(crate) fn new(provider: Provider) -> Self {
        let result = Self {
            onshot: OnshotCompleteQ::new(),
            channel: ChannelCompleteQ::new(),
            events: Default::default(),
            duration: Arc::new(Mutex::new(Duration::from_secs(5))),
        };

        let mut poller = Poller::new(
            result.duration.clone(),
            result.onshot.clone(),
            result.channel.clone(),
            provider,
            result.events.clone(),
        );

        static THREAD_POOL: OnceCell<ThreadPool> = OnceCell::new();

        let thread_pool = THREAD_POOL.get_or_init(|| ThreadPool::new().unwrap());

        thread_pool
            .spawn(async move { poller.poll().await })
            .unwrap();

        result
    }

    /// Add tx completed event listener
    pub fn wait_transaction(&mut self, tx: H256) -> TransactionWaitable<Timeout> {
        self.events.lock().unwrap().push(EventType::Transaction(tx));

        TransactionWaitable {
            tx,
            receiver: Some(self.onshot.wait_for(EventType::Transaction(tx))),
        }
    }

    /// Add event filter listener
    pub fn event_filter(&mut self, filter: Filter) -> EventReceiver<Timeout> {
        let event_type = EventType::Log(filter.clone());

        self.events.lock().unwrap().push(event_type.clone());

        EventReceiver {
            filter,
            receiver: self.channel.wait_for(event_type, 20),
        }
    }
}

#[allow(unused)]
struct Poller {
    last_execute_block_number: Option<U256>,
    duration_onchain: bool,
    duration: Arc<Mutex<Duration>>,
    onshot: OnshotCompleteQ,
    channel: ChannelCompleteQ,
    provider: Provider,
    events: Arc<Mutex<Vec<EventType>>>,
}

impl Poller {
    fn new(
        duration: Arc<Mutex<Duration>>,
        onshot: OnshotCompleteQ,
        channel: ChannelCompleteQ,
        provider: Provider,
        events: Arc<Mutex<Vec<EventType>>>,
    ) -> Self {
        Self {
            last_execute_block_number: None,
            duration_onchain: false,
            duration,
            onshot,
            channel,
            provider,
            events,
        }
    }

    async fn poll(&mut self) {
        loop {
            // Only poll_loop instance is aliving, exit loop.
            if Arc::strong_count(&self.events) == 1 {
                log::trace!("event emitter poll thread stopped.");
                return;
            }

            let block_number = match self.provider.eth_block_number().await {
                Ok(block_number) => block_number,
                Err(err) => {
                    log::error!("get block number err, {}", err.to_string());
                    self.wait_timeout().await;
                    continue;
                }
            };

            let process_events = {
                let mut events = self.events.lock().unwrap();

                let temp = events.clone();

                events.clear();

                temp
            };

            let mut reserved = vec![];

            for event in process_events {
                match event {
                    EventType::Transaction(tx_hash) => {
                        log::debug!("Check tx status {}", tx_hash);
                        match self.provider.eth_get_transaction_receipt(tx_hash).await {
                            Ok(Some(receipt)) => {
                                log::trace!(
                                    "Get tx {} receipt returns {}",
                                    tx_hash,
                                    serde_json::to_string(&receipt).unwrap()
                                );

                                self.onshot
                                    .complete_one(event, EventArg::Transaction(receipt));
                            }
                            Ok(None) => {
                                log::trace!("Get tx receipt return None");
                                reserved.push(event);
                            }
                            Err(err) => {
                                log::error!("Get tx {} receipt error, {}", tx_hash, err);
                            }
                        }
                    }
                    EventType::Log(filter) => {
                        let from_block = if let Some(last_execute_block_number) =
                            self.last_execute_block_number
                        {
                            last_execute_block_number
                        } else {
                            block_number
                        };

                        let mut filter = filter.clone();

                        // filter.from_block = Some(from_block);
                        // filter.to_block = Some(block_number);

                        match self.provider.eth_get_logs(filter.clone()).await {
                            Ok(logs) => {
                                if let Some(logs) = logs {
                                    log::debug!("get logs({:?}) returns {:?}", filter, logs);

                                    self.channel.complete_one(
                                        EventType::Log(filter.clone()),
                                        EventArg::Log(logs),
                                    );
                                } else {
                                    log::debug!("get logs({:?}) returns None", filter);
                                }

                                reserved.push(EventType::Log(filter));
                            }
                            Err(err) => {
                                log::error!("Get logs({:?}) error, {}", filter, err);
                            }
                        }
                    }
                }
            }

            self.last_execute_block_number = Some(block_number);

            self.events.lock().unwrap().append(&mut reserved);

            match self.calc_poll_duration().await {
                Err(err) => {
                    log::error!("calc poll duration failed, {}", err);
                }
                _ => {}
            }

            self.wait_timeout().await;
        }
    }

    async fn wait_timeout(&self) {
        let duration = self.duration.lock().unwrap().clone();
        Timeout::new(duration).await;
    }

    async fn calc_poll_duration(&mut self) -> anyhow::Result<Duration> {
        if !self.duration_onchain {
            // Calculate the time interval between the two most recent blocks

            let mut block_number = self.provider.eth_block_number().await?;

            if block_number < 2.into() {
                return Ok(self.duration.lock().unwrap().clone());
            }

            let last = self
                .provider
                .eth_get_block_by_number(block_number, true)
                .await?
                .ok_or(anyhow::format_err!(
                    "Get block {} return None",
                    block_number
                ))?;

            block_number.sub_assign(1.into());

            let prev = self
                .provider
                .eth_get_block_by_number(block_number, true)
                .await?
                .ok_or(anyhow::format_err!(
                    "Get block {} return None",
                    block_number
                ))?;

            let duration = last.timestamp.sub(prev.timestamp);

            let duration = Duration::from_secs(duration.as_u64());

            log::debug!("calc poll duration {:?}", duration);

            *self.duration.lock().unwrap() = duration;
        }

        return Ok(self.duration.lock().unwrap().clone());
    }
}

/// Transaction instance provide extra wait fn
pub struct TransactionWaitable<T: Timer> {
    /// Transaction id
    pub tx: H256,

    receiver: Option<oneshot::EventReceiver<Event, T>>,
}

impl<T: Timer> TransactionWaitable<T>
where
    T: Unpin,
{
    pub async fn wait(&mut self) -> anyhow::Result<TransactionReceipt> {
        let value = self.receiver.take().unwrap().await.success()?;

        match value {
            EventArg::Transaction(receipt) => Ok(receipt),
            _ => {
                panic!("Inner error, returns event arg type error!!!")
            }
        }
    }
}

pub struct EventReceiver<T: Timer> {
    pub filter: Filter,

    receiver: channel::EventReceiver<Event, T>,
}

impl<T: Timer> EventReceiver<T>
where
    T: Unpin,
{
    pub async fn next(&mut self) -> Option<PollLogs> {
        (&mut self.receiver).await.ok().map(|arg| match arg {
            EventArg::Log(logs) => logs,
            _ => {
                panic!("Inner error, returns event arg type error!!!")
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_timer_rs::{hashed::Timeout, Timer};
    use ethers_hardhat_rs::{cmds::HardhatNetwork, utils::get_hardhat_network_provider};

    use super::EventEmitter;

    #[async_std::test]
    async fn test_poll_calc_duration() {
        _ = pretty_env_logger::try_init();

        let mut network = HardhatNetwork::new().expect("Create hardhat network instance");

        network.start().await.expect("Start hardhat network");

        let provider = get_hardhat_network_provider();

        let emitter = EventEmitter::new(provider);

        Timeout::new(Duration::from_secs(30)).await;

        let duration = emitter.duration.lock().unwrap().clone();

        assert!(duration >= Duration::from_secs(10));

        assert!(duration <= Duration::from_secs(12));
    }
}
