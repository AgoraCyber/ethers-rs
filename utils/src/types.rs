mod block;
pub use block::*;

mod block_num_tag;
pub use block_num_tag::*;

mod syncing;
pub use syncing::*;

mod accesslist;
pub use accesslist::*;

mod fee;
pub use fee::*;

mod filter;
pub use filter::*;

use crate::{error, hash::keccak256, hex::bytes_to_hex, hex_def, hex_fixed_def};

use crate::error::UtilsError;

// pub type UncleHash = ethabi::Hash;
// pub type Sha3Hash = ethabi::Hash;
// pub type BlockHash = ethabi::Hash;
pub type Address = ethabi::Address;
// pub type MerkleHash = ethabi::Hash;
// pub type TransactionHash = ethabi::Hash;
// pub type TransactionsRoot = ethabi::Hash;
// pub type ReceiptsRoot = ethabi::Hash;
// pub type MixHash = ethabi::Hash;
// pub type Number = ethabi::Uint;

hex_fixed_def!(UncleHash, 32);
hex_fixed_def!(BlockHash, 32);
hex_fixed_def!(Sha3Hash, 32);
// hex_fixed_def!(Address, 20);
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
hex_fixed_def!(Status, 1);
hex_fixed_def!(Topic, 32);
hex_fixed_def!(Signature, 65);

pub type Uint = ethabi::Uint;
pub type Int = ethabi::Int;
pub type Hash32 = ethabi::Hash;

pub trait AddressEx {
    fn from_pub_key(key: [u8; 65]) -> Address {
        let buf: [u8; 20] = keccak256(&key[1..])[12..]
            .try_into()
            .expect("To address array");

        buf.into()
    }

    #[cfg(feature = "rust_crypto")]
    fn from_pub_key_compressed(key: [u8; 33]) -> Result<Address, UtilsError> {
        let key = k256::EncodedPoint::from_bytes(&key)
            .map_err(|err| UtilsError::Address(format!("{}", err)))?;

        let key = k256::ecdsa::VerifyingKey::from_encoded_point(&key)
            .map_err(|err| UtilsError::Address(format!("{}", err)))?;

        Ok(Self::from_pub_key(
            key.to_encoded_point(false)
                .as_bytes()
                .try_into()
                .expect("Encode uncompress public key"),
        ))
    }

    fn from_pub_key_vec(key: Vec<u8>) -> Result<Address, UtilsError> {
        match key.len() {
            33 => Ok(Self::from_pub_key(key.try_into().expect(""))),
            65 => Ok(Self::from_pub_key(key.try_into().expect(""))),
            _ => Err(UtilsError::Address(
                "Address public key len either 33 or 65".to_owned(),
            )),
        }
    }
}

impl AddressEx for Address {}

pub trait Eip55: Sized {
    fn to_checksum_string(&self) -> String;

    fn from_checksum_string(source: &str) -> Result<Self, error::UtilsError>;
}

impl Eip55 for Address {
    fn to_checksum_string(&self) -> String {
        let mut data = bytes_to_hex(self.as_bytes());

        let digest = keccak256(&data.as_bytes()[2..]);

        let addr = unsafe { &mut data.as_bytes_mut()[2..] };

        for i in 0..addr.len() {
            let byte = digest[i / 2];
            let nibble = 0xf & if i % 2 == 0 { byte >> 4 } else { byte };
            if nibble >= 8 {
                addr[i] = addr[i].to_ascii_uppercase();
            }
        }

        data
    }

    fn from_checksum_string(source: &str) -> Result<Self, error::UtilsError> {
        let address: Address =
            Self::try_from(source).map_err(|err| error::UtilsError::Address(format!("{}", err)))?;

        let expected = address.to_checksum_string();

        if expected != source {
            return Err(error::UtilsError::Address(format!(
                "Expect address {}",
                expected
            )));
        }

        Ok(address)
    }
}

#[cfg(test)]
mod tests {

    use super::{Address, Eip55, Number};
    use serde::{Deserialize, Serialize};

    use crate::types::BlockTag;

    use super::{Block, BlockNumberOrTag, Transaction};

    #[test]
    fn test_address_checksum() {
        Address::from_checksum_string("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .expect("From checksum string");
    }

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
            ("empty_txs", include_str!("test-data/block/empty_txs.json")),
            ("block", include_str!("test-data/block/block.json")),
            ("hadhat", include_str!("test-data/block/hardhat.json")),
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
            BlockNumberOrTag::Number(Number([0x10, 0x01].to_vec())),
        );

        check_block_number_or_tag("earliest", BlockNumberOrTag::Tag(BlockTag::Earliest));

        check_block_number_or_tag("finalized", BlockNumberOrTag::Tag(BlockTag::Finalized));

        check_block_number_or_tag("safe", BlockNumberOrTag::Tag(BlockTag::Safe));

        check_block_number_or_tag("latest", BlockNumberOrTag::Tag(BlockTag::Latest));

        check_block_number_or_tag("pending", BlockNumberOrTag::Tag(BlockTag::Pending));
    }
}
