#[macro_export]
macro_rules! hex_def {
    ($name:ident) => {
        /// 32 bytes $name
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(pub Vec<u8>);

        impl Default for $name {
            fn default() -> Self {
                Self([0].to_vec())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }

        impl From<Vec<u8>> for $name {
            fn from(value: Vec<u8>) -> Self {
                Self(value)
            }
        }

        impl From<&[u8]> for $name {
            fn from(value: &[u8]) -> Self {
                Self(value.to_owned())
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $crate::anyhow::Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                let bytes = $crate::hex::hex_to_bytes(value)?;

                Ok(Self(bytes))
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::anyhow::Error;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }

        impl<'a> AsRef<[u8]> for $name
        where
            Self: 'a,
        {
            fn as_ref(&self) -> &[u8] {
                self.0.as_slice()
            }
        }

        impl $crate::rlp::Encodable for $name {
            fn rlp_append(&self, s: &mut $crate::rlp::RlpStream) {
                s.append(&self.0.as_slice());
            }
        }

        impl $crate::rlp::Decodable for $name {
            fn decode(rlp: &$crate::rlp::Rlp) -> Result<Self, $crate::rlp::DecoderError> {
                rlp.decoder()
                    .decode_value(|bytes| Ok(bytes.to_vec().into()))
            }
        }

        impl $name {
            /// Convert `$name` instance to hex string.
            pub fn to_string(&self) -> String {
                $crate::hex::bytes_to_hex(self.0.as_slice())
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

                    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                        write!(formatter, "hex string for {}", stringify!($name))
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
                        Ok($name(v.to_vec()))
                    }

                    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok($name(v))
                    }

                    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok($name(v.to_vec()))
                    }
                }

                let hex = deserializer.deserialize_any(Visitor)?;

                hex.try_into().map_err(serde::de::Error::custom)
            }
        }
    };
}
