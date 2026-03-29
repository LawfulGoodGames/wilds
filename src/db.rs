use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

pub async fn init() -> color_eyre::Result<sqlx::SqlitePool> {
    let opts = SqliteConnectOptions::from_str("sqlite:wilds.db")?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
