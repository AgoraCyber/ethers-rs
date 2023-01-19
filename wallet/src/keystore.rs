use std::str::FromStr;

use aes::{
    cipher::{self, InnerIvInit, KeyInit, StreamCipherCore},
    Aes128,
};
use ethers_utils_rs::{
    hash::pbkdf2::pbkdf2_hmac,
    hex_def,
    types::{Address, AddressEx},
};
use rand::{CryptoRng, Rng};
use scrypt::{scrypt, Params as ScryptParams};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sha3::{Digest, Keccak256};
use uuid::Uuid;

use crate::{wallet::KeyProvider, WalletError};

hex_def!(IV);
hex_def!(CipherText);
hex_def!(MAC);
hex_def!(Salt);

const DEFAULT_CIPHER: &str = "aes-128-ctr";
const DEFAULT_KEY_SIZE: usize = 32usize;
const DEFAULT_IV_SIZE: usize = 16usize;
const DEFAULT_KDF_PARAMS_DKLEN: u8 = 32u8;
const DEFAULT_KDF_PARAMS_LOG_N: u8 = 13u8;
const DEFAULT_KDF_PARAMS_R: u32 = 8u32;
const DEFAULT_KDF_PARAMS_P: u32 = 1u32;

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("Scrypt error,{0}")]
    Scrypt(String),
    #[error("Keystore from json, mac mismatch")]
    MacMismatch,

    #[error("Load key error,{0}")]
    KeyProvider(WalletError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyStore {
    pub address: Option<Address>,
    pub crypto: CryptoJson,
    pub id: Uuid,
    pub version: u8,
}

#[derive(Debug, Deserialize, Serialize)]
/// Represents the "crypto" part of an encrypted JSON keystore.
pub struct CryptoJson {
    pub cipher: String,
    pub cipherparams: CipherparamsJson,
    pub ciphertext: CipherText,
    pub kdf: KdfType,
    pub kdfparams: KdfparamsType,
    pub mac: MAC,
}

#[derive(Debug, Deserialize, Serialize)]
/// Represents the "cipherparams" part of an encrypted JSON keystore.
pub struct CipherparamsJson {
    pub iv: IV,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
/// Types of key derivition functions supported by the Web3 Secret Storage.
pub enum KdfType {
    Pbkdf2,
    Scrypt,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
/// Defines the various parameters used in the supported KDFs.
pub enum KdfparamsType {
    Pbkdf2 {
        c: u32,
        dklen: u8,
        prf: String,
        salt: Salt,
    },
    Scrypt {
        dklen: u8,
        n: u32,
        p: u32,
        r: u32,
        salt: Salt,
    },
}

impl TryInto<String> for KeyStore {
    type Error = serde_json::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string_pretty(&self)
    }
}

impl FromStr for KeyStore {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

pub trait KeyStoreEncrypt {
    /// encrypt private_key or other data into keystore format with provide `password`
    fn encrypt<B, P>(pk: B, password: P) -> anyhow::Result<KeyStore>
    where
        P: AsRef<[u8]>,
        B: KeyProvider,
    {
        Self::encrypt_with(&mut rand::rngs::OsRng, pk, password)
    }
    fn encrypt_with<R, B, P>(rng: &mut R, pk: B, password: P) -> anyhow::Result<KeyStore>
    where
        R: Rng + CryptoRng,
        P: AsRef<[u8]>,
        B: KeyProvider,
    {
        // Generate a random salt.
        let mut salt = vec![0u8; DEFAULT_KEY_SIZE];
        rng.fill_bytes(salt.as_mut_slice());

        // Derive the key.
        let mut key = vec![0u8; DEFAULT_KDF_PARAMS_DKLEN as usize];
        let scrypt_params = ScryptParams::new(
            DEFAULT_KDF_PARAMS_LOG_N,
            DEFAULT_KDF_PARAMS_R,
            DEFAULT_KDF_PARAMS_P,
        )
        .map_err(|err| KeyStoreError::Scrypt(err.to_string()))?;

        scrypt(password.as_ref(), &salt, &scrypt_params, key.as_mut_slice())
            .map_err(|err| KeyStoreError::Scrypt(err.to_string()))?;

        // Encrypt the private key using AES-128-CTR.
        let mut iv = vec![0u8; DEFAULT_IV_SIZE];
        rng.fill_bytes(iv.as_mut_slice());

        let encryptor = Aes128Ctr::new(&key[..16], &iv[..16]).expect("invalid length");

        let mut ciphertext = pk.load()?;

        let address = Address::from_private_key(&ciphertext).ok();

        encryptor.apply_keystream(&mut ciphertext);

        // Calculate the MAC.
        let mut mac = Keccak256::new();

        mac.update(&key[16..32]);
        mac.update(&ciphertext);

        let mac = mac.finalize();

        // If a file name is not specified for the keystore, simply use the strigified uuid.
        let id = Uuid::new_v4();

        // Construct and serialize the encrypted JSON keystore.
        Ok(KeyStore {
            id,
            version: 3,
            crypto: CryptoJson {
                cipher: String::from(DEFAULT_CIPHER),
                cipherparams: CipherparamsJson { iv: iv.into() },
                ciphertext: ciphertext.to_vec().into(),
                kdf: KdfType::Scrypt,
                kdfparams: KdfparamsType::Scrypt {
                    dklen: DEFAULT_KDF_PARAMS_DKLEN,
                    n: 2u32.pow(DEFAULT_KDF_PARAMS_LOG_N as u32),
                    p: DEFAULT_KDF_PARAMS_P,
                    r: DEFAULT_KDF_PARAMS_R,
                    salt: salt.into(),
                },
                mac: mac.to_vec().into(),
            },
            address,
        })
    }
}

struct Aes128Ctr {
    inner: ctr::CtrCore<Aes128, ctr::flavors::Ctr128BE>,
}

impl Aes128Ctr {
    fn new(key: &[u8], iv: &[u8]) -> Result<Self, cipher::InvalidLength> {
        let cipher = aes::Aes128::new_from_slice(key).unwrap();
        let inner = ctr::CtrCore::inner_iv_slice_init(cipher, iv).unwrap();
        Ok(Self { inner })
    }

    fn apply_keystream(self, buf: &mut [u8]) {
        self.inner.apply_keystream_partial(buf.into());
    }
}

impl KeyStoreEncrypt for KeyStore {}

impl KeyStore {
    pub fn decrypt_into<S>(self, password: S) -> Result<Vec<u8>, KeyStoreError>
    where
        S: AsRef<[u8]>,
    {
        let keystore = self;

        // Derive the key.
        let key = match keystore.crypto.kdfparams {
            KdfparamsType::Pbkdf2 {
                c,
                dklen,
                prf: _,
                salt,
            } => {
                let mut key = vec![0u8; dklen as usize];
                pbkdf2_hmac::<Sha256>(password.as_ref(), &salt.0, c, key.as_mut_slice());
                key
            }
            KdfparamsType::Scrypt {
                dklen,
                n,
                p,
                r,
                salt,
            } => {
                let mut key = vec![0u8; dklen as usize];
                let log_n = (n as f32).log2() as u8;
                let scrypt_params = ScryptParams::new(log_n, r, p)
                    .map_err(|err| KeyStoreError::Scrypt(err.to_string()))?;

                scrypt(
                    password.as_ref(),
                    &salt.0,
                    &scrypt_params,
                    key.as_mut_slice(),
                )
                .map_err(|err| KeyStoreError::Scrypt(err.to_string()))?;
                key
            }
        };

        // Calculate the MAC.
        let mut derived_mac = Keccak256::new();

        derived_mac.update(&key[16..32]);
        derived_mac.update(&keystore.crypto.ciphertext.0);
        let derived_mac = derived_mac.finalize();

        if derived_mac.as_slice() != keystore.crypto.mac.0.as_slice() {
            return Err(KeyStoreError::MacMismatch);
        }

        // Decrypt the private key bytes using AES-128-CTR
        let decryptor = Aes128Ctr::new(&key[..16], &keystore.crypto.cipherparams.iv.0[..16])
            .expect("invalid length");

        let mut pk = keystore.crypto.ciphertext.0;

        decryptor.apply_keystream(&mut pk);

        Ok(pk)
    }
}

#[cfg(test)]
mod tests {
    use ethers_utils_rs::{
        hex::bytes_to_hex,
        types::{Address, AddressEx},
    };

    use super::{KeyStore, KeyStoreEncrypt};

    #[test]
    fn test_encrypt_decrypt() {
        let keystore = KeyStore::encrypt(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "thebestrandompassword",
        )
        .expect("Encrypt into keystore format");

        // Check generate address.
        assert_eq!(
            keystore.address,
            Some(
                "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
                    .try_into()
                    .expect("")
            )
        );

        let pk = keystore
            .decrypt_into("thebestrandompassword")
            .expect("Descrypt keystore");

        assert_eq!(
            bytes_to_hex(&pk),
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );
    }

    #[test]
    fn test_decrypt_pbkdf2() {
        let keystore: KeyStore = include_str!("test-keys/pbkdf2.json")
            .parse()
            .expect("Load keystore from file");

        let pk = keystore
            .decrypt_into("testpassword")
            .expect("Decrypt keystore with password");

        assert_eq!(
            bytes_to_hex(&pk),
            "0x7a28b5ba57c53603b0b07b56bba752f7784bf506fa95edc395f5cf6c7514fe9d"
        );
    }

    #[test]
    fn test_decrypt_scrypt() {
        let _ = pretty_env_logger::try_init();

        let keystore: KeyStore = include_str!("test-keys/scrypt.json")
            .parse()
            .expect("Load keystore from file");

        let pk = keystore
            .decrypt_into("grOQ8QDnGHvpYJf")
            .expect("Decrypt keystore with password");

        assert_eq!(
            bytes_to_hex(&pk),
            "0x80d3a6ed7b24dcd652949bc2f3827d2f883b3722e3120b15a93a2e0790f03829"
        );
    }

    #[test]
    fn test_decrypt_ethersjs_keystore() {
        let keystore: KeyStore = include_str!("test-keys/ethersjs.json")
            .parse()
            .expect("Load keystore from file");

        let expect_address = keystore.address.clone();

        let pk = keystore
            .decrypt_into("test")
            .expect("Decrypt keystore with password");

        let address = Address::from_private_key(&pk).expect("Get address from descrypt pk");

        assert_eq!(expect_address, Some(address));
    }
}
