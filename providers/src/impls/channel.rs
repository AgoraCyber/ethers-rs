use futures::channel::mpsc::{Receiver, SendError, Sender};
use jsonrpc_rs::channel::RPCData;
use jsonrpc_rs::{channel::TransportChannel, RPCResult};
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

/// Ethererum network provider tokio io event driver channel.
///
pub(crate) struct TokioProviderChannel {
    pub(crate) receiver: Receiver<RPCResult<RPCData>>,
    pub(crate) sender: Sender<RPCData>,
}

impl TransportChannel for TokioProviderChannel {
    type StreamError = jsonrpc_rs::RPCError;

    type SinkError = SendError;

    type Input = Receiver<RPCResult<RPCData>>;

    type Output = Sender<RPCData>;

    fn spawn<Fut>(future: Fut)
    where
        Fut: futures::Future<Output = RPCResult<()>> + Send + 'static,
    {
        static INSTANCE: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

        let executor = INSTANCE.get_or_init(|| Runtime::new().unwrap());

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
