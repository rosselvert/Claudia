use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct User {
    pub id:  Uuid,
    pub full_name: String,
    pub email: String,
    pub password: String
}