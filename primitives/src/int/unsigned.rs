use std::ops::{Add, Mul, Sub};

use num::{bigint::ToBigUint, BigUint, FromPrimitive, ToPrimitive};
use serde::{de, Deserialize, Serialize};

use crate::{BytesVisitor, FromEtherHex, ToEtherHex};

use thiserror::Error;

use concat_idents::concat_idents;

#[derive(Debug, Error)]
pub enum UintError {
    #[error("OutOfRange: {0}")]
    OutOfRange(String),

    #[error("ToBigUnit: {0}")]
    ToBigUnit(String),
}

/// unit<M> type mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Default)]
pub struct Uint<const BITS: usize>(pub [u8; 32]);

fn to_bytes32(value: BigUint, bits: usize) -> Result<[u8; 32], UintError> {
    if value.bits() as usize > bits {
        return Err(UintError::OutOfRange(format!(
            "{} convert to uint<{}> failed",
            value, bits
        )));
    }

    let mut buff = [0u8; 32];

    let bytes = value.to_bytes_be();

    buff[(32 - bytes.len())..].copy_from_slice(&bytes);

    Ok(buff)
}

impl<const BITS: usize> Uint<BITS> {
    /// Create `Unit<BITS>` from [`ToBigUint`].
    /// Returns [`OutOfRange`](UintError::OutOfRange) or [`ToBigUnit`](UintError::ToBigUnit) if failed.
    pub fn new<N: ToBigUint>(value: N) -> Result<Self, UintError> {
        if let Some(value) = value.to_biguint() {
            to_bytes32(value, BITS).map(|c| Self(c))
        } else {
            Err(UintError::ToBigUnit(
                "convert input into BigUnit failed".to_owned(),
            ))
        }
    }
}

impl<const BITS: usize> From<Uint<BITS>> for BigUint {
    fn from(value: Uint<BITS>) -> Self {
        BigUint::from_bytes_be(&value.0)
    }
}

impl<const BITS: usize> Sub for Uint<BITS> {
    type Output = Uint<BITS>;

    fn sub(self, rhs: Self) -> Self::Output {
        // underflow will panic
        Self::new(BigUint::from(self) - BigUint::from(rhs)).unwrap()
    }
}

impl<const BITS: usize, N> Sub<N> for Uint<BITS>
where
    N: ToBigUint,
{
    type Output = Uint<BITS>;

    fn sub(self, rhs: N) -> Self::Output {
        // underflow will panic
        Self::new(BigUint::from(self) - rhs.to_biguint().unwrap()).unwrap()
    }
}

impl<const BITS: usize> Mul for Uint<BITS> {
    type Output = Uint<BITS>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(BigUint::from(self) + BigUint::from(rhs)).unwrap()
    }
}

impl<const BITS: usize, N> Mul<N> for Uint<BITS>
where
    N: ToBigUint,
{
    type Output = Uint<BITS>;

    fn mul(self, rhs: N) -> Self::Output {
        Self::new(BigUint::from(self) * rhs.to_biguint().unwrap()).unwrap()
    }
}

impl<const BITS: usize> Add for Uint<BITS> {
    type Output = Uint<BITS>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(BigUint::from(self) + BigUint::from(rhs)).unwrap()
    }
}

impl<const BITS: usize, N> Add<N> for Uint<BITS>
where
    N: ToBigUint,
{
    type Output = Uint<BITS>;

    fn add(self, rhs: N) -> Self::Output {
        Self::new(BigUint::from(self) + rhs.to_biguint().unwrap()).unwrap()
    }
}

impl<const BITS: usize> Serialize for Uint<BITS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let lead_zeros = self.0.iter().take_while(|c| **c == 0).count();

            serializer.serialize_str(&(&self.0[lead_zeros..]).to_eth_hex())
        } else {
            // for rlp/eip712/abi serializers
            let name = format!("uint{}", BITS);

            let static_name = unsafe { &*(&name as *const String) };

            serializer.serialize_newtype_struct(&static_name, &self.0)
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

        let value = to_bytes32(value, BITS).map_err(de::Error::custom)?;

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

        let value = to_bytes32(value, BITS).map_err(de::Error::custom)?;

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

            let value =
                to_bytes32(BigUint::from_bytes_be(&buff), BITS).map_err(de::Error::custom)?;

            Ok(Self(value))
        }
    }
}

pub type U256 = Uint<256>;
pub type U64 = Uint<64>;

/// impl from builin num
macro_rules! convert_builtin_unsigned {
    ($t: ident,$expr: tt,$($tails:tt),+) => {
        impl<const BITS: usize> From<$expr> for $t<BITS> {
            fn from(value: $expr) -> Self {
                let value: BigUint = value.to_biguint().expect("usize to biguint");
                // overflow panic
                Uint::new(value).expect("Overflow")
            }
        }

        concat_idents!(to_fn=to_,$expr  {
            impl<const BITS: usize> From<$t<BITS>> for Option<$expr> {
                fn from(value: $t<BITS>) -> Self {
                    BigUint::from(value).to_fn()
                }
            }
        });



        convert_builtin_unsigned!($t,$($tails),*);
    };
    ($t: ident, $expr: tt) => {
        impl<const BITS: usize> From<$expr> for $t<BITS> {
            fn from(value: $expr) -> Self {
                let value: BigUint = value.to_biguint().expect("usize to biguint");
                // overflow panic
                Uint::new(value).expect("Overflow")
            }
        }

        concat_idents!(to_fn=to_,$expr  {
            impl<const BITS: usize> From<$t<BITS>> for Option<$expr> {
                fn from(value: $t<BITS>) -> Self {
                    BigUint::from(value).to_fn()
                }
            }
        });
    };
}

convert_builtin_unsigned!(Uint, usize, u128, u64, u32, u16, u8);
