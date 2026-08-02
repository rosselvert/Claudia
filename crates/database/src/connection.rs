use sqlx::postgres::PgPoolOptions;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    database_url: String
}

#[tokio::main]
pub async fn connection() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();

    let config = envy::from_env::<Config>();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.unwrap().database_url)
        .await?;

    println!("Connected");

    Ok(())
}