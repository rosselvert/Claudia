pub mod routes;
pub mod handlers;
pub mod dto;

use axum::Router;
use std::{sync::Arc};
use database::connection::{connection};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool
}
#[tokio::main]
async fn main() {
    let pool = connection().await.expect("Unable to initiate DB Pool");

    let shared_state = Arc::new(AppState {db: pool});

    let app = Router::new()
        .nest("/api/v1", routes::auth_routes::router())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await.unwrap();

    println!("Server running on http//127.0.0.1:5000");

    axum::serve(listener, app).await.unwrap();
}