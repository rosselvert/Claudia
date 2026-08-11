use serde::Deserialize;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, Deserialize)]
struct Config {
    database_url: String,
}

pub async fn connection() -> anyhow::Result<PgPool> {
    dotenvy::dotenv().ok();

    let config = envy::from_env::<Config>()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    Ok(pool)
}
