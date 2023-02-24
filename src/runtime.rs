pub use ethers_eip2718::*;
pub use ethers_macros::*;
pub use ethers_primitives::*;
pub use ethers_provider::*;
pub use ethers_signer::signer::Signer;
pub use serde::{Deserialize, Serialize};
pub use serde_ethabi::from_abi;
pub use serde_ethabi::to_abi;

pub use anyhow::Error;
use serde_json::json;

/// Contract client errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// Tx failed
    #[error("TxFailure: {0}")]
    TxFailure(H256),

    /// Address field is none of deploy contract tx receipt.
    #[error("DeployContract: contract_address field of receipt is null,{0}")]
    DeployContract(H256),
    /// Expect signer to execute send_raw_transaction
    #[error("SignerExpect: method {0} expect signer")]
    SignerExpect(String),
    /// Expect signer to execute send_raw_transaction
    #[error("Accounts: signer return empty accounts list")]
    Accounts,
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

/// Client to communicate with ethereum contract.
#[derive(Clone)]
pub struct Client {
    /// rpc client provider for ethereum rpc node.
    pub provider: Provider,
    /// tx signer.
    pub signer: Option<Signer>,
}

impl From<(Provider, Signer)> for Client {
    fn from((provider, signer): (Provider, Signer)) -> Self {
        Self {
            provider,
            signer: Some(signer),
        }
    }
}

impl Client {
    pub async fn deploy_contract(
        &self,
        constract_name: &str,
        mut call_data: Vec<u8>,
        deploy_data: &str,
        ops: TxOptions,
    ) -> anyhow::Result<Address> {
        let mut buff = Vec::<u8>::from_eth_hex(deploy_data)?;

        buff.append(&mut call_data);

        let tx_hash = self
            ._send_raw_transaction(constract_name, None, buff, ops, false)
            .await?;

        let receipt = self
            .provider
            .register_transaction_listener(tx_hash.clone())?
            .wait()
            .await?;

        let status = receipt
            .status
            .ok_or(ClientError::TxFailure(tx_hash.clone()))?;

        match status {
            Status::Success => {
                if let Some(contract_address) = receipt.contract_address {
                    return Ok(contract_address);
                } else {
                    return Err(ClientError::TxFailure(tx_hash).into());
                }
            }
            Status::Failure => return Err(ClientError::TxFailure(tx_hash).into()),
        }
    }

    /// Invoke contract pure/view method without send transaction.
    pub async fn eth_call(
        &self,
        method_name: &str,
        to: &Address,
        mut call_data: Vec<u8>,
    ) -> anyhow::Result<Vec<u8>> {
        log::debug!("eth_call {}", method_name);

        let mut provider = self.provider.clone();

        let mut selector_name = keccak256(method_name.as_bytes())[0..4].to_vec();

        selector_name.append(&mut call_data);

        let call_data: Bytes = selector_name.into();

        let tx: LegacyTransactionRequest = json!({
            "to": to,
            "data":call_data,
        })
        .try_into()?;

        let result = provider.eth_call(tx, None::<BlockNumberOrTag>).await?;

        Ok(result.0)
    }

    /// Send raw transaction contract `to`.
    ///
    /// # Parameters
    ///
    /// - `to` Contract address
    /// - `call_data` Transaction [`data`](LegacyTransactionRequest::data) field
    pub async fn send_raw_transaction(
        &self,
        method_name: &str,
        to: &Address,
        call_data: Vec<u8>,
        ops: TxOptions,
    ) -> anyhow::Result<DefaultTransactionReceipter> {
        let tx_hash = self
            ._send_raw_transaction(method_name, Some(to), call_data, ops, true)
            .await?;

        self.provider.register_transaction_listener(tx_hash.clone())
    }

    async fn _send_raw_transaction(
        &self,
        method_name: &str,
        to: Option<&Address>,
        mut call_data: Vec<u8>,
        ops: TxOptions,
        selector: bool,
    ) -> anyhow::Result<H256> {
        let mut provider = self.provider.clone();

        let mut signer = self
            .signer
            .clone()
            .ok_or(ClientError::SignerExpect(method_name.to_owned()))?;

        let address = {
            let mut accounts = signer.accounts().await?;

            if accounts.is_empty() {
                return Err(ClientError::Accounts.into());
            }

            accounts.remove(0)
        };

        // Get nonce first.
        let nonce = provider.eth_get_transaction_count(address).await?;

        log::debug!(
            target: method_name,
            "Fetch account {} nonce, {}",
            address.to_checksum_string(),
            nonce
        );

        // Get chain id
        let chain_id = provider.eth_chain_id().await?;

        log::debug!(target: method_name, "Fetch chain_id, {}", chain_id);

        let call_data = if selector {
            let mut selector_name = keccak256(method_name.as_bytes())[0..4].to_vec();

            selector_name.append(&mut call_data);

            selector_name
        } else {
            call_data
        };

        let mut tx = LegacyTransactionRequest {
            chain_id: Some(chain_id),
            nonce: Some(nonce),
            to: to.map(|c| c.clone()),
            data: Some(call_data.into()),
            value: ops.value,
            ..Default::default()
        };

        // estimate gas
        let gas = provider
            .eth_estimate_gas(tx.clone(), None::<BlockNumberOrTag>)
            .await?;

        log::debug!(target: method_name, "Fetch estimate gas, {}", gas);

        tx.gas = Some(gas);

        // Get gas price

        let gas_price = if let Some(gas_price) = ops.gas_price {
            gas_price
        } else {
            provider.eth_gas_price().await?
        };

        log::debug!(target: method_name, "Fetch gas price, {}", gas_price);

        tx.gas_price = Some(gas_price);

        log::debug!(
            target: method_name,
            "Try sign transaction, {}",
            serde_json::to_string(&tx)?,
        );

        let signed_tx = signer.sign_eth_transaction(tx).await?;

        log::debug!(
            target: method_name,
            "Signed transaction, {}",
            signed_tx.to_string()
        );

        let hash = provider.eth_send_raw_transaction(signed_tx).await?;

        log::debug!(target: method_name, "Send transaction success, {}", hash);

        Ok(hash)
    }

    /// Get balance of client bound signer.
    ///
    /// If client signer is [`None`], returns error [`ClientError::SignerExpect`].
    pub async fn balance(&self) -> anyhow::Result<U256> {
        let mut signer = self
            .signer
            .clone()
            .ok_or(ClientError::SignerExpect("balance".to_owned()))?;

        let address = signer.address().await?;

        Ok(self.provider.clone().eth_get_balance(address).await?)
    }
}
