use std::fmt::{Debug, Display};

use futures::{
    channel::mpsc::{self, Receiver, Sender},
    Sink, SinkExt, Stream, StreamExt, TryStreamExt,
};
use jsonrpc_rs::{bytes::Bytes, channel::TransportChannel, RPCResult};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::Provider;

use super::channel::TokioProviderChannel;

use jsonrpc_rs::channel::RPCData;

pub async fn connect_to<S: AsRef<str> + Clone + Send + 'static + Sync>(
    url: S,
) -> RPCResult<Provider> {
    let (client_output, dispatcher_input) = mpsc::channel::<RPCData>(20);
    let (dispatcher_output, client_input) = mpsc::channel::<RPCResult<RPCData>>(20);

    // Create mpsc transport, real send/recv network message are in procedure dispatcher.
    let client_transport = TokioProviderChannel {
        receiver: client_input,
        sender: client_output,
    };

    let (ws_stream, _) = connect_async(url.as_ref())
        .await
        .map_err(jsonrpc_rs::map_error)?;

    let (write, read) = ws_stream.split();

    let requester_url = url.clone();

    TokioProviderChannel::spawn(async move {
        match requester_loop(write, dispatcher_input).await {
            Err(err) => {
                log::error!("requester_loop err, {}", err);
            }
            _ => {
                log::trace!("requester_loop stop,{}", requester_url.as_ref());
            }
        }

        Ok(())
    });

    let responser_url = url.clone();

    TokioProviderChannel::spawn(async move {
        match responser_loop(read, dispatcher_output).await {
            Err(err) => {
                log::error!("responser_loop err, {}", err);
            }
            _ => {
                log::trace!("responser_loop stop,{}", responser_url.as_ref());
            }
        }

        Ok(())
    });
    let tag = format!("eth-provider-ws_{}", url.as_ref());

    Ok(Provider::new(
        tag.clone(),
        jsonrpc_rs::Client::new(tag, client_transport),
    ))
}

#[allow(unused)]
async fn requester_loop<S>(mut sink: S, mut dispatcher_input: Receiver<RPCData>) -> RPCResult<()>
where
    S: Sink<Message> + Unpin,
    S::Error: Display + Debug,
{
    while let Some(message) = dispatcher_input.next().await {
        sink.send(Message::Text(String::from_utf8_lossy(&message).to_string()))
            .await
            .map_err(jsonrpc_rs::map_error);
    }

    // Ok(())

    unimplemented!()
}

async fn responser_loop<S, E>(
    mut stream: S,
    mut response_input: Sender<RPCResult<RPCData>>,
) -> RPCResult<()>
where
    S: Stream<Item = Result<Message, E>> + Unpin,
    E: Display + Debug,
{
    while let Some(message) = stream.try_next().await.map_err(jsonrpc_rs::map_error)? {
        match message {
            Message::Text(data) => {
                response_input.send(Ok(Bytes::from(data))).await?;
            }
            _ => {
                log::trace!("skip handle ws recv msg, {}", message);
            }
        }
    }

    Ok(())
}
