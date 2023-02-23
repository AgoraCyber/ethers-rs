use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use serde_eip712::*;

use ethers_primitives::{Address, Bytes32, H256, U256};
use sha3::{Digest, Keccak256};

pub fn keccak256<S>(bytes: S) -> [u8; 32]
where
    S: AsRef<[u8]>,
{
    let mut hasher = Keccak256::new();

    hasher.update(bytes.as_ref());

    hasher.finalize().into()
}
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EIP712Domain {
    /// the user readable name of signing domain, i.e. the name of the DApp or the protocol.
    pub name: String,
    /// the current major version of the signing domain. Signatures from different versions are not compatible.
    pub version: String,
    /// the EIP-155 chain id. The user-agent should refuse signing if it does not match the currently active chain.
    pub chain_id: U256,
    /// the address of the contract that will verify the signature. The user-agent may do contract specific phishing prevention
    pub verifying_contract: Address,
    /// an disambiguating salt for the protocol. This can be used as a domain separator of last resort
    pub salt: Option<Bytes32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TypedData<M>
where
    M: Serialize,
{
    /// The custom types used by signing message.
    pub types: HashMap<String, TypeDefinition>,
    /// The type of the message.
    pub primary_type: String,
    /// Signing domain metadata. The signing domain is the intended context for the signature (e.g.
    /// the dapp, protocol, etc. that it's intended for). This data is used to construct the domain
    /// seperator of the message.
    pub domain: EIP712Domain,
    /// The message to be signed.
    pub message: M,
}

impl<M> TypedData<M>
where
    M: Serialize,
{
    pub fn sign_hash(&self) -> anyhow::Result<H256> {
        Ok(keccak256(self.encode()?).into())
    }

    pub fn encode(&self) -> anyhow::Result<[u8; 66]> {
        let domain = eip712_hash_struct("EIP712Domain", &self.types, &self.domain)?;

        let message = eip712_hash_struct(&self.primary_type, &self.types, &self.message)?;

        // "\x19\x01" ‖ domainSeparator ‖ hashStruct(message)
        let mut buff = [0u8; 66];

        buff[0..2].copy_from_slice(&[0x19, 0x01]);
        buff[2..34].copy_from_slice(&domain);
        buff[34..66].copy_from_slice(&message);

        Ok(buff)
    }
}
// /// encode(domainSeparator : 𝔹²⁵⁶, message : 𝕊)
// pub fn eip712_encode<S: Serialize>(domain: &EIP712Domain, value: &S) -> anyhow::Result<[u8; 66]> {
//     let type_definitions = eip712_type_definitions(&domain)?;
//     let domain = eip712_hash_struct("EIP712Domain".to_owned(), type_definitions, domain)?;

//     let encode_type = eip712_encode_type(&value)?;

//     let pos = encode_type.find("(").unwrap();

//     let primary_type = encode_type[..pos].to_string();

//     let type_definitions = eip712_type_definitions(&value)?;

//     let message = eip712_hash_struct(primary_type, type_definitions, value)?;

//     // "\x19\x01" ‖ domainSeparator ‖ hashStruct(message)
//     let mut buff = [0u8; 66];

//     buff[0..2].copy_from_slice(&[0x19, 0x01]);
//     buff[2..34].copy_from_slice(&domain);
//     buff[34..66].copy_from_slice(&message);

//     Ok(buff)
// }
/// Create eth_signTypedData payload.
pub fn eip712_into_request<S: Serialize>(
    domain: EIP712Domain,
    value: S,
) -> anyhow::Result<TypedData<S>> {
    // Get primary type.

    let encode_type = eip712_encode_type(&value)?;

    let pos = encode_type.find("(").unwrap();

    let primary_type = encode_type[..pos].to_string();

    let mut type_definitions = eip712_type_definitions(&domain)?;

    type_definitions.extend(eip712_type_definitions(&value)?.into_iter());

    Ok(TypedData {
        types: type_definitions,
        primary_type,
        domain,
        message: value,
    })
}
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use ethers_primitives::{Address, ToEtherHex};

    use super::*;

    #[test]
    fn test_mail() {
        _ = pretty_env_logger::try_init();

        let domain = json!({
            "name": "Ether Mail",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC",
        });

        let domain: EIP712Domain = serde_json::from_value(domain).unwrap();

        let type_definitions = eip712_type_definitions(&domain).unwrap();

        assert_eq!(
            eip712_hash_struct("EIP712Domain", &type_definitions, &domain)
                .unwrap()
                .to_eth_hex(),
            "0xf2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090f"
        );

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Person {
            pub name: String,
            pub wallet: Address,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Mail {
            pub from: Person,
            pub to: Person,
            pub contents: String,
        }

        let json = json!({
          "from": {
            "name": "Cow",
            "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
          },
          "to": {
            "name": "Bob",
            "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
          },
          "contents": "Hello, Bob!"
        });

        let mail: Mail = serde_json::from_value(json).expect("parse domain");

        assert_eq!(
            "Mail(Person from,Person to,string contents)Person(string name,address wallet)",
            eip712_encode_type(&mail).expect("generate e712 types")
        );

        let expect_request: TypedData<Mail> =
            serde_json::from_str(include_str!("./eip712.json")).unwrap();

        assert_eq!(eip712_into_request(domain, mail).unwrap(), expect_request);

        assert_eq!(
            expect_request.sign_hash().unwrap().to_eth_hex(),
            "0xbe609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2"
        );

        let expect_request: TypedData<serde_json::Value> =
            serde_json::from_str(include_str!("./eip712.json")).unwrap();

        assert_eq!(
            expect_request.sign_hash().unwrap().to_eth_hex(),
            "0xbe609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2"
        );

        expect_request.sign_hash().unwrap();
    }
}
