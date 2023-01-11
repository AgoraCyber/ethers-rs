use crate::{hex_def, hex_fixed_def};

hex_fixed_def!(UncleHash, 32);
hex_fixed_def!(BlockHash, 32);
hex_fixed_def!(Sha3Hash, 32);
hex_fixed_def!(Address, 20);
hex_fixed_def!(MerkleHash, 32);
hex_fixed_def!(TransactionHash, 32);
hex_def!(BloomFilter);
hex_fixed_def!(TransactionsRoot, 32);
hex_fixed_def!(ReceiptsRoot, 32);
hex_def!(Difficulty);
hex_def!(Number);
hex_def!(ExtraData);
hex_def!(Bytecode);
hex_def!(Input);
hex_fixed_def!(MixHash, 32);
hex_fixed_def!(Nonce, 8);
hex_fixed_def!(Type, 1);
