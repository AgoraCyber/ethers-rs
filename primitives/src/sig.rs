use std::{fmt::Display, str::FromStr};

#[cfg(feature = "rust_crypto")]
use k256::ecdsa::{RecoveryId, Signature};
use num::BigUint;
use serde::{Deserialize, Serialize};

use crate::{FromEtherHex, ToEtherHex};

use super::U256;

#[derive(Debug, thiserror::Error)]
pub enum Eip1559SigError {
    #[error("{0}")]
    K256EcdsaSignature(#[from] k256::ecdsa::signature::Error),
    #[error("InvalidRecoveryId: {0}")]
    InvalidRecoveryId(u8),
}

/// Ethereum signature structure.
///
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Eip1559Signature {
    pub v: u8,
    pub r: U256,
    pub s: U256,
}

impl Display for Eip1559Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buff = [0u8; 65];

        buff[0] = self.v;

        buff[1..33].copy_from_slice(&self.r.0);
        buff[33..].copy_from_slice(&self.s.0);

        write!(f, "{}", buff.to_eth_hex())
    }
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

#[cfg(feature = "rust_crypto")]
impl TryFrom<Eip1559Signature> for (Signature, RecoveryId) {
    type Error = Eip1559SigError;
    fn try_from(sig: Eip1559Signature) -> Result<(Signature, RecoveryId), Self::Error> {
        let mut buff = [0u8; 64];

        buff[..32].copy_from_slice(&sig.r.0);
        buff[32..].copy_from_slice(&sig.s.0);

        let recover_id = sig.v;

        let sig = Signature::try_from(buff.as_slice())?;

        let recover_id = RecoveryId::from_byte(recover_id)
            .ok_or(Eip1559SigError::InvalidRecoveryId(recover_id))?;

        Ok((sig, recover_id))
    }
}

impl FromStr for Eip1559Signature {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let buff = Vec::<u8>::from_eth_hex(s)?;

        if buff.len() != 65 {
            return Err(anyhow::format_err!("signature length != 65"));
        }

        Ok(Self {
            v: buff[0],
            r: U256::new(BigUint::from_bytes_be(&buff[1..33])).unwrap(),
            s: U256::new(BigUint::from_bytes_be(&buff[33..])).unwrap(),
        })
    }
}
