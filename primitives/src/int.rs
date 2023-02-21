//! Primitive int/uint type support for the ethereum rpc/abi
//!

use std::fmt::Display;

use num::{PrimInt, Signed, Unsigned};
use serde::{de, Deserialize, Serialize};

use crate::{
    hex::{FromEtherHex, ToEtherHex},
    BytesVisitor,
};

#[derive(Debug, thiserror::Error)]
pub enum IntError {
    #[error("Deserialize int from other format, out of range")]
    OutOfRange,
    #[error("Invalid input int string,{0}")]
    InvalidHumanReadableInput(String),
}

///
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Int<const SIGNED: bool, const LENGTH: usize>(pub [u8; 32]);

impl<const SIGNED: bool, const LENGTH: usize> Int<SIGNED, LENGTH> {
    /// Create new Int type from bytes
    pub fn new(bytes: &[u8; LENGTH]) -> Self {
        assert!(LENGTH <= 32);

        let mut buf = [0u8; 32];

        buf[(32 - bytes.len())..].clone_from_slice(bytes);

        Self(buf)
    }
}

impl<const SIGNED: bool, const LENGTH: usize> Display for Int<SIGNED, LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_eth_hex())
    }
}

impl<const SIGNED: bool, const LENGTH: usize> Serialize for Int<SIGNED, LENGTH> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        assert!(LENGTH <= 32);

        if serializer.is_human_readable() {
            if LENGTH > 16 {
                serializer.serialize_str(&self.0.to_eth_hex())
            } else {
                let mut buff = [0u8; 16];
                buff.copy_from_slice(&self.0[16..]);

                if SIGNED {
                    serializer.serialize_i128(i128::from_be_bytes(buff))
                } else {
                    serializer.serialize_u128(u128::from_be_bytes(buff))
                }
            }
        } else {
            let name = if SIGNED {
                format!("int{}", LENGTH * 8)
            } else {
                format!("uint{}", LENGTH * 8)
            };

            let static_name = unsafe { &*(&name as *const String) };

            serializer.serialize_newtype_struct(static_name.as_str(), &self.0)
        }
    }
}

struct IntVisitor<const SIGNED: bool, const LENGTH: usize>;

impl<'de, const SIGNED: bool, const LENGTH: usize> de::Visitor<'de> for IntVisitor<SIGNED, LENGTH> {
    type Value = Int<SIGNED, LENGTH>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expect String/uint/int type")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = Vec::<u8>::from_eth_hex(v).map_err(de::Error::custom)?;

        if value.len() > 32 {
            return Err(IntError::OutOfRange).map_err(de::Error::custom)?;
        }

        let mut buff = [0u8; 32];

        buff.copy_from_slice(&value);

        Ok(Int::<SIGNED, LENGTH>(buff))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = v.to_be_bytes();

        let mut buff = if v.is_negative() {
            [0xffu8; 32]
        } else {
            [0u8; 32]
        };

        buff[16..].copy_from_slice(&value);

        Ok(Int::<SIGNED, LENGTH>(buff))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = v.to_be_bytes();

        let mut buff = [0u8; 32];

        buff[16..].copy_from_slice(&value);

        Ok(Int::<SIGNED, LENGTH>(buff))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u128(v as u128)
    }
}

impl<'de, const SIGNED: bool, const LENGTH: usize> Deserialize<'de> for Int<SIGNED, LENGTH> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        assert!(LENGTH <= 32);

        if deserializer.is_human_readable() {
            deserializer.deserialize_any(IntVisitor)
        } else {
            let name = if SIGNED {
                format!("int{}", LENGTH * 8)
            } else {
                format!("uint{}", LENGTH * 8)
            };

            let static_name = unsafe { &*(&name as *const String) };

            let buff = deserializer
                .deserialize_newtype_struct(static_name.as_str(), BytesVisitor::default())?;

            if buff.len() > 32 {
                return Err(IntError::OutOfRange).map_err(serde::de::Error::custom);
            }

            let mut temp = [0u8; 32];

            temp.copy_from_slice(&buff[..32]);

            Ok(Self(temp))
        }
    }
}

impl<N, const LENGTH: usize> From<N> for Int<true, LENGTH>
where
    N: Signed + PrimInt,
{
    fn from(value: N) -> Self {
        assert!(LENGTH <= 32);

        let bytes = value.to_i128().unwrap().to_be_bytes();

        let mut buf = if value.is_negative() {
            [0xffu8; 32]
        } else {
            [0u8; 32]
        };

        buf[(32 - bytes.len())..].clone_from_slice(&bytes);

        Self(buf)
    }
}

impl<N, const LENGTH: usize> From<N> for Int<false, LENGTH>
where
    N: Unsigned + PrimInt,
{
    fn from(value: N) -> Self {
        assert!(LENGTH / 8 <= 32 && LENGTH % 8 == 0);

        let bytes = value.to_i128().unwrap().to_be_bytes();

        let mut buf = [0u8; 32];

        buf[(32 - bytes.len())..].clone_from_slice(&bytes);

        Self(buf)
    }
}

pub type U256 = Int<false, 32>;
pub type U160 = Int<false, 20>;

#[cfg(test)]
mod tests {

    // use crate::{hex::ToEtherHex, rlp::rlp_encode};

    use serde_ethabi::{from_abi, to_abi};

    use super::*;

    #[test]
    fn test_abi() {
        // _ = pretty_env_logger::try_init();

        let v = Int::<true, 30>::from(1);

        assert_eq!(
            v.to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );

        let buff = to_abi(&v).expect("encode abi");

        let v1: Int<true, 30> = from_abi(buff).expect("decode abi");

        assert_eq!(v1, v);
    }

    // #[test]
    // fn test_rlp() {
    //     let v = U256::from(1usize);

    //     let rlp_data = rlp_encode(&v).expect("rlp encode").to_eth_hex();

    //     assert_eq!(rlp_data, "0x01");
    // }
}
