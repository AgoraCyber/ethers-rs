use futures::{
    channel::mpsc::{self, Receiver, Sender},
    SinkExt, StreamExt,
};
use jsonrpc_rs::{channel::TransportChannel, Error, ErrorCode, RPCResult};
use reqwest::header::{HeaderValue, CONTENT_TYPE};

use crate::Provider;

use super::channel::ProviderChannel;

/// Use http/https protocol connect to ethereum node.
///
/// Visit [`Ethereum JSON-RPC Specification`](https://ethereum.github.io/execution-apis/api-documentation/) for more details
///
/// # Parameters
///
/// * `url` - Ethereum node public JSONRPC server url
pub fn connect_to<S: AsRef<str> + Clone + Send + 'static + Sync>(url: S) -> Provider {
    let (client_output, dispatcher_input) = mpsc::channel::<String>(20);
    let (dispatcher_output, client_input) = mpsc::channel::<RPCResult<String>>(20);

    // Create mpsc transport, real send/recv network message are in procedure dispatcher.
    let client_transport = ProviderChannel {
        receiver: client_input,
        sender: client_output,
    };

    let dispatcher_url = url.clone();

    ProviderChannel::spawn(async move {
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
    mut dispatcher_input: Receiver<String>,
    dispatcher_output: Sender<RPCResult<String>>,
) -> RPCResult<()>
where
    S: AsRef<str> + Clone + Send + 'static + Sync,
{
    while let Some(message) = dispatcher_input.next().await {
        let response_input = dispatcher_output.clone();
        let request_url = dispatcher_url.clone();

        // panic!("hello");

        ProviderChannel::spawn(async move {
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
    message: String,
    mut response_input: Sender<RPCResult<String>>,
) -> RPCResult<()>
where
    S: AsRef<str> + Clone + Send + 'static + Sync,
{
    log::debug!("open proxy");
    let client = reqwest::Client::new();

    log::debug!("try send message: {}", message);

    let response = client
        .post(request_url.as_ref())
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(message)
        .send()
        .await
        .map_err(|err| jsonrpc_rs::Error::<String, ()>::from_std_error(err));

    log::debug!("response {:?}", response);

    match response {
        Ok(response) => {
            if response.status().is_success() {
                response_input
                    .send(
                        response
                            .text()
                            .await
                            .map_err(|err| Error::<String, ()>::from_std_error(err)),
                    )
                    .await?;
            } else {
                let err = Error::<String, ()> {
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
