use ethers_utils_rs::eth::*;
use serde::{Deserialize, Serialize};

use crate::error::ProviderError;

#[derive(Serialize, Deserialize)]
pub struct Block {
    /// Current block hash value
    pub hash: BlockHash,
    /// Parent block hash
    #[serde(rename = "parentHash")]
    pub parent_hash: BlockHash,

    /// Ommers hash
    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles: Sha3Hash,

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
    pub number: Number,

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
    pub mix_hash: MixHash,

    /// Nonce
    pub nonce: Nonce,

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
    pub transactions: Vec<TransactionOrHash>,

    /// Uncles
    pub uncles: Vec<UncleHash>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionOrHash {
    Hash(TransactionHash),
    Transaction(Transaction),
}

#[derive(Serialize, Deserialize)]
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
    transaction_index: Number,
    /// Block hash
    #[serde(rename = "blockHash")]
    pub block_hash: BlockHash,
    /// Block number
    #[serde(rename = "blockNumber")]
    pub block_number: Number,
    /// Gas limit
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<Number>,
    /// Transaction hash
    pub hash: TransactionHash,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<AccessList>>,
    /// Chain ID tha this transaction is valid on
    #[serde(rename = "chainId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<Number>,
    /// The parity(0 for even, 1 for odd) of the y-value of the secp256k1 signature.
    pub v: Number,
    /// r-value of the secp256k1
    pub r: Number,
    /// s-value of the secp256k1
    pub s: Number,
}

#[derive(Serialize, Deserialize)]
pub struct AccessList {
    /// address that the transaction plans to access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    /// address storage keys that the transaction plans to access
    #[serde(rename = "storageKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_keys: Option<Vec<Sha3Hash>>,
}

/// eth_getBlockByNumber parameter `Block`
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum BlockNumberOrTag {
    Number(Number),
    Tag(BlockTag),
}

impl TryFrom<Number> for BlockNumberOrTag {
    type Error = ProviderError;
    fn try_from(value: Number) -> Result<Self, Self::Error> {
        Ok(BlockNumberOrTag::Number(value))
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
    type Error = ProviderError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with("0x") {
            Ok(BlockNumberOrTag::Number(
                value.try_into().map_err(|err| ProviderError::Number(err))?,
            ))
        } else {
            Ok(BlockNumberOrTag::Tag(
                serde_json::from_str(&format!("\"{}\"", value))
                    .map_err(|err| ProviderError::BlockTag(err))?,
            ))
        }
    }
}

#[cfg(test)]
mod tests {

    use ethers_utils_rs::hex::Hex;
    use serde::{Deserialize, Serialize};

    use crate::types::BlockTag;

    use super::{Block, BlockNumberOrTag, Transaction};

    fn check_serde<'de, T: Serialize + Deserialize<'de>>(tag: &str, data: &'de str) {
        // let _: serde_json::Value =
        //     serde_json::from_str(data).expect(format!("Deserialize {}", tag).as_str());

        let _: T = serde_json::from_str(data).expect(format!("Deserialize {}", tag).as_str());

        // assert_eq!(
        //     serde_json::to_value(&json).expect("Serialize json"),
        //     serde_json::to_value(&t).expect("Serialize json")
        // );
    }

    #[test]
    fn test_serde() {
        _ = pretty_env_logger::try_init();

        let blocks = vec![
            ("block", include_str!("test-data/block/block.json")),
            ("0x30e49e13258f051e6ea8ec36f3e4e15df663396cf307299dbf5830441fd8ed98", include_str!("test-data/block/0x30e49e13258f051e6ea8ec36f3e4e15df663396cf307299dbf5830441fd8ed98.json"))
        ];

        for (tag, data) in blocks {
            check_serde::<Block>(tag, data);
        }

        let txs = vec![
            ("block", include_str!("test-data/tx/0x0bb3c2388383f714a8070dc6078a5edbe78f23c96646d4148d63cf964197ccc5.json"))
            ];

        for (tag, data) in txs {
            check_serde::<Transaction>(tag, data);
        }
    }

    #[test]
    fn test_block_tag() {
        fn check_block_number_or_tag(source: &str, expect: BlockNumberOrTag) {
            let t: BlockNumberOrTag = source.try_into().expect(&format!("Parse {} error", source));

            assert_eq!(expect, t);
        }

        check_block_number_or_tag(
            "0x1001",
            BlockNumberOrTag::Number(Hex([0x10, 0x01].to_vec())),
        );

        check_block_number_or_tag("earliest", BlockNumberOrTag::Tag(BlockTag::Earliest));

        check_block_number_or_tag("finalized", BlockNumberOrTag::Tag(BlockTag::Finalized));

        check_block_number_or_tag("safe", BlockNumberOrTag::Tag(BlockTag::Safe));

        check_block_number_or_tag("latest", BlockNumberOrTag::Tag(BlockTag::Latest));

        check_block_number_or_tag("pending", BlockNumberOrTag::Tag(BlockTag::Pending));
    }
}
