use ethabi::ethereum_types::H256;
use ethers_utils_rs::{
    hash::keccak256,
    hex_fixed_def,
    types::{Address, Bytecode, Signature, TransactionHash},
};
use rlp::Encodable;
use serde::{Deserialize, Serialize};

use super::LegacyTransactionRequest;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Eip2930TransactionRequest {
    #[serde(flatten)]
    pub tx: LegacyTransactionRequest,

    pub access_list: AccessList,
}

impl Encodable for Eip2930TransactionRequest {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_unbounded_list();

        self.rlp_base(s);

        s.finalize_unbounded_list();
    }
}

impl Eip2930TransactionRequest {
    /// Generate legacy transaction sign hash.
    pub fn sign_hash(&self) -> TransactionHash {
        keccak256(self.rlp()).into()
    }

    fn to_bytecode(mut buff: Vec<u8>) -> Bytecode {
        buff.insert(0, 0x1);

        buff.into()
    }
    pub fn rlp(&self) -> Bytecode {
        let mut s = rlp::RlpStream::new();

        self.rlp_append(&mut s);

        Self::to_bytecode(s.out().freeze().to_vec())
    }

    fn rlp_base(&self, s: &mut rlp::RlpStream) {
        s.append(&self.tx.chain_id);

        self.tx.rlp_base(s);

        s.append(&self.access_list);
    }

    /// Returns signed tx rlp encoding stream.
    pub fn rlp_signed(&self, signature: Signature) -> Bytecode {
        let mut rlp = rlp::RlpStream::new();

        rlp.begin_unbounded_list();

        self.rlp_base(&mut rlp);

        rlp.append(&signature.v());
        rlp.append(&signature.r());
        rlp.append(&signature.s());

        rlp.finalize_unbounded_list();

        Self::to_bytecode(rlp.out().freeze().to_vec())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AccessList(Vec<Access>);

impl Encodable for AccessList {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_unbounded_list();

        // s.append_list(&self.0);

        for v in &self.0 {
            s.append(v);
        }

        s.finalize_unbounded_list();
    }
}

hex_fixed_def!(StorageKey, 32);

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Access {
    pub address: Address,

    pub storage_keys: Vec<H256>,
}

impl Encodable for Access {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_unbounded_list();
        s.append(&self.address);

        {
            s.begin_unbounded_list();

            for k in &self.storage_keys {
                s.append(k);
            }

            s.finalize_unbounded_list();
        }

        s.finalize_unbounded_list();
    }
}

impl Encodable for StorageKey {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.append(&self.as_ref());
    }
}
