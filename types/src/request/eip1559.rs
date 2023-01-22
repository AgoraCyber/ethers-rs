use ethabi::ethereum_types::{Address, Signature, H256, U256};
use ethers_utils_rs::hash::keccak256;
use rlp::Encodable;
use serde::{Deserialize, Serialize};

use crate::{signature::SignatureVRS, Bytecode};

use super::{rlp_opt, AccessList};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Eip1559TransactionRequest {
    pub chain_id: U256,

    /// Transaction nonce
    pub nonce: U256,
    /// Gas price
    pub max_priority_fee_per_gas: U256,

    pub max_fee_per_gas: U256,
    /// Supplied gas
    pub gas: U256,
    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
    /// Transferred value
    pub value: Option<U256>,
    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Bytecode>,

    pub access_list: AccessList,
}

impl Encodable for Eip1559TransactionRequest {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_unbounded_list();

        self.rlp_base(s);

        s.finalize_unbounded_list();
    }
}

impl Eip1559TransactionRequest {
    /// Generate legacy transaction sign hash.
    pub fn sign_hash(&self) -> H256 {
        keccak256(self.rlp()).into()
    }
    fn to_bytecode(mut buff: Vec<u8>) -> Bytecode {
        buff.insert(0, 0x2);

        buff.into()
    }
    pub fn rlp(&self) -> Bytecode {
        let mut s = rlp::RlpStream::new();

        self.rlp_append(&mut s);

        Self::to_bytecode(s.out().freeze().to_vec())
    }

    fn rlp_base(&self, s: &mut rlp::RlpStream) {
        s.append(&self.chain_id);
        s.append(&self.nonce);
        s.append(&self.max_priority_fee_per_gas);
        s.append(&self.max_fee_per_gas);
        s.append(&self.gas);

        rlp_opt(s, &self.to);
        rlp_opt(s, &self.value);
        rlp_opt(s, &self.input);

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
