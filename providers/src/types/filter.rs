use ethers_utils_rs::eth::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Filter {
    /// The lowest number block of returned range.
    #[serde(rename = "fromBlock")]
    from_block: Option<Number>,
    #[serde(rename = "toBlock")]
    to_block: Option<Number>,

    address: Option<FilterAddress>,

    topics: Option<FilterTopic>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FilterAddress {
    Address(Address),
    Addresses(Vec<Address>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FilterTopic {
    Signle(Topic),
    Multi(Vec<Topic>),
}

impl TryFrom<&str> for Filter {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<String> for Filter {
    type Error = serde_json::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl TryFrom<serde_json::Value> for Filter {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    pub removed: bool,

    #[serde(rename = "logIndex")]
    pub log_index: Number,

    #[serde(rename = "transactionIndex")]
    pub transaction_index: Number,

    #[serde(rename = "transactionHash")]
    pub transaction_hash: Number,

    #[serde(rename = "blockHash")]
    pub block_hash: BlockHash,

    #[serde(rename = "blockNumber")]
    pub block_number: Number,

    pub address: Address,

    pub data: Bytecode,

    pub topics: Vec<Topic>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PollLogs {
    BlocksOrTransactions(Vec<Sha3Hash>),

    Logs(Vec<Log>),
}
