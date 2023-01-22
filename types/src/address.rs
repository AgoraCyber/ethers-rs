use ethers_utils_rs::{hash::keccak256, hex::bytes_to_hex};
use hex::FromHexError;

use ethabi::ethereum_types::Address;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AddressError {
    #[error("Convert address from str error,{0}")]
    HexFormat(FromHexError),

    #[error("Convert bytes to compressed public key error, {0}")]
    CompressedPubKey(String),

    #[error("Address public key len either 33 or 65")]
    InvalidPubKeyLength,

    #[error("Address checksum mismatch")]
    AddressChecksum,
}

/// Extend `Address` structure, add some usefull helper fns.
pub trait AddressEx {
    /// Create address from public key.
    fn from_pub_key<K>(key: K) -> anyhow::Result<Address>
    where
        K: TryInto<[u8; 65]>,
        K::Error: std::error::Error + Send + Sync + 'static,
    {
        let key = key.try_into()?;

        let buf: [u8; 20] = keccak256(&key[1..])[12..]
            .try_into()
            .expect("To address array");

        Ok(buf.into())
    }

    /// Create address from compressed public key.
    #[cfg(feature = "rust_crypto")]
    fn from_pub_key_compressed<K>(key: K) -> anyhow::Result<Address>
    where
        K: TryInto<[u8; 33]>,
        K::Error: std::error::Error + Send + Sync + 'static,
    {
        let key = key.try_into()?;

        let key = k256::EncodedPoint::from_bytes(&key)
            .map_err(|err| AddressError::CompressedPubKey(err.to_string()))?;

        let key = k256::ecdsa::VerifyingKey::from_encoded_point(&key)?;

        Self::from_pub_key(key.to_encoded_point(false).as_bytes())
    }

    /// Create address from `AsRef<[u8]>`, and auto detecting public key type.
    fn from_any_pub_key<S>(key: S) -> anyhow::Result<Address>
    where
        S: AsRef<[u8]>,
    {
        let key = key.as_ref();

        match key.len() {
            33 => Self::from_pub_key_compressed(key),
            65 => Self::from_pub_key(key),
            _ => Err(AddressError::InvalidPubKeyLength.into()),
        }
    }

    /// Create address from private key
    #[cfg(feature = "rust_crypto")]
    fn from_private_key(key: &[u8]) -> anyhow::Result<Address> {
        let pk = k256::ecdsa::SigningKey::from_bytes(key)?;

        Self::from_pub_key(pk.verifying_key().to_encoded_point(false).as_bytes())
    }
}

impl AddressEx for Address {}

pub trait Eip55: Sized {
    fn to_checksum_string(&self) -> String;

    fn from_checksum_string(source: &str) -> anyhow::Result<Self>;
}

impl Eip55 for Address {
    fn to_checksum_string(&self) -> String {
        let mut data = bytes_to_hex(self.as_bytes());

        let digest = keccak256(&data.as_bytes()[2..]);

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

    fn from_checksum_string(source: &str) -> anyhow::Result<Self> {
        let address: Address = Self::try_from(source)?;

        let expected = address.to_checksum_string();

        if expected != source {
            return Err(AddressError::AddressChecksum.into());
        }

        Ok(address)
    }
}
