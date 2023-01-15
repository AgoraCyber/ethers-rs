use ethabi::Address;
use ethers_providers_rs::Provider;

/// Ether client signer impl placeholer
///
#[derive(Clone)]
pub struct Signer {}

pub struct ContractContext {
    pub address: Address,
    pub provider: Option<Provider>,
    pub signer: Option<Signer>,
}
