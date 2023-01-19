use crate::{rlp::rlp_opt, types::*};
use rlp::RlpStream;
use serde::{de, Deserialize, Serialize};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_bloom: Option<BloomFilter>,

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

#[derive(Debug, Clone)]
pub enum TypedTransaction {
    // 0x00
    Legacy,
    // 0x01
    Eip2930,
    // 0x02
    Eip1559,
}

impl Serialize for TypedTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Legacy => serializer.serialize_str("0x00"),
            Self::Eip2930 => serializer.serialize_str("0x01"),
            Self::Eip1559 => serializer.serialize_str("0x02"),
        }
    }
}

impl<'de> Deserialize<'de> for TypedTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TypedVisitor;

        impl<'de> de::Visitor<'de> for TypedVisitor {
            type Value = TypedTransaction;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "Expect 0x00/0x01/0x02")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "0x00" => Ok(TypedTransaction::Legacy),
                    "0x01" => Ok(TypedTransaction::Eip2930),
                    "0x02" => Ok(TypedTransaction::Eip1559),
                    _ => Err(UtilsError::InvalidTypedTransaction(v.to_owned()))
                        .map_err(de::Error::custom),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(TypedVisitor {})
    }
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
    pub r#type: Option<TypedTransaction>,
    /// transaction nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Number>,
    /// To address
    pub to: Address,
    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<Number>,

    /// Transaction index in block
    #[serde(rename = "transactionIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction_index: Option<Number>,
    /// Block hash
    #[serde(rename = "blockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<BlockHash>,
    /// Block number
    #[serde(rename = "blockNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<Number>,
    /// Gas limit
    #[serde(rename = "gasPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<Number>,
    /// Transaction hash
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<AccessList>>,
    /// Chain ID tha this transaction is valid on
    #[serde(rename = "chainId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<Number>,
    /// The parity(0 for even, 1 for odd) of the y-value of the secp256k1 signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<Number>,
    /// r-value of the secp256k1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<Number>,
    /// s-value of the secp256k1
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl Transaction {
    const NUM_TX_FIELDS: usize = 9;
    /// Hashes the transaction's data with the provided chain id
    pub fn sighash(&self) -> TransactionHash {
        match self.chain_id {
            Some(_) => keccak256(self.rlp()).into(),
            None => keccak256(self.rlp_unsigned()).into(),
        }
    }

    /// Gets the transaction's RLP encoding, prepared with the chain_id and extra fields for
    /// signing. Assumes the chainid exists.
    pub fn rlp(&self) -> Vec<u8> {
        let mut rlp = RlpStream::new();

        if let Some(chain_id) = &self.chain_id {
            rlp.begin_list(Self::NUM_TX_FIELDS);
            self.rlp_base(&mut rlp);

            rlp.append(chain_id);
            rlp.append(&0u8);
            rlp.append(&0u8);
        } else {
            rlp.begin_list(Self::NUM_TX_FIELDS - 3);
            self.rlp_base(&mut rlp);
        }
        rlp.out().freeze().into()
    }

    /// Gets the unsigned transaction's RLP encoding
    pub fn rlp_unsigned(&self) -> Vec<u8> {
        let mut rlp = RlpStream::new();
        // rlp.begin_list(Self::NUM_TX_FIELDS - 3);
        rlp.begin_unbounded_list();
        self.rlp_base(&mut rlp);
        rlp.finalize_unbounded_list();
        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed(&self, signature: &Signature) -> Bytecode {
        let mut rlp = RlpStream::new();

        rlp.begin_unbounded_list();

        self.rlp_base(&mut rlp);
        // append the signature
        rlp.append(&signature.v());

        rlp.append(&signature.r());

        rlp.append(&signature.s());

        rlp.finalize_unbounded_list();

        rlp.out().freeze().to_vec().into()
    }

    pub(crate) fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.gas_price);
        rlp_opt(rlp, &self.gas);

        rlp.append(&self.to);
        rlp.append(&self.value);
        rlp.append(&self.input);
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
