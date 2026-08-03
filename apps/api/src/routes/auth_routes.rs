use std::sync::Arc;
use axum::{routing::post, Router};
use crate::{AppState, handlers::auth_handler::{handle_login, handle_register}};


pub fn router() -> Router<Arc<AppState>>{
    Router::new()
        .route("/login", post(handle_login))
        .route("/register", post(handle_register))
}