use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

use super::SessionError;
use super::paths;

pub async fn connect_pool() -> Result<SqlitePool, SessionError> {
    let path = paths::memory_db_path();
    let opts = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .map_err(SessionError::Sql)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(SessionError::Migrate)?;
    Ok(pool)
}
