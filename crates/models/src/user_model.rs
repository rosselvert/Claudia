use sqlx::FromRow;
use serde::{Serialize, Deserialize};

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct User {
    pub id:  i32,
    pub full_name: String,
    pub email: String,
    pub password: String
}