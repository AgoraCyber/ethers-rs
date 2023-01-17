use k256::{
    ecdsa::{
        self,
        hazmat::{bits2field, SignPrimitive},
        RecoveryId, Signature, VerifyingKey,
    },
    schnorr::signature::hazmat::PrehashVerifier,
    Secp256k1,
};
use sha2::Sha256;

use crate::{Result, WalletError};

use super::{KeyProvider, WalletProvider};

pub struct LocalWalletRustCrypto {
    sign_key: ecdsa::SigningKey,
}

impl LocalWalletRustCrypto {
    /// Create new local wallet from key provider
    pub fn new<P: KeyProvider>(mut provider: P) -> Result<Self> {
        let key_data = provider.load()?;

        let sign_key = ecdsa::SigningKey::from_bytes(&key_data)
            .map_err(|e| WalletError::ECDSA(format!("{}", e)))?;

        Ok(Self { sign_key })
    }
}

impl WalletProvider for LocalWalletRustCrypto {
    fn recover(&self, hashed: &[u8], signature: &[u8], recover_id: u8) -> Result<Vec<u8>> {
        let sig = Signature::from_der(signature)
            .map_err(|err| WalletError::ECDSA(format!("Convert signature error, {}", err)))?;

        let recover_id =
            RecoveryId::from_byte(recover_id).ok_or(WalletError::RecoverId(recover_id))?;

        let key = VerifyingKey::recover_from_prehash(hashed, &sig, recover_id)
            .map_err(|err| WalletError::ECDSA(format!("Recover public key error, {}", err)))?;

        Ok(key.to_encoded_point(true).as_bytes().to_vec())
    }

    fn sign(&self, hashed: &[u8]) -> Result<Vec<u8>> {
        let z = bits2field::<Secp256k1>(hashed)
            .map_err(|err| WalletError::ECDSA(format!("Convert bits to field error, {}", err)))?;

        let (signature, recid) = self
            .sign_key
            .as_nonzero_scalar()
            .try_sign_prehashed_rfc6979::<Sha256>(z, b"")
            .map_err(|err| WalletError::ECDSA(format!("sign rfc6979 error, {}", err)))?;

        let r = signature.r().to_bytes();
        let s = signature.s().to_bytes();

        let mut result = vec![];

        result.append(&mut r.to_vec());
        result.append(&mut s.to_vec());
        result.push(recid.expect("Recover id").to_byte());

        assert_eq!(result.len(), 65);

        Ok(result)
    }

    fn verify(&self, hashed: &[u8], signature: &[u8]) -> Result<bool> {
        let verifying_key = self.sign_key.verifying_key();

        let sig = Signature::from_der(signature)
            .map_err(|err| WalletError::ECDSA(format!("Convert signature error, {}", err)))?;

        Ok(verifying_key.verify_prehash(hashed, &sig).is_ok())
    }

    fn public_key(&self, comppressed: bool) -> Result<Vec<u8>> {
        Ok(self
            .sign_key
            .verifying_key()
            .to_encoded_point(comppressed)
            .as_bytes()
            .to_vec())
    }
}
