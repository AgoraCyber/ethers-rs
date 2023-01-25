use ethers_providers_rs::Provider;
use ethers_signer_rs::signer::Signer;
use ethers_types_rs::{
    Address, BlockNumberOrTag, Bytecode, Eip55, EthereumUnit, LegacyTransactionRequest, Status,
    H256, U256,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::Error;

/// The client to access ether blockchain functions.
#[derive(Clone)]
pub struct Client {
    pub provider: Provider,
    pub signer: Option<Signer>,
}

impl From<Provider> for Client {
    fn from(value: Provider) -> Self {
        Client {
            provider: value,
            signer: None,
        }
    }
}

impl From<(Provider, Signer)> for Client {
    fn from(value: (Provider, Signer)) -> Self {
        Client {
            provider: value.0,
            signer: Some(value.1),
        }
    }
}

impl Client {
    pub async fn eth_call(&mut self, to: Address, call_data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let mut provider = self.provider.clone();

        let call_data: Bytecode = call_data.into();

        let tx: LegacyTransactionRequest = json!({
            "to": to,
            "data":call_data,
        })
        .try_into()?;

        let result = provider.eth_call(tx, None::<BlockNumberOrTag>).await?;

        Ok(result.0)
    }

    pub async fn deploy_contract(
        &mut self,
        call_data: Vec<u8>,
        ops: TxOptions,
    ) -> anyhow::Result<Address> {
        let tx_hash = self
            .send_raw_transaction("deploy", None, call_data, ops)
            .await?;

        let mut provider = self.provider.clone();

        let receipt = provider
            .eth_get_transaction_receipt(tx_hash.clone())
            .await?;

        if let Some(receipt) = receipt {
            if let Some(status) = receipt.status {
                match status {
                    Status::Success => {
                        if let Some(contract_address) = receipt.contract_address {
                            return Ok(contract_address);
                        } else {
                            return Err(Error::ContractAddress(tx_hash).into());
                        }
                    }
                    Status::Failure => return Err(Error::TxFailure(tx_hash).into()),
                }
            }
        }

        unimplemented!("Impl tx completed event")
    }

    /// Send raw transaction to contract address.
    ///
    /// # Parameters
    ///
    /// * `value` - Amount of native token transferring to contract.
    /// * `call_data` - Invoke contract method call data. visit solidity [`abi`](https://docs.soliditylang.org/en/v0.8.17/abi-spec.html) for more information
    ///
    pub async fn send_raw_transaction(
        &mut self,
        tag: &str,
        to: Option<Address>,
        call_data: Vec<u8>,
        ops: TxOptions,
    ) -> anyhow::Result<H256> {
        let mut provider = self.provider.clone();

        let mut signer = self
            .signer
            .clone()
            .ok_or(Error::InvokeMethodExpectSigner(tag.to_owned()))?;

        let accounts = signer.accounts().await?;

        if accounts.is_empty() {
            return Err(Error::SignerAccounts.into());
        }

        let address = accounts[0];

        // Get nonce first.
        let nonce = provider.eth_get_transaction_count(address).await?;

        log::debug!(
            target: tag,
            "Fetch account {} nonce, {:#x}",
            address.to_checksum_string(),
            nonce
        );

        // Get chain id
        let chain_id = provider.eth_chain_id().await?;

        log::debug!(target: tag, "Fetch chain_id, {}", chain_id);

        let mut tx = LegacyTransactionRequest {
            chain_id: Some(chain_id),
            nonce: Some(nonce),
            to,
            data: Some(call_data.into()),
            value: ops.value,
            ..Default::default()
        };

        // estimate gas
        let gas = provider
            .eth_estimate_gas(tx.clone(), None::<BlockNumberOrTag>)
            .await?;

        log::debug!(target: tag, "Fetch estimate gas, {:#x}", gas);

        tx.gas = Some(gas);

        // Get gas price

        let gas_price = if let Some(gas_price) = ops.gas_price {
            gas_price
        } else {
            provider.eth_gas_price().await?
        };

        log::debug!(target: tag, "Fetch gas price, {:#x}", gas_price);

        tx.gas_price = Some(gas_price);

        log::debug!(
            target: tag,
            "Try sign transaction, {}",
            serde_json::to_string(&tx)?,
        );

        let signed_tx = signer.sign_eth_transaction(tx).await?;

        log::debug!(target: tag, "Signed transaction, {}", signed_tx.to_string());

        let hash = provider.eth_send_raw_transaction(signed_tx).await?;

        log::debug!(target: tag, "Send transaction success, {:#?}", hash);

        Ok(hash)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TxOptions {
    /// Mannul set gas price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    /// Transferring ether values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
}

impl<'a> TryFrom<&'a str> for TxOptions {
    type Error = anyhow::Error;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}

impl TryFrom<String> for TxOptions {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}

impl TryFrom<serde_json::Value> for TxOptions {
    type Error = anyhow::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(value)?)
    }
}

pub trait ToTxOptions {
    fn to_tx_options(self) -> TxOptions;
}

impl<T: EthereumUnit> ToTxOptions for T {
    fn to_tx_options(self) -> TxOptions {
        TxOptions {
            gas_price: None,
            value: Some(self.to_u256()),
        }
    }
}
