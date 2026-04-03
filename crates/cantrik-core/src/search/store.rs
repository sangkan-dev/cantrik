//! LanceDB location, schema helpers, and table utilities.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, Int64Array, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures_util::TryStreamExt;
use lancedb::error::Error as LanceError;
use lancedb::query::{ExecutableQuery, QueryBase, Select};

use super::types::SearchError;

pub const VECTOR_TABLE: &str = "code_chunks";
pub const SCHEMA_VERSION: u32 = 1;
pub const VECTOR_META_FILE: &str = "vector_meta.json";

/// Embedded DB under `.cantrik/index/lance/` (separate from AST artifacts in `ast/`).
pub fn lance_db_path(project_root: &Path) -> PathBuf {
    project_root.join(".cantrik").join("index").join("lance")
}

pub fn vector_meta_path(project_root: &Path) -> PathBuf {
    lance_db_path(project_root).join(VECTOR_META_FILE)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct VectorMeta {
    pub schema_version: u32,
    pub embedding_dims: u32,
    pub vector_model: String,
}

pub fn chunk_table_schema(dims: i32) -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("chunk_id", DataType::Utf8, false),
        Field::new("content_hash", DataType::Utf8, false),
        Field::new("path", DataType::Utf8, false),
        Field::new("symbol", DataType::Utf8, false),
        Field::new("language", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("start_row", DataType::Int64, false),
        Field::new("start_byte", DataType::Int64, false),
        Field::new("end_byte", DataType::Int64, false),
        Field::new("preview", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dims),
            false,
        ),
    ]))
}

pub fn read_vector_meta(project_root: &Path) -> Result<Option<VectorMeta>, SearchError> {
    let p = vector_meta_path(project_root);
    if !p.exists() {
        return Ok(None);
    }
    let s = std::fs::read_to_string(&p)?;
    Ok(Some(serde_json::from_str(&s)?))
}

pub fn write_vector_meta(project_root: &Path, meta: &VectorMeta) -> Result<(), SearchError> {
    let dir = lance_db_path(project_root);
    std::fs::create_dir_all(&dir)?;
    let p = vector_meta_path(project_root);
    std::fs::write(&p, serde_json::to_string_pretty(meta)?)?;
    Ok(())
}

pub fn vector_dims_from_schema(schema: &Schema) -> Result<i32, SearchError> {
    let field = schema
        .field_with_name("vector")
        .map_err(|_| SearchError::LanceDb("table schema missing `vector` column".into()))?;
    match field.data_type() {
        DataType::FixedSizeList(_, n) => Ok(*n),
        _ => Err(SearchError::LanceDb(
            "`vector` column is not FixedSizeList".into(),
        )),
    }
}

pub async fn connect_lance(project_root: &Path) -> Result<lancedb::Connection, SearchError> {
    let dir = lance_db_path(project_root);
    std::fs::create_dir_all(&dir)?;
    let uri = dir
        .to_str()
        .ok_or_else(|| SearchError::LanceDb("lance path is not valid UTF-8".into()))?;
    lancedb::connect(uri)
        .execute()
        .await
        .map_err(|e| SearchError::LanceDb(e.to_string()))
}

pub async fn open_chunks_table(
    conn: &lancedb::Connection,
) -> Result<Option<lancedb::Table>, SearchError> {
    match conn.open_table(VECTOR_TABLE).execute().await {
        Ok(t) => Ok(Some(t)),
        Err(LanceError::TableNotFound { .. }) => Ok(None),
        Err(e) => Err(SearchError::LanceDb(e.to_string())),
    }
}

/// Rows keyed by `chunk_id`; values are `(content_hash, vector)`.
pub async fn load_stored_embeddings(
    table: &lancedb::Table,
) -> Result<HashMap<String, (String, Vec<f32>)>, SearchError> {
    let stream = table
        .query()
        .select(Select::columns(&["chunk_id", "content_hash", "vector"]))
        .execute()
        .await
        .map_err(|e| SearchError::LanceDb(e.to_string()))?;

    let batches: Vec<RecordBatch> = stream
        .try_collect()
        .await
        .map_err(|e| SearchError::LanceDb(e.to_string()))?;

    let mut out = HashMap::new();
    for batch in batches {
        let ids = batch
            .column_by_name("chunk_id")
            .ok_or_else(|| SearchError::Arrow("missing chunk_id".into()))?
            .as_string::<i32>();
        let hashes = batch
            .column_by_name("content_hash")
            .ok_or_else(|| SearchError::Arrow("missing content_hash".into()))?
            .as_string::<i32>();
        let vectors = batch
            .column_by_name("vector")
            .ok_or_else(|| SearchError::Arrow("missing vector".into()))?
            .as_fixed_size_list();

        for row in 0..batch.num_rows() {
            let id = ids.value(row).to_string();
            let hash = hashes.value(row).to_string();
            let list = vectors.value(row);
            let floats = list.as_primitive::<Float32Type>();
            let v: Vec<f32> = floats.values().to_vec();
            out.insert(id, (hash, v));
        }
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
pub fn record_batch_for_chunks(
    schema: Arc<Schema>,
    dims: i32,
    chunk_ids: &[String],
    content_hashes: &[String],
    paths: &[String],
    symbols: &[String],
    languages: &[String],
    kinds: &[String],
    start_rows: &[i64],
    start_bytes: &[i64],
    end_bytes: &[i64],
    previews: &[String],
    vectors: &[Vec<f32>],
) -> Result<RecordBatch, SearchError> {
    let n = chunk_ids.len();
    if !(n == content_hashes.len()
        && n == paths.len()
        && n == symbols.len()
        && n == languages.len()
        && n == kinds.len()
        && n == start_rows.len()
        && n == start_bytes.len()
        && n == end_bytes.len()
        && n == previews.len()
        && n == vectors.len())
    {
        return Err(SearchError::Arrow(
            "column length mismatch building RecordBatch".into(),
        ));
    }
    for v in vectors {
        if v.len() != dims as usize {
            return Err(SearchError::Arrow(format!(
                "vector len {} != dims {}",
                v.len(),
                dims
            )));
        }
    }

    let list = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vectors
            .iter()
            .map(|emb| Some(emb.iter().map(|&x| Some(x)).collect::<Vec<_>>())),
        dims,
    );

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(chunk_ids.to_vec())),
            Arc::new(StringArray::from(content_hashes.to_vec())),
            Arc::new(StringArray::from(paths.to_vec())),
            Arc::new(StringArray::from(symbols.to_vec())),
            Arc::new(StringArray::from(languages.to_vec())),
            Arc::new(StringArray::from(kinds.to_vec())),
            Arc::new(Int64Array::from(start_rows.to_vec())),
            Arc::new(Int64Array::from(start_bytes.to_vec())),
            Arc::new(Int64Array::from(end_bytes.to_vec())),
            Arc::new(StringArray::from(previews.to_vec())),
            Arc::new(list),
        ],
    )
    .map_err(|e| SearchError::Arrow(e.to_string()))
}

pub async fn upsert_chunk_batch(
    conn: &lancedb::Connection,
    schema: Arc<Schema>,
    batch: RecordBatch,
) -> Result<(), SearchError> {
    match open_chunks_table(conn).await? {
        None => {
            conn.create_table(VECTOR_TABLE, batch)
                .execute()
                .await
                .map_err(|e| SearchError::LanceDb(e.to_string()))?;
        }
        Some(table) => {
            let reader = RecordBatchIterator::new(std::iter::once(Ok(batch)), schema.clone());
            let mut m = table.merge_insert(&["chunk_id"]);
            m.when_matched_update_all(None)
                .when_not_matched_insert_all()
                .when_not_matched_by_source_delete(None);
            m.execute(Box::new(reader))
                .await
                .map_err(|e| SearchError::LanceDb(e.to_string()))?;
        }
    }
    Ok(())
}

pub async fn table_row_count(project_root: &Path) -> Result<Option<usize>, SearchError> {
    if !lance_db_path(project_root).exists() {
        return Ok(None);
    }
    let conn = connect_lance(project_root).await?;
    let Some(table) = open_chunks_table(&conn).await? else {
        return Ok(None);
    };
    let n = table
        .count_rows(None)
        .await
        .map_err(|e| SearchError::LanceDb(e.to_string()))?;
    Ok(Some(n))
}

#[cfg(test)]
mod tests {
    use super::VectorMeta;

    #[test]
    fn vector_meta_serde_roundtrip() {
        let m = VectorMeta {
            schema_version: 1,
            embedding_dims: 768,
            vector_model: "nomic-embed-text".into(),
        };
        let j = serde_json::to_string(&m).expect("json");
        let m2: VectorMeta = serde_json::from_str(&j).expect("parse");
        assert_eq!(m, m2);
    }
}
