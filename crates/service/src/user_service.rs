use models::user_model::User;
use sqlx::PgPool;

pub async fn get_all_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(r#"SELECT id, full_name, email, password FROM users"#)
        .fetch_all(pool)
        .await
}
