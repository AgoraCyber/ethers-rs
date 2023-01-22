use ethers_providers_rs::Provider;
use ethers_types_rs::{Address, BlockNumberOrTag, Bytecode, Eip55, LegacyTransactionRequest};
use serde_json::json;

/// Ether client signer impl placeholer
///
#[derive(Clone)]
pub struct Signer {}

pub struct ContractContext {
    pub address: Address,
    pub provider: Provider,
    pub signer: Option<Signer>,
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
}
