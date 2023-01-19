#[macro_export]
macro_rules! hex_fixed_def {
    ($name:ident,$len:literal) => {
        /// 32 bytes $name
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(pub [u8; $len]);

        impl Default for $name {
            fn default() -> Self {
                Self([0; $len])
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $crate::anyhow::Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                let bytes = $crate::hex::hex_to_bytes(value)?;

                if bytes.len() != $len {
                    return Err(hex::FromHexError::InvalidStringLength.into());
                }

                Ok(Self(
                    bytes
                        .try_into()
                        .map_err(|_| hex::FromHexError::InvalidStringLength)?,
                ))
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::anyhow::Error;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }

        impl TryFrom<Vec<u8>> for $name {
            type Error = crate::error::UtilsError;
            fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
                Ok(Self(value.try_into().map_err(|_| {
                    crate::error::UtilsError::Hex(hex::FromHexError::InvalidStringLength)
                })?))
            }
        }

        impl<'a> TryFrom<&'a [u8]> for $name {
            type Error = crate::error::UtilsError;
            fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
                Ok(Self(value.try_into().map_err(|_| {
                    crate::error::UtilsError::Hex(hex::FromHexError::InvalidStringLength)
                })?))
            }
        }

        impl $name {
            /// Convert `$name` instance to hex string.
            pub fn to_string(&self) -> String {
                crate::hex::bytes_to_hex(self.0.as_slice())
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use std::fmt::Formatter;

                use serde::de;

                struct Visitor;

                impl<'de> de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                        write!(f, "fixed hex string for {}", stringify!($name))
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
                        if v.len() != $len {
                            Err(hex::FromHexError::InvalidStringLength)
                                .map_err(serde::de::Error::custom)
                        } else {
                            Ok($name(v.try_into().map_err(serde::de::Error::custom)?))
                        }
                    }

                    fn visit_none<E>(self) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok($name::default())
                    }

                    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        if v.len() != $len {
                            Err(hex::FromHexError::InvalidStringLength)
                                .map_err(serde::de::Error::custom)
                        } else {
                            Ok($name(
                                v.as_slice().try_into().map_err(serde::de::Error::custom)?,
                            ))
                        }
                    }

                    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        if v.len() != $len {
                            Err(hex::FromHexError::InvalidStringLength)
                                .map_err(serde::de::Error::custom)
                        } else {
                            Ok($name(v.try_into().map_err(serde::de::Error::custom)?))
                        }
                    }
                }

                let hex = deserializer.deserialize_any(Visitor)?;

                hex.try_into().map_err(serde::de::Error::custom)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Display};

    hex_fixed_def!(H1, 1);

    #[test]
    fn test_less_than() {
        let hex: H1 = "0x1".try_into().expect("Parse hex string error");

        assert_eq!(hex, H1([1]));

        call(hex);
    }

    fn call<H>(h: H)
    where
        H: TryInto<H1>,
        H::Error: Debug + Display,
    {
        assert_eq!(h.try_into().unwrap(), H1([1]));
    }
}
