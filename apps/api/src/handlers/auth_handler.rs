use crate::{dto::auth_dto::LoginRequest, dto::auth_dto::RegisterRequest, AppState};
use axum::{extract::{ Path, State}, Json, http::StatusCode};
use std::sync::Arc;
use service::{auth_service::login, register};

#[axum::debug_handler]
pub async fn handle_login( 
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>
) -> Result<StatusCode, (StatusCode, String)> {
    login(&state.db, &payload.email, &payload.password).await;

    Ok(StatusCode::OK)
}

pub async fn handle_register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>
) -> Result<StatusCode, (StatusCode, String)> {
    register(&state.db, &payload.full_name, &payload.email, &payload.password).await;

    Ok(StatusCode::OK)
}