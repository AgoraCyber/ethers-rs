use std::fmt::{Debug, Display};

use ethers_utils_rs::types::{Bytecode, Transaction};
use jsonrpc_rs::RPCResult;

pub mod error;
pub mod signers;
mod typed;
pub use typed::*;

/// Signer `Result` type alias with [`error::SignerError`]
pub type Result<T> = std::result::Result<T, error::SignerError>;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Signer {
    rpc_client: jsonrpc_rs::Client,
}

impl Signer {
    pub fn new(rpc_client: jsonrpc_rs::Client) -> Self {
        Self { rpc_client }
    }

    /// Returns the signed transaction of the parameter `transaction_request`
    pub async fn sign_eth_transaction<T>(&mut self, transaction_request: T) -> RPCResult<Bytecode>
    where
        T: TryInto<Transaction>,
        T::Error: Display + Debug,
    {
        let transaction_request = transaction_request
            .try_into()
            .map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("signer_signEthTransaction", vec![transaction_request])
            .await
    }
}
