use sqlx::PgPool;

pub async fn register(pool: &PgPool, full_name: String, email: String, password: String) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO users (email, full_name, password) VALUES ($1, $2, $3)"#, email, full_name, password
    )
    .execute(pool)
    .await
}