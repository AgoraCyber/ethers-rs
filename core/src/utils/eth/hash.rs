use serde::{Deserialize, Serialize};

use crate::utils::hex::{bytes_to_hex, hex_to_bytes};

/// 32 bytes hash
#[derive(Debug, Clone)]
pub struct Hash<const LEN: usize>(pub [u8; LEN]);

impl<const LEN: usize> Default for Hash<LEN> {
    fn default() -> Self {
        Self([0; LEN])
    }
}

impl<const LEN: usize> TryFrom<&str> for Hash<LEN> {
    type Error = hex::FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = hex_to_bytes(value)?;

        Ok(Self(
            bytes
                .try_into()
                .map_err(|_| hex::FromHexError::InvalidStringLength)?,
        ))
    }
}

impl<const LEN: usize> TryFrom<String> for Hash<LEN> {
    type Error = hex::FromHexError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl<const LEN: usize> Hash<LEN> {
    /// Convert `Hash` instance to hex string.
    pub fn to_string(&self) -> String {
        bytes_to_hex(self.0.as_slice())
    }
}

impl<const LEN: usize> Serialize for Hash<LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

mod visitor {
    use std::fmt::Formatter;

    use serde::de;

    use super::Hash;

    pub struct HashVisitor<const LEN: usize>;

    impl<'de, const LEN: usize> de::Visitor<'de> for HashVisitor<LEN> {
        type Value = Hash<LEN>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("Hash ")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.try_into().map_err(serde::de::Error::custom)?)
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.try_into().map_err(serde::de::Error::custom)?)
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() != 32 {
                Err(hex::FromHexError::InvalidStringLength).map_err(serde::de::Error::custom)
            } else {
                Ok(Hash(v.try_into().map_err(serde::de::Error::custom)?))
            }
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() != 32 {
                Err(hex::FromHexError::InvalidStringLength).map_err(serde::de::Error::custom)
            } else {
                Ok(Hash(
                    v.as_slice().try_into().map_err(serde::de::Error::custom)?,
                ))
            }
        }

        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() != 32 {
                Err(hex::FromHexError::InvalidStringLength).map_err(serde::de::Error::custom)
            } else {
                Ok(Hash(v.try_into().map_err(serde::de::Error::custom)?))
            }
        }
    }
}

impl<'de, const LEN: usize> Deserialize<'de> for Hash<LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = deserializer.deserialize_any(visitor::HashVisitor)?;

        hex.try_into().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Hash;

    #[test]
    pub fn test_hash_cast() {
        let block_hash: Hash<32> =
            "0x0bb3c2388383f714a8070dc6078a5edbe78f23c96646d4148d63cf964197ccc5"
                .try_into()
                .expect("Parse Hash error");

        assert_eq!(
            block_hash.to_string(),
            "0x0bb3c2388383f714a8070dc6078a5edbe78f23c96646d4148d63cf964197ccc5"
        );
    }
}
