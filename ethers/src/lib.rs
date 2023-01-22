//! rexport ethers macros.
pub use ethers_macros_rs::*;

pub mod contract;

pub use ethers_types_rs::ethabi;

pub use ethers_types_rs::*;

pub use contract::*;
pub use ethers_providers_rs::Provider;

pub mod error;
pub use error::*;

pub type Result<T> = std::result::Result<T, Error>;
