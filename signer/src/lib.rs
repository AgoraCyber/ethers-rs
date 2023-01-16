use std::fmt::{Debug, Display};

use ethers_utils_rs::{
    eip712::{self, EIP712Domain},
    types::{Bytecode, Transaction},
};
use jsonrpc_rs::RPCResult;

pub mod error;
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
            .call("signer_ethTransaction", vec![transaction_request])
            .await
    }

    /// Returns the signed typed data, using [`eip-712`](https://eips.ethereum.org/EIPS/eip-712) algorithm
    pub async fn sign_typed_data<D, T, S, V>(
        &mut self,
        domain: D,
        types: T,
        primary_type: S,
        value: V,
    ) -> RPCResult<Bytecode>
    where
        S: AsRef<str>,
        D: TryInto<EIP712Domain>,
        D::Error: Display + Debug,
        T: TryInto<eip712::Types>,
        T::Error: Display + Debug,
        V: TryInto<eip712::Value>,
        V::Error: Display + Debug,
    {
        let domain = domain.try_into().map_err(jsonrpc_rs::map_error)?;
        let types = types.try_into().map_err(jsonrpc_rs::map_error)?;
        let value = value.try_into().map_err(jsonrpc_rs::map_error)?;

        let primary_type = primary_type.as_ref().to_owned();

        self.rpc_client
            .call("signer_typedData", (domain, types, primary_type, value))
            .await
    }

    /// Decript data using signer private key.
    pub async fn decrypt<B>(&mut self, encrypt_data: B) -> RPCResult<Bytecode>
    where
        B: TryInto<Bytecode>,
        B::Error: Display + Debug,
    {
        let encrypt_data = encrypt_data.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("signer_decrypt", vec![encrypt_data])
            .await
    }
}
