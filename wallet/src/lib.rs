pub mod error;
mod vtable;
pub mod wallet;

/// Helper `Result` structure for wallet crate.
pub type Result<T> = std::result::Result<T, error::WalletError>;

pub use error::WalletError;
