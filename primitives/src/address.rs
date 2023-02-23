//! Rust type for ethereum account address with builtin eip55 support
//!
//!

use std::fmt::Display;

use hex::FromHexError;
#[cfg(feature = "rust_crypto")]
use k256::elliptic_curve::sec1::ToEncodedPoint;
#[cfg(feature = "rust_crypto")]
use k256::PublicKey;
#[cfg(feature = "rust_crypto")]
use k256::SecretKey;

use serde::Deserialize;
use serde::Serialize;

use sha3::Digest;
use sha3::Keccak256;

use crate::hex::FromEtherHex;
use crate::hex::ToEtherHex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddressError {
    #[error("Invalid address string length,{0}")]
    Length(String),

    #[error("Eip155 format check failed,{0}")]
    Eip155(String),

    #[error("{0}")]
    FromHexError(#[from] FromHexError),

    #[cfg(feature = "rust_crypto")]
    #[error("{0}")]
    EllipticCurve(#[from] k256::elliptic_curve::Error),
}

/// Ethereum address type in binary bytes with format [`rlp`](crate::rlp) and format [`abi`](crate::abi) supports
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Address(
    /// Ethereum address's length is 20 in bytes
    pub [u8; 20],
);

impl Address {
    pub fn zero_address() -> Address {
        Address([0; 20])
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.to_checksum_string().serialize(serializer)
        } else {
            let mut buff = [0u8; 32];

            buff[12..].copy_from_slice(&self.0);

            serializer.serialize_newtype_struct("address", &buff)
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let data = String::deserialize(deserializer)?;

            Address::from_str(&data, false).map_err(serde::de::Error::custom)
        } else {
            use super::bytes::Bytes32;

            let bytes32 = Bytes32::deserialize(deserializer)?;

            let mut buff = [0u8; 20];

            buff.copy_from_slice(&bytes32.0[12..]);

            Ok(Self(buff))
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_checksum_string())
    }
}

impl TryFrom<&str> for Address {
    type Error = AddressError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Address::from_str(value, false)
    }
}

/// Eip55 support trait
pub trait Eip55: Sized {
    /// Convert address to eip55 string
    fn to_checksum_string(&self) -> String;

    /// Load address from string and make a eip55 checksum comparison
    fn from_str(source: &str, checksum: bool) -> Result<Self, AddressError>;
}

impl Eip55 for Address {
    fn to_checksum_string(&self) -> String {
        let mut data = self.0.to_eth_hex();

        let digest: [u8; 32] = Keccak256::new()
            .chain_update(&data.as_bytes()[2..])
            .finalize()
            .into();

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

    fn from_str(source: &str, checksum: bool) -> Result<Self, AddressError> {
        let buff = Vec::<u8>::from_eth_hex(source)?;

        if buff.len() != 20 {
            return Err(AddressError::Length(source.to_owned()).into());
        }

        let address = Self(buff.try_into().unwrap());

        if checksum {
            let expected = address.to_checksum_string();

            if expected != source {
                return Err(AddressError::Eip155(source.to_owned()));
            }
        }

        Ok(address)
    }
}

#[cfg(feature = "rust_crypto")]
impl From<PublicKey> for Address {
    fn from(value: PublicKey) -> Self {
        let buff = value.to_encoded_point(false);

        let digest: [u8; 32] = Keccak256::new()
            .chain_update(&buff.as_bytes()[1..])
            .finalize()
            .into();

        Self(digest[12..].try_into().unwrap())
    }
}

#[cfg(feature = "rust_crypto")]
impl From<SecretKey> for Address {
    fn from(value: SecretKey) -> Self {
        Address::from(&value)
    }
}

#[cfg(feature = "rust_crypto")]
impl From<&SecretKey> for Address {
    fn from(value: &SecretKey) -> Self {
        let value = value.public_key();
        let buff = value.to_encoded_point(false);

        let digest: [u8; 32] = Keccak256::new()
            .chain_update(&buff.as_bytes()[1..])
            .finalize()
            .into();

        Self(digest[12..].try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use k256::{PublicKey, SecretKey};
    use serde_ethabi::to_abi;

    use super::*;

    #[test]
    fn test_address() {
        let pk = Vec::<u8>::from_eth_hex(
            "0xbc7a746a0e8e299dea3be1f1d31fa9ba706a514e9d167b131fd3caa00d108881",
        )
        .unwrap();

        let pk = SecretKey::from_be_bytes(&pk).unwrap();

        let pub_key = Vec::<u8>::from_eth_hex(
            "0x023ffa007b44f4a635b69911f15b1af1dc4441b5bc4f1b197ab508f8e26f6fe784",
        )
        .unwrap();

        let pub_key = PublicKey::from_sec1_bytes(&pub_key).unwrap();

        assert_eq!(
            Address::from_str("0x8d57B06Cb8E7C8a0515C71B76B019EF4F3ed680d", true).unwrap(),
            Address::from(pub_key)
        );

        assert_eq!(
            Address::from_str("0x8d57B06Cb8E7C8a0515C71B76B019EF4F3ed680d", true).unwrap(),
            Address::from(pk)
        );
    }

    #[test]
    fn test_address_abi() {
        let address =
            Address::from_str("0x8d57B06Cb8E7C8a0515C71B76B019EF4F3ed680d", true).unwrap();

        let mut buff = [0u8; 32];

        buff[12..].copy_from_slice(&address.0);

        assert_eq!(to_abi(&address).unwrap(), buff,);
    }
}
