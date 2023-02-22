//! ethereum hex string serialize/deserialize supporting,
//! generally ethereum hex string start with **"0x"**
//!
//! Also, this mod reexport types from crate [`hex`] .

pub use hex::*;

/// Implement this trait to support serialize `type` to ethereum hex string, "0x..."
pub trait ToEtherHex {
    fn to_eth_hex(&self) -> String;
}

impl<T: ToHex> ToEtherHex for T {
    fn to_eth_hex(&self) -> String {
        let hex_str = self.encode_hex::<String>();
        if hex_str == "00" {
            "0x0".to_owned()
        } else {
            format!("0x{}", hex_str)
        }
    }
}

/// Implement this trait to support deserialize `type` from ethereum hex string, "0x..."
pub trait FromEtherHex: Sized {
    type Error;
    fn from_eth_hex<T: AsRef<str>>(t: T) -> Result<Self, Self::Error>;
}

impl<T: FromHex> FromEtherHex for T {
    type Error = T::Error;

    fn from_eth_hex<S: AsRef<str>>(t: S) -> Result<Self, Self::Error> {
        let buff = t.as_ref().trim_start_matches("0x");
        if buff.len() % 2 != 0 {
            Self::from_hex(format!("0{}", buff))
        } else {
            Self::from_hex(buff)
        }
    }
}
