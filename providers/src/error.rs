use ethers_utils_rs::error::UtilsError;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Parse block tag error, {0}")]
    BlockTag(serde_json::Error),
    #[error("Parse number error, {0}")]
    Number(UtilsError),
}
