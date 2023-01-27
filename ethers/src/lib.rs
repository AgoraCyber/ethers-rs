//! rexport ethers macros.
pub use ethers_macros_rs::*;

pub mod contract;

pub mod client;
pub use client::*;

pub use ethers_types_rs::ethabi;

pub use ethers_types_rs::*;

pub use contract::*;
pub use ethers_providers_rs::{
    DefaultFilterReceiver, DefaultTransactionReceipter, FilterReceiver, Provider,
    TransactionReceipter,
};

pub mod error;
pub use error::*;

pub type Result<T> = anyhow::Result<T>;

pub use ethers_signer_rs::signer::Signer;

pub use ethers_hardhat_rs as hardhat;

pub use async_timer_rs::hashed::Timeout;
