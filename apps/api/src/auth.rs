use crate::{AppState, error::ApiError};
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthUser {
    pub id: Uuid,
}

pub struct AdminUser {
    pub id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Authentication required"))?;
        let token = Uuid::parse_str(value)
            .map_err(|_| ApiError::new(StatusCode::UNAUTHORIZED, "Invalid session"))?;
        let session =
            sqlx::query("SELECT user_id FROM sessions WHERE token = $1 AND expires_at > NOW()")
                .bind(token)
                .fetch_optional(&state.db)
                .await?;

        session
            .map(|row| Self {
                id: row.get("user_id"),
            })
            .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Session expired or invalid"))
    }
}

impl FromRequestParts<Arc<AppState>> for AdminUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        let is_admin: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND role = 'admin')",
        )
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;
        if !is_admin {
            return Err(ApiError::new(
                StatusCode::FORBIDDEN,
                "Administrator access required",
            ));
        }
        Ok(Self { id: user.id })
    }
}
