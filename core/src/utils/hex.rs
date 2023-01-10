use num::Num;

/// Parse hex string to integer
///
/// If `source` have the 0x prefix, skip over them.
pub fn hex_to_integer<T: Num>(source: &str) -> Result<T, T::FromStrRadixErr> {
    T::from_str_radix(source.trim_start_matches("0x"), 16)
}
