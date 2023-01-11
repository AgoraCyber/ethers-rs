use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilsError {
    #[error("Parse hex string error, {0}")]
    Hex(hex::FromHexError),
}
