use ethers_utils_rs::types::Number;
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

use super::KeyProvider;

#[derive(Clone)]
pub struct LocalWalletRustCrypto {
    sign_key: ecdsa::SigningKey,
}

impl LocalWalletRustCrypto {
    /// Create new local wallet from key provider
    pub fn new<P: KeyProvider>(provider: P) -> Result<Self> {
        let key_data = provider.load()?;

        let sign_key = ecdsa::SigningKey::from_bytes(&key_data)
            .map_err(|e| WalletError::ECDSA(format!("{}", e)))?;

        Ok(Self { sign_key })
    }
}

impl LocalWalletRustCrypto {
    pub fn recover<H>(
        &self,
        hashed: H,
        signature: ethers_utils_rs::types::Signature,
        compressed: bool,
    ) -> anyhow::Result<Vec<u8>>
    where
        H: AsRef<[u8]>,
    {
        let sig = Signature::try_from(&signature.0[1..])
            .map_err(|err| WalletError::ECDSA(format!("Convert signature error, {}", err)))?;

        let recover_id = signature.0[0];

        let recover_id =
            RecoveryId::from_byte(recover_id).ok_or(WalletError::RecoverId(recover_id))?;

        let key = VerifyingKey::recover_from_prehash(hashed.as_ref(), &sig, recover_id)
            .map_err(|err| WalletError::ECDSA(format!("Recover public key error, {}", err)))?;

        Ok(key.to_encoded_point(compressed).as_bytes().to_vec())
    }

    /// Sign hashed data and returns signature
    pub fn sign<S>(&self, hashed: S) -> anyhow::Result<ethers_utils_rs::types::Signature>
    where
        S: AsRef<[u8]>,
    {
        let hashed = hashed.as_ref();

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

        result.push(recid.expect("Recover id").to_byte());
        result.append(&mut r.to_vec());
        result.append(&mut s.to_vec());

        assert_eq!(result.len(), 65);

        Ok(result.try_into()?)
    }

    pub fn verify<R, S>(&self, hashed: &[u8], r: R, s: S) -> anyhow::Result<bool>
    where
        R: TryInto<Number>,
        S: TryInto<Number>,
        R::Error: std::error::Error + Sync + Send + 'static,
        S::Error: std::error::Error + Sync + Send + 'static,
    {
        let r = r.try_into()?;
        let mut s = s.try_into()?;

        let mut signature = r.0;

        signature.append(&mut s.0);

        let verifying_key = self.sign_key.verifying_key();

        let sig = Signature::try_from(signature.as_slice())
            .map_err(|err| WalletError::ECDSA(format!("Convert signature error, {}", err)))?;

        Ok(verifying_key.verify_prehash(hashed, &sig).is_ok())
    }

    pub fn public_key(&self, comppressed: bool) -> anyhow::Result<Vec<u8>> {
        Ok(self
            .sign_key
            .verifying_key()
            .to_encoded_point(comppressed)
            .as_bytes()
            .to_vec())
    }
}
