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

        let mut buff = vec![0x02u8];

        let mut append = s.finalize()?.into();

        buff.append(&mut append);

        Ok(buff.into())
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
            signature.v,
            signature.r,
            signature.s,
        )
            .serialize(&mut s)?;

        let mut buff = vec![0x02u8];

        let mut append = s.finalize()?.into();

        buff.append(&mut append);

        Ok(buff.into())
    }
}

#[cfg(test)]
mod tests {
    use ethers_primitives::Eip1559Signature;
    use serde_json::json;

    use crate::Eip1559TransactionRequest;

    #[test]
    fn test_rlp() {
        let tx = json!({
          "maxPriorityFeePerGas": "0x0",
          "maxFeePerGas": "0x0",
          "gas": "0x0",
          "nonce": "0x0",
          "to": null,
          "value": "0x0",
          "chainId": "0x1",
          "type": "0x02",
          "data": "0x00",
          "accessList": [
            {
              "address": "0x0000000000000000000000000000000000000000",
              "storageKeys": [
                "0x0000000000000000000000000000000000000000000000000000000000000000"
              ]
            },
            {
              "address": "0x0000000000000000000000000000000000000000",
              "storageKeys": [
                "0x0000000000000000000000000000000000000000000000000000000000000000"
              ]
            }
          ]
        });

        let tx: Eip1559TransactionRequest = serde_json::from_value(tx).unwrap();

        assert_eq!(
            tx.rlp().unwrap().to_string(),
            "0x02f87a0180808080808000f870f7940000000000000000000000000000000000000000e1a00000000000000000000000000000000000000000000000000000000000000000f7940000000000000000000000000000000000000000e1a00000000000000000000000000000000000000000000000000000000000000000"
        );

        let sig: Eip1559Signature = "0x007a53fb20b46d9cc2600d8dc3168a698d41c0dec029d46db4ba88ffe359bbe4092536bd58c593edcda36c5f2e35ed4db158b0cab202b6b2648403117e483a9b30".parse().unwrap();

        assert_eq!(
            tx.rlp_signed(sig).unwrap().to_string(),
            "0x02f8bd0180808080808000f870f7940000000000000000000000000000000000000000e1a00000000000000000000000000000000000000000000000000000000000000000f7940000000000000000000000000000000000000000e1a0000000000000000000000000000000000000000000000000000000000000000080a07a53fb20b46d9cc2600d8dc3168a698d41c0dec029d46db4ba88ffe359bbe409a02536bd58c593edcda36c5f2e35ed4db158b0cab202b6b2648403117e483a9b30"
        );
    }
}
