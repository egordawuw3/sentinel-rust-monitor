use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, Pool, Sqlite, FromRow};
use std::str::FromStr;
use anyhow::Result;

#[derive(Debug, Clone, FromRow)]
pub struct Site {
    pub id: i64,
    pub url: String,
    pub user_id: i64,
}

pub async fn init_pool(database_url: &str) -> Result<Pool<Sqlite>> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .log_statements(log::LevelFilter::Debug);

    let pool = Pool::<Sqlite>::connect_with(options).await?;

    // Schema
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            user_id INTEGER NOT NULL
        );"
    )
    .execute(&pool)
    .await?;

    // Performance Index
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_id ON sites (user_id);")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn add_site(pool: &Pool<Sqlite>, chat_id: i64, url: String) -> Result<i64> {
    let id = sqlx::query("INSERT INTO sites (user_id, url) VALUES (?, ?)")
        .bind(chat_id)
        .bind(url)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn get_all_sites(pool: &Pool<Sqlite>) -> Result<Vec<Site>> {
    let sites = sqlx::query_as::<_, Site>("SELECT * FROM sites").fetch_all(pool).await?;
    Ok(sites)
}

pub async fn get_user_sites(pool: &Pool<Sqlite>, user_id: i64) -> Result<Vec<Site>> {
    let sites = sqlx::query_as::<_, Site>("SELECT * FROM sites WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(sites)
}

pub async fn get_site(pool: &Pool<Sqlite>, id: i64) -> Result<Site> {
    let site = sqlx::query_as::<_, Site>("SELECT * FROM sites WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(site)
}

pub async fn remove_site(pool: &Pool<Sqlite>, id: i64, user_id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM sites WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
