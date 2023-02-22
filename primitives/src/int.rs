//! Primitive int/uint type support for the ethereum rpc/abi
//!

mod uint;
pub use uint::*;

use num::{bigint::ToBigUint, BigUint};

/// impl from builin num
#[macro_export]
macro_rules! from_builtin_num {
    ($t: ident,$expr: tt,$($tails:tt),+) => {
        impl<const BITS: usize> From<$expr> for $t<BITS> {
            fn from(value: $expr) -> Self {
                let value: BigUint = value.to_biguint().expect("usize to biguint");
                // overflow panic
                Uint::new(value).expect("Overflow")
            }
        }

        from_builtin_num!($t,$($tails),*);
    };
    ($t: ident, $expr: tt) => {
        impl<const BITS: usize> From<$expr> for $t<BITS> {
            fn from(value: $expr) -> Self {
                let value: BigUint = value.to_biguint().expect("usize to biguint");
                // overflow panic
                Uint::new(value).expect("Overflow")
            }
        }
    };
}

from_builtin_num!(Uint, u128, u64, u32, u16, u8);
