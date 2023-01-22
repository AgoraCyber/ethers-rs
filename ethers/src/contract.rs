use ethers_providers_rs::Provider;
use ethers_types_rs::Address;

/// Ether client signer impl placeholer
///
#[derive(Clone)]
pub struct Signer {}

pub struct ContractContext {
    pub address: Address,
    pub provider: Option<Provider>,
    pub signer: Option<Signer>,
}
