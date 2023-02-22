//! Primitive int/uint type support for the ethereum rpc/abi
//!

mod unsigned;
pub use unsigned::*;

mod signed;
pub use signed::*;

// from_to_builtin_num!(Int, isize, i128, i64, i32, i16, i8);

#[cfg(test)]
mod tests {
    use serde_ethabi::to_abi;
    use serde_rlp::rlp_encode;

    use crate::ToEtherHex;

    use super::*;

    #[test]
    fn test_arith() {
        let lhs = Uint::<8>::new(1u8).unwrap();
        let rhs = Uint::<8>::new(4u8).unwrap();

        assert_eq!(lhs < rhs, true);

        assert_eq!((rhs - lhs), Uint::<8>::new(3u8).unwrap());
    }

    #[test]
    fn test_rlp() {
        _ = pretty_env_logger::try_init();

        assert_eq!(
            rlp_encode(&U256::from(0usize)).unwrap(),
            rlp_encode(&0usize).unwrap()
        );

        assert_eq!(
            rlp_encode(&U256::from(100000usize)).unwrap(),
            rlp_encode(&100000usize).unwrap()
        );
    }

    #[test]
    fn test_abi() {
        assert_eq!(
            to_abi(&U256::from(69usize)).unwrap().to_eth_hex(),
            "0x0000000000000000000000000000000000000000000000000000000000000045"
        );

        assert_eq!(
            to_abi(&U256::from(1usize)).unwrap().to_eth_hex(),
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
    }
}
