use sqlx::PgPool;
use bcrypt::{ verify, hash, DEFAULT_COST};

pub async fn register(
    pool: &PgPool,
    full_name: &str,
    email: &str,
    password: &str,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {

    println!("Register dipanggil");

    let password_hash = hash(password, DEFAULT_COST).unwrap();

    let result = sqlx::query!(
        r#"
        INSERT INTO users(email, full_name, password)
        VALUES ($1, $2, $3)
        "#,
        email,
        full_name,
        password_hash
    )
    .execute(pool)
    .await?;

    println!("Rows affected = {}", result.rows_affected());

    Ok(result)
}

pub async fn login(pool: &PgPool, email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let user = sqlx::query!(
        r#"SELECT id, email, password FROM users WHERE email = $1"#
        ,email
    )
    .fetch_optional(pool)
    .await?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err("Invalid email or password".into());
        }
    };

    let valid = verify(password, &user.password)?;

    if !valid {
        return Err("Invalid email or password".into());
    }

    Ok(())
}