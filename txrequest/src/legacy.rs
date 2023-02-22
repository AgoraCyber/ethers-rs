use serde::{Deserialize, Serialize};

use ethers_primitives::*;

use serde_rlp::RlpEncoder;

use super::{keccak256, H256};

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
    pub data: Option<Bytes>,
    /// Chain id for EIP-155
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<U64>,
}

impl LegacyTransactionRequest {
    /// Generate legacy transaction sign hash.
    pub fn sign_hash(&self) -> anyhow::Result<H256> {
        Ok(keccak256(self.rlp()?.0).into())
    }

    pub fn rlp(&self) -> anyhow::Result<Bytes> {
        let mut s = RlpEncoder::default();

        (
            &self.nonce,
            &self.gas_price,
            &self.gas,
            &self.to,
            &self.value,
            &self.data,
        )
            .serialize(&mut s)?;

        Ok(s.finalize()?.into())
    }

    /// Returns signed tx rlp encoding stream.
    pub fn rlp_signed(&self, signature: Eip1559Signature) -> anyhow::Result<Bytes> {
        let mut rlp = RlpEncoder::default();

        // encode v,r,s
        let chain_id = self
            .chain_id
            .as_ref()
            .map(|c| c.0.clone())
            .unwrap_or(Default::default());

        let v: U64 = U64::new(signature.v + 35 + chain_id * 2usize).unwrap();

        (
            &self.nonce,
            &self.gas_price,
            &self.gas,
            &self.to,
            &self.value,
            &self.data,
            v,
            signature.r,
            signature.s,
        )
            .serialize(&mut rlp)?;

        Ok(rlp.finalize()?.into())
    }
}
