use ethabi::ethereum_types::{Signature, H256};
use serde::{Deserialize, Serialize};

fn rlp_opt<T: rlp::Encodable>(rlp: &mut rlp::RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

macro_rules! tx_json_support {
    ($ty:ident) => {
        impl TryFrom<&str> for $ty {
            type Error = serde_json::Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                serde_json::from_str(value)
            }
        }

        impl TryFrom<String> for $ty {
            type Error = serde_json::Error;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_ref())
            }
        }

        impl TryFrom<serde_json::Value> for $ty {
            type Error = serde_json::Error;
            fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
                serde_json::from_value(value)
            }
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NameOrAddress {
    Name(String),
    Address(crate::Address),
}

mod legacy;
pub use legacy::*;

mod eip2930;
pub use eip2930::*;

mod eip1559;
pub use eip1559::*;

use crate::Bytecode;

tx_json_support!(LegacyTransactionRequest);
tx_json_support!(Eip2930TransactionRequest);
tx_json_support!(Eip1559TransactionRequest);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum TypedTransactionRequest {
    // 0x00
    #[serde(rename = "0x00")]
    Legacy(LegacyTransactionRequest),
    // 0x01
    #[serde(rename = "0x01")]
    Eip2930(Eip2930TransactionRequest),
    // 0x02
    #[serde(rename = "0x02")]
    Eip1559(Eip1559TransactionRequest),
}

impl From<LegacyTransactionRequest> for TypedTransactionRequest {
    fn from(tx: LegacyTransactionRequest) -> Self {
        TypedTransactionRequest::Legacy(tx)
    }
}

impl From<Eip2930TransactionRequest> for TypedTransactionRequest {
    fn from(tx: Eip2930TransactionRequest) -> Self {
        TypedTransactionRequest::Eip2930(tx)
    }
}

impl From<Eip1559TransactionRequest> for TypedTransactionRequest {
    fn from(tx: Eip1559TransactionRequest) -> Self {
        TypedTransactionRequest::Eip1559(tx)
    }
}

impl TypedTransactionRequest {
    pub fn sign_hash(&self) -> H256 {
        match self {
            Self::Legacy(tx) => tx.sign_hash(),
            Self::Eip2930(tx) => tx.sign_hash(),
            Self::Eip1559(tx) => tx.sign_hash(),
        }
    }

    pub fn rlp_signed(&self, signature: Signature) -> Bytecode {
        match self {
            Self::Legacy(tx) => tx.rlp_signed(signature),
            Self::Eip2930(tx) => tx.rlp_signed(signature),
            Self::Eip1559(tx) => tx.rlp_signed(signature),
        }
    }
}
