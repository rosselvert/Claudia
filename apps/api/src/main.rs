pub mod auth;
pub mod error;
pub mod handlers;
pub mod routes;

use axum::Router;
use database::connection::connection;
use std::{env, sync::Arc};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = connection().await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    let shared_state = Arc::new(AppState { db: pool });

    let frontend_dir = env::var("FRONTEND_DIR").unwrap_or_else(|_| "apps/web/dist".to_owned());
    let index_file = format!("{frontend_dir}/index.html");
    let app = Router::new()
        .nest("/api/v1", routes::store_routes::router())
        .fallback_service(ServeDir::new(frontend_dir).fallback(ServeFile::new(index_file)))
        .with_state(shared_state);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "5001".to_owned());
    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await?;

    println!("Claudia API running on http://{address}");

    axum::serve(listener, app).await?;
    Ok(())
}
