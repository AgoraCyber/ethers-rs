use std::ptr::NonNull;

use crate::{wallet::Wallet, wallet::WalletProvider, Result};

#[repr(C)]
pub(crate) struct WalletVTable {
    pub(crate) public_key: unsafe fn(NonNull<WalletVTable>, compressed: bool) -> Result<Vec<u8>>,

    pub(crate) sign: unsafe fn(NonNull<WalletVTable>, hashed: &[u8]) -> Result<Vec<u8>>,

    pub(crate) verify:
        unsafe fn(NonNull<WalletVTable>, hashed: &[u8], signature: &[u8]) -> Result<bool>,

    /// Recover pub_key from hashed
    pub(crate) recover: unsafe fn(
        NonNull<WalletVTable>,
        hashed: &[u8],
        signature: &[u8],
        recover_id: u8,
    ) -> Result<Vec<u8>>,
}

unsafe fn public_key<W: WalletProvider>(
    vtable: NonNull<WalletVTable>,
    compressed: bool,
) -> Result<Vec<u8>> {
    let mut raw = vtable.cast::<WalletInner<W>>();

    raw.as_mut().wallet_impl.public_key(compressed)
}

unsafe fn sign<W: WalletProvider>(vtable: NonNull<WalletVTable>, hashed: &[u8]) -> Result<Vec<u8>> {
    let mut raw = vtable.cast::<WalletInner<W>>();

    raw.as_mut().wallet_impl.sign(hashed)
}

unsafe fn verify<W: WalletProvider>(
    vtable: NonNull<WalletVTable>,
    hashed: &[u8],
    signature: &[u8],
) -> Result<bool> {
    let mut raw = vtable.cast::<WalletInner<W>>();

    raw.as_mut().wallet_impl.verify(hashed, signature)
}

unsafe fn recover<W: WalletProvider>(
    vtable: NonNull<WalletVTable>,
    hashed: &[u8],
    signature: &[u8],
    recover_id: u8,
) -> Result<Vec<u8>> {
    let mut raw = vtable.cast::<WalletInner<W>>();

    raw.as_mut()
        .wallet_impl
        .recover(hashed, signature, recover_id)
}

impl WalletVTable {
    fn new<W: WalletProvider>() -> Self {
        Self {
            sign: sign::<W>,
            verify: verify::<W>,
            recover: recover::<W>,
            public_key: public_key::<W>,
        }
    }
}

#[repr(C)]
struct WalletInner<W: WalletProvider> {
    vtable: WalletVTable,
    wallet_impl: W,
}

impl Wallet {
    /// Create new wallet instance with wallet implementation type `W`
    pub fn new_with_impl<W: WalletProvider>(wallet_impl: W) -> Self {
        let boxed = Box::new(WalletInner::<W> {
            vtable: WalletVTable::new::<W>(),
            wallet_impl,
        });

        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(boxed) as *mut WalletVTable) };

        Self { inner: ptr }
    }
}
