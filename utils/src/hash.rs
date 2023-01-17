use sha3::{Digest, Keccak256};

/// Compute the Keccak-256 hash of input bytes.
pub fn keccak256<S>(bytes: S) -> [u8; 32]
where
    S: AsRef<[u8]>,
{
    let mut hasher = Keccak256::new();

    hasher.update(bytes.as_ref());

    hasher.finalize().into()
}
