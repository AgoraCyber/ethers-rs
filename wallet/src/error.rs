use ethers_primitives::FromHexError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("ecdsa error,{0}")]
    ECDSA(String),

    #[error("Invalid recover id,{0}")]
    RecoverId(u8),

    #[error("Load key error, {0}")]
    LoadKey(String),

    #[error("{0}")]
    FromHexError(#[from] FromHexError),
}
