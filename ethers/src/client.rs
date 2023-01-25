use ethers_providers_rs::Provider;
use ethers_signer_rs::signer::Signer;
use ethers_types_rs::{Address, BlockNumberOrTag, Bytecode, LegacyTransactionRequest};
use serde_json::json;

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

    pub async fn deploy_contract(&mut self, call_data: Vec<u8>) -> anyhow::Result<Address> {
        unimplemented!()
    }
}
