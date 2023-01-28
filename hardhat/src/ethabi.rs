use std::io;

use serde::{Deserialize, Serialize};

pub use ethers_types_rs::ethabi::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardhatArtifact {
    pub contract_name: String,
    pub source_name: String,
    pub abi: ethers_types_rs::ethabi::Contract,
    pub bytecode: String,
    pub deployed_bytecode: String,
}

impl HardhatArtifact {
    /// Loads artifact from json.
    pub fn load<T: io::Read>(reader: T) -> anyhow::Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }
}
