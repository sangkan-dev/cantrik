use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub path: String,
    pub symbol: String,
    pub language: String,
    pub score: f32,
    pub preview: String,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("HTTP / transport: {0}")]
    Http(String),
    #[error("embeddings API: {0}")]
    EmbedApi(String),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LanceDB: {0}")]
    LanceDb(String),
    #[error("Arrow: {0}")]
    Arrow(String),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("index: {0}")]
    Index(#[from] crate::indexing::IndexError),
}
