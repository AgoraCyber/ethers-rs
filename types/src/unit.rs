use std::{fmt::Display, str::FromStr};

use ethabi::ethereum_types::U256;
use num::{BigInt, BigRational, ToPrimitive};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error)]
pub enum UnitError {
    #[error("Parse ethereum unit from float string literal failed. {0}")]
    ParseFloatString(String),
}

pub trait EthereumUnit {
    fn decimals() -> usize;

    fn new(v: U256) -> Self;
}

macro_rules! def_eth_unit {
    ($name: ident, $decimals: literal) => {
        #[derive(Clone, Deserialize, Serialize)]
        pub struct $name(pub U256);

        impl EthereumUnit for $name {
            fn decimals() -> usize {
                $decimals
            }

            fn new(v: U256) -> Self {
                Self(v)
            }
        }

        impl $name {
            pub fn unit_to<T: EthereumUnit>(&self) -> T {
                T::new(self.0.clone())
            }
        }

        impl From<u128> for $name {
            fn from(v: u128) -> Self {
                $name(U256::from(v))
            }
        }

        impl From<u64> for $name {
            fn from(v: u64) -> Self {
                $name(U256::from(v))
            }
        }

        impl From<u32> for $name {
            fn from(v: u32) -> Self {
                $name(U256::from(v))
            }
        }

        impl From<u16> for $name {
            fn from(v: u16) -> Self {
                $name(U256::from(v))
            }
        }

        impl From<u8> for $name {
            fn from(v: u8) -> Self {
                $name(U256::from(v))
            }
        }

        impl FromStr for $name {
            type Err = anyhow::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let v: f64 = s.parse()?;

                let v =
                    BigRational::from_float(v).ok_or(UnitError::ParseFloatString(s.to_owned()))?;

                let exponent = BigInt::from(10u32).pow($name::decimals() as u32);

                let v = v * exponent;

                log::debug!("{:#x}", v);

                let v = v.floor();

                log::debug!("{:#x}", v);

                let v = v.to_integer();

                log::debug!("{:#x}", v);

                let (_, bytes) = v.to_bytes_be();

                Ok($name(U256::from_big_endian(&bytes)))
            }
        }

        impl<'a> TryFrom<&'a str> for $name {
            type Error = anyhow::Error;
            fn try_from(value: &'a str) -> Result<Self, Self::Error> {
                value.parse()
            }
        }

        impl From<$name> for String {
            fn from(v: $name) -> Self {
                let mut bytes = vec![0; 32];

                v.0.to_big_endian(&mut bytes);

                let v = BigInt::from_bytes_be(num::bigint::Sign::Plus, &bytes);

                let v = BigRational::from_integer(v);

                let v = v / BigInt::from(10u32).pow($name::decimals() as u32);

                format!(
                    "{}",
                    v.numer().to_f64().unwrap() / v.denom().to_f64().unwrap()
                )
            }
        }

        impl From<$name> for U256 {
            fn from(v: $name) -> Self {
                v.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s: String = String::from(self.clone());

                write!(f, "{}", s)
            }
        }
    };
}

def_eth_unit!(Wei, 0);
def_eth_unit!(Kwei, 3);
def_eth_unit!(Mwei, 6);
def_eth_unit!(Gwei, 9);
def_eth_unit!(Szabo, 12);
def_eth_unit!(Finney, 15);
def_eth_unit!(Ether, 18);

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_convert() {
        _ = pretty_env_logger::try_init();

        let v = "1.1".parse::<Gwei>().expect("Parse ether");

        let gwei = serde_json::to_string(&v).expect("");

        let v: Wei = v.unit_to();

        let wei = serde_json::to_string(&v).expect("");

        assert_eq!(gwei, wei);

        log::debug!("{}", gwei);

        // let expected = U256::from_dec_str("1100000000000000000").expect("Parse u256");
    }
}
