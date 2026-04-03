//! Orchestration: read AST chunks, embed via Ollama, sync LanceDB.

use arrow_array::RecordBatch;
use arrow_array::cast::AsArray;
use arrow_array::types::Float32Type;
use futures_util::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use reqwest::Client;
use std::collections::HashMap;
use std::path::Path;

use crate::config::AppConfig;
use crate::indexing::{self, read_all_source_chunks};

use super::embed_ollama::{
    chunk_id_for, content_hash, embed_batch, embed_input_for_chunk, embed_one,
    resolve_embed_base_url,
};
use super::store::{
    SCHEMA_VERSION, VECTOR_TABLE, VectorMeta, chunk_table_schema, connect_lance,
    load_stored_embeddings, open_chunks_table, read_vector_meta, record_batch_for_chunks,
    upsert_chunk_batch, vector_dims_from_schema, write_vector_meta,
};
use super::types::{ScoredChunk, SearchError};

const DEFAULT_VECTOR_MODEL: &str = "nomic-embed-text";
const EMBED_BATCH: usize = 8;
const PREVIEW_CHARS: usize = 4096;

#[derive(Debug, Clone, Default)]
pub struct VectorBuildReport {
    pub chunks_total: usize,
    pub chunks_embedded: usize,
    pub chunks_reused: usize,
}

/// Build / refresh the LanceDB vector table from `chunks.jsonl` (after `cantrik index`).
pub async fn build_vector_index(
    project_root: &Path,
    config: &AppConfig,
) -> Result<VectorBuildReport, SearchError> {
    let root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());

    let ast_dir = indexing::ast_index_dir(&root);
    let chunks_path = indexing::chunks_path(&ast_dir);
    let chunks = read_all_source_chunks(&chunks_path)?;
    if chunks.is_empty() {
        return Err(SearchError::InvalidState(
            "no chunks found; run `cantrik index` first".into(),
        ));
    }

    let model = config
        .index
        .vector_model
        .as_deref()
        .unwrap_or(DEFAULT_VECTOR_MODEL);
    let base = resolve_embed_base_url(config);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| SearchError::Http(e.to_string()))?;

    let conn = connect_lance(&root).await?;
    let meta_file = read_vector_meta(&root)?;
    let existing = open_chunks_table(&conn).await?;

    if let (Some(m), Some(_)) = (&meta_file, &existing)
        && m.vector_model != model
    {
        return Err(SearchError::InvalidState(format!(
            "vector model is `{}` but index was built for `{}`; remove {:?} to rebuild",
            model,
            m.vector_model,
            super::store::lance_db_path(&root)
        )));
    }

    let mut stored: HashMap<String, (String, Vec<f32>)> = HashMap::new();
    let mut known_dims: Option<i32> = None;

    if let Some(table) = &existing {
        stored = load_stored_embeddings(table).await?;
        known_dims = if let Some(m) = &meta_file {
            Some(m.embedding_dims as i32)
        } else {
            let sch = table
                .schema()
                .await
                .map_err(|e| SearchError::LanceDb(e.to_string()))?;
            Some(vector_dims_from_schema(&sch)?)
        };
    }

    let n = chunks.len();
    let mut embeddings: Vec<Option<Vec<f32>>> = vec![None; n];
    let mut reused = 0usize;

    for (i, c) in chunks.iter().enumerate() {
        let id = chunk_id_for(&c.path, c.start_byte, c.end_byte);
        let h = content_hash(&c.source);
        if let Some((prev_h, vec)) = stored.get(&id)
            && prev_h == &h
        {
            embeddings[i] = Some(vec.clone());
            reused += 1;
        }
    }

    let mut need_indices: Vec<usize> = Vec::new();
    for (i, e) in embeddings.iter().enumerate() {
        if e.is_none() {
            need_indices.push(i);
        }
    }

    let mut embedded = 0usize;
    for win in need_indices.chunks(EMBED_BATCH) {
        let texts: Vec<String> = win
            .iter()
            .map(|&i| {
                let c = &chunks[i];
                embed_input_for_chunk(&c.path, &c.symbol, &c.language, &c.source)
            })
            .collect();
        let vecs = embed_batch(&client, &base, model, &texts).await?;
        for (&idx, v) in win.iter().zip(vecs) {
            embeddings[idx] = Some(v);
            embedded += 1;
        }
    }

    let dims = if let Some(d) = known_dims {
        d
    } else {
        let first = embeddings
            .iter()
            .find_map(|e| e.as_ref())
            .ok_or_else(|| SearchError::InvalidState("no embedding vectors produced".into()))?;
        first.len() as i32
    };

    if let Some(m) = &meta_file
        && m.embedding_dims as i32 != dims
    {
        return Err(SearchError::InvalidState(format!(
            "embedding dimension mismatch: meta has {} but vectors are {}",
            m.embedding_dims, dims
        )));
    }

    for v in embeddings.iter().flatten() {
        if v.len() != dims as usize {
            return Err(SearchError::InvalidState(format!(
                "inconsistent embedding length: expected {}, got {}",
                dims,
                v.len()
            )));
        }
    }

    let schema = chunk_table_schema(dims);
    let mut chunk_ids = Vec::with_capacity(n);
    let mut hashes = Vec::with_capacity(n);
    let mut paths = Vec::with_capacity(n);
    let mut symbols = Vec::with_capacity(n);
    let mut languages = Vec::with_capacity(n);
    let mut kinds = Vec::with_capacity(n);
    let mut start_rows = Vec::with_capacity(n);
    let mut start_bytes = Vec::with_capacity(n);
    let mut end_bytes = Vec::with_capacity(n);
    let mut previews = Vec::with_capacity(n);
    let mut vectors: Vec<Vec<f32>> = Vec::with_capacity(n);

    for (i, c) in chunks.iter().enumerate() {
        let vec = embeddings[i]
            .as_ref()
            .ok_or_else(|| SearchError::InvalidState("missing embedding row".into()))?
            .clone();
        chunk_ids.push(chunk_id_for(&c.path, c.start_byte, c.end_byte));
        hashes.push(content_hash(&c.source));
        paths.push(c.path.clone());
        symbols.push(c.symbol.clone());
        languages.push(c.language.clone());
        kinds.push(c.kind.clone());
        start_rows.push(
            i64::try_from(c.start_row)
                .map_err(|_| SearchError::InvalidState("start_row too large".into()))?,
        );
        start_bytes.push(
            i64::try_from(c.start_byte)
                .map_err(|_| SearchError::InvalidState("start_byte too large".into()))?,
        );
        end_bytes.push(
            i64::try_from(c.end_byte)
                .map_err(|_| SearchError::InvalidState("end_byte too large".into()))?,
        );
        previews.push(c.source.chars().take(PREVIEW_CHARS).collect());
        vectors.push(vec);
    }

    let batch = record_batch_for_chunks(
        schema.clone(),
        dims,
        &chunk_ids,
        &hashes,
        &paths,
        &symbols,
        &languages,
        &kinds,
        &start_rows,
        &start_bytes,
        &end_bytes,
        &previews,
        &vectors,
    )?;

    upsert_chunk_batch(&conn, schema, batch).await?;

    write_vector_meta(
        &root,
        &VectorMeta {
            schema_version: SCHEMA_VERSION,
            embedding_dims: dims as u32,
            vector_model: model.to_string(),
        },
    )?;

    Ok(VectorBuildReport {
        chunks_total: n,
        chunks_embedded: embedded,
        chunks_reused: reused,
    })
}

/// Semantic search: embed `query`, run ANN on `VECTOR_TABLE`, return ranked chunks.
pub async fn semantic_search(
    project_root: &Path,
    config: &AppConfig,
    query: &str,
    top_k: usize,
) -> Result<Vec<ScoredChunk>, SearchError> {
    let root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let meta = read_vector_meta(&root)?.ok_or_else(|| {
        SearchError::InvalidState(
            "vector index missing; run `cantrik index` (with vectors) first".into(),
        )
    })?;

    let model = config
        .index
        .vector_model
        .as_deref()
        .unwrap_or(DEFAULT_VECTOR_MODEL);
    if model != meta.vector_model {
        return Err(SearchError::InvalidState(format!(
            "config vector model `{model}` does not match index `{}`",
            meta.vector_model
        )));
    }

    let base = resolve_embed_base_url(config);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| SearchError::Http(e.to_string()))?;
    let qvec = embed_one(&client, &base, model, query).await?;
    if qvec.len() != meta.embedding_dims as usize {
        return Err(SearchError::InvalidState(format!(
            "query embedding dim {} != index {}",
            qvec.len(),
            meta.embedding_dims
        )));
    }

    let conn = connect_lance(&root).await?;
    let table = open_chunks_table(&conn)
        .await?
        .ok_or_else(|| SearchError::InvalidState(format!("table `{VECTOR_TABLE}` not found")))?;

    let stream = table
        .query()
        .select(Select::columns(&[
            "path",
            "symbol",
            "language",
            "preview",
            "_distance",
        ]))
        .nearest_to(qvec.as_slice())
        .map_err(|e| SearchError::LanceDb(e.to_string()))?
        .limit(top_k)
        .execute()
        .await
        .map_err(|e| SearchError::LanceDb(e.to_string()))?;

    let batches: Vec<RecordBatch> = stream
        .try_collect()
        .await
        .map_err(|e| SearchError::LanceDb(e.to_string()))?;

    let mut out = Vec::new();
    for batch in batches {
        let paths = batch
            .column_by_name("path")
            .ok_or_else(|| SearchError::Arrow("missing path".into()))?
            .as_string::<i32>();
        let symbols = batch
            .column_by_name("symbol")
            .ok_or_else(|| SearchError::Arrow("missing symbol".into()))?
            .as_string::<i32>();
        let langs = batch
            .column_by_name("language")
            .ok_or_else(|| SearchError::Arrow("missing language".into()))?
            .as_string::<i32>();
        let previews = batch
            .column_by_name("preview")
            .ok_or_else(|| SearchError::Arrow("missing preview".into()))?
            .as_string::<i32>();
        let distances = batch
            .column_by_name("_distance")
            .ok_or_else(|| SearchError::Arrow("missing _distance".into()))?
            .as_primitive::<Float32Type>();

        for row in 0..batch.num_rows() {
            let dist = distances.value(row);
            let score = 1.0 / (1.0 + dist);
            out.push(ScoredChunk {
                path: paths.value(row).to_string(),
                symbol: symbols.value(row).to_string(),
                language: langs.value(row).to_string(),
                score,
                preview: previews.value(row).to_string(),
            });
        }
    }
    Ok(out)
}
