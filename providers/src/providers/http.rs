use futures::{
    channel::mpsc::{self, Receiver, Sender},
    SinkExt, StreamExt,
};
use jsonrpc_rs::{channel::TransportChannel, map_error, ErrorCode, RPCError, RPCResult};
use reqwest::header::{HeaderValue, CONTENT_TYPE};

use crate::Provider;

use super::channel::TokioProviderChannel;

use jsonrpc_rs::channel::RPCData;

/// Use http/https protocol connect to ethereum node.
///
/// Visit [`Ethereum JSON-RPC Specification`](https://ethereum.github.io/execution-apis/api-documentation/) for more details
///
/// # Parameters
///
/// * `url` - Ethereum node public JSONRPC server url
pub fn connect_to<S: AsRef<str> + Clone + Send + 'static + Sync>(url: S) -> Provider {
    let (client_output, dispatcher_input) = mpsc::channel::<RPCData>(20);
    let (dispatcher_output, client_input) = mpsc::channel::<RPCResult<RPCData>>(20);

    // Create mpsc transport, real send/recv network message are in procedure dispatcher.
    let client_transport = TokioProviderChannel {
        receiver: client_input,
        sender: client_output,
    };

    let dispatcher_url = url.clone();

    TokioProviderChannel::spawn(async move {
        match requester_loop(dispatcher_url.clone(), dispatcher_input, dispatcher_output).await {
            Err(err) => {
                log::error!("requester_loop err, {}", err);
            }
            _ => {
                log::trace!("requester_loop stop,{}", dispatcher_url.as_ref());
            }
        }

        Ok(())
    });

    Provider::new(jsonrpc_rs::Client::new(
        format!("eth-provider-https_{}", url.as_ref()),
        client_transport,
    ))
}

#[allow(unused)]
async fn requester_loop<S>(
    dispatcher_url: S,
    mut dispatcher_input: Receiver<RPCData>,
    dispatcher_output: Sender<RPCResult<RPCData>>,
) -> RPCResult<()>
where
    S: AsRef<str> + Clone + Send + 'static + Sync,
{
    while let Some(message) = dispatcher_input.next().await {
        let response_input = dispatcher_output.clone();
        let request_url = dispatcher_url.clone();

        // panic!("hello");

        TokioProviderChannel::spawn(async move {
            match send_and_recv(request_url, message, response_input).await {
                Err(err) => {
                    log::error!("send_and_recv error, {}", err);
                }
                _ => {}
            }
            Ok(())
        });
    }

    Ok(())
}

async fn send_and_recv<S>(
    request_url: S,
    message: RPCData,
    mut response_input: Sender<RPCResult<RPCData>>,
) -> RPCResult<()>
where
    S: AsRef<str> + Clone + Send + 'static + Sync,
{
    let client = reqwest::Client::new();

    log::trace!("try send message: {:?}", message);

    let response = client
        .post(request_url.as_ref())
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(message)
        .send()
        .await
        .map_err(map_error);

    log::trace!("response {:?}", response);

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let recv_data = response.bytes().await.map_err(map_error);

                if let Ok(recv_data) = &recv_data {
                    log::trace!("recv response {}", String::from_utf8_lossy(&recv_data));
                }

                response_input.send(recv_data).await?;
            } else {
                let err = RPCError {
                    code: ErrorCode::InternalError,
                    message: response.status().to_string(),
                    data: None,
                };

                response_input.send(Err(err)).await?;
            }
        }
        Err(err) => {
            response_input.send(Err(err)).await?;
        }
    }

    Ok(())
}
