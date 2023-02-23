use ethers_primitives::{Address, U256};
use k256::{
    ecdsa::{
        self,
        hazmat::{bits2field, SignPrimitive},
        Signature, VerifyingKey,
    },
    schnorr::signature::hazmat::PrehashVerifier,
    PublicKey, Secp256k1,
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
        signature: ethers_primitives::Eip1559Signature,
    ) -> anyhow::Result<Address>
    where
        H: AsRef<[u8]>,
    {
        let (sig, recover_id) = signature.try_into()?;

        let key = VerifyingKey::recover_from_prehash(hashed.as_ref(), &sig, recover_id)
            .map_err(|err| WalletError::ECDSA(format!("Recover public key error, {}", err)))?;

        let pubkey: PublicKey = key.into();

        Ok(pubkey.into())
    }

    /// Sign hashed data and returns signature
    pub fn sign<S>(&self, hashed: S) -> anyhow::Result<ethers_primitives::Eip1559Signature>
    where
        S: AsRef<[u8]>,
    {
        let hashed = hashed.as_ref();

        let z = bits2field::<Secp256k1>(hashed)
            .map_err(|err| WalletError::ECDSA(format!("Convert bits to field error, {}", err)))?;

        let (sig, recid) = self
            .sign_key
            .as_nonzero_scalar()
            .try_sign_prehashed_rfc6979::<Sha256>(z, b"")
            .map_err(|err| WalletError::ECDSA(format!("sign rfc6979 error, {}", err)))?;

        let recid = recid.expect("Recover id");

        Ok((sig, recid).into())
    }

    pub fn verify<R, S>(&self, hashed: &[u8], r: R, s: S) -> anyhow::Result<bool>
    where
        R: TryInto<U256>,
        S: TryInto<U256>,
        R::Error: std::error::Error + Sync + Send + 'static,
        S::Error: std::error::Error + Sync + Send + 'static,
    {
        let mut buff = [0; 64];

        let r: U256 = r.try_into()?;
        let s: U256 = s.try_into()?;

        buff[..32].copy_from_slice(&r.0);
        buff[32..].copy_from_slice(&s.0);

        let verifying_key = self.sign_key.verifying_key();

        let sig = Signature::try_from(buff.as_slice())
            .map_err(|err| WalletError::ECDSA(format!("Convert signature error, {}", err)))?;

        Ok(verifying_key.verify_prehash(hashed, &sig).is_ok())
    }

    pub fn public_key(&self) -> anyhow::Result<PublicKey> {
        Ok(self.sign_key.verifying_key().into())
    }
}
