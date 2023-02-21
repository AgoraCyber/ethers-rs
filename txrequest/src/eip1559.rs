use ethers_primitives::*;

use serde::{Deserialize, Serialize};
use serde_rlp::RlpEncoder;

use super::{keccak256, AccessList, H256};

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub data: Option<Bytes>,

    pub access_list: AccessList,
}

impl Eip1559TransactionRequest {
    /// Generate legacy transaction sign hash.
    pub fn sign_hash(&self) -> anyhow::Result<H256> {
        Ok(keccak256(self.rlp()?.0).into())
    }

    pub fn rlp(&self) -> anyhow::Result<Bytes> {
        let mut s = RlpEncoder::default();
        (
            &self.chain_id,
            &self.nonce,
            &self.max_priority_fee_per_gas,
            &self.max_fee_per_gas,
            &self.gas,
            &self.to,
            &self.value,
            &self.data,
            &self.access_list,
        )
            .serialize(&mut s)?;

        Ok(s.finalize()?.into())
    }

    /// Returns signed tx rlp encoding stream.
    pub fn rlp_signed(&self, signature: Eip1559Signature) -> anyhow::Result<Bytes> {
        let mut s = RlpEncoder::default();

        (
            &self.chain_id,
            &self.nonce,
            &self.max_priority_fee_per_gas,
            &self.max_fee_per_gas,
            &self.gas,
            &self.to,
            &self.value,
            &self.data,
            &self.access_list,
            signature,
        )
            .serialize(&mut s)?;

        Ok(s.finalize()?.into())
    }
}
