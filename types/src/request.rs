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
pub enum TypedTransaction {
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

impl From<LegacyTransactionRequest> for TypedTransaction {
    fn from(tx: LegacyTransactionRequest) -> Self {
        TypedTransaction::Legacy(tx)
    }
}

impl From<Eip2930TransactionRequest> for TypedTransaction {
    fn from(tx: Eip2930TransactionRequest) -> Self {
        TypedTransaction::Eip2930(tx)
    }
}

impl From<Eip1559TransactionRequest> for TypedTransaction {
    fn from(tx: Eip1559TransactionRequest) -> Self {
        TypedTransaction::Eip1559(tx)
    }
}

impl TypedTransaction {
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

// #[cfg(test)]
// mod tests {

//     use ethers_utils_rs::hex::bytes_to_hex;
//     use ethers_wallet_rs::wallet::Wallet;
//     use serde_json::json;

//     use super::{Eip1559TransactionRequest, Eip2930TransactionRequest, LegacyTransactionRequest};

//     #[test]
//     fn test_rlp_legacy_tx() {
//         let wallet =
//             Wallet::new("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
//                 .expect("Create hardhat account 0 wallet");

//         let tx: LegacyTransactionRequest = json!({
//             "chainId":"0x1",
//             "nonce": "0x1",
//             "to": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
//             "value":"0x1",
//             "input":"0x",
//             "gas":"0x60000",
//             "gasPrice": "0x60000111"
//         })
//         .try_into()
//         .expect("Create tx");

//         let signature = wallet.sign(tx.sign_hash()).expect("Sign tx");

//         let raw_tx = bytes_to_hex(tx.rlp_signed(signature));

//         assert_eq!(
//             raw_tx,
//             "0xf864018460000111830600009470997970c51812dc3a010c7d01b50e0d17dc79c8018026a06c7e1e13070e6f10e51d7d20e986c59fd080fc6afc5508f44e8b0a84a58b7d1aa013c20fa2b6d77ae6814a41b674946387dde6401c73eb0cab2246a2981c48e344"
//         );
//     }

//     #[test]
//     fn test_eip155_encode() {
//         let tx: LegacyTransactionRequest = json!({
//             "chainId":"0x1",
//             "nonce": "0x9",
//             "to": "0x3535353535353535353535353535353535353535",
//             "value":"0xDE0B6B3A7640000",
//             "input":"0x",
//             "gas":"0x5208",
//             "gasPrice": "0x4A817C800"
//         })
//         .try_into()
//         .expect("Create tx");

//         assert_eq!(tx.rlp().to_string(),"0xec098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000080018080");
//     }

//     #[test]
//     fn test_eip2903_encode() {
//         _ = pretty_env_logger::try_init();
//         let tx: Eip2930TransactionRequest  = json!({
//           "input": "0x0672a1f01755187ba5e8c5c8c62d7dd30e1248d3525d2496eb371c32df1930f6fc33e1b2f5edd4de57e781a5195a8d7905c5f067bf7e9c13390ba0d1be0f0f32d278d1a2cad3d1c8d3724c41bfafc46717d2b2520867ee8feb7af32c6957d93808553ae5f252239c1fac67ca48cb3accdb401c8fa084355735fa89fee49fc6a2672b03d90184743ffe3f364799ee0ebb61f1237745ecd70dbe321ad1ab6e880f5c6aa296077b4733003d4b945ef0b6a301da68434e29d785ad37e9a129aed27bea53c21cbfdab5aecb68b1197334ed8754ed16d2d7431683072da60e745e4b60c1605e4df5aa8a968f9533748551e3b7204f521f36fecf8b624fbedfaa747142d9d6ac5d829ca292d2f2ef86766cd860548d199a431e908c71b4f5b652deaec467c5f2d1182fd32fde9e62fe340aabb924fc31b1de3da4711ccd27e9f63d06b13d8df7e94a58879c4361d5081fd1c61d966b5d14b4d82842e90035ee09df8ff345f9355590f53caeb392d3b1549b9560b68838969564b9c477be922a6c920512df4071ddeb8cd75016312dffa34d1c74656b77852f463546d375a112ebaea24de0b88083d96a8de0bd68eabf1572525259b345a42d3452a86b3c769af15609294ed0d14e34db14d0f4cde6e73a7eec7058bb473d54d090f558621bcaa9d2d6a607bf921c48291adb71a1a474d57d8bae58bde9ced11a1e57899c62f687a07029532c4110288886f2102fd842b0d66a6383103df0116df4471869ce489175da935dd32f77977a77aa8b5869d386c59c7b948282a7d5ffd18fa73ab547ce5db8e2d211fd05e217dc8c2cb2e20cf89075217fa26fd63dd9d4db5826e337f821b4a913b01af26b3e8b355c6c49791baf88a09bb3c23baf364617694c89756e3bb47659f14d67a3271dc220a9c76e36cf307479c37aee52ab8d2077a2de615c667db5bffdaacfce184c713502077dc5516b67c16daa8dc35c6861ad068cb093417990432cfe4539be09e9f166ad9f00233f8ce1c02ea5abc94fe6a6616668eae27676054c27d7dad029d12b89eb308f1bdce99d8f58672ec7fa7effedfd32be6e5742593b867262f26330fdeef29e210fb53040ac066e3bd53edb5abe9dbdf085f8d218d605b7700f2d8a29f876c65757f85ae543a358ce67be557819e40a59e5dd2ecae0cc5ffedec2cc319e5f28341c8f633ac543c4fd37996128ff6eb41dc0bce27121abbf8f65e5166f76145a7ca8540244e686dd7fa2104f2c2e5c81fa062024b53e56bc48c79964bd3d4fc6733212afdf24988f17ff083a43b8f09a8ee2aefb7cd25c9f8967096421f8aa952d8ffa8db268031dea25cba42aad04a98d1220857558d1d707a8cecec8add9df4d44f83d92b84393a01702fcfc5d89a8eee337d8f2c1965c0815a8aa4ad5b89630c29f07c1446df5caabfef0e539d458a0c4d0d0",
//           "gasPrice":"0x96D88891",
//           "nonce": "0xA7B0D",
//           "to": "0x7165d013987fd68ecc4b672314e250019dc487f6",
//           "value": "0x1FFFFFFFFFFFFF",
//           "chainId": "0x1",
//           "accessList": [
//             {
//               "address": "0x47cb9a27c50bce108942364e6d04ae25b90e0928",
//               "storageKeys": [
//                 "0x82b146500f164ea95af8239f97649abc5a761216a34ffe4a031e86a0284ef6d5",
//                 "0x2c6b1d11e089bbb19db030cf20780ff23abf22d1a17ad091139252488b13f9d7",
//                 "0xfc23a8020066958a1c88dd6b9b5abdbfd5d1144d7cacc46845c0ac6205425317",
//                 "0x7f2592bc59f9ef55d9f4b92f43c4d0d38e90386b7a5e961dbc0ab38f7eebe28e"
//               ]
//             },
//             {
//               "address": "0xc4eed2c25f92a411ade4565b2c798cb04411fb2b",
//               "storageKeys": [
//                 "0x88d053bb8d2ed6d00fae9bcb7119a2c8ce4a2b923589f947c95331de50ee915c",
//                 "0x1da0be68898bfb2ae70cc7c54092ff67eb434955a4705843ad8f32e26825ca08"
//               ]
//             },
//             {
//               "address": "0x10ce4236c1e2e2da270dc8c276b54f76d418097d",
//               "storageKeys": [
//                 "0xa1223e28b324306d48a410b4481af4527e34e726442733488d4726098784f6ff",
//                 "0xe622a5f3b5337506c6c0a659059e2e891312b276b7387d2249bf3e3c844d381f"
//               ]
//             },
//             {
//               "address": "0x59d79b32c0bc047d091d798e59947a825668098c",
//               "storageKeys": [
//                 "0xc07f11eb154186ecca71c85d0a3005032b3d976e77d063a5d81a1078609fd934",
//                 "0x53619da8665e72f0080f074a8a7e37f91cb589303d36ed84819db8cf15254254",
//                 "0x5fb098b793bf963cd28c144f03426f66aa7035edab7e03aa8b50014d6515bfb5"
//               ]
//             }
//           ],
//           "type": "0x01",
//           "gas": "0x924F97D3"
//         }).try_into().expect("Parse into eip2930 tx");

//         assert_eq!(
//             tx.rlp().to_string(),
//             "0x01f9060101830a7b0d8496d8889184924f97d3947165d013987fd68ecc4b672314e250019dc487f6871fffffffffffffb904000672a1f01755187ba5e8c5c8c62d7dd30e1248d3525d2496eb371c32df1930f6fc33e1b2f5edd4de57e781a5195a8d7905c5f067bf7e9c13390ba0d1be0f0f32d278d1a2cad3d1c8d3724c41bfafc46717d2b2520867ee8feb7af32c6957d93808553ae5f252239c1fac67ca48cb3accdb401c8fa084355735fa89fee49fc6a2672b03d90184743ffe3f364799ee0ebb61f1237745ecd70dbe321ad1ab6e880f5c6aa296077b4733003d4b945ef0b6a301da68434e29d785ad37e9a129aed27bea53c21cbfdab5aecb68b1197334ed8754ed16d2d7431683072da60e745e4b60c1605e4df5aa8a968f9533748551e3b7204f521f36fecf8b624fbedfaa747142d9d6ac5d829ca292d2f2ef86766cd860548d199a431e908c71b4f5b652deaec467c5f2d1182fd32fde9e62fe340aabb924fc31b1de3da4711ccd27e9f63d06b13d8df7e94a58879c4361d5081fd1c61d966b5d14b4d82842e90035ee09df8ff345f9355590f53caeb392d3b1549b9560b68838969564b9c477be922a6c920512df4071ddeb8cd75016312dffa34d1c74656b77852f463546d375a112ebaea24de0b88083d96a8de0bd68eabf1572525259b345a42d3452a86b3c769af15609294ed0d14e34db14d0f4cde6e73a7eec7058bb473d54d090f558621bcaa9d2d6a607bf921c48291adb71a1a474d57d8bae58bde9ced11a1e57899c62f687a07029532c4110288886f2102fd842b0d66a6383103df0116df4471869ce489175da935dd32f77977a77aa8b5869d386c59c7b948282a7d5ffd18fa73ab547ce5db8e2d211fd05e217dc8c2cb2e20cf89075217fa26fd63dd9d4db5826e337f821b4a913b01af26b3e8b355c6c49791baf88a09bb3c23baf364617694c89756e3bb47659f14d67a3271dc220a9c76e36cf307479c37aee52ab8d2077a2de615c667db5bffdaacfce184c713502077dc5516b67c16daa8dc35c6861ad068cb093417990432cfe4539be09e9f166ad9f00233f8ce1c02ea5abc94fe6a6616668eae27676054c27d7dad029d12b89eb308f1bdce99d8f58672ec7fa7effedfd32be6e5742593b867262f26330fdeef29e210fb53040ac066e3bd53edb5abe9dbdf085f8d218d605b7700f2d8a29f876c65757f85ae543a358ce67be557819e40a59e5dd2ecae0cc5ffedec2cc319e5f28341c8f633ac543c4fd37996128ff6eb41dc0bce27121abbf8f65e5166f76145a7ca8540244e686dd7fa2104f2c2e5c81fa062024b53e56bc48c79964bd3d4fc6733212afdf24988f17ff083a43b8f09a8ee2aefb7cd25c9f8967096421f8aa952d8ffa8db268031dea25cba42aad04a98d1220857558d1d707a8cecec8add9df4d44f83d92b84393a01702fcfc5d89a8eee337d8f2c1965c0815a8aa4ad5b89630c29f07c1446df5caabfef0e539d458a0c4d0d0f901cff89b9447cb9a27c50bce108942364e6d04ae25b90e0928f884a082b146500f164ea95af8239f97649abc5a761216a34ffe4a031e86a0284ef6d5a02c6b1d11e089bbb19db030cf20780ff23abf22d1a17ad091139252488b13f9d7a0fc23a8020066958a1c88dd6b9b5abdbfd5d1144d7cacc46845c0ac6205425317a07f2592bc59f9ef55d9f4b92f43c4d0d38e90386b7a5e961dbc0ab38f7eebe28ef85994c4eed2c25f92a411ade4565b2c798cb04411fb2bf842a088d053bb8d2ed6d00fae9bcb7119a2c8ce4a2b923589f947c95331de50ee915ca01da0be68898bfb2ae70cc7c54092ff67eb434955a4705843ad8f32e26825ca08f8599410ce4236c1e2e2da270dc8c276b54f76d418097df842a0a1223e28b324306d48a410b4481af4527e34e726442733488d4726098784f6ffa0e622a5f3b5337506c6c0a659059e2e891312b276b7387d2249bf3e3c844d381ff87a9459d79b32c0bc047d091d798e59947a825668098cf863a0c07f11eb154186ecca71c85d0a3005032b3d976e77d063a5d81a1078609fd934a053619da8665e72f0080f074a8a7e37f91cb589303d36ed84819db8cf15254254a05fb098b793bf963cd28c144f03426f66aa7035edab7e03aa8b50014d6515bfb5"
//         );

//         let wallet =
//             Wallet::new("0xa25f2b7436f87382fd91813aeea356cdb11d288dcc3a547bb2e39ce983de7f07")
//                 .expect("Create hardhat account 0 wallet");

//         let signature = wallet.sign(tx.sign_hash()).expect("Sign tx");

//         let tx_signed = tx.rlp_signed(signature);

//         assert_eq!(tx_signed.to_string(),"0x01f9064401830a7b0d8496d8889184924f97d3947165d013987fd68ecc4b672314e250019dc487f6871fffffffffffffb904000672a1f01755187ba5e8c5c8c62d7dd30e1248d3525d2496eb371c32df1930f6fc33e1b2f5edd4de57e781a5195a8d7905c5f067bf7e9c13390ba0d1be0f0f32d278d1a2cad3d1c8d3724c41bfafc46717d2b2520867ee8feb7af32c6957d93808553ae5f252239c1fac67ca48cb3accdb401c8fa084355735fa89fee49fc6a2672b03d90184743ffe3f364799ee0ebb61f1237745ecd70dbe321ad1ab6e880f5c6aa296077b4733003d4b945ef0b6a301da68434e29d785ad37e9a129aed27bea53c21cbfdab5aecb68b1197334ed8754ed16d2d7431683072da60e745e4b60c1605e4df5aa8a968f9533748551e3b7204f521f36fecf8b624fbedfaa747142d9d6ac5d829ca292d2f2ef86766cd860548d199a431e908c71b4f5b652deaec467c5f2d1182fd32fde9e62fe340aabb924fc31b1de3da4711ccd27e9f63d06b13d8df7e94a58879c4361d5081fd1c61d966b5d14b4d82842e90035ee09df8ff345f9355590f53caeb392d3b1549b9560b68838969564b9c477be922a6c920512df4071ddeb8cd75016312dffa34d1c74656b77852f463546d375a112ebaea24de0b88083d96a8de0bd68eabf1572525259b345a42d3452a86b3c769af15609294ed0d14e34db14d0f4cde6e73a7eec7058bb473d54d090f558621bcaa9d2d6a607bf921c48291adb71a1a474d57d8bae58bde9ced11a1e57899c62f687a07029532c4110288886f2102fd842b0d66a6383103df0116df4471869ce489175da935dd32f77977a77aa8b5869d386c59c7b948282a7d5ffd18fa73ab547ce5db8e2d211fd05e217dc8c2cb2e20cf89075217fa26fd63dd9d4db5826e337f821b4a913b01af26b3e8b355c6c49791baf88a09bb3c23baf364617694c89756e3bb47659f14d67a3271dc220a9c76e36cf307479c37aee52ab8d2077a2de615c667db5bffdaacfce184c713502077dc5516b67c16daa8dc35c6861ad068cb093417990432cfe4539be09e9f166ad9f00233f8ce1c02ea5abc94fe6a6616668eae27676054c27d7dad029d12b89eb308f1bdce99d8f58672ec7fa7effedfd32be6e5742593b867262f26330fdeef29e210fb53040ac066e3bd53edb5abe9dbdf085f8d218d605b7700f2d8a29f876c65757f85ae543a358ce67be557819e40a59e5dd2ecae0cc5ffedec2cc319e5f28341c8f633ac543c4fd37996128ff6eb41dc0bce27121abbf8f65e5166f76145a7ca8540244e686dd7fa2104f2c2e5c81fa062024b53e56bc48c79964bd3d4fc6733212afdf24988f17ff083a43b8f09a8ee2aefb7cd25c9f8967096421f8aa952d8ffa8db268031dea25cba42aad04a98d1220857558d1d707a8cecec8add9df4d44f83d92b84393a01702fcfc5d89a8eee337d8f2c1965c0815a8aa4ad5b89630c29f07c1446df5caabfef0e539d458a0c4d0d0f901cff89b9447cb9a27c50bce108942364e6d04ae25b90e0928f884a082b146500f164ea95af8239f97649abc5a761216a34ffe4a031e86a0284ef6d5a02c6b1d11e089bbb19db030cf20780ff23abf22d1a17ad091139252488b13f9d7a0fc23a8020066958a1c88dd6b9b5abdbfd5d1144d7cacc46845c0ac6205425317a07f2592bc59f9ef55d9f4b92f43c4d0d38e90386b7a5e961dbc0ab38f7eebe28ef85994c4eed2c25f92a411ade4565b2c798cb04411fb2bf842a088d053bb8d2ed6d00fae9bcb7119a2c8ce4a2b923589f947c95331de50ee915ca01da0be68898bfb2ae70cc7c54092ff67eb434955a4705843ad8f32e26825ca08f8599410ce4236c1e2e2da270dc8c276b54f76d418097df842a0a1223e28b324306d48a410b4481af4527e34e726442733488d4726098784f6ffa0e622a5f3b5337506c6c0a659059e2e891312b276b7387d2249bf3e3c844d381ff87a9459d79b32c0bc047d091d798e59947a825668098cf863a0c07f11eb154186ecca71c85d0a3005032b3d976e77d063a5d81a1078609fd934a053619da8665e72f0080f074a8a7e37f91cb589303d36ed84819db8cf15254254a05fb098b793bf963cd28c144f03426f66aa7035edab7e03aa8b50014d6515bfb580a06e9eb2f76b4ccc9374852061f37d1880a3354e8675b4178914107a179db909a6a013ecde698f7ddbfc0770f7681cb343db64b64624241acdeb314f18e2d5e47e02");
//     }

//     #[test]
//     fn test_eip1559_encode() {
//         _ = pretty_env_logger::try_init();

//         let tx: Eip1559TransactionRequest = json!({
//           "maxPriorityFeePerGas": "0x0",
//           "maxFeePerGas": "0x0",
//           "gas": "0x0",
//           "nonce": "0x0",
//           "to": null,
//           "value": "0x0",
//           "chainId": "0x1",
//           "type": "0x02",
//           "input": "0x00",
//           "accessList": [
//             {
//               "address": "0x0000000000000000000000000000000000000000",
//               "storageKeys": [
//                 "0x0000000000000000000000000000000000000000000000000000000000000000"
//               ]
//             },
//             {
//               "address": "0x0000000000000000000000000000000000000000",
//               "storageKeys": [
//                 "0x0000000000000000000000000000000000000000000000000000000000000000"
//               ]
//             }
//           ]
//         })
//         .try_into()
//         .expect("Create tx");

//         let wallet =
//             Wallet::new("0x0000000000000000000000000000000000000000000000000000000000000001")
//                 .expect("Create hardhat account 0 wallet");

//         let signature = wallet.sign(tx.sign_hash()).expect("Sign tx");

//         let tx_signed = tx.rlp_signed(signature);

//         assert_eq!(tx_signed.to_string(),"0x02f8bd0180808080808000f870f7940000000000000000000000000000000000000000e1a00000000000000000000000000000000000000000000000000000000000000000f7940000000000000000000000000000000000000000e1a0000000000000000000000000000000000000000000000000000000000000000080a07a53fb20b46d9cc2600d8dc3168a698d41c0dec029d46db4ba88ffe359bbe409a02536bd58c593edcda36c5f2e35ed4db158b0cab202b6b2648403117e483a9b30");
//     }
// }
