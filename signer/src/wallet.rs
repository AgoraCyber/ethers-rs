//! Create signer from local wallet.

use ethers_types_rs::*;

use ethers_wallet_rs::wallet::Wallet;
use futures::{
    channel::mpsc::{self, SendError, Sender},
    executor::ThreadPool,
    stream::BoxStream,
    task::SpawnExt,
    StreamExt,
};
use jsonrpc_rs::{channel::TransportChannel, map_error, RPCData, RPCResult, Server};
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

            #[allow(unused_parens)]
            server.async_handle("signer_ethTransaction", move |tx| {
                sign_transaction(wallet.clone(), tx)
            });

            let wallet = self.clone();

            #[allow(unused_parens)]
            server.async_handle("signer_typedData", move |typed_data| {
                sign_typed_data(wallet.clone(), typed_data)
            });

            let wallet = self.clone();

            #[allow(unused_parens)]
            server.async_handle("signer_decrypt", move |data| decrypt(wallet.clone(), data));

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
async fn sign_transaction(wallet: Wallet, t: TypedTransaction) -> RPCResult<Option<Signature>> {
    let hashed = t.sign_hash();

    let signature = wallet.sign(hashed).map_err(map_error)?;

    Ok(Some(signature))
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

    use ethers_types_rs::{Eip2930TransactionRequest, LegacyTransactionRequest, SignatureVRS};
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

        let tx: LegacyTransactionRequest = json!({
            "nonce": "0x1",
            "to": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "value":"0x1",
            "input":"0x",
            "gas":"0x60000",
            "gasPrice": "0x60000111",
            "chainId": "0x1"

        })
        .try_into()
        .expect("Create tx");

        let signature = signer
            .sign_eth_transaction(tx.clone())
            .await
            .expect("Sign tx");

        log::debug!("v {}", signature.v());
        log::debug!("r {}", signature.r());
        log::debug!("s {}", signature.s());

        assert_eq!(
            "0xf864018460000111830600009470997970c51812dc3a010c7d01b50e0d17dc79c8018026a06c7e1e13070e6f10e51d7d20e986c59fd080fc6afc5508f44e8b0a84a58b7d1aa013c20fa2b6d77ae6814a41b674946387dde6401c73eb0cab2246a2981c48e344",
            tx.rlp_signed(signature).to_string()
        );

        let tx: Eip2930TransactionRequest = json!({
            "type": "0x01",
            "nonce": "0x1",
            "to": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "value":"0x1",
            "input":"0x",
            "gas":"0x60000",
            "gasPrice": "0x60000111",
            "chainId": "0x1",
            "accessList":[]

        })
        .try_into()
        .expect("Create tx");

        let signature = signer
            .sign_eth_transaction(tx.clone())
            .await
            .expect("Sign tx");

        log::debug!("v {}", signature.v());
        log::debug!("r {}", signature.r());
        log::debug!("s {}", signature.s());

        assert_eq!(
            "0x01f86601018460000111830600009470997970c51812dc3a010c7d01b50e0d17dc79c80180c080a0fd6402ef803609fce890c09304abd681fe29c25616e960c1f44db1a026f5d03ba00867e433e4ddd6e8b0c8f283c08a7f81d2cc41cc29aaa8631d7b979a8ec9e8ec",
            tx.rlp_signed(signature).to_string()
        );
    }
}
