#[cfg(feature = "rust_crypto")]
use k256::ecdsa::{RecoveryId, Signature};
use num::BigUint;
use serde::{Deserialize, Serialize};

use super::U256;

/// Ethereum signature structure.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct Eip1559Signature {
    pub v: u8,
    pub r: U256,
    pub s: U256,
}

/// Convert tuple ([`Signature`], [`RecoveryId`]) to [`Eip1559Signature`]
#[cfg(feature = "rust_crypto")]
impl From<(Signature, RecoveryId)> for Eip1559Signature {
    fn from(value: (Signature, RecoveryId)) -> Self {
        Self {
            v: value.1.to_byte(),
            r: U256::new(BigUint::from_bytes_be(&value.0.r().to_bytes())).unwrap(),
            s: U256::new(BigUint::from_bytes_be(&value.0.s().to_bytes())).unwrap(),
        }
    }
}
