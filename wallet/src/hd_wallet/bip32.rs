use std::io::Write;

use ethers_utils_rs::hash::pbkdf2::pbkdf2_hmac_array;
use hmac::{Hmac, Mac};
use k256::ecdsa::SigningKey;
use sha2::Sha512;
use thiserror::Error;

use super::bip44::{Bip44Error, Bip44Node, Bip44Path};

#[derive(Debug, Error)]
pub enum Bip32Error {
    #[error("Parse bip44 path error,{0}")]
    Bip44(Bip44Error),
}

fn mnemonic_to_send<M, P>(mnemonic: M, password: P) -> [u8; 64]
where
    M: AsRef<[u8]>,
    P: AsRef<[u8]>,
{
    let mut salt = "mnemonic".as_bytes().to_vec();

    salt.append(&mut password.as_ref().to_vec());

    pbkdf2_hmac_array::<Sha512, 64>(mnemonic.as_ref(), salt.as_ref(), 2048)
}

pub struct DriveKey {
    pub public_key: [u8; 33],
    pub private_key: [u8; 32],
    pub chain_code: [u8; 32],
    pub node: Option<Bip44Node>,
}

type HmacSha512 = Hmac<Sha512>;

impl DriveKey {
    pub fn new<M, P>(mnemonic: M, password: P) -> Self
    where
        M: AsRef<[u8]>,
        P: AsRef<[u8]>,
    {
        let seed = mnemonic_to_send(mnemonic, password);

        Self::new_master_key(seed)
    }

    pub(crate) fn new_master_key(seed: [u8; 64]) -> Self {
        let mut h = HmacSha512::new_from_slice(b"Bitcoin seed").expect("Create HmacSha512");

        h.write(&seed).expect("Write hmac data");

        let intermediary = h.finalize().into_bytes();

        let key_data = &intermediary[..32];
        let chain_code = &intermediary[32..];

        let key = SigningKey::from_bytes(key_data).expect("Create ecdsa signing key");

        let public_key = key.verifying_key().to_encoded_point(true);

        DriveKey {
            public_key: public_key
                .as_bytes()
                .try_into()
                .expect("Convert public key"),
            private_key: key_data.try_into().expect("Convert private key"),
            chain_code: chain_code.try_into().expect("Convert chain code"),
            node: None,
        }
    }

    pub(crate) fn child_key(&self, node: Bip44Node) -> Self {
        let mut h = HmacSha512::new_from_slice(&self.chain_code).expect("HmacSha512 new");

        match node {
            Bip44Node::Hardened(_) => {
                h.write(&[0]).expect("Hmac write");

                h.write(&self.private_key).expect("Hmac write");
            }
            Bip44Node::Normal(_) => {
                h.write(&self.public_key).expect("Hmac write");
            }
        }

        let index_bytes = (u64::from(&node) as u32).to_be_bytes();

        h.write(&index_bytes).expect("Hmac write");

        let intermediary = h.finalize().into_bytes();

        let key_data = &intermediary[..32];
        let chain_code = &intermediary[32..];

        let key = SigningKey::from_bytes(key_data).expect("Create ecdsa signing key");

        let scalar = key.as_nonzero_scalar().add(
            SigningKey::from_bytes(&self.private_key)
                .expect("Create ecdsa signing key")
                .as_nonzero_scalar(),
        );

        let key = SigningKey::from_bytes(&scalar.to_bytes()).expect("Create ecdsa signing key");

        let public_key = key.verifying_key().to_encoded_point(true);

        DriveKey {
            public_key: public_key
                .as_bytes()
                .try_into()
                .expect("Convert public key"),
            private_key: key.to_bytes().try_into().expect("Convert private key"),
            chain_code: chain_code.try_into().expect("Convert chain code"),
            node: Some(node),
        }
    }

    pub fn drive<P>(&self, path: P) -> Result<DriveKey, Bip32Error>
    where
        P: AsRef<str>,
    {
        let path: Bip44Path = path
            .as_ref()
            .parse()
            .map_err(|err| Bip32Error::Bip44(err))?;

        Ok(self
            .child_key(path.purpose)
            .child_key(path.coin)
            .child_key(path.account)
            .child_key(path.change)
            .child_key(path.address))
    }
}

#[cfg(test)]
mod tests {
    use ethers_types_rs::{Address, AddressEx, Eip55};
    use ethers_utils_rs::hex::bytes_to_hex;

    use super::{mnemonic_to_send, DriveKey};

    #[test]
    fn test_gen_seed() {
        let seed = mnemonic_to_send(
            "canal walnut regular license dust liberty story expect repeat design picture medal",
            "",
        );

        assert_eq!("0x15cba277c500b4e0c777d563278130f4c24b52532b3c8c45e051d417bebc5c007243c07d2e341a2d7c17bbd3880b968ca60869edab8f015be30674ad4d3d260f",bytes_to_hex(&seed));
    }

    #[test]
    fn test_hardhat_default_accounts() {
        let _ = pretty_env_logger::try_init();

        let drive_key = DriveKey::new(
            "test test test test test test test test test test test junk",
            "",
        );

        fn check_drive(drive_key: &DriveKey, id: usize, pk: &str, expect_address: &str) {
            let key = drive_key
                .drive(format!("m/44'/60'/0'/0/{}", id))
                .expect("Bip32 drive key");

            assert_eq!(bytes_to_hex(&key.private_key), pk);

            let address = Address::from_pub_key_compressed(key.public_key).expect("Parse address");

            assert_eq!(address.to_checksum_string(), expect_address);
        }

        check_drive(
            &drive_key,
            0,
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        );

        check_drive(
            &drive_key,
            19,
            "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e",
            "0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199",
        );
    }
}
