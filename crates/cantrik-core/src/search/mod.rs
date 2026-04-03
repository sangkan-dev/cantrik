//! Semantic index: Ollama embeddings + LanceDB (Sprint 6).

mod embed_ollama;
mod pipeline;
mod store;
mod types;

pub use embed_ollama::{resolve_embed_base_url, resolve_ollama_base};
pub use pipeline::{VectorBuildReport, build_vector_index, semantic_search};
pub use store::{VECTOR_TABLE, lance_db_path, table_row_count};
pub use types::{ScoredChunk, SearchError};
