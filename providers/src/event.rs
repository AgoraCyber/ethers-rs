use crate::Provider;

use std::{fmt::Display, sync::Arc, time::Duration};

use async_timer_rs::{hashed::Timeout, Timer};
use completeq_rs::{error::CompleteQError, result::EmitResult, user_event::UserEvent};
use ethers_types_rs::{
    ethabi::{ParseLog, RawLog},
    *,
};
use futures::{executor::ThreadPool, task::SpawnExt};
use once_cell::sync::OnceCell;

/// Ether client support event types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Transaction mint event
    Transaction(H256),
    /// filter.
    Filter(Filter),
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transaction(hash) => {
                write!(f, "event_tx: {}", hash)
            }
            Self::Filter(filter) => {
                write!(f, "event_filter: {:?}", filter)
            }
        }
    }
}

/// Event emit arg
pub enum EventArg {
    /// Transaction mint event return value [`TransactionReceipt`]
    Transaction(TransactionReceipt),
    /// `get_ethLogs`/`eth_getFilterLogs` return value [`PollLogs`]
    Log(Vec<Log>),
}

#[derive(Default, Clone)]
pub struct Event;

impl UserEvent for Event {
    type Argument = EventArg;
    type ID = EventType;
}

pub(crate) type OneshotCompleteQ = completeq_rs::oneshot::CompleteQ<Event>;
pub(crate) type ChannelCompleteQ = completeq_rs::channel::CompleteQ<Event>;

impl Provider {
    pub(crate) fn start_event_poll(&self) {
        static THREAD_POOL: OnceCell<ThreadPool> = OnceCell::new();

        let mut poller = Poller::new(self.clone());

        let thread_pool = THREAD_POOL.get_or_init(|| ThreadPool::new().unwrap());

        thread_pool
            .spawn(async move { poller.poll_loop().await })
            .unwrap();
    }

    pub fn register_filter_listener<A, T>(
        &self,
        address: Option<A>,
        topic_filter: Option<T>,
    ) -> anyhow::Result<DefaultFilterReceiver>
    where
        A: TryInto<AddressFilter>,
        A::Error: std::error::Error + Sync + Send + 'static,
        T: TryInto<TopicFilter>,
        T::Error: std::error::Error + Sync + Send + 'static,
    {
        let address = if let Some(address) = address {
            Some(address.try_into()?)
        } else {
            None
        };

        let topic_filter = if let Some(topic_filter) = topic_filter {
            Some(topic_filter.try_into()?)
        } else {
            None
        };

        let filter = Filter {
            from_block: None,
            to_block: None,
            address,
            topics: topic_filter,
        };

        let event_type = EventType::Filter(filter.clone());

        self.events.lock().unwrap().push(event_type.clone());

        Ok(FilterReceiver {
            filter,
            receiver: self.channel.wait_for(event_type, 100),
        })
    }

    pub fn register_transaction_listener<H>(
        &self,
        tx_hash: H,
    ) -> anyhow::Result<DefaultTransactionReceipter>
    where
        H: TryInto<H256>,
        H::Error: std::error::Error + Sync + Send + 'static,
    {
        let tx_hash = tx_hash.try_into()?;

        let event_type = EventType::Transaction(tx_hash);

        self.events.lock().unwrap().push(event_type.clone());

        Ok(TransactionReceipter {
            tx: tx_hash,
            receiver: self.oneshot.wait_for(event_type),
        })
    }
}

struct Poller {
    max_filter_block_range: U256,
    last_poll_block_number: U256,
    poll_interval_duration: Duration,
    provider: Provider,
    tx_listeners: Vec<H256>,
    filter_listeners: Vec<Filter>,
}

impl Poller {
    fn new(provider: Provider) -> Self {
        Self {
            last_poll_block_number: 0.into(),
            max_filter_block_range: 10.into(),
            provider,
            tx_listeners: Default::default(),
            filter_listeners: Default::default(),
            poll_interval_duration: Duration::from_secs(5),
        }
    }
    async fn poll_loop(&mut self) {
        loop {
            log::debug!("start events poll for {}", self.provider.id());
            // If only `Self` handle strong reference of provider, stop polll loop
            if Arc::strong_count(&self.provider.events) == 1 {
                log::debug!("stop events poller for {}", self.provider.id());
                return;
            }

            self.handle_new_listeners();

            Self::handle_result("Poll one block events", self.poll_one().await);

            log::debug!(
                "events poll_once for {}, sleeping {:?}",
                self.provider.id(),
                self.poll_interval_duration
            );

            let timer = Timeout::new(self.poll_interval_duration);

            timer.await;
        }
    }

    async fn check_if_poll(&mut self) -> anyhow::Result<Option<U256>> {
        if self.filter_listeners.is_empty() && self.tx_listeners.is_empty() {
            return Ok(None);
        }

        let block_number = self.provider.eth_block_number().await?;

        if block_number == self.last_poll_block_number {
            Ok(None)
        } else {
            Ok(Some(block_number))
        }
    }

    async fn recalc_poll_internval_duration(&mut self, block_number: U256) -> anyhow::Result<()> {
        let last_block = self
            .provider
            .eth_get_block_by_number(block_number, true)
            .await?;

        let one: U256 = 1.into();

        let prev_block = self
            .provider
            .eth_get_block_by_number(block_number - one, true)
            .await?;

        if let Some(prev_block) = prev_block {
            if let Some(last_block) = last_block {
                self.poll_interval_duration =
                    Duration::from_secs((last_block.timestamp - prev_block.timestamp).as_u64());

                log::debug!(
                    "new poll interval duration is {:?}",
                    self.poll_interval_duration
                );
            }
        }

        Ok(())
    }

    async fn poll_one(&mut self) -> anyhow::Result<()> {
        if let Some(block_number) = self.check_if_poll().await? {
            self.fetch_and_emit_tx_events().await?;

            self.fetch_and_emit_filter_events(block_number).await?;

            if block_number - self.last_poll_block_number > 1.into() {
                self.recalc_poll_internval_duration(block_number).await?;
            }

            self.last_poll_block_number = block_number;
        }

        Ok(())
    }

    /// Get and remove new incoming listeners from provider events pipe.
    fn handle_new_listeners(&mut self) {
        let new_listeners = self
            .provider
            .events
            .lock()
            .unwrap()
            .drain(0..)
            .collect::<Vec<_>>();

        for listener in new_listeners {
            match listener {
                EventType::Transaction(tx_hash) => {
                    self.tx_listeners.push(tx_hash);
                }
                EventType::Filter(filter_id) => {
                    self.filter_listeners.push(filter_id);
                }
            }
        }
    }

    async fn fetch_and_emit_tx_events(&mut self) -> anyhow::Result<()> {
        let mut remaining = vec![];

        for tx_hash in &self.tx_listeners {
            match self
                .provider
                .eth_get_transaction_receipt(tx_hash.clone())
                .await?
            {
                Some(receipt) => {
                    log::trace!(
                        "Get tx {} receipt returns {}",
                        tx_hash,
                        serde_json::to_string(&receipt).unwrap()
                    );

                    // Oneshot event ignore to checking returns value
                    self.provider.oneshot.complete_one(
                        EventType::Transaction(tx_hash.clone()),
                        EventArg::Transaction(receipt),
                    );
                }
                None => {
                    log::trace!("Get tx receipt return None");
                    remaining.push(tx_hash.clone());
                }
            }
        }

        self.tx_listeners = remaining;

        Ok(())
    }

    async fn fetch_and_emit_filter_events(&mut self, block_number: U256) -> anyhow::Result<()> {
        let mut remaining = vec![];

        for filter in &self.filter_listeners {
            let mut filter_send = filter.clone();

            let from_block = if block_number > self.max_filter_block_range {
                let mut from_block = block_number - self.max_filter_block_range;

                if from_block < self.last_poll_block_number {
                    from_block = self.last_poll_block_number;
                }

                from_block
            } else {
                self.last_poll_block_number
            };

            filter_send.from_block = Some(from_block);
            filter_send.to_block = Some(block_number);

            log::debug!("try poll filter logs {:?}", filter_send);

            let logs = self.provider.eth_get_logs(filter_send).await?;

            match logs {
                FilterEvents::Logs(logs) if logs.len() > 0 => {
                    let result = self
                        .provider
                        .channel
                        .complete_one(EventType::Filter(filter.clone()), EventArg::Log(logs))
                        .await;

                    match result {
                        EmitResult::Closed => {
                            log::debug!("Remove filter listener {:?}", filter);
                            continue;
                        }
                        _ => {}
                    }
                }
                _ => {
                    log::debug!("poll empty logs for filter {:?}", filter);
                }
            }

            remaining.push(filter.clone());
        }

        self.filter_listeners = remaining;

        Ok(())
    }
    fn handle_result<T, E>(tag: &str, result: std::result::Result<T, E>)
    where
        E: Display,
    {
        if result.is_err() {
            log::error!("{} error, {}", tag, result.err().unwrap())
        }
    }
}

/// Transaction instance provide extra wait fn
pub struct TransactionReceipter<T: Timer> {
    /// Transaction id
    pub tx: H256,

    receiver: completeq_rs::oneshot::EventReceiver<Event, T>,
}

impl<T: Timer> TransactionReceipter<T>
where
    T: Unpin,
{
    pub async fn wait(&mut self) -> anyhow::Result<TransactionReceipt> {
        let value = (&mut self.receiver).await.success()?;

        match value {
            Some(EventArg::Transaction(receipt)) => Ok(receipt),
            None => Err(CompleteQError::PipeBroken.into()),
            _ => {
                panic!("Inner error, returns event arg type error!!!")
            }
        }
    }
}

pub type DefaultTransactionReceipter = TransactionReceipter<Timeout>;

pub struct FilterReceiver<T: Timer> {
    pub filter: Filter,

    receiver: completeq_rs::channel::EventReceiver<Event, T>,
}

impl<T: Timer> FilterReceiver<T>
where
    T: Unpin,
{
    pub async fn try_next(&mut self) -> anyhow::Result<Option<Vec<Log>>> {
        Ok((&mut self.receiver).await.success().map(|c| {
            c.map(|c| match c {
                EventArg::Log(logs) => logs,
                _ => {
                    panic!("Inner error, returns event arg type error!!!")
                }
            })
        })?)
    }
}

pub type DefaultFilterReceiver = FilterReceiver<Timeout>;

pub struct TypedFilterReceiver<T, LOG>
where
    LOG: ParseLog,
    T: Timer + Unpin,
{
    log: LOG,
    receiver: FilterReceiver<T>,
}

impl<T, LOG> TypedFilterReceiver<T, LOG>
where
    LOG: ParseLog,
    T: Timer + Unpin,
{
    pub fn new(log: LOG, receiver: FilterReceiver<T>) -> Self {
        Self { log, receiver }
    }
    pub async fn try_next(&mut self) -> anyhow::Result<Option<Vec<LOG::Log>>> {
        if let Some(logs) = self.receiver.try_next().await? {
            let mut result = vec![];
            for log in logs {
                result.push(self.log.parse_log(RawLog {
                    data: log.data.0,
                    topics: log.topics,
                })?);
            }

            Ok(Some(result))
        } else {
            // end receive loop
            Ok(None)
        }
    }
}
