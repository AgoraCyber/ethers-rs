use ethers_utils_rs::eth::*;
use serde::{Deserialize, Serialize};
use serde_with::*;

use super::AccessList;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Block {
    /// Current block hash value
    pub hash: Option<BlockHash>,
    /// Parent block hash
    #[serde(rename = "parentHash")]
    pub parent_hash: BlockHash,

    /// Ommers hash
    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles: Option<Sha3Hash>,

    /// Coinbase
    pub miner: Address,

    /// State root
    #[serde(rename = "stateRoot")]
    pub state_root: MerkleHash,

    /// Transactions root
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: TransactionsRoot,

    /// Receipts root
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: ReceiptsRoot,

    /// Bloom filter
    #[serde(rename = "logsBloom")]
    pub logs_bloom: BloomFilter,

    /// Difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,

    /// Number
    pub number: Option<Number>,

    /// Gas limit

    #[serde(rename = "gasLimit")]
    pub gas_limit: Number,

    /// Gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: Number,

    /// Timestamp
    pub timestamp: Number,

    /// Extra data
    #[serde(rename = "extraData")]
    pub extra_data: ExtraData,

    /// Mix hash
    #[serde(rename = "mixHash")]
    pub mix_hash: Option<MixHash>,

    /// Nonce
    pub nonce: Option<Nonce>,

    /// Total difficult
    #[serde(rename = "totalDeffficult")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_deffficult: Option<Difficulty>,

    /// Base fee per gas
    #[serde(rename = "baseFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee_per_gas: Option<Number>,

    /// Block size
    pub size: Number,

    /// transactions
    #[serde_as(as = "VecSkipError<_>")]
    pub transactions: Vec<TransactionOrHash>,

    /// Uncles
    pub uncles: Vec<UncleHash>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionOrHash {
    Null,
    Hash(TransactionHash),
    Transaction(Transaction),
}

#[serde_as]
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Transaction {
    /// transaction type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    /// transaction nonce
    pub nonce: Number,
    /// To address
    pub to: Address,
    /// Gas limit
    pub gas: Number,
    #[serde(rename = "transactionIndex")]
    transaction_index: Option<Number>,
    /// Block hash
    #[serde(rename = "blockHash")]
    pub block_hash: Option<BlockHash>,
    /// Block number
    #[serde(rename = "blockNumber")]
    pub block_number: Option<Number>,
    /// Gas limit
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<Number>,
    /// Transaction hash
    pub hash: Option<TransactionHash>,
    /// Transfer eth value
    pub value: Number,
    /// Input data to call contract.
    pub input: Input,
    /// Maximum fee per gas the sender is willing to pay to miners in wei
    #[serde(rename = "maxPriorityFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<Number>,
    /// Maximum total fee per gas the sender is willing to
    /// pay(includes the network/base fee and miner/ priority fee) in wei
    #[serde(rename = "maxFeePerGas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<Number>,
    /// EIP-2930 access list
    pub access_list: Option<Vec<AccessList>>,
    /// Chain ID tha this transaction is valid on
    #[serde(rename = "chainId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<Number>,
    /// The parity(0 for even, 1 for odd) of the y-value of the secp256k1 signature.
    pub v: Option<Number>,
    /// r-value of the secp256k1
    pub r: Option<Number>,
    /// s-value of the secp256k1
    pub s: Option<Number>,
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

#[serde_as]
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct TransactionReceipt {
    /// From address
    pub from: Address,
    /// To address
    pub to: Option<Address>,
    /// Contract address created by this transaction.
    pub constract_address: Option<Address>,
    /// Gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: Number,
    /// Gas used
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: Number,

    #[serde(rename = "effectiveGasPrice")]
    pub effective_gas_price: Number,

    #[serde(rename = "transactionIndex")]
    transaction_index: Number,
    /// Block hash
    #[serde(rename = "blockHash")]
    pub block_hash: BlockHash,
    /// Block number
    #[serde(rename = "blockNumber")]
    pub block_number: Number,
    /// 1 for success, 0 for failure.
    pub status: Option<Status>,
    /// Logs
    pub logs: Vec<super::filter::Log>,
    /// Logs bloom filter string
    pub logs_bloom: BloomFilter,
    /// Only include before the Byzantium upgrade
    pub root: Option<MerkleHash>,
}
