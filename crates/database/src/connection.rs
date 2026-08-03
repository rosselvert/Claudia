use sqlx::{PgPool, postgres::PgPoolOptions};
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
struct Config {
    database_url: String
}

pub async fn connection() -> anyhow::Result<PgPool> {
    dotenvy::dotenv().ok();

    let config = envy::from_env::<Config>();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.unwrap().database_url)
        .await?;

    println!("Connected");

    Ok(pool)
}