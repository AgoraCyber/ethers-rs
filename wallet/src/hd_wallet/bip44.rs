use std::str::FromStr;

use once_cell::sync::OnceCell;
use regex::Regex;

use thiserror::Error;

/// Path parsing regex
fn get_path_regex() -> &'static Regex {
    static REGEX: OnceCell<Regex> = OnceCell::new();

    REGEX.get_or_init(|| {
        Regex::new("^m/(\\d+'?)/(\\d+'?)/(\\d+'?)/([0,1])/(\\d+'?)$").expect("Compile path regex")
    })
}

#[derive(Debug, Error)]
pub enum Bip44Error {
    #[error("Invalid bip44 node string, {0}")]
    InvalidBip44NodeString(String),
    #[error("Invalid bip44 path, {0}")]
    InvalidPath(String),
}

static HARDENED_BIT: u64 = 0x80000000;

/// BIP44 path number instance
#[derive(Debug, PartialEq, Clone)]
pub enum Bip44Node {
    Normal(u64),
    Hardened(u64),
}

// impl Into<u64> for Bip44Node {
//     fn into(self) -> u64 {
//         match self {
//             Self::Normal(v) => v,
//             Self::Hardened(v) => v | HARDENED_BIT,
//         }
//     }
// }

impl From<Bip44Node> for u64 {
    fn from(node: Bip44Node) -> Self {
        match node {
            Bip44Node::Normal(v) => v,
            Bip44Node::Hardened(v) => v | HARDENED_BIT,
        }
    }
}

impl<'a> From<&'a Bip44Node> for u64 {
    fn from(node: &'a Bip44Node) -> Self {
        match node {
            Bip44Node::Normal(v) => *v,
            Bip44Node::Hardened(v) => v | HARDENED_BIT,
        }
    }
}

impl From<u64> for Bip44Node {
    fn from(value: u64) -> Self {
        if (value & HARDENED_BIT) == HARDENED_BIT {
            Bip44Node::Hardened(value | HARDENED_BIT)
        } else {
            Bip44Node::Normal(value)
        }
    }
}

impl FromStr for Bip44Node {
    type Err = Bip44Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hardened = false;

        let s = if s.ends_with("'") {
            hardened = true;
            s.trim_end_matches("'")
        } else {
            s
        };

        let value =
            u64::from_str(s).map_err(|_| Bip44Error::InvalidBip44NodeString(s.to_string()))?;

        if hardened {
            Ok(Self::Hardened(value))
        } else {
            Ok(Self::Normal(value))
        }
    }
}

// // Deterministic bip44 path
pub struct Bip44Path {
    pub purpose: Bip44Node,
    pub coin: Bip44Node,
    pub account: Bip44Node,
    pub change: Bip44Node,
    pub address: Bip44Node,
}

impl FromStr for Bip44Path {
    type Err = Bip44Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = get_path_regex().captures(value) {
            Ok(Bip44Path {
                purpose: captures.get(1).unwrap().as_str().parse()?,
                coin: captures.get(2).unwrap().as_str().parse()?,
                account: captures.get(3).unwrap().as_str().parse()?,
                change: captures.get(4).unwrap().as_str().parse()?,
                address: captures.get(5).unwrap().as_str().parse()?,
            })
        } else {
            Err(Bip44Error::InvalidPath(value.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bip44Path, HARDENED_BIT};

    #[test]
    fn test_parse_path() {
        let path: Bip44Path = "m/44'/60'/0'/0/1".parse().expect("eip44 path");

        assert_eq!(u64::from(&path.purpose), 44u64 | HARDENED_BIT);

        assert_eq!(u64::from(&path.coin), 60u64 | HARDENED_BIT);

        assert_eq!(u64::from(&path.account), 0u64 | HARDENED_BIT);

        assert_eq!(u64::from(&path.change), 0u64);

        assert_eq!(u64::from(&path.address), 1);
    }
}
