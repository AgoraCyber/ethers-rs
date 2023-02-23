//! Contract abi bytes<M> and bytes type support. those types can aslo be used with eip715 or tx signature.

use std::fmt::Display;

use crate::hex::{FromEtherHex, ToEtherHex};

use hex::FromHexError;
// use concat_idents::concat_idents;
use serde::{de, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BytesErrors {
    #[error("Inputs data is out of bytes<M> range ")]
    BytesMOutOfRange,

    #[error("{0}")]
    FromHexError(#[from] FromHexError),
}

/// Type mapping for `bytes<M>` of contract abi
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct BytesM<const LEN: usize>(pub [u8; 32]);

impl<const LEN: usize> Display for BytesM<LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_eth_hex())
    }
}

impl<const LEN: usize> TryFrom<&str> for BytesM<LEN> {
    type Error = BytesErrors;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let buff = Vec::<u8>::from_eth_hex(value)?;

        if buff.len() > 32 {
            return Err(BytesErrors::BytesMOutOfRange);
        }

        let mut temp = [0u8; 32];

        temp[..LEN].copy_from_slice(&buff[..LEN]);

        Ok(Self(temp))
    }
}

impl<const LEN: usize> Serialize for BytesM<LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.0.to_eth_hex())
        } else {
            // AbiSerialize/EIp715Serializer use struct name to handle dispatch.
            let buff = &self.0;

            let name = format!("bytes{}", LEN);

            let static_name = unsafe { &*(&name as *const String) };

            serializer.serialize_newtype_struct(static_name.as_str(), buff)
        }
    }
}

impl<'de, const LEN: usize> Deserialize<'de> for BytesM<LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let buff = if deserializer.is_human_readable() {
            let data = String::deserialize(deserializer)?;

            let buff = Vec::<u8>::from_eth_hex(data).map_err(serde::de::Error::custom)?;

            if buff.len() > 32 {
                return Err(BytesErrors::BytesMOutOfRange).map_err(serde::de::Error::custom);
            }

            buff
        } else {
            let name = format!("bytes{}", LEN);

            let static_name = unsafe { &*(&name as *const String) };

            let buff = deserializer
                .deserialize_newtype_struct(static_name.as_str(), BytesVisitor::default())?;

            if buff.len() > 32 {
                return Err(BytesErrors::BytesMOutOfRange).map_err(serde::de::Error::custom);
            }

            buff
        };

        let mut temp = [0u8; 32];

        temp[..LEN].copy_from_slice(&buff[..LEN]);

        Ok(Self(temp))
    }
}

impl<const LEN: usize> From<[u8; LEN]> for BytesM<LEN> {
    fn from(value: [u8; LEN]) -> Self {
        Self::from(&value)
    }
}

impl<const LEN: usize> From<&[u8; LEN]> for BytesM<LEN> {
    fn from(value: &[u8; LEN]) -> Self {
        let mut buff = [0u8; 32];

        buff[..LEN].copy_from_slice(value);

        Self(buff)
    }
}

#[derive(Debug, Default)]
pub(crate) struct BytesVisitor;

impl<'de> de::Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expect bytes/bytes<M>")
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }
}

/// Type mapping for `bytes` of contract abi
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Bytes(pub Vec<u8>);

impl Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_eth_hex())
    }
}

impl TryFrom<&str> for Bytes {
    type Error = BytesErrors;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(Vec::<u8>::from_eth_hex(value)?))
    }
}

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.0.to_eth_hex())
        } else {
            serializer.serialize_newtype_struct("bytes", &self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let data = String::deserialize(deserializer)?;

            let buff = Vec::<u8>::from_eth_hex(data).map_err(serde::de::Error::custom)?;

            Ok(Self(buff))
        } else {
            let buff = deserializer.deserialize_newtype_struct("bytes", BytesVisitor::default())?;

            Ok(Self(buff))
        }
    }
}

impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl<const LEN: usize> From<&[u8; LEN]> for Bytes {
    fn from(value: &[u8; LEN]) -> Self {
        Self(value.to_vec())
    }
}

impl<const LEN: usize> From<[u8; LEN]> for Bytes {
    fn from(value: [u8; LEN]) -> Self {
        Self(value.to_vec())
    }
}

pub type Bytes1 = BytesM<1>;
pub type Bytes32 = BytesM<32>;

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use serde::Deserialize;
    use serde_ethabi::{from_abi, to_abi};

    use crate::hex::{FromEtherHex, ToEtherHex};

    use super::*;

    fn check<'de, V: Deserialize<'de> + Debug + PartialEq>(v: V, data: &str) {
        let data = Vec::<u8>::from_eth_hex(data).unwrap();

        let expected: V = from_abi(data).unwrap();

        assert_eq!(v, expected);
    }
    #[test]
    fn test_bytes() {
        assert_eq!(
            to_abi(#[allow(unused)]&([BytesM::from(b"abc"),BytesM::from(b"def")])).unwrap().to_eth_hex(),
            "0x61626300000000000000000000000000000000000000000000000000000000006465660000000000000000000000000000000000000000000000000000000000"
        );

        check(#[allow(unused)]([BytesM::from(b"abc"),BytesM::from(b"def")]), "0x61626300000000000000000000000000000000000000000000000000000000006465660000000000000000000000000000000000000000000000000000000000");

        assert_eq!(
            to_abi(&(Bytes::from(b"dave"),true,[1usize,2,3].as_slice())).unwrap().to_eth_hex(),
            "0x0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000464617665000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
        );

        check((Bytes::from(b"dave"),true,[1usize,2,3].to_vec()), "0x0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000464617665000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003");

        assert_eq!(
            to_abi(&(0x123usize,[0x456usize,0x789].as_slice(),BytesM::from(b"1234567890"),Bytes::from(b"Hello, world!"))).unwrap().to_eth_hex(),
            "0x00000000000000000000000000000000000000000000000000000000000001230000000000000000000000000000000000000000000000000000000000000080313233343536373839300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000004560000000000000000000000000000000000000000000000000000000000000789000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );

        check((0x123usize,[0x456usize,0x789].to_vec(),BytesM::from(b"1234567890"),Bytes::from(b"Hello, world!")),
            "0x00000000000000000000000000000000000000000000000000000000000001230000000000000000000000000000000000000000000000000000000000000080313233343536373839300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000004560000000000000000000000000000000000000000000000000000000000000789000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000");

        assert_eq!(
            to_abi(&(
                [[1usize, 2].as_slice(), [3usize].as_slice()].as_slice(),
                ["one", "two", "three"].as_slice()
            ))
            .unwrap()
            .to_eth_hex(),
            "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000036f6e650000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000374776f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000057468726565000000000000000000000000000000000000000000000000000000"
        );

        check((
                [[1usize, 2].to_vec(), [3usize].to_vec()].to_vec(),
                ["one".to_owned(), "two".to_owned(), "three".to_owned()].to_vec()
            ),"0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000036f6e650000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000374776f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000057468726565000000000000000000000000000000000000000000000000000000");
    }
}
