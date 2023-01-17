use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilsError {
    #[error("Parse hex string error, {0}")]
    Hex(hex::FromHexError),

    #[error("Invalid block tag, {0}")]
    BlockTag(String),

    #[error("Parsing sync statuts failed")]
    Syncing,

    #[error("Parse address error, {0}")]
    Address(String),
}
