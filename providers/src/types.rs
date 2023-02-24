use ethers_primitives::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ProviderError;

pub use ethers_eip2718::AccessList;

macro_rules! from_json {
    ($name: ident) => {
        impl TryFrom<&str> for $name {
            type Error = serde_json::Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                serde_json::from_str(value)
            }
        }

        impl TryFrom<String> for $name {
            type Error = serde_json::Error;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_ref())
            }
        }

        impl TryFrom<serde_json::Value> for $name {
            type Error = serde_json::Error;
            fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
                serde_json::from_value(value)
            }
        }
    };
}

/// When a Contract creates a log, it can include up to 4 pieces of data to be indexed by.
/// The indexed data is hashed and included in a Bloom Filter, which is a data structure
/// that allows for efficient filtering.
///
/// So, a filter may correspondingly have up to 4 topic-sets, where each topic-set refers
/// to a condition that must match the indexed log topic in that position
/// (i.e. each condition is AND-ed together).
///
/// If a topic-set is null, a log topic in that position is not filtered at all and any
/// value matches.
///
/// If a topic-set is a single topic, a log topic in that position must match that topic.
///
/// If a topic-set is an array of topics, a log topic in that position must match any one
/// of the topics (i.e. the topic in this position are OR-ed).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TopicFilter(Topic, Topic, Topic, Topic);

/// Topic filter expr
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Topic {
    /// Not filtered at all and any value matches
    Unset,
    /// Match any of the hashes.
    OneOf(Vec<H256>),
    /// Match only this hash.
    This(H256),
}

impl Default for Topic {
    fn default() -> Self {
        Self::Unset
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum AddressFilter {
    Address(Address),
    Addresses(Vec<Address>),
}

impl From<Address> for AddressFilter {
    fn from(value: Address) -> Self {
        AddressFilter::Address(value)
    }
}

impl From<Vec<Address>> for AddressFilter {
    fn from(value: Vec<Address>) -> Self {
        AddressFilter::Addresses(value)
    }
}

from_json!(AddressFilter);

/// Filter argument for events
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    /// The lowest number block of returned range.
    pub from_block: Option<U256>,
    /// The highest number block of returned range.
    pub to_block: Option<U256>,

    pub address: Option<AddressFilter>,

    pub topics: Option<TopicFilter>,
}

impl From<(Address, TopicFilter)> for Filter {
    fn from(value: (Address, TopicFilter)) -> Self {
        Filter {
            from_block: None,
            to_block: None,
            address: Some(AddressFilter::Address(value.0)),
            topics: Some(value.1),
        }
    }
}

impl From<(Vec<Address>, TopicFilter)> for Filter {
    fn from(value: (Vec<Address>, TopicFilter)) -> Self {
        Filter {
            from_block: None,
            to_block: None,
            address: Some(AddressFilter::Addresses(value.0)),
            topics: Some(value.1),
        }
    }
}

from_json!(Filter);

/// eth_getBlockByNumber parameter `Block`
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum BlockNumberOrTag {
    U256(U256),
    Tag(BlockTag),
}

impl From<U256> for BlockNumberOrTag {
    fn from(v: U256) -> Self {
        BlockNumberOrTag::U256(v)
    }
}

impl From<BlockTag> for BlockNumberOrTag {
    fn from(v: BlockTag) -> Self {
        BlockNumberOrTag::Tag(v)
    }
}

impl Default for BlockNumberOrTag {
    fn default() -> Self {
        BlockNumberOrTag::Tag(BlockTag::Latest)
    }
}

/// eth_getBlockByNumber parameter `Block` valid tag enum
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum BlockTag {
    Earliest,
    Finalized,
    Safe,
    Latest,
    Pending,
}

impl<'a> TryFrom<&'a str> for BlockNumberOrTag {
    type Error = anyhow::Error;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.starts_with("0x") {
            Ok(BlockNumberOrTag::U256(serde_json::from_str(&format!(
                "\"{}\"",
                value
            ))?))
        } else {
            Ok(BlockNumberOrTag::Tag(serde_json::from_str(&format!(
                "\"{}\"",
                value
            ))?))
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    /// The lowest number block of returned range.
    pub oldest_block: U256,

    pub base_fee_per_gas: Vec<U256>,

    pub reward: Vec<Vec<U256>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub removed: bool,

    pub log_index: U256,

    pub transaction_index: U256,

    pub transaction_hash: U256,

    pub block_hash: H256,

    pub block_number: U256,

    pub address: Address,

    pub data: Bytes,

    pub topics: Vec<H256>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FilterEvents {
    BlocksOrTransactions(Vec<H256>),

    Logs(Vec<Log>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SyncingStatus {
    Syncing(Syncing),

    #[serde(deserialize_with = "from_bool", serialize_with = "as_bool")]
    False,
}

impl Default for SyncingStatus {
    fn default() -> Self {
        SyncingStatus::False
    }
}

fn from_bool<'de, D>(d: D) -> std::result::Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    bool::deserialize(d).and_then(|flag| {
        if !flag {
            Ok(())
        } else {
            Err(ProviderError::Syncing).map_err(serde::de::Error::custom)
        }
    })
}

fn as_bool<S>(serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bool(false)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Syncing {
    /// Starting block
    starting_block: U256,

    /// Current block
    current_block: U256,

    /// Highest block
    highest_block: U256,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// From address
    pub from: Address,
    /// To address
    pub to: Option<Address>,
    /// Contract address created by this transaction.
    pub contract_address: Option<Address>,
    /// Gas used
    pub gas_used: U256,
    /// Gas used
    pub cumulative_gas_used: U256,

    pub effective_gas_price: U256,

    transaction_index: U256,
    /// Block hash
    pub block_hash: H256,
    /// Block number
    pub block_number: U256,
    /// 1 for success, 0 for failure.
    pub status: Option<Status>,
    /// Logs
    pub logs: Vec<Log>,
    /// Logs bloom filter string
    pub logs_bloom: Bytes,
    /// Only include before the Byzantium upgrade
    pub root: Option<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    // 0x00
    #[serde(rename = "0x1")]
    Success,
    // 0x01
    #[serde(rename = "0x0")]
    Failure,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    /// Current block hash value
    pub hash: Option<H256>,
    /// Parent block hash
    #[serde(rename = "parentHash")]
    pub parent_hash: H256,

    /// Ommers hash
    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles: Option<H256>,

    /// Coinbase
    pub miner: Address,

    /// State root
    #[serde(rename = "stateRoot")]
    pub state_root: H256,

    /// Transactions root
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: H256,

    /// Receipts root
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: H256,

    /// Bloom filter
    #[serde(rename = "logsBloom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_bloom: Option<Bytes>,

    /// Difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Bytes>,

    /// U256
    pub number: Option<U256>,

    /// Gas limit

    #[serde(rename = "gasLimit")]
    pub gas_limit: U256,

    /// Gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,

    /// Timestamp
    pub timestamp: U256,

    /// Extra data
    #[serde(rename = "extraData")]
    pub extra_data: Bytes,

    /// Mix hash
    #[serde(rename = "mixHash")]
    pub mix_hash: Option<H256>,

    /// Nonce
    pub nonce: Option<U256>,

    /// Total difficult
    #[serde(rename = "totalDeffficult")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_deffficult: Option<Bytes>,

    /// Base fee per gas
    #[serde(rename = "baseFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee_per_gas: Option<U256>,

    /// Block size
    pub size: U256,

    /// transactions
    pub transactions: Vec<TransactionOrHash>,

    /// Uncles
    pub uncles: Vec<H256>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionOrHash {
    Null,
    Hash(H256),
    Transaction(Transaction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    // 0x00
    #[serde(rename = "0x00")]
    Legacy,
    // 0x01
    #[serde(rename = "0x01")]
    Eip2930,
    // 0x02
    #[serde(rename = "0x02")]
    Eip1559,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    /// transaction type
    ///
    /// 1. Legacy (pre-EIP2718) `0x00`
    /// 2. EIP2930 (state access lists) `0x01`
    /// 3. EIP1559 0x02
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<TransactionType>,
    /// transaction nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,
    /// To address
    pub to: Address,
    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,

    /// Transaction index in block
    #[serde(rename = "transactionIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction_index: Option<U256>,
    /// Block hash
    #[serde(rename = "blockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<H256>,
    /// Block number
    #[serde(rename = "blockNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U256>,
    /// Gas limit
    #[serde(rename = "gasPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    /// Transaction hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<H256>,
    /// Transfer eth value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// Input data to call contract.
    pub input: Bytes,
    /// Maximum fee per gas the sender is willing to pay to miners in wei
    #[serde(rename = "maxPriorityFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
    /// Maximum total fee per gas the sender is willing to
    /// pay(includes the network/base fee and miner/ priority fee) in wei
    #[serde(rename = "maxFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>,
    /// EIP-2930 access list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    /// Chain ID tha this transaction is valid on
    #[serde(rename = "chainId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<U256>,
    /// The parity(0 for even, 1 for odd) of the y-value of the secp256k1 signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<U256>,
    /// r-value of the secp256k1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<U256>,
    /// s-value of the secp256k1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<U256>,
}

from_json!(Transaction);
