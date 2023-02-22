use std::ops::Sub;

use num::{bigint::ToBigUint, BigUint, FromPrimitive};
use serde::{de, Deserialize, Serialize};

use crate::{BytesVisitor, FromEtherHex, ToEtherHex};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UintError {
    #[error("OutOfRange: {0}")]
    OutOfRange(String),

    #[error("ToBigUnit: {0}")]
    ToBigUnit(String),
}

/// unit<M> type mapping,a wrapper of [`BigUnit`](num::BigUint)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Uint<const BITS: usize>(pub BigUint);

impl<const BITS: usize> Uint<BITS> {
    /// Create `Unit<BITS>` from [`ToBigUint`].
    /// Returns [`OutOfRange`](UintError::OutOfRange) or [`ToBigUnit`](UintError::ToBigUnit) if failed.
    pub fn new<N: ToBigUint>(value: N) -> Result<Self, UintError> {
        if let Some(value) = value.to_biguint() {
            if value.bits() as usize > BITS {
                return Err(UintError::OutOfRange(format!(
                    "{} convert to uint<{}> failed",
                    value, BITS
                )));
            }

            Ok(Self(value))
        } else {
            Err(UintError::ToBigUnit(
                "convert input into BigUnit failed".to_owned(),
            ))
        }
    }
}

impl<const BITS: usize> From<Uint<BITS>> for BigUint {
    fn from(value: Uint<BITS>) -> Self {
        value.0
    }
}

impl<const BITS: usize> Sub for Uint<BITS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // underflow will panic
        Self::new(self.0 - rhs.0).unwrap()
    }
}

impl<const BITS: usize> Serialize for Uint<BITS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let buff = self.0.to_bytes_be();

            serializer.serialize_str(&buff.to_eth_hex())
        } else {
            // for rlp/eip712/abi serializers
            let name = format!("uint{}", BITS);

            let static_name = unsafe { &*(&name as *const String) };

            let bytes = self.0.to_bytes_be();

            assert!(bytes.len() * 8 <= BITS, "Always success !!!");

            let mut buff = [0u8; 32];

            // padding the left side with 0 until the length is 32 bytes
            buff[(32 - bytes.len())..].copy_from_slice(&bytes);

            serializer.serialize_newtype_struct(&static_name, &buff)
        }
    }
}

struct UintVisitor<const BITS: usize>;

impl<'de, const BITS: usize> de::Visitor<'de> for UintVisitor<BITS> {
    type Value = Uint<BITS>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expect string/number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = Vec::<u8>::from_eth_hex(v).map_err(de::Error::custom)?;

        let value = BigUint::from_bytes_be(&value);

        if value.bits() as usize > BITS {
            return Err(UintError::OutOfRange(format!(
                "{} convert to uint<{}> failed",
                value, BITS
            )))
            .map_err(de::Error::custom);
        }

        Ok(Uint(value))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u128(v as u128)
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = BigUint::from_u128(v)
            .ok_or(UintError::ToBigUnit(format!(
                "convert {} to BigUint failed",
                v
            )))
            .map_err(de::Error::custom)?;

        if value.bits() as usize > BITS {
            return Err(UintError::OutOfRange(format!(
                "{} convert to uint<{}> failed",
                value, BITS
            )))
            .map_err(de::Error::custom);
        }

        Ok(Uint(value))
    }
}

impl<'de, const BITS: usize> Deserialize<'de> for Uint<BITS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(UintVisitor)
        } else {
            // for rlp/eip712/abi serializers
            let name = format!("uint{}", BITS);

            let static_name = unsafe { &*(&name as *const String) };

            let buff = deserializer
                .deserialize_newtype_struct(static_name.as_str(), BytesVisitor::default())?;

            if buff.len() > 32 {
                return Err(UintError::OutOfRange(buff.to_eth_hex()))
                    .map_err(serde::de::Error::custom);
            }

            Ok(Self(BigUint::from_bytes_be(&buff)))
        }
    }
}

pub type U256 = Uint<256>;
pub type U64 = Uint<64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arith() {
        let lhs = Uint::<8>::new(1u8).unwrap();
        let rhs = Uint::<8>::new(4u8).unwrap();

        assert_eq!(lhs < rhs, true);

        assert_eq!((rhs - lhs), Uint::<8>::new(3u8).unwrap());
    }
}
