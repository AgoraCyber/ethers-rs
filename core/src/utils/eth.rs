mod hash;
pub use hash::*;

pub type BlockHash = Hash<32>;
pub type Sha3Hash = Hash<32>;
pub type Address = Hash<20>;
pub type MerkleHash = Hash<32>;
pub type TransactionHash = Hash<32>;
pub type BloomFilter = Hash<32>;
pub type TransactionsRoot = Hash<32>;
pub type ReceiptsRoot = Hash<32>;
pub type Difficulty = Hash<1>;
