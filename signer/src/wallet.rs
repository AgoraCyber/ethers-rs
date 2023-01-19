//! Create signer from local wallet.

use ethers_utils_rs::{
    eip712::TypedData,
    types::{Address, AddressEx, Bytecode, Eip55, Transaction},
};
use ethers_wallet_rs::wallet::Wallet;
use futures::{
    channel::mpsc::{self, SendError, Sender},
    executor::ThreadPool,
    stream::BoxStream,
    task::SpawnExt,
    StreamExt,
};
use jsonrpc_rs::{channel::TransportChannel, RPCData, RPCResult, Server};
use once_cell::sync::OnceCell;

use crate::signer::Signer;

/// Ethererum network provider tokio io event driver channel.
///
pub(crate) struct LocalWalletChannel {
    pub(crate) receiver: BoxStream<'static, RPCResult<RPCData>>,
    pub(crate) sender: Sender<RPCData>,
}

impl TransportChannel for LocalWalletChannel {
    type StreamError = jsonrpc_rs::RPCError;

    type SinkError = SendError;

    type Input = BoxStream<'static, RPCResult<RPCData>>;

    type Output = Sender<RPCData>;

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

pub trait WalletSigner {
    /// Convert wallet into signer
    fn try_into_signer(self) -> anyhow::Result<Signer>;
}

impl WalletSigner for Wallet {
    fn try_into_signer(self) -> anyhow::Result<Signer> {
        let address = self.public_key(false)?;

        let address = Address::from_pub_key(address.as_slice())?;

        let (client_output, dispatcher_input) = mpsc::channel(20);
        let (dispatcher_output, client_input) = mpsc::channel(20);

        // Create mpsc transport, real send/recv network message are in procedure dispatcher.
        let client_transport = LocalWalletChannel {
            receiver: client_input.map(|c| Ok(c)).boxed(),
            sender: client_output,
        };

        LocalWalletChannel::spawn(async move {
            let server_transport = LocalWalletChannel {
                receiver: dispatcher_input.map(|c| Ok(c)).boxed(),
                sender: dispatcher_output,
            };

            let mut server = Server::default();

            let wallet = self.clone();

            server.async_handle("signer_ethTransaction", move |tx: Transaction| {
                sign_transaction(wallet.clone(), tx)
            });

            let wallet = self.clone();

            server.async_handle("signer_typedData", move |typed_data: TypedData| {
                sign_typed_data(wallet.clone(), typed_data)
            });

            let wallet = self.clone();

            server.async_handle("signer_decrypt", move |data: Bytecode| {
                decrypt(wallet.clone(), data)
            });

            server.accept(server_transport);

            Ok(())
        });

        Ok(Signer::new(jsonrpc_rs::Client::new(
            format!("local_wallet_{}", address.to_checksum_string()),
            client_transport,
        )))
    }
}

#[allow(unused)]
async fn sign_transaction(wallet: Wallet, t: Transaction) -> RPCResult<Option<Bytecode>> {
    unimplemented!()
}

#[allow(unused)]
async fn sign_typed_data(wallet: Wallet, data: TypedData) -> RPCResult<Option<Bytecode>> {
    unimplemented!()
}

#[allow(unused)]
async fn decrypt(wallet: Wallet, data: Bytecode) -> RPCResult<Option<Bytecode>> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use ethers_wallet_rs::wallet::Wallet;
    use serde_json::json;

    use super::WalletSigner;

    #[async_std::test]
    async fn test_sign_tx() {
        let _ = pretty_env_logger::try_init();

        let wallet =
            Wallet::new("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("Create hardhat account 0 wallet");

        let mut signer = wallet
            .try_into_signer()
            .expect("Try convert wallet into signer");

        signer
            .sign_eth_transaction(json!({}))
            .await
            .expect("Sign tx");
    }
}
