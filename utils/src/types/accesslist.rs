use crate::{rlp::rlp_opt, types::*};
use rlp::Encodable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessList {
    /// address that the transaction plans to access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    /// address storage keys that the transaction plans to access
    #[serde(rename = "storageKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_keys: Option<Vec<Sha3Hash>>,
}

impl Encodable for AccessList {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_unbounded_list();

        rlp_opt(s, &self.address);

        if let Some(_) = &self.storage_keys {
            // s.append_list(&keys);
        } else {
            s.begin_list(0);
        }

        s.finalize_unbounded_list();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessListResult {
    /// Access list created
    #[serde(rename = "accessList")]
    access_list: Option<AccessList>,
    /// Error message when created failed.
    error: Option<String>,
    /// Gas used
    #[serde(rename = "gasUsed")]
    gas_used: Number,
}
