use std::fmt::{Debug, Display};

use ethers_eip2718::*;
use ethers_eip712::TypedData;
use ethers_primitives::*;
use jsonrpc_rs::RPCResult;
use serde::Serialize;

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
    pub async fn sign_eth_transaction<T>(&mut self, transaction_request: T) -> RPCResult<Bytes>
    where
        T: TryInto<TypedTransactionRequest>,
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
    pub async fn sign_typed_data<T, V>(&mut self, typed_data: T) -> RPCResult<Eip1559Signature>
    where
        T: TryInto<TypedData<V>>,
        T::Error: Display + Debug,
        V: Serialize,
    {
        let typed_data = typed_data.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("signer_typedData", vec![typed_data])
            .await
    }

    /// Decript data using signer private key.
    pub async fn decrypt<B>(&mut self, encrypt_data: B) -> RPCResult<Bytes>
    where
        B: TryInto<Bytes>,
        B::Error: Display + Debug,
    {
        let encrypt_data = encrypt_data.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("signer_decrypt", vec![encrypt_data])
            .await
    }

    /// Get associating signer account addresses
    pub async fn accounts(&mut self) -> RPCResult<Vec<Address>> {
        self.rpc_client.call("signer_accounts", ()).await
    }

    /// Get associating signer account addresses
    pub async fn address(&mut self) -> RPCResult<Address> {
        self.rpc_client.call("signer_address", ()).await
    }
}
