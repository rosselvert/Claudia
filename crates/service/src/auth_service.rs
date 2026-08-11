use bcrypt::{DEFAULT_COST, hash, verify};
use sqlx::{PgPool, Row};

#[derive(Debug)]
pub enum AuthError {
    Database(sqlx::Error),
    Password(bcrypt::BcryptError),
    InvalidCredentials,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "{error}"),
            Self::Password(error) => write!(f, "{error}"),
            Self::InvalidCredentials => write!(f, "Invalid email or password"),
        }
    }
}

impl std::error::Error for AuthError {}
impl From<sqlx::Error> for AuthError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}
impl From<bcrypt::BcryptError> for AuthError {
    fn from(value: bcrypt::BcryptError) -> Self {
        Self::Password(value)
    }
}

pub async fn register(
    pool: &PgPool,
    full_name: &str,
    email: &str,
    password: &str,
) -> Result<sqlx::postgres::PgQueryResult, AuthError> {
    let password_hash = hash(password, DEFAULT_COST)?;

    let result = sqlx::query(
        r#"
        INSERT INTO users(email, full_name, password)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(email)
    .bind(full_name)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(result)
}

pub async fn login(pool: &PgPool, email: &str, password: &str) -> Result<(), AuthError> {
    let user = sqlx::query(r#"SELECT id, email, password FROM users WHERE email = $1"#)
        .bind(email)
        .fetch_optional(pool)
        .await?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err(AuthError::InvalidCredentials);
        }
    };

    let password_hash: String = user.get("password");
    let valid = verify(password, &password_hash)?;

    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(())
}
