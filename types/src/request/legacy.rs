use ethabi::ethereum_types::{H256, U256, U64};
use ethers_hash_rs::keccak256;
use rlp::{Encodable, RlpStream};
use serde::{Deserialize, Serialize};

use super::rlp_opt;

use crate::block::Bytecode;
use crate::signature::SignatureVRS;
use crate::Address;
use crate::Signature;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct LegacyTransactionRequest {
    /// Transaction nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,
    /// Gas price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    /// Supplied gas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,
    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
    /// Transferred value
    pub value: Option<U256>,
    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytecode>,
    /// Chain id for EIP-155
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<U64>,
}

impl Encodable for LegacyTransactionRequest {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_unbounded_list();

        self.rlp_base(s);

        // EIP-155 suggest encoding fields
        // rlp_opt(s, &self.chain_id);
        s.append(&self.chain_id.unwrap());
        s.append(&0u8);
        s.append(&0u8);
        s.finalize_unbounded_list();
    }
}

impl LegacyTransactionRequest {
    /// Generate legacy transaction sign hash.
    pub fn sign_hash(&self) -> H256 {
        keccak256(self.rlp()).into()
    }

    pub fn rlp(&self) -> Bytecode {
        let mut s = rlp::RlpStream::new();

        self.rlp_append(&mut s);

        s.out().freeze().to_vec().into()
    }

    pub(crate) fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.gas_price);
        rlp_opt(rlp, &self.gas);

        rlp_opt(rlp, &self.to);
        rlp_opt(rlp, &self.value);
        rlp_opt(rlp, &self.data);

        // rlp.append(&self.to);
        // rlp.append(&self.value);
        // rlp.append(&self.input);
    }

    /// Returns signed tx rlp encoding stream.
    pub fn rlp_signed(&self, signature: Signature) -> Bytecode {
        let mut rlp = rlp::RlpStream::new();

        rlp.begin_unbounded_list();

        self.rlp_base(&mut rlp);

        // encode v,r,s
        let chain_id: u64 = self.chain_id.unwrap_or(Default::default()).as_u64();

        let v = signature.v() as u64 + 35 + chain_id * 2;

        rlp.append(&v);
        rlp.append(&signature.r());
        rlp.append(&signature.s());

        rlp.finalize_unbounded_list();

        rlp.out().freeze().to_vec().into()
    }
}
