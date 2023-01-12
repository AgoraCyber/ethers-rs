use ethers_utils_rs::eth::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessList {
    /// address that the transaction plans to access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    /// address storage keys that the transaction plans to access
    #[serde(rename = "storageKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_keys: Option<Vec<Sha3Hash>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessListResult {
    /// Access list created
    #[serde(rename = "accessList")]
    access_list: Option<AccessList>,
    /// Error message when created failed.
    error: Option<String>,
    /// Gas used
    #[serde(rename = "gasUsed")]
    gas_used: Number,
}
