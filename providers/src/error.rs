#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Parse block tag error, {0}")]
    BlockTag(serde_json::Error),

    #[error("Parse number error")]
    Number,

    #[error("Parse syncing status err, should always return false if not syncing")]
    Syncing,
}
