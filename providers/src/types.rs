mod block;
pub use block::*;

mod block_num_tag;
pub use block_num_tag::*;

mod syncing;
pub use syncing::*;

#[cfg(test)]
mod tests {

    use ethers_utils_rs::eth::Number;
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
            BlockNumberOrTag::Number(Number([0x10, 0x01].to_vec())),
        );

        check_block_number_or_tag("earliest", BlockNumberOrTag::Tag(BlockTag::Earliest));

        check_block_number_or_tag("finalized", BlockNumberOrTag::Tag(BlockTag::Finalized));

        check_block_number_or_tag("safe", BlockNumberOrTag::Tag(BlockTag::Safe));

        check_block_number_or_tag("latest", BlockNumberOrTag::Tag(BlockTag::Latest));

        check_block_number_or_tag("pending", BlockNumberOrTag::Tag(BlockTag::Pending));
    }
}
