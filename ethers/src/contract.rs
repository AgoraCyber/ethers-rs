use ethers_providers_rs::Provider;
use ethers_signer_rs::signer::Signer;
use ethers_types_rs::{
    Address, BlockNumberOrTag, Bytecode, Eip55, LegacyTransactionRequest, H256, U256,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::Error;

pub struct ContractContext {
    pub address: Address,
    pub provider: Provider,
    pub signer: Option<Signer>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TxOptions {
    /// Mannul set gas price.
    #[serde(skip_serializing_if = "Option::is_none")]
    gas_price: Option<U256>,
    /// Transferring ether values.
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<U256>,
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

impl ContractContext {
    pub async fn eth_call(&mut self, call_data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let mut provider = self.provider.clone();

        let call_data: Bytecode = call_data.into();

        let address = self.address.to_checksum_string();

        let tx: LegacyTransactionRequest = json!({
            "to": address,
            "data":call_data,
        })
        .try_into()?;

        let result = provider.eth_call(tx, None::<BlockNumberOrTag>).await?;

        Ok(result.0)
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
        method: &str,
        call_data: Vec<u8>,
        ops: TxOptions,
    ) -> anyhow::Result<H256> {
        let mut provider = self.provider.clone();

        let mut signer = self
            .signer
            .clone()
            .ok_or(Error::InvokeMethodExpectSigner(method.to_owned()))?;

        let accounts = signer.accounts().await?;

        if accounts.is_empty() {
            return Err(Error::SignerAccounts.into());
        }

        let address = accounts[0];

        // Get nonce first.
        let nonce = provider.eth_get_transaction_count(address).await?;

        log::debug!(
            target: method,
            "Fetch account {} nonce, {:#x}",
            address.to_checksum_string(),
            nonce
        );

        // Get chain id
        let chain_id = provider.eth_chain_id().await?;

        log::debug!(target: method, "Fetch chain_id, {}", chain_id);

        let mut tx = LegacyTransactionRequest {
            chain_id: Some(chain_id),
            nonce: Some(nonce),
            to: Some(self.address.clone()),
            data: Some(call_data.into()),
            value: ops.value,
            ..Default::default()
        };

        // estimate gas
        let gas = provider
            .eth_estimate_gas(tx.clone(), None::<BlockNumberOrTag>)
            .await?;

        log::debug!(target: method, "Fetch estimate gas, {:#x}", gas);

        tx.gas = Some(gas);

        // Get gas price

        let gas_price = if let Some(gas_price) = ops.gas_price {
            gas_price
        } else {
            provider.eth_gas_price().await?
        };

        log::debug!(target: method, "Fetch gas price, {:#x}", gas_price);

        tx.gas_price = Some(gas_price);

        log::debug!(
            target: method,
            "Try sign transaction, {}",
            serde_json::to_string(&tx)?,
        );

        let signed_tx = signer.sign_eth_transaction(tx).await?;

        log::debug!(
            target: method,
            "Signed transaction, {}",
            signed_tx.to_string()
        );

        let hash = provider.eth_send_raw_transaction(signed_tx).await?;

        log::debug!(target: method, "Send transaction success, {:#?}", hash);

        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use ethers_types_rs::{Ether, Gwei};
    use serde_json::json;

    use crate::TxOptions;

    #[test]
    fn test_tx_ops() {
        let value: Ether = "1.01".try_into().expect("");

        let gas_price: Gwei = "1.01".try_into().expect("");

        let ops: TxOptions = json!({
            "gas_price":gas_price,
            "value": value,
        })
        .try_into()
        .expect("To ops");

        assert_eq!(ops.gas_price, Some(gas_price.into()));

        assert_eq!(ops.value, Some(value.into()));
    }
}
