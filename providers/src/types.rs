use ethers_core_rs::utils::eth::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Block {
    /// Parent block hash
    parent_hash: BlockHash,
    /// Ommers hash
    sha3_uncles: Sha3Hash,
    /// Coinbase
    miner: Address,
    /// State root
    state_root: MerkleHash,
    /// Transactions root
    transactions_root: TransactionsRoot,
    /// Receipts root
    receipts_root: ReceiptsRoot,
    /// Bloom filter
    logs_bloom: BloomFilter,
    /// Difficulty
    difficulty: Difficulty,
}
