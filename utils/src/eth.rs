use super::hex::*;

pub type UncleHash = HexFixed<32>;
pub type BlockHash = HexFixed<32>;
pub type Sha3Hash = HexFixed<32>;
pub type Address = HexFixed<20>;
pub type MerkleHash = HexFixed<32>;
pub type TransactionHash = HexFixed<32>;
pub type BloomFilter = Hex;
pub type TransactionsRoot = HexFixed<32>;
pub type ReceiptsRoot = HexFixed<32>;
pub type Difficulty = Hex;
pub type Number = Hex;
pub type ExtraData = Hex;
pub type Input = Hex;
pub type MixHash = HexFixed<32>;
pub type Nonce = HexFixed<8>;
pub type Type = HexFixed<1>;
