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

#[cfg(feature = "rust_crypto")]
pub mod pbkdf2 {
    use digest::{
        block_buffer::Eager,
        core_api::{BlockSizeUser, BufferKindUser, CoreProxy, FixedOutputCore, UpdateCore},
        generic_array::typenum::{IsLess, Le, NonZero, U256},
        HashMarker,
    };

    /// A variant of the [`pbkdf2`][crate::pbkdf2] function which uses HMAC for PRF.
    /// It's generic over (eager) hash functions.
    ///
    /// ```
    /// use hex_literal::hex;
    /// use pbkdf2::pbkdf2_hmac;
    /// use sha2::Sha256;
    ///
    /// let mut buf = [0u8; 20];
    /// pbkdf2_hmac::<Sha256>(b"password", b"salt", 4096, &mut buf);
    /// assert_eq!(buf, hex!("c5e478d59288c841aa530db6845c4c8d962893a0"));
    /// ```
    pub fn pbkdf2_hmac<D>(password: &[u8], salt: &[u8], rounds: u32, res: &mut [u8])
    where
        D: CoreProxy,
        D::Core: Sync
            + HashMarker
            + UpdateCore
            + FixedOutputCore
            + BufferKindUser<BufferKind = Eager>
            + Default
            + Clone,
        <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
        Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero,
    {
        pbkdf2::pbkdf2::<hmac::Hmac<D>>(password, salt, rounds, res);
    }

    /// A variant of the [`pbkdf2_hmac`] function which returns an array
    /// instead of filling an input slice.
    ///
    /// ```
    /// use hex_literal::hex;
    /// use pbkdf2::pbkdf2_hmac_array;
    /// use sha2::Sha256;
    ///
    /// assert_eq!(
    ///     pbkdf2_hmac_array::<Sha256, 20>(b"password", b"salt", 4096),
    ///     hex!("c5e478d59288c841aa530db6845c4c8d962893a0"),
    /// );
    /// ```
    pub fn pbkdf2_hmac_array<D, const N: usize>(
        password: &[u8],
        salt: &[u8],
        rounds: u32,
    ) -> [u8; N]
    where
        D: CoreProxy,
        D::Core: Sync
            + HashMarker
            + UpdateCore
            + FixedOutputCore
            + BufferKindUser<BufferKind = Eager>
            + Default
            + Clone,
        <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
        Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero,
    {
        let mut buf = [0u8; N];
        pbkdf2_hmac::<D>(password, salt, rounds, &mut buf);
        buf
    }
}
