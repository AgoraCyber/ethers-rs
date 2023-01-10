use num::Num;

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
    hex::decode(source.trim_start_matches("0x"))
}

/// Convert bytes to hex string with prefix `0x`
pub fn bytes_to_hex(source: &[u8]) -> String {
    format!("0x{}", hex::encode(source))
}
