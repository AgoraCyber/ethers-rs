use std::{
    fmt::Display,
    ops::{Add, Mul, Sub},
};

use hex::FromHexError;
use num::{bigint::ToBigInt, BigInt, FromPrimitive, Signed, ToPrimitive};
use serde::{de, Deserialize, Serialize};

use crate::{BytesVisitor, FromEtherHex, ToEtherHex};

use concat_idents::concat_idents;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignedError {
    #[error("OutOfRange: {0}")]
    OutOfRange(String),

    #[error("ToBigUnit: {0}")]
    ToBigUnit(String),
    #[error("FromHex: {0}")]
    FromHex(#[from] FromHexError),
}

/// unit<M> type mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Int<const BITS: usize>(pub [u8; 32]);

fn to_bytes32(value: BigInt, bits: usize) -> Result<[u8; 32], SignedError> {
    if value.bits() as usize > bits {
        return Err(SignedError::OutOfRange(format!(
            "{} convert to uint<{}> failed",
            value, bits
        )));
    }

    let bytes = value.to_signed_bytes_be();

    let mut buff = if value.is_negative() {
        [0xffu8; 32]
    } else {
        [0u8; 32]
    };

    buff[(32 - bytes.len())..].copy_from_slice(&bytes);

    Ok(buff)
}

impl<const BITS: usize> Int<BITS> {
    /// Create `Unit<BITS>` from [`ToBigInt`].
    /// Returns [`OutOfRange`](SignedError::OutOfRange) or [`ToBigUnit`](SignedError::ToBigUnit) if failed.
    pub fn new<N: ToBigInt + Signed>(value: N) -> Result<Self, SignedError> {
        if let Some(value) = value.to_bigint() {
            to_bytes32(value, BITS).map(|c| Self(c))
        } else {
            Err(SignedError::ToBigUnit(
                "convert input into BigUnit failed".to_owned(),
            ))
        }
    }
}

impl<const BITS: usize> Display for Int<BITS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lead_ones = self.0.iter().take_while(|c| **c == 0xff).count();
        let lead_zeros = self.0.iter().take_while(|c| **c == 0x00).count();

        if lead_ones > 0 {
            write!(f, "{}", (&self.0[(lead_ones - 1)..]).to_eth_value_hex())
        } else {
            write!(f, "{}", (&self.0[lead_zeros..]).to_eth_value_hex())
        }
    }
}

impl<const BITS: usize> TryFrom<&str> for Int<BITS> {
    type Error = SignedError;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        let value = Vec::<u8>::from_eth_hex(v)?;

        let lead_ones = value.iter().take_while(|c| **c == 0xff).count();

        if value.len() - lead_ones + 1 > BITS / 8 {
            return Err(SignedError::OutOfRange(format!(
                "{} convert to int<{}> failed",
                v, BITS
            )));
        }

        let mut buff = if lead_ones > 0 {
            [0xffu8; 32]
        } else {
            [0x0u8; 32]
        };

        buff[(32 - (value.len() - lead_ones))..].copy_from_slice(&value[lead_ones..]);

        Ok(Int(buff))
    }
}

impl<const BITS: usize> From<Int<BITS>> for BigInt {
    fn from(value: Int<BITS>) -> Self {
        BigInt::from(&value)
    }
}

impl<const BITS: usize> From<&Int<BITS>> for BigInt {
    fn from(value: &Int<BITS>) -> Self {
        BigInt::from_signed_bytes_be(&value.0[(32 - BITS / 8)..])
    }
}

impl<const BITS: usize> PartialOrd for Int<BITS> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        BigInt::from(self).partial_cmp(&BigInt::from(other))
    }
}

impl<const BITS: usize> Sub for Int<BITS> {
    type Output = Int<BITS>;

    fn sub(self, rhs: Self) -> Self::Output {
        // underflow will panic
        Self::new(BigInt::from(self) - BigInt::from(rhs)).unwrap()
    }
}

impl<const BITS: usize, N> Sub<N> for Int<BITS>
where
    N: ToBigInt,
{
    type Output = Int<BITS>;

    fn sub(self, rhs: N) -> Self::Output {
        // underflow will panic
        Self::new(BigInt::from(self) - rhs.to_bigint().unwrap()).unwrap()
    }
}

impl<const BITS: usize> Mul for Int<BITS> {
    type Output = Int<BITS>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(BigInt::from(self) + BigInt::from(rhs)).unwrap()
    }
}

impl<const BITS: usize, N> Mul<N> for Int<BITS>
where
    N: ToBigInt,
{
    type Output = Int<BITS>;

    fn mul(self, rhs: N) -> Self::Output {
        Self::new(BigInt::from(self) * rhs.to_bigint().unwrap()).unwrap()
    }
}

impl<const BITS: usize> Add for Int<BITS> {
    type Output = Int<BITS>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(BigInt::from(self) + BigInt::from(rhs)).unwrap()
    }
}

impl<const BITS: usize, N> Add<N> for Int<BITS>
where
    N: ToBigInt,
{
    type Output = Int<BITS>;

    fn add(self, rhs: N) -> Self::Output {
        Self::new(BigInt::from(self) + rhs.to_bigint().unwrap()).unwrap()
    }
}

impl<const BITS: usize> Serialize for Int<BITS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            // for rlp/eip712/abi serializers
            let name = format!("int{}", BITS);

            let static_name = unsafe { &*(&name as *const String) };

            serializer.serialize_newtype_struct(&static_name, &self.0)
        }
    }
}

struct IntVisitor<const BITS: usize>;

impl<'de, const BITS: usize> de::Visitor<'de> for IntVisitor<BITS> {
    type Value = Int<BITS>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expect string/number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Int::<BITS>::try_from(v).map_err(de::Error::custom)
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
        let value = BigInt::from_u128(v)
            .ok_or(SignedError::ToBigUnit(format!(
                "convert {} to BigInt failed",
                v
            )))
            .map_err(de::Error::custom)?;

        if value.bits() as usize > BITS {
            return Err(SignedError::OutOfRange(format!(
                "{} convert to uint<{}> failed",
                value, BITS
            )))
            .map_err(de::Error::custom);
        }

        let value = to_bytes32(value, BITS).map_err(de::Error::custom)?;

        Ok(Int(value))
    }
}

impl<'de, const BITS: usize> Deserialize<'de> for Int<BITS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(IntVisitor)
        } else {
            // for rlp/eip712/abi serializers
            let name = format!("int{}", BITS);

            let static_name = unsafe { &*(&name as *const String) };

            let value = deserializer
                .deserialize_newtype_struct(static_name.as_str(), BytesVisitor::default())?;

            let lead_ones = value.iter().take_while(|c| **c == 0xff).count();

            if value.len() - lead_ones + 1 > BITS / 8 {
                return Err(SignedError::OutOfRange(format!(
                    "{} convert to uint<{}> failed",
                    value.to_eth_hex(),
                    BITS
                )))
                .map_err(de::Error::custom);
            }

            let mut buff = if lead_ones > 0 {
                [0xffu8; 32]
            } else {
                [0x0u8; 32]
            };

            buff[(32 - (value.len() - lead_ones))..].copy_from_slice(&value[lead_ones..]);

            Ok(Int(buff))
        }
    }
}

/// impl from builin num
macro_rules! convert_builtin_signed {
    ($t: ident,$expr: tt,$($tails:tt),+) => {
        impl<const BITS: usize> From<$expr> for $t<BITS> {
            fn from(value: $expr) -> Self {
                let value: BigInt = value.to_bigint().expect("usize to bigint");
                // overflow panic
                Int::new(value).expect("Overflow")
            }
        }

        concat_idents!(to_fn=to_,$expr  {
            impl<const BITS: usize> From<$t<BITS>> for Option<$expr> {
                fn from(value: $t<BITS>) -> Self {
                    BigInt::from(value).to_fn()
                }
            }
        });



        convert_builtin_signed!($t,$($tails),*);
    };
    ($t: ident, $expr: tt) => {
        impl<const BITS: usize> From<$expr> for $t<BITS> {
            fn from(value: $expr) -> Self {
                let value: BigInt = value.to_bigint().expect("usize to biguint");
                // overflow panic
                Int::new(value).expect("Overflow")
            }
        }

        concat_idents!(to_fn=to_,$expr  {
            impl<const BITS: usize> From<$t<BITS>> for Option<$expr> {
                fn from(value: $t<BITS>) -> Self {
                    BigInt::from(value).to_fn()
                }
            }
        });
    };
}

convert_builtin_signed!(Int, isize, i128, i64, i32, i16, i8);

pub type I256 = Int<256>;
pub type I64 = Int<64>;

#[cfg(test)]
mod tests {
    use serde_ethabi::{from_abi, to_abi};
    use serde_rlp::rlp_encode;

    use crate::ToEtherHex;

    use super::*;

    #[test]
    fn test_arith() {
        if let Some(value) = Option::<i8>::from(I256::new(-1i8).unwrap()) {
            assert_eq!(value, -1i8);
        }

        if let Some(value) = Option::<i8>::from(I256::new(-4i8).unwrap()) {
            assert_eq!(value, -4i8);
        }

        let lhs = Int::<8>::new(-1i8).unwrap();
        let rhs = Int::<8>::new(-4i8).unwrap();

        assert_eq!(lhs > rhs, true);

        assert_eq!((rhs - lhs), Int::<8>::new(-3i8).unwrap());

        assert_eq!(
            BigInt::from(I256::new(-4isize).unwrap()),
            BigInt::from_i8(-4).unwrap()
        );
    }

    #[test]
    fn test_rlp() {
        _ = pretty_env_logger::try_init();

        assert_eq!(
            rlp_encode(&I256::from(0isize)).unwrap(),
            rlp_encode(&0usize).unwrap()
        );

        assert_eq!(
            rlp_encode(&I256::from(100000isize)).unwrap(),
            rlp_encode(&100000isize).unwrap()
        );
    }

    #[test]
    fn test_abi() {
        fn check<const BITS: usize>(value: Int<BITS>, expect: &str) {
            assert_eq!(to_abi(&value).unwrap().to_eth_hex(), expect);

            let buff = Vec::<u8>::from_eth_hex(expect).unwrap();

            let expect: Int<BITS> = from_abi(buff).unwrap();

            assert_eq!(value, expect);
        }

        check(
            I256::from(-69isize),
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffbb",
        );

        check(
            I256::from(-1isize),
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
    }
}
