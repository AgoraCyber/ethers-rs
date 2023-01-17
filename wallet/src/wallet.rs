//! ethers-rs wallet facade
//!

use std::ptr::NonNull;

use ethers_utils_rs::hex;

use crate::{vtable::WalletVTable, Result, WalletError};

#[cfg(feature = "pure")]
mod pure;

#[cfg(feature = "openssl")]
mod openssl;

/// Blockchain in memeory wallet instance.
pub struct Wallet {
    pub(crate) inner: NonNull<WalletVTable>,
}

impl Wallet {
    pub fn new<P: KeyProvider>(provider: P) -> Result<Self> {
        Ok(Self::new_with_impl(pure::LocalWalletRustCrypto::new(
            provider,
        )?))
    }

    /// Get public key data .
    pub fn public_key(&self, compressed: bool) -> Result<Vec<u8>> {
        unsafe { (self.inner.as_ref().public_key)(self.inner, compressed) }
    }

    /// Sign provider hashed data.
    pub fn sign(&self, hashed: &[u8]) -> Result<Vec<u8>> {
        unsafe { (self.inner.as_ref().sign)(self.inner, hashed) }
    }

    /// Verify signature with public key and hashed data
    pub fn verify(&self, hashed: &[u8], signature: &[u8]) -> Result<bool> {
        unsafe { (self.inner.as_ref().verify)(self.inner, hashed, signature) }
    }

    /// Recover public key with hashed data, signature and recover id.
    pub fn recover(
        &self,
        hashed: &[u8],
        signature: &[u8],
        recover_id: u8,
        compressed: bool,
    ) -> Result<Vec<u8>> {
        unsafe {
            (self.inner.as_ref().recover)(self.inner, hashed, signature, recover_id, compressed)
        }
    }
}

/// Private key provider trait
pub trait KeyProvider {
    /// Load private key to memory
    fn load(&mut self) -> Result<Vec<u8>>;
}

/// Wallet implementation trait.
pub trait WalletProvider {
    fn public_key(&self, compressed: bool) -> Result<Vec<u8>>;
    /// Sign provider hashed data.
    fn sign(&self, hashed: &[u8]) -> Result<Vec<u8>>;

    /// Verify signature with public key and hashed data
    fn verify(&self, hashed: &[u8], signature: &[u8]) -> Result<bool>;
    /// Recover public key with hashed data, signature and recover id.
    fn recover(
        &self,
        hashed: &[u8],
        signature: &[u8],
        recover_id: u8,
        compressed: bool,
    ) -> Result<Vec<u8>>;
}

impl<'a> KeyProvider for &'a str {
    fn load(&mut self) -> Result<Vec<u8>> {
        hex::hex_to_bytes(self).map_err(|err| WalletError::LoadKey(format!("{}", err)))
    }
}

impl<'a> KeyProvider for &'a [u8] {
    fn load(&mut self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }
}

impl KeyProvider for Vec<u8> {
    fn load(&mut self) -> Result<Vec<u8>> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {

    use ethers_utils_rs::{
        hash::keccak256,
        types::{Address, AddressEx, Eip55, Signature},
    };

    use super::Wallet;

    #[test]
    fn test_public_key() {
        let _ = pretty_env_logger::try_init();

        let wallet =
            Wallet::new("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("Create wallet from private key");

        let address = Address::from_pub_key(
            wallet
                .public_key(false)
                .expect("Public key")
                .try_into()
                .expect(""),
        );

        assert_eq!(
            address.to_checksum_string(),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
    }

    #[test]
    fn test_sign_and_recover() {
        let _ = pretty_env_logger::try_init();

        let expected = "0xf16ea9a3478698f695fd1401bfe27e9e4a7e8e3da94aa72b021125e31fa899cc573c48ea3fe1d4ab61a9db10c19032026e3ed2dbccba5a178235ac27f94504311c";

        let data = "hello";

        let hashed = format!("\x19Ethereum Signed Message:\n{}{}", data.len(), data);

        let hashed = keccak256(hashed);

        let wallet =
            Wallet::new("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("Create wallet from private key");

        let signature = wallet.sign(&hashed).expect("personal_sign");

        let signature: Signature = signature.try_into().expect("0");

        assert_eq!(expected, signature.to_string());

        assert!(wallet
            .verify(&hashed, &signature.0[0..64])
            .expect("Verify signature"));

        let address = Address::from_pub_key(
            wallet
                .public_key(false)
                .expect("Public key")
                .try_into()
                .expect(""),
        );

        assert_eq!(
            address.to_checksum_string(),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
    }
}
