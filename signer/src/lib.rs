pub mod error;
pub mod signers;

/// Signer `Result` type alias with [`error::SignerError`]
pub type Result<T> = std::result::Result<T, error::SignerError>;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Signer {
    rpc_client: jsonrpc_rs::Client,
}

impl Signer {
    pub fn new(rpc_client: jsonrpc_rs::Client) -> Self {
        Self { rpc_client }
    }
}
