#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ether abi error,{0}")]
    AbiError(ethabi::Error),
    #[error("Ether provider error, {0}")]
    ProviderError(ethers_providers_rs::ProviderError),
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
