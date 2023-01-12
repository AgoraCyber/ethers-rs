use num::Num;

#[macro_use]
pub mod hex_fixed;
#[macro_use]
pub mod hex_var_len;

/// Parse hex string to integer
///
/// If `source` have the 0x prefix, skip over them.
pub fn hex_to_integer<T: Num>(source: &str) -> Result<T, T::FromStrRadixErr> {
    T::from_str_radix(source.trim_start_matches("0x"), 16)
}

/// Decode hex string to [`Vec<u8>`]
///
/// If `source` have the 0x prefix, skip over them.
pub fn hex_to_bytes(source: &str) -> Result<Vec<u8>, hex::FromHexError> {
    let mut source = source.trim_start_matches("0x").to_string();

    if source.len() % 2 != 0 {
        source = format!("0{}", source);
    }

    hex::decode(source)
}

/// Convert bytes to hex string with prefix `0x`
pub fn bytes_to_hex(source: &[u8]) -> String {
    let hex_str = hex::encode(source);

    format!(
        "0x{}",
        if hex_str.len() == 2 {
            hex_str.trim_start_matches("0")
        } else {
            hex_str.as_str()
        }
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        hex::{bytes_to_hex, hex_to_bytes},
        hex_fixed_def,
    };

    hex_fixed_def!(H32, 32);

    #[test]
    pub fn test_hex_fixed_cast() {
        let block_hash: H32 = "0x0bb3c2388383f714a8070dc6078a5edbe78f23c96646d4148d63cf964197ccc5"
            .try_into()
            .expect("Parse HexFixed error");

        assert_eq!(
            block_hash.to_string(),
            "0x0bb3c2388383f714a8070dc6078a5edbe78f23c96646d4148d63cf964197ccc5"
        );
    }

    #[test]
    fn test_zero_hex() {
        let zero = hex_to_bytes("0x1").expect("success");

        assert_eq!(zero, [1; 1]);

        assert_eq!("0x1", bytes_to_hex(&zero));

        assert_eq!(
            bytes_to_hex(&hex_to_bytes("0x10").expect("hex_to_bytes")),
            "0x10"
        );
    }
}
