use serde::{Deserialize, Serialize};

use crate::AccessList;

use super::LegacyTransactionRequest;

use ethers_primitives::*;

use serde_ethrlp::RlpEncoder;

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

        let chain_id = self.tx.chain_id.unwrap_or(U64::new(1u8).unwrap());

        (
            &chain_id,
            &self.tx.nonce,
            &self.tx.gas_price,
            &self.tx.gas,
            &self.tx.to,
            &self.tx.value,
            &self.tx.data,
            &self.access_list,
        )
            .serialize(&mut s)?;

        let mut buff = vec![0x01u8];

        let mut append = s.finalize()?.into();

        buff.append(&mut append);

        Ok(buff.into())
    }

    /// Returns signed tx rlp encoding stream.
    pub fn rlp_signed(&self, signature: Eip1559Signature) -> anyhow::Result<Bytes> {
        let mut s = RlpEncoder::default();

        let chain_id = self.tx.chain_id.unwrap_or(U64::new(1u8).unwrap());

        (
            &chain_id,
            &self.tx.nonce,
            &self.tx.gas_price,
            &self.tx.gas,
            &self.tx.to,
            &self.tx.value,
            &self.tx.data,
            &self.access_list,
            signature.v,
            signature.r,
            signature.s,
        )
            .serialize(&mut s)?;

        let mut buff = vec![0x01u8];

        let mut append = s.finalize()?.into();

        buff.append(&mut append);

        Ok(buff.into())
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use crate::*;

    #[test]
    fn test_rlp_failed() {
        _ = pretty_env_logger::try_init();

        let tx = json!({
            "nonce": "0x3",
            "gasPrice": "0x1",
            "gas":"0x61A8",
            "to":"0xb94f5374fce5edbc8e2a8697c15331677e6ebf0b",
            "value":"0x0a",
            "data":"0x5544",
            "accessList":[]
        });

        let tx: Eip2930TransactionRequest = serde_json::from_value(tx).unwrap();

        assert_eq!(tx.tx.nonce, Some(U256::new(3usize).unwrap()));
        assert_eq!(tx.tx.gas_price, Some(U256::new(1usize).unwrap()));
        assert_eq!(tx.tx.gas, Some(U256::new(25000usize).unwrap()));
        assert_eq!(tx.tx.value, Some(U256::new(10usize).unwrap()));
        assert_eq!(tx.tx.data, Some(Bytes::from([0x55, 0x44])));
        assert_eq!(tx.access_list, AccessList(vec![]));

        let sign_hash = tx.sign_hash().unwrap();

        assert_eq!(
            sign_hash.to_string(),
            "0x49b486f0ec0a60dfbbca2d30cb07c9e8ffb2a2ff41f29a1ab6737475f6ff69f3"
        );
    }

    #[test]
    fn test_rlp() {
        _ = pretty_env_logger::try_init();

        let tx = json!({
          "data": "0x0672a1f01755187ba5e8c5c8c62d7dd30e1248d3525d2496eb371c32df1930f6fc33e1b2f5edd4de57e781a5195a8d7905c5f067bf7e9c13390ba0d1be0f0f32d278d1a2cad3d1c8d3724c41bfafc46717d2b2520867ee8feb7af32c6957d93808553ae5f252239c1fac67ca48cb3accdb401c8fa084355735fa89fee49fc6a2672b03d90184743ffe3f364799ee0ebb61f1237745ecd70dbe321ad1ab6e880f5c6aa296077b4733003d4b945ef0b6a301da68434e29d785ad37e9a129aed27bea53c21cbfdab5aecb68b1197334ed8754ed16d2d7431683072da60e745e4b60c1605e4df5aa8a968f9533748551e3b7204f521f36fecf8b624fbedfaa747142d9d6ac5d829ca292d2f2ef86766cd860548d199a431e908c71b4f5b652deaec467c5f2d1182fd32fde9e62fe340aabb924fc31b1de3da4711ccd27e9f63d06b13d8df7e94a58879c4361d5081fd1c61d966b5d14b4d82842e90035ee09df8ff345f9355590f53caeb392d3b1549b9560b68838969564b9c477be922a6c920512df4071ddeb8cd75016312dffa34d1c74656b77852f463546d375a112ebaea24de0b88083d96a8de0bd68eabf1572525259b345a42d3452a86b3c769af15609294ed0d14e34db14d0f4cde6e73a7eec7058bb473d54d090f558621bcaa9d2d6a607bf921c48291adb71a1a474d57d8bae58bde9ced11a1e57899c62f687a07029532c4110288886f2102fd842b0d66a6383103df0116df4471869ce489175da935dd32f77977a77aa8b5869d386c59c7b948282a7d5ffd18fa73ab547ce5db8e2d211fd05e217dc8c2cb2e20cf89075217fa26fd63dd9d4db5826e337f821b4a913b01af26b3e8b355c6c49791baf88a09bb3c23baf364617694c89756e3bb47659f14d67a3271dc220a9c76e36cf307479c37aee52ab8d2077a2de615c667db5bffdaacfce184c713502077dc5516b67c16daa8dc35c6861ad068cb093417990432cfe4539be09e9f166ad9f00233f8ce1c02ea5abc94fe6a6616668eae27676054c27d7dad029d12b89eb308f1bdce99d8f58672ec7fa7effedfd32be6e5742593b867262f26330fdeef29e210fb53040ac066e3bd53edb5abe9dbdf085f8d218d605b7700f2d8a29f876c65757f85ae543a358ce67be557819e40a59e5dd2ecae0cc5ffedec2cc319e5f28341c8f633ac543c4fd37996128ff6eb41dc0bce27121abbf8f65e5166f76145a7ca8540244e686dd7fa2104f2c2e5c81fa062024b53e56bc48c79964bd3d4fc6733212afdf24988f17ff083a43b8f09a8ee2aefb7cd25c9f8967096421f8aa952d8ffa8db268031dea25cba42aad04a98d1220857558d1d707a8cecec8add9df4d44f83d92b84393a01702fcfc5d89a8eee337d8f2c1965c0815a8aa4ad5b89630c29f07c1446df5caabfef0e539d458a0c4d0d0",
          "gasPrice":"0x96D88891",
          "nonce": "0xA7B0D",
          "to": "0x7165d013987fd68ecc4b672314e250019dc487f6",
          "value": "0x1FFFFFFFFFFFFF",
          "chainId": "0x1",
          "accessList": [
            {
              "address": "0x47cb9a27c50bce108942364e6d04ae25b90e0928",
              "storageKeys": [
                "0x82b146500f164ea95af8239f97649abc5a761216a34ffe4a031e86a0284ef6d5",
                "0x2c6b1d11e089bbb19db030cf20780ff23abf22d1a17ad091139252488b13f9d7",
                "0xfc23a8020066958a1c88dd6b9b5abdbfd5d1144d7cacc46845c0ac6205425317",
                "0x7f2592bc59f9ef55d9f4b92f43c4d0d38e90386b7a5e961dbc0ab38f7eebe28e"
              ]
            },
            {
              "address": "0xc4eed2c25f92a411ade4565b2c798cb04411fb2b",
              "storageKeys": [
                "0x88d053bb8d2ed6d00fae9bcb7119a2c8ce4a2b923589f947c95331de50ee915c",
                "0x1da0be68898bfb2ae70cc7c54092ff67eb434955a4705843ad8f32e26825ca08"
              ]
            },
            {
              "address": "0x10ce4236c1e2e2da270dc8c276b54f76d418097d",
              "storageKeys": [
                "0xa1223e28b324306d48a410b4481af4527e34e726442733488d4726098784f6ff",
                "0xe622a5f3b5337506c6c0a659059e2e891312b276b7387d2249bf3e3c844d381f"
              ]
            },
            {
              "address": "0x59d79b32c0bc047d091d798e59947a825668098c",
              "storageKeys": [
                "0xc07f11eb154186ecca71c85d0a3005032b3d976e77d063a5d81a1078609fd934",
                "0x53619da8665e72f0080f074a8a7e37f91cb589303d36ed84819db8cf15254254",
                "0x5fb098b793bf963cd28c144f03426f66aa7035edab7e03aa8b50014d6515bfb5"
              ]
            }
          ],
          "type": "0x01",
          "gas": "0x924F97D3"
        });

        let tx: Eip2930TransactionRequest = serde_json::from_value(tx).unwrap();

        assert_eq!(
            tx.rlp().unwrap().to_string(),
            "0x01f9060101830a7b0d8496d8889184924f97d3947165d013987fd68ecc4b672314e250019dc487f6871fffffffffffffb904000672a1f01755187ba5e8c5c8c62d7dd30e1248d3525d2496eb371c32df1930f6fc33e1b2f5edd4de57e781a5195a8d7905c5f067bf7e9c13390ba0d1be0f0f32d278d1a2cad3d1c8d3724c41bfafc46717d2b2520867ee8feb7af32c6957d93808553ae5f252239c1fac67ca48cb3accdb401c8fa084355735fa89fee49fc6a2672b03d90184743ffe3f364799ee0ebb61f1237745ecd70dbe321ad1ab6e880f5c6aa296077b4733003d4b945ef0b6a301da68434e29d785ad37e9a129aed27bea53c21cbfdab5aecb68b1197334ed8754ed16d2d7431683072da60e745e4b60c1605e4df5aa8a968f9533748551e3b7204f521f36fecf8b624fbedfaa747142d9d6ac5d829ca292d2f2ef86766cd860548d199a431e908c71b4f5b652deaec467c5f2d1182fd32fde9e62fe340aabb924fc31b1de3da4711ccd27e9f63d06b13d8df7e94a58879c4361d5081fd1c61d966b5d14b4d82842e90035ee09df8ff345f9355590f53caeb392d3b1549b9560b68838969564b9c477be922a6c920512df4071ddeb8cd75016312dffa34d1c74656b77852f463546d375a112ebaea24de0b88083d96a8de0bd68eabf1572525259b345a42d3452a86b3c769af15609294ed0d14e34db14d0f4cde6e73a7eec7058bb473d54d090f558621bcaa9d2d6a607bf921c48291adb71a1a474d57d8bae58bde9ced11a1e57899c62f687a07029532c4110288886f2102fd842b0d66a6383103df0116df4471869ce489175da935dd32f77977a77aa8b5869d386c59c7b948282a7d5ffd18fa73ab547ce5db8e2d211fd05e217dc8c2cb2e20cf89075217fa26fd63dd9d4db5826e337f821b4a913b01af26b3e8b355c6c49791baf88a09bb3c23baf364617694c89756e3bb47659f14d67a3271dc220a9c76e36cf307479c37aee52ab8d2077a2de615c667db5bffdaacfce184c713502077dc5516b67c16daa8dc35c6861ad068cb093417990432cfe4539be09e9f166ad9f00233f8ce1c02ea5abc94fe6a6616668eae27676054c27d7dad029d12b89eb308f1bdce99d8f58672ec7fa7effedfd32be6e5742593b867262f26330fdeef29e210fb53040ac066e3bd53edb5abe9dbdf085f8d218d605b7700f2d8a29f876c65757f85ae543a358ce67be557819e40a59e5dd2ecae0cc5ffedec2cc319e5f28341c8f633ac543c4fd37996128ff6eb41dc0bce27121abbf8f65e5166f76145a7ca8540244e686dd7fa2104f2c2e5c81fa062024b53e56bc48c79964bd3d4fc6733212afdf24988f17ff083a43b8f09a8ee2aefb7cd25c9f8967096421f8aa952d8ffa8db268031dea25cba42aad04a98d1220857558d1d707a8cecec8add9df4d44f83d92b84393a01702fcfc5d89a8eee337d8f2c1965c0815a8aa4ad5b89630c29f07c1446df5caabfef0e539d458a0c4d0d0f901cff89b9447cb9a27c50bce108942364e6d04ae25b90e0928f884a082b146500f164ea95af8239f97649abc5a761216a34ffe4a031e86a0284ef6d5a02c6b1d11e089bbb19db030cf20780ff23abf22d1a17ad091139252488b13f9d7a0fc23a8020066958a1c88dd6b9b5abdbfd5d1144d7cacc46845c0ac6205425317a07f2592bc59f9ef55d9f4b92f43c4d0d38e90386b7a5e961dbc0ab38f7eebe28ef85994c4eed2c25f92a411ade4565b2c798cb04411fb2bf842a088d053bb8d2ed6d00fae9bcb7119a2c8ce4a2b923589f947c95331de50ee915ca01da0be68898bfb2ae70cc7c54092ff67eb434955a4705843ad8f32e26825ca08f8599410ce4236c1e2e2da270dc8c276b54f76d418097df842a0a1223e28b324306d48a410b4481af4527e34e726442733488d4726098784f6ffa0e622a5f3b5337506c6c0a659059e2e891312b276b7387d2249bf3e3c844d381ff87a9459d79b32c0bc047d091d798e59947a825668098cf863a0c07f11eb154186ecca71c85d0a3005032b3d976e77d063a5d81a1078609fd934a053619da8665e72f0080f074a8a7e37f91cb589303d36ed84819db8cf15254254a05fb098b793bf963cd28c144f03426f66aa7035edab7e03aa8b50014d6515bfb5"
        );

        let sig: Eip1559Signature = "0x006e9eb2f76b4ccc9374852061f37d1880a3354e8675b4178914107a179db909a613ecde698f7ddbfc0770f7681cb343db64b64624241acdeb314f18e2d5e47e02".parse().unwrap();

        assert_eq!(tx.rlp_signed(sig).unwrap().to_string(), "0x01f9064401830a7b0d8496d8889184924f97d3947165d013987fd68ecc4b672314e250019dc487f6871fffffffffffffb904000672a1f01755187ba5e8c5c8c62d7dd30e1248d3525d2496eb371c32df1930f6fc33e1b2f5edd4de57e781a5195a8d7905c5f067bf7e9c13390ba0d1be0f0f32d278d1a2cad3d1c8d3724c41bfafc46717d2b2520867ee8feb7af32c6957d93808553ae5f252239c1fac67ca48cb3accdb401c8fa084355735fa89fee49fc6a2672b03d90184743ffe3f364799ee0ebb61f1237745ecd70dbe321ad1ab6e880f5c6aa296077b4733003d4b945ef0b6a301da68434e29d785ad37e9a129aed27bea53c21cbfdab5aecb68b1197334ed8754ed16d2d7431683072da60e745e4b60c1605e4df5aa8a968f9533748551e3b7204f521f36fecf8b624fbedfaa747142d9d6ac5d829ca292d2f2ef86766cd860548d199a431e908c71b4f5b652deaec467c5f2d1182fd32fde9e62fe340aabb924fc31b1de3da4711ccd27e9f63d06b13d8df7e94a58879c4361d5081fd1c61d966b5d14b4d82842e90035ee09df8ff345f9355590f53caeb392d3b1549b9560b68838969564b9c477be922a6c920512df4071ddeb8cd75016312dffa34d1c74656b77852f463546d375a112ebaea24de0b88083d96a8de0bd68eabf1572525259b345a42d3452a86b3c769af15609294ed0d14e34db14d0f4cde6e73a7eec7058bb473d54d090f558621bcaa9d2d6a607bf921c48291adb71a1a474d57d8bae58bde9ced11a1e57899c62f687a07029532c4110288886f2102fd842b0d66a6383103df0116df4471869ce489175da935dd32f77977a77aa8b5869d386c59c7b948282a7d5ffd18fa73ab547ce5db8e2d211fd05e217dc8c2cb2e20cf89075217fa26fd63dd9d4db5826e337f821b4a913b01af26b3e8b355c6c49791baf88a09bb3c23baf364617694c89756e3bb47659f14d67a3271dc220a9c76e36cf307479c37aee52ab8d2077a2de615c667db5bffdaacfce184c713502077dc5516b67c16daa8dc35c6861ad068cb093417990432cfe4539be09e9f166ad9f00233f8ce1c02ea5abc94fe6a6616668eae27676054c27d7dad029d12b89eb308f1bdce99d8f58672ec7fa7effedfd32be6e5742593b867262f26330fdeef29e210fb53040ac066e3bd53edb5abe9dbdf085f8d218d605b7700f2d8a29f876c65757f85ae543a358ce67be557819e40a59e5dd2ecae0cc5ffedec2cc319e5f28341c8f633ac543c4fd37996128ff6eb41dc0bce27121abbf8f65e5166f76145a7ca8540244e686dd7fa2104f2c2e5c81fa062024b53e56bc48c79964bd3d4fc6733212afdf24988f17ff083a43b8f09a8ee2aefb7cd25c9f8967096421f8aa952d8ffa8db268031dea25cba42aad04a98d1220857558d1d707a8cecec8add9df4d44f83d92b84393a01702fcfc5d89a8eee337d8f2c1965c0815a8aa4ad5b89630c29f07c1446df5caabfef0e539d458a0c4d0d0f901cff89b9447cb9a27c50bce108942364e6d04ae25b90e0928f884a082b146500f164ea95af8239f97649abc5a761216a34ffe4a031e86a0284ef6d5a02c6b1d11e089bbb19db030cf20780ff23abf22d1a17ad091139252488b13f9d7a0fc23a8020066958a1c88dd6b9b5abdbfd5d1144d7cacc46845c0ac6205425317a07f2592bc59f9ef55d9f4b92f43c4d0d38e90386b7a5e961dbc0ab38f7eebe28ef85994c4eed2c25f92a411ade4565b2c798cb04411fb2bf842a088d053bb8d2ed6d00fae9bcb7119a2c8ce4a2b923589f947c95331de50ee915ca01da0be68898bfb2ae70cc7c54092ff67eb434955a4705843ad8f32e26825ca08f8599410ce4236c1e2e2da270dc8c276b54f76d418097df842a0a1223e28b324306d48a410b4481af4527e34e726442733488d4726098784f6ffa0e622a5f3b5337506c6c0a659059e2e891312b276b7387d2249bf3e3c844d381ff87a9459d79b32c0bc047d091d798e59947a825668098cf863a0c07f11eb154186ecca71c85d0a3005032b3d976e77d063a5d81a1078609fd934a053619da8665e72f0080f074a8a7e37f91cb589303d36ed84819db8cf15254254a05fb098b793bf963cd28c144f03426f66aa7035edab7e03aa8b50014d6515bfb580a06e9eb2f76b4ccc9374852061f37d1880a3354e8675b4178914107a179db909a6a013ecde698f7ddbfc0770f7681cb343db64b64624241acdeb314f18e2d5e47e02");
    }
}
