use super::{bytes_to_hex, hex_to_bytes};
use serde::{Deserialize, Serialize};

/// 32 bytes HexFixed
#[derive(Debug, Clone)]
pub struct Hex(pub Vec<u8>);

impl Default for Hex {
    fn default() -> Self {
        Self([0].to_vec())
    }
}

impl TryFrom<&str> for Hex {
    type Error = hex::FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = hex_to_bytes(value)?;

        Ok(Self(bytes))
    }
}

impl TryFrom<String> for Hex {
    type Error = hex::FromHexError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl Hex {
    /// Convert `HexFixed` instance to hex string.
    pub fn to_string(&self) -> String {
        bytes_to_hex(self.0.as_slice())
    }
}

impl Serialize for Hex {
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

    use super::Hex;

    pub struct HexVisitor;

    impl<'de> de::Visitor<'de> for HexVisitor {
        type Value = Hex;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("HexFixed ")
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
                Ok(Hex(v.try_into().map_err(serde::de::Error::custom)?))
            }
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() != 32 {
                Err(hex::FromHexError::InvalidStringLength).map_err(serde::de::Error::custom)
            } else {
                Ok(Hex(v
                    .as_slice()
                    .try_into()
                    .map_err(serde::de::Error::custom)?))
            }
        }

        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() != 32 {
                Err(hex::FromHexError::InvalidStringLength).map_err(serde::de::Error::custom)
            } else {
                Ok(Hex(v.try_into().map_err(serde::de::Error::custom)?))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = deserializer.deserialize_any(visitor::HexVisitor)?;

        hex.try_into().map_err(serde::de::Error::custom)
    }
}
