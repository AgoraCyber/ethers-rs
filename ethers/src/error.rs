use ethers_types_rs::{ethabi, H256};
use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ether abi error,{0}")]
    AbiError(ethabi::Error),
    #[error("Ether provider error, {0}")]
    ProviderError(ethers_providers_rs::ProviderError),

    #[error("Uknown error, {0}")]
    Unknown(String),

    /// Contract call expect signer
    #[error("Expect signer to invoke {0}")]
    InvokeMethodExpectSigner(String),

    #[error("Signer accounts method empty accounts array")]
    SignerAccounts,

    #[error("Execute tx return failed, {0}")]
    TxFailure(H256),

    #[error("Tx {0} return success, but contract address is null")]
    ContractAddress(H256),
}

impl From<ethabi::Error> for Error {
    fn from(inner: ethabi::Error) -> Self {
        Error::AbiError(inner)
    }
}

impl From<ethers_providers_rs::ProviderError> for Error {
    fn from(inner: ethers_providers_rs::ProviderError) -> Self {
        Error::ProviderError(inner)
    }
}

pub fn custom_error<E>(e: E) -> Error
where
    E: Display,
{
    Error::Unknown(format!("{}", e))
}
