use futures::{
    channel::mpsc::{Receiver, SendError, Sender},
    executor::ThreadPool,
    task::SpawnExt,
};
use jsonrpc_rs::{channel::TransportChannel, RPCResult};
use once_cell::sync::OnceCell;

/// Ethererum network provider common channel.
///
pub(crate) struct ProviderChannel {
    pub(crate) receiver: Receiver<RPCResult<String>>,
    pub(crate) sender: Sender<String>,
}

impl TransportChannel for ProviderChannel {
    type StreamError = jsonrpc_rs::Error<String, ()>;

    type SinkError = SendError;

    type Input = Receiver<RPCResult<String>>;

    type Output = Sender<String>;

    fn spawn<Fut>(future: Fut)
    where
        Fut: futures::Future<Output = RPCResult<()>> + Send + 'static,
    {
        static INSTANCE: OnceCell<ThreadPool> = OnceCell::new();

        let executor = INSTANCE.get_or_init(|| ThreadPool::new().unwrap());

        _ = executor.spawn(async move {
            let result = future.await;

            match result {
                Err(err) => {
                    log::error!("{}", err);
                }
                _ => {}
            }
        });
    }

    fn framed(self) -> (Self::Input, Self::Output) {
        (self.receiver, self.sender)
    }
}
