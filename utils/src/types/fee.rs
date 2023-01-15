use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FeeHistory {
    /// The lowest number block of returned range.
    #[serde(rename = "oldestBlock")]
    oldest_block: Number,
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: Vec<Number>,

    reward: Vec<Vec<Number>>,
}
