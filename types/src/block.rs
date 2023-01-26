use crate::{bytes_def, request::AccessList};

use ethabi::ethereum_types::{H256, U256};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::*;

use crate::Address;

#[derive(Debug, Clone, thiserror::Error)]
pub enum BlockError {
    #[error("Invalid syning status, expect bool or status")]
    InvalidSyning,
}

bytes_def!(BloomFilter);
bytes_def!(Difficulty);
bytes_def!(Bytecode);

#[serde_as]
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
    pub logs_bloom: Option<BloomFilter>,

    /// Difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,

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
    pub extra_data: Bytecode,

    /// Mix hash
    #[serde(rename = "mixHash")]
    pub mix_hash: Option<H256>,

    /// Nonce
    pub nonce: Option<U256>,

    /// Total difficult
    #[serde(rename = "totalDeffficult")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_deffficult: Option<Difficulty>,

    /// Base fee per gas
    #[serde(rename = "baseFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee_per_gas: Option<U256>,

    /// Block size
    pub size: U256,

    /// transactions
    #[serde_as(as = "VecSkipError<_>")]
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

#[serde_as]
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
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
    pub input: Bytecode,
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

impl TryFrom<&str> for Transaction {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<String> for Transaction {
    type Error = serde_json::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl TryFrom<serde_json::Value> for Transaction {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

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
            Ok(BlockNumberOrTag::U256(value.parse()?))
        } else {
            Ok(BlockNumberOrTag::Tag(serde_json::from_str(&format!(
                "\"{}\"",
                value
            ))?))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FeeHistory {
    /// The lowest number block of returned range.
    #[serde(rename = "oldestBlock")]
    pub oldest_block: U256,
    #[serde(rename = "baseFeePerGas")]
    pub base_fee_per_gas: Vec<U256>,

    pub reward: Vec<Vec<U256>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Filter {
    /// The lowest number block of returned range.
    #[serde(rename = "fromBlock")]
    pub from_block: Option<U256>,
    #[serde(rename = "toBlock")]
    pub to_block: Option<U256>,

    pub address: Option<FilterAddress>,

    pub topics: Option<FilterTopic>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum FilterAddress {
    Address(Address),
    Addresses(Vec<Address>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum FilterTopic {
    Signle(U256),
    Multi(Vec<U256>),
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
    pub log_index: U256,

    #[serde(rename = "transactionIndex")]
    pub transaction_index: U256,

    #[serde(rename = "transactionHash")]
    pub transaction_hash: U256,

    #[serde(rename = "blockHash")]
    pub block_hash: H256,

    #[serde(rename = "blockNumber")]
    pub block_number: U256,

    pub address: Address,

    pub data: Bytecode,

    pub topics: Vec<U256>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PollLogs {
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
            Err(BlockError::InvalidSyning).map_err(serde::de::Error::custom)
        }
    })
}

fn as_bool<S>(serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bool(false)
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Syncing {
    /// Starting block
    #[serde(rename = "startingBlock")]
    starting_block: U256,

    /// Current block
    #[serde(rename = "currentBlock")]
    current_block: U256,

    /// Highest block
    #[serde(rename = "highestBlock")]
    highest_block: U256,
}

#[cfg(test)]
mod tests {
    use jsonrpc_rs::Response;
    use serde_json::json;

    use super::*;

    #[test]
    fn dser_syncing() {
        let value = json!({
            "jsonrpc": "2.0",
            "result": false,
            "id": 0
        });

        let status: Response<String, SyncingStatus, ()> =
            serde_json::from_value(value).expect("Parse syncing false");

        assert_eq!(status.result, Some(SyncingStatus::False));

        let value = json!({
            "jsonrpc": "2.0",
            "result": {
                "startingBlock": "0x11",
                "currentBlock": "0x12",
                "highestBlock": "0x33",
            },
            "id": 0
        });

        let status: Response<String, SyncingStatus, ()> =
            serde_json::from_value(value).expect("Parse syncing status object");

        assert_eq!(
            status.result,
            Some(SyncingStatus::Syncing(Syncing {
                starting_block: 0x11.into(),
                current_block: 0x12.into(),
                highest_block: 0x33.into(),
            }))
        );
    }
}

#[serde_as]
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct TransactionReceipt {
    /// From address
    pub from: Address,
    /// To address
    pub to: Option<Address>,
    /// Contract address created by this transaction.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<Address>,
    /// Gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    /// Gas used
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U256,

    #[serde(rename = "effectiveGasPrice")]
    pub effective_gas_price: U256,

    #[serde(rename = "transactionIndex")]
    transaction_index: U256,
    /// Block hash
    #[serde(rename = "blockHash")]
    pub block_hash: H256,
    /// Block number
    #[serde(rename = "blockNumber")]
    pub block_number: U256,
    /// 1 for success, 0 for failure.
    pub status: Option<Status>,
    /// Logs
    pub logs: Vec<Log>,
    /// Logs bloom filter string
    #[serde(rename = "logsBloom")]
    pub logs_bloom: BloomFilter,
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
