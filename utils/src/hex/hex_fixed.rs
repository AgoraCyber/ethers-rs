use crate::error::UtilsError;

use super::{bytes_to_hex, hex_to_bytes};
use serde::{Deserialize, Serialize};
/// 32 bytes HexFixed
#[derive(Debug, Clone, PartialEq)]
pub struct HexFixed<const LEN: usize>(pub [u8; LEN]);

impl<const LEN: usize> Default for HexFixed<LEN> {
    fn default() -> Self {
        Self([0; LEN])
    }
}

impl<const LEN: usize> TryFrom<&str> for HexFixed<LEN> {
    type Error = UtilsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = hex_to_bytes(value).map_err(|err| UtilsError::Hex(err))?;

        Ok(Self(bytes.try_into().map_err(|_| {
            UtilsError::Hex(hex::FromHexError::InvalidStringLength)
        })?))
    }
}

impl<const LEN: usize> TryFrom<String> for HexFixed<LEN> {
    type Error = UtilsError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl<const LEN: usize> HexFixed<LEN> {
    /// Convert `HexFixed` instance to hex string.
    pub fn to_string(&self) -> String {
        bytes_to_hex(self.0.as_slice())
    }
}

impl<const LEN: usize> Serialize for HexFixed<LEN> {
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

    use super::HexFixed;

    pub struct HexFixedVisitor<const LEN: usize>;

    impl<'de, const LEN: usize> de::Visitor<'de> for HexFixedVisitor<LEN> {
        type Value = HexFixed<LEN>;

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
                Ok(HexFixed(v.try_into().map_err(serde::de::Error::custom)?))
            }
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() != 32 {
                Err(hex::FromHexError::InvalidStringLength).map_err(serde::de::Error::custom)
            } else {
                Ok(HexFixed(
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
                Ok(HexFixed(v.try_into().map_err(serde::de::Error::custom)?))
            }
        }
    }
}

impl<'de, const LEN: usize> Deserialize<'de> for HexFixed<LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = deserializer.deserialize_any(visitor::HexFixedVisitor)?;

        hex.try_into().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Display};

    use super::HexFixed;

    #[test]
    fn test_less_than_len() {
        let hex: HexFixed<1> = "0x1".try_into().expect("Parse hex string error");

        assert_eq!(hex, HexFixed([1]));

        call(hex);
    }

    fn call<H>(h: H)
    where
        H: TryInto<HexFixed<1>>,
        H::Error: Debug + Display,
    {
        assert_eq!(h.try_into().unwrap(), HexFixed([1]));
    }
}
