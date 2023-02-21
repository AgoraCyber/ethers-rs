use serde::{Deserialize, Serialize};

use super::LegacyTransactionRequest;

use ethers_primitives::*;

use serde_rlp::RlpEncoder;

use super::{keccak256, H256};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Eip2930TransactionRequest {
    #[serde(flatten)]
    pub tx: LegacyTransactionRequest,

    pub access_list: AccessList,
}

impl Eip2930TransactionRequest {
    /// Generate legacy transaction sign hash.
    pub fn sign_hash(&self) -> anyhow::Result<H256> {
        Ok(keccak256(self.rlp()?.0).into())
    }

    pub fn rlp(&self) -> anyhow::Result<Bytes> {
        let mut s = RlpEncoder::default();

        (
            0x1,
            &self.tx.chain_id,
            &self.tx.nonce,
            &self.tx.gas_price,
            &self.tx.gas,
            &self.tx.to,
            &self.tx.value,
            &self.tx.data,
            &self.access_list,
        )
            .serialize(&mut s)?;

        Ok(s.finalize()?.into())
    }

    /// Returns signed tx rlp encoding stream.
    pub fn rlp_signed(&self, signature: Eip1559Signature) -> anyhow::Result<Bytes> {
        let mut s = RlpEncoder::default();

        (
            &self.tx.chain_id,
            &self.tx.nonce,
            &self.tx.gas_price,
            &self.tx.gas,
            &self.tx.to,
            &self.tx.value,
            &self.tx.data,
            &self.access_list,
            signature,
        )
            .serialize(&mut s)?;

        Ok(s.finalize()?.into())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AccessList(Vec<Access>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Access {
    pub address: Address,

    pub storage_keys: Vec<H256>,
}
