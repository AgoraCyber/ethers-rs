//! rexport ethers macros.
pub use ethers_macros_rs::*;

pub use ethabi;
pub use ethabi::{Address, Bytes, FixedBytes, Hash, Int, Uint};

pub mod contract;

pub use contract::*;
pub use ethers_providers_rs::Provider;

pub mod error;
pub use error::*;

pub type Result<T> = std::result::Result<T, Error>;
