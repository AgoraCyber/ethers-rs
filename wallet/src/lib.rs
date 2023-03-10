pub mod error;
pub mod wallet;

/// Helper `Result` structure for wallet crate.
pub type Result<T> = std::result::Result<T, error::WalletError>;

pub use error::WalletError;

pub mod hd_wallet;

pub mod keystore;

mod hash;
