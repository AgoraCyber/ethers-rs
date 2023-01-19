use super::*;
use crate::error::UtilsError;
use serde::{Deserialize, Serialize};

/// eth_getBlockByNumber parameter `Block`
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum BlockNumberOrTag {
    Number(Number),
    Tag(BlockTag),
}

impl TryFrom<Number> for BlockNumberOrTag {
    type Error = UtilsError;
    fn try_from(value: Number) -> Result<Self, Self::Error> {
        Ok(BlockNumberOrTag::Number(value))
    }
}

impl TryFrom<BlockTag> for BlockNumberOrTag {
    type Error = UtilsError;
    fn try_from(tag: BlockTag) -> Result<Self, Self::Error> {
        Ok(BlockNumberOrTag::Tag(tag))
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
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with("0x") {
            Ok(BlockNumberOrTag::Number(value.try_into()?))
        } else {
            Ok(BlockNumberOrTag::Tag(
                serde_json::from_str(&format!("\"{}\"", value))
                    .map_err(|_| UtilsError::BlockTag(value.to_string()))?,
            ))
        }
    }
}
