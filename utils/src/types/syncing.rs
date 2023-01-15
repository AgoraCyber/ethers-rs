use super::*;
use crate::error::UtilsError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SyncingStatus {
    Syncing(Syncing),

    #[serde(deserialize_with = "from_bool", serialize_with = "as_bool")]
    False,
}

impl Default for SyncingStatus {
    fn default() -> Self {
        SyncingStatus::False
    }
}

fn from_bool<'de, D>(d: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    bool::deserialize(d).and_then(|flag| {
        if !flag {
            Ok(())
        } else {
            Err(UtilsError::Syncing).map_err(serde::de::Error::custom)
        }
    })
}

fn as_bool<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bool(false)
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Syncing {
    /// Starting block
    #[serde(rename = "startingBlock")]
    starting_block: Number,

    /// Current block
    #[serde(rename = "currentBlock")]
    current_block: Number,

    /// Highest block
    #[serde(rename = "highestBlock")]
    highest_block: Number,
}

#[cfg(test)]
mod tests {
    use jsonrpc_rs::Response;
    use serde_json::json;

    use super::*;

    #[test]
    fn dser_syncing() {
        let value = json!({
            "jsonrpc": "2.0",
            "result": false,
            "id": 0
        });

        let status: Response<String, SyncingStatus, ()> =
            serde_json::from_value(value).expect("Parse syncing false");

        assert_eq!(status.result, Some(SyncingStatus::False));

        let value = json!({
            "jsonrpc": "2.0",
            "result": {
                "startingBlock": "0x11",
                "currentBlock": "0x12",
                "highestBlock": "0x33",
            },
            "id": 0
        });

        let status: Response<String, SyncingStatus, ()> =
            serde_json::from_value(value).expect("Parse syncing status object");

        assert_eq!(
            status.result,
            Some(SyncingStatus::Syncing(Syncing {
                starting_block: Number(vec![0x11]),
                current_block: Number(vec![0x12]),
                highest_block: Number(vec![0x33]),
            }))
        );
    }
}
