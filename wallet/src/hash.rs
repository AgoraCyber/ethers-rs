#[cfg(feature = "rust_crypto")]
pub mod pbkdf2 {
    use digest::{
        block_buffer::Eager,
        core_api::{BlockSizeUser, BufferKindUser, CoreProxy, FixedOutputCore, UpdateCore},
        generic_array::typenum::{IsLess, Le, NonZero, U256},
        HashMarker,
    };

    pub use ::pbkdf2::*;

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
