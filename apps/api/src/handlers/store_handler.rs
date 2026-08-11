use crate::{AppState, auth::AuthUser, error::ApiError};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub full_name: String,
    pub email: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    full_name: String,
}
#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}
#[derive(Serialize)]
pub struct SessionResponse {
    token: Uuid,
}
#[derive(Serialize)]
pub struct AuthResponse {
    token: Uuid,
    user: PublicUser,
}
#[derive(Serialize, FromRow)]
pub struct PublicUser {
    id: Uuid,
    full_name: String,
    email: String,
    role: String,
}

#[derive(Serialize, FromRow)]
pub struct Product {
    id: Uuid,
    name: String,
    slug: String,
    description: String,
    category: String,
    price_cents: i64,
    stock: i32,
    image_url: Option<String>,
    featured: bool,
}
#[derive(Deserialize, Default)]
pub struct ProductFilter {
    search: Option<String>,
    category: Option<String>,
    featured: Option<bool>,
}

#[derive(Deserialize)]
pub struct AddCartItem {
    product_id: Uuid,
    quantity: i32,
}
#[derive(Deserialize)]
pub struct UpdateQuantity {
    quantity: i32,
}
#[derive(Serialize, FromRow)]
pub struct CartItem {
    product_id: Uuid,
    name: String,
    slug: String,
    price_cents: i64,
    stock: i32,
    image_url: Option<String>,
    quantity: i32,
}
#[derive(Serialize)]
pub struct Cart {
    items: Vec<CartItem>,
    item_count: i64,
    subtotal_cents: i64,
}

#[derive(Deserialize)]
pub struct CheckoutRequest {
    recipient_name: String,
    phone: String,
    shipping_address: String,
    payment_method: Option<String>,
}
#[derive(Deserialize)]
pub struct AddressInput {
    label: String,
    recipient_name: String,
    phone: String,
    address: String,
    is_default: bool,
}
#[derive(Serialize, FromRow)]
pub struct Address {
    id: Uuid,
    label: String,
    recipient_name: String,
    phone: String,
    address: String,
    is_default: bool,
}
#[derive(Serialize, FromRow)]
pub struct Order {
    id: Uuid,
    status: String,
    payment_method: String,
    payment_status: String,
    subtotal_cents: i64,
    shipping_cents: i64,
    total_cents: i64,
    recipient_name: String,
    phone: String,
    shipping_address: String,
    created_at: String,
}
#[derive(Serialize, FromRow)]
pub struct OrderItem {
    product_id: Uuid,
    product_name: String,
    unit_price_cents: i64,
    quantity: i32,
    subtotal_cents: i64,
}
#[derive(Serialize)]
pub struct OrderDetail {
    order: Order,
    items: Vec<OrderItem>,
}

fn clean(value: &str) -> &str {
    value.trim()
}

fn validate_account(full_name: &str, email: &str, password: &str) -> Result<(), ApiError> {
    if clean(full_name).len() < 2 || clean(full_name).len() > 100 {
        return Err(ApiError::bad_request(
            "Full name must contain 2 to 100 characters",
        ));
    }
    if !email.contains('@') || email.len() > 254 {
        return Err(ApiError::bad_request("Enter a valid email address"));
    }
    if password.len() < 8 || password.len() > 72 {
        return Err(ApiError::bad_request(
            "Password must contain 8 to 72 characters",
        ));
    }
    Ok(())
}

async fn create_session(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Uuid, ApiError> {
    let token = Uuid::new_v4();
    sqlx::query("INSERT INTO sessions(token, user_id) VALUES ($1, $2)")
        .bind(token)
        .bind(user_id)
        .execute(&mut **transaction)
        .await?;
    Ok(token)
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    let email = clean(&payload.email).to_lowercase();
    let full_name = clean(&payload.full_name);
    validate_account(full_name, &email, &payload.password)?;
    let password = hash(&payload.password, DEFAULT_COST).map_err(|_| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not secure password",
        )
    })?;
    let mut transaction = state.db.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO users(full_name, email, password) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(full_name)
    .bind(&email)
    .bind(password)
    .fetch_one(&mut *transaction)
    .await;
    let user_id: Uuid = match inserted {
        Ok(row) => row.get("id"),
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "An account with this email already exists",
            ));
        }
        Err(error) => return Err(error.into()),
    };
    let token = create_session(&mut transaction, user_id).await?;
    transaction.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: PublicUser {
                id: user_id,
                full_name: full_name.to_owned(),
                email,
                role: "user".to_owned(),
            },
        }),
    ))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let email = clean(&payload.email).to_lowercase();
    let row = sqlx::query(
        "SELECT id, full_name, email, password, role FROM users WHERE LOWER(email) = $1",
    )
    .bind(&email)
    .fetch_optional(&state.db)
    .await?;
    let row =
        row.ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Invalid email or password"))?;
    let password_hash: String = row.get("password");
    if !verify(&payload.password, &password_hash).unwrap_or(false) {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "Invalid email or password",
        ));
    }
    let user = PublicUser {
        id: row.get("id"),
        full_name: row.get("full_name"),
        email: row.get("email"),
        role: row.get("role"),
    };
    let mut transaction = state.db.begin().await?;
    sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW()")
        .execute(&mut *transaction)
        .await?;
    let token = create_session(&mut transaction, user.id).await?;
    transaction.commit().await?;
    Ok(Json(AuthResponse { token, user }))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(auth.id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<PublicUser>, ApiError> {
    let user = sqlx::query_as::<_, PublicUser>(
        "SELECT id, full_name, email, role FROM users WHERE id = $1",
    )
    .bind(auth.id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(user))
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<PublicUser>, ApiError> {
    let full_name = clean(&payload.full_name);
    if full_name.len() < 2 || full_name.len() > 100 {
        return Err(ApiError::bad_request(
            "Full name must contain 2 to 100 characters",
        ));
    }
    let user = sqlx::query_as::<_, PublicUser>(
        r#"
        UPDATE users SET full_name = $2 WHERE id = $1
        RETURNING id, full_name, email, role
    "#,
    )
    .bind(auth.id)
    .bind(full_name)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(user))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    if payload.new_password.len() < 8 || payload.new_password.len() > 72 {
        return Err(ApiError::bad_request(
            "New password must contain 8 to 72 characters",
        ));
    }
    if payload.current_password == payload.new_password {
        return Err(ApiError::bad_request(
            "New password must be different from the current password",
        ));
    }
    let current_hash = sqlx::query_scalar::<_, String>("SELECT password FROM users WHERE id = $1")
        .bind(auth.id)
        .fetch_one(&state.db)
        .await?;
    let valid = verify(&payload.current_password, &current_hash).unwrap_or(false);
    if !valid {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "Current password is incorrect",
        ));
    }
    let new_hash = hash(&payload.new_password, DEFAULT_COST).map_err(|_| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not secure password",
        )
    })?;
    let mut transaction = state.db.begin().await?;
    sqlx::query("UPDATE users SET password = $2 WHERE id = $1")
        .bind(auth.id)
        .bind(new_hash)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(auth.id)
        .execute(&mut *transaction)
        .await?;
    let token = create_session(&mut transaction, auth.id).await?;
    transaction.commit().await?;
    Ok(Json(SessionResponse { token }))
}

pub async fn products(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<ProductFilter>,
) -> Result<Json<Vec<Product>>, ApiError> {
    let search = filter.search.map(|value| format!("%{}%", clean(&value)));
    let products = sqlx::query_as::<_, Product>(
        r#"
        SELECT id, name, slug, description, category, price_cents, stock, image_url, featured
        FROM products
        WHERE active = TRUE
          AND ($1::TEXT IS NULL OR name ILIKE $1 OR description ILIKE $1)
          AND ($2::TEXT IS NULL OR LOWER(category) = LOWER($2))
          AND ($3::BOOLEAN IS NULL OR featured = $3)
        ORDER BY featured DESC, created_at DESC
        LIMIT 100
    "#,
    )
    .bind(search)
    .bind(filter.category)
    .bind(filter.featured)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(products))
}

pub async fn product(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Product>, ApiError> {
    sqlx::query_as::<_, Product>(
        r#"
        SELECT id, name, slug, description, category, price_cents, stock, image_url, featured
        FROM products WHERE slug = $1 AND active = TRUE
    "#,
    )
    .bind(slug)
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| ApiError::not_found("Product not found"))
}

pub async fn wishlist(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<Product>>, ApiError> {
    let products = sqlx::query_as::<_, Product>(
        r#"
        SELECT p.id, p.name, p.slug, p.description, p.category, p.price_cents,
               p.stock, p.image_url, p.featured
        FROM wishlist_items w JOIN products p ON p.id = w.product_id
        WHERE w.user_id = $1 AND p.active = TRUE
        ORDER BY w.created_at DESC
    "#,
    )
    .bind(auth.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(products))
}

pub async fn add_wishlist_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE id = $1 AND active = TRUE)")
            .bind(product_id)
            .fetch_one(&state.db)
            .await?;
    if !exists {
        return Err(ApiError::not_found("Product not found"));
    }
    sqlx::query(
        r#"
        INSERT INTO wishlist_items(user_id, product_id) VALUES ($1, $2)
        ON CONFLICT(user_id, product_id) DO NOTHING
    "#,
    )
    .bind(auth.id)
    .bind(product_id)
    .execute(&state.db)
    .await?;
    Ok(StatusCode::CREATED)
}

pub async fn remove_wishlist_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM wishlist_items WHERE user_id = $1 AND product_id = $2")
        .bind(auth.id)
        .bind(product_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn validate_address(input: &AddressInput) -> Result<(), ApiError> {
    if input.label.trim().is_empty() || input.label.trim().len() > 40 {
        return Err(ApiError::bad_request(
            "Address label is required and cannot exceed 40 characters",
        ));
    }
    if input.recipient_name.trim().len() < 2 || input.recipient_name.trim().len() > 100 {
        return Err(ApiError::bad_request(
            "Recipient name must contain 2 to 100 characters",
        ));
    }
    if input.phone.trim().len() < 7 || input.phone.trim().len() > 30 {
        return Err(ApiError::bad_request("Enter a valid phone number"));
    }
    if input.address.trim().len() < 10 || input.address.trim().len() > 500 {
        return Err(ApiError::bad_request(
            "Address must contain 10 to 500 characters",
        ));
    }
    Ok(())
}

pub async fn addresses(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<Address>>, ApiError> {
    let addresses = sqlx::query_as::<_, Address>(
        r#"
        SELECT id, label, recipient_name, phone, address, is_default
        FROM addresses WHERE user_id = $1 ORDER BY is_default DESC, created_at DESC
    "#,
    )
    .bind(auth.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(addresses))
}

pub async fn create_address(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<AddressInput>,
) -> Result<(StatusCode, Json<Address>), ApiError> {
    validate_address(&input)?;
    let mut transaction = state.db.begin().await?;
    let address_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM addresses WHERE user_id = $1")
            .bind(auth.id)
            .fetch_one(&mut *transaction)
            .await?;
    let is_default = input.is_default || address_count == 0;
    if is_default {
        sqlx::query("UPDATE addresses SET is_default = FALSE WHERE user_id = $1")
            .bind(auth.id)
            .execute(&mut *transaction)
            .await?;
    }
    let address = sqlx::query_as::<_, Address>(
        r#"
        INSERT INTO addresses(user_id, label, recipient_name, phone, address, is_default)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, label, recipient_name, phone, address, is_default
    "#,
    )
    .bind(auth.id)
    .bind(input.label.trim())
    .bind(input.recipient_name.trim())
    .bind(input.phone.trim())
    .bind(input.address.trim())
    .bind(is_default)
    .fetch_one(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(address)))
}

pub async fn update_address(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(address_id): Path<Uuid>,
    Json(input): Json<AddressInput>,
) -> Result<Json<Address>, ApiError> {
    validate_address(&input)?;
    let mut transaction = state.db.begin().await?;
    let currently_default = sqlx::query_scalar::<_, bool>(
        "SELECT is_default FROM addresses WHERE id = $1 AND user_id = $2 FOR UPDATE",
    )
    .bind(address_id)
    .bind(auth.id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::not_found("Address not found"))?;
    let is_default = input.is_default || currently_default;
    if is_default {
        sqlx::query("UPDATE addresses SET is_default = FALSE WHERE user_id = $1 AND id <> $2")
            .bind(auth.id)
            .bind(address_id)
            .execute(&mut *transaction)
            .await?;
    }
    let address = sqlx::query_as::<_, Address>(
        r#"
        UPDATE addresses SET label = $3, recipient_name = $4, phone = $5,
            address = $6, is_default = $7 WHERE id = $1 AND user_id = $2
        RETURNING id, label, recipient_name, phone, address, is_default
    "#,
    )
    .bind(address_id)
    .bind(auth.id)
    .bind(input.label.trim())
    .bind(input.recipient_name.trim())
    .bind(input.phone.trim())
    .bind(input.address.trim())
    .bind(is_default)
    .fetch_one(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Json(address))
}

pub async fn delete_address(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(address_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut transaction = state.db.begin().await?;
    let was_default = sqlx::query_scalar::<_, bool>(
        r#"
        DELETE FROM addresses WHERE id = $1 AND user_id = $2 RETURNING is_default
    "#,
    )
    .bind(address_id)
    .bind(auth.id)
    .fetch_optional(&mut *transaction)
    .await?;
    if was_default.is_none() {
        return Err(ApiError::not_found("Address not found"));
    }
    if was_default == Some(true) {
        sqlx::query(
            r#"
            UPDATE addresses SET is_default = TRUE WHERE id = (
                SELECT id FROM addresses WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1
            )
        "#,
        )
        .bind(auth.id)
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn load_cart(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Cart, ApiError> {
    let items = sqlx::query_as::<_, CartItem>(
        r#"
        SELECT p.id AS product_id, p.name, p.slug, p.price_cents, p.stock, p.image_url, ci.quantity
        FROM cart_items ci JOIN products p ON p.id = ci.product_id
        WHERE ci.user_id = $1 AND p.active = TRUE ORDER BY ci.created_at
    "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let item_count = items.iter().map(|item| i64::from(item.quantity)).sum();
    let subtotal_cents = items
        .iter()
        .map(|item| item.price_cents * i64::from(item.quantity))
        .sum();
    Ok(Cart {
        items,
        item_count,
        subtotal_cents,
    })
}

pub async fn cart(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Cart>, ApiError> {
    Ok(Json(load_cart(&state.db, auth.id).await?))
}

pub async fn add_cart_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<AddCartItem>,
) -> Result<(StatusCode, Json<Cart>), ApiError> {
    if !(1..=99).contains(&payload.quantity) {
        return Err(ApiError::bad_request("Quantity must be between 1 and 99"));
    }
    let stock: Option<i32> =
        sqlx::query_scalar("SELECT stock FROM products WHERE id = $1 AND active = TRUE")
            .bind(payload.product_id)
            .fetch_optional(&state.db)
            .await?;
    let stock = stock.ok_or_else(|| ApiError::not_found("Product not found"))?;
    let result = sqlx::query(r#"
        INSERT INTO cart_items(user_id, product_id, quantity) VALUES ($1, $2, $3)
        ON CONFLICT(user_id, product_id) DO UPDATE SET quantity = cart_items.quantity + EXCLUDED.quantity
        WHERE cart_items.quantity + EXCLUDED.quantity <= $4
    "#).bind(auth.id).bind(payload.product_id).bind(payload.quantity).bind(stock.min(99))
        .execute(&state.db).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::bad_request(
            "Requested quantity exceeds available stock",
        ));
    }
    Ok((
        StatusCode::CREATED,
        Json(load_cart(&state.db, auth.id).await?),
    ))
}

pub async fn update_cart_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<UpdateQuantity>,
) -> Result<Json<Cart>, ApiError> {
    if payload.quantity == 0 {
        sqlx::query("DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2")
            .bind(auth.id)
            .bind(product_id)
            .execute(&state.db)
            .await?;
    } else {
        if !(1..=99).contains(&payload.quantity) {
            return Err(ApiError::bad_request("Quantity must be between 0 and 99"));
        }
        let result = sqlx::query(
            r#"
            UPDATE cart_items SET quantity = $3
            WHERE user_id = $1 AND product_id = $2
              AND $3 <= (SELECT stock FROM products WHERE id = $2 AND active = TRUE)
        "#,
        )
        .bind(auth.id)
        .bind(product_id)
        .bind(payload.quantity)
        .execute(&state.db)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ApiError::bad_request(
                "Cart item was not found or stock is insufficient",
            ));
        }
    }
    Ok(Json(load_cart(&state.db, auth.id).await?))
}

pub async fn remove_cart_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2")
        .bind(auth.id)
        .bind(product_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn checkout(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CheckoutRequest>,
) -> Result<(StatusCode, Json<OrderDetail>), ApiError> {
    if clean(&payload.recipient_name).len() < 2
        || clean(&payload.phone).len() < 7
        || clean(&payload.shipping_address).len() < 10
    {
        return Err(ApiError::bad_request(
            "Complete recipient name, phone, and shipping address are required",
        ));
    }
    let mut tx = state.db.begin().await?;
    let items = sqlx::query_as::<_, CartItem>(
        r#"
        SELECT p.id AS product_id, p.name, p.slug, p.price_cents, p.stock, p.image_url, ci.quantity
        FROM cart_items ci JOIN products p ON p.id = ci.product_id
        WHERE ci.user_id = $1 AND p.active = TRUE ORDER BY ci.created_at FOR UPDATE OF p
    "#,
    )
    .bind(auth.id)
    .fetch_all(&mut *tx)
    .await?;
    if items.is_empty() {
        return Err(ApiError::bad_request("Your cart is empty"));
    }
    if let Some(item) = items.iter().find(|item| item.quantity > item.stock) {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            format!("Insufficient stock for {}", item.name),
        ));
    }
    let subtotal: i64 = items
        .iter()
        .map(|item| item.price_cents * i64::from(item.quantity))
        .sum();
    let payment_method = payload.payment_method.as_deref().unwrap_or("bank_transfer");
    if !["bank_transfer", "credit_card", "cash_on_delivery"].contains(&payment_method) {
        return Err(ApiError::bad_request("Unsupported payment method"));
    }
    let payment_status = if payment_method == "credit_card" {
        "paid"
    } else {
        "pending"
    };
    let shipping_cents = if subtotal >= 1_000_000 { 0 } else { 30_000 };
    let total = subtotal + shipping_cents;
    let order_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO orders(
            user_id, subtotal_cents, shipping_cents, total_cents, payment_method,
            payment_status, recipient_name, phone, shipping_address
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id
    "#,
    )
    .bind(auth.id)
    .bind(subtotal)
    .bind(shipping_cents)
    .bind(total)
    .bind(payment_method)
    .bind(payment_status)
    .bind(clean(&payload.recipient_name))
    .bind(clean(&payload.phone))
    .bind(clean(&payload.shipping_address))
    .fetch_one(&mut *tx)
    .await?;
    for item in &items {
        let subtotal = item.price_cents * i64::from(item.quantity);
        sqlx::query(r#"
            INSERT INTO order_items(order_id, product_id, product_name, unit_price_cents, quantity, subtotal_cents)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#).bind(order_id).bind(item.product_id).bind(&item.name).bind(item.price_cents)
            .bind(item.quantity).bind(subtotal).execute(&mut *tx).await?;
        sqlx::query("UPDATE products SET stock = stock - $1 WHERE id = $2")
            .bind(item.quantity)
            .bind(item.product_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
        .bind(auth.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let detail = load_order(&state.db, auth.id, order_id).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

pub async fn orders(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<Order>>, ApiError> {
    let orders = sqlx::query_as::<_, Order>(
        r#"
        SELECT id, status, payment_method, payment_status, subtotal_cents,
               shipping_cents, total_cents, recipient_name, phone, shipping_address,
               TO_CHAR(created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM orders WHERE user_id = $1 ORDER BY created_at DESC
    "#,
    )
    .bind(auth.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(orders))
}

async fn load_order(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    order_id: Uuid,
) -> Result<OrderDetail, ApiError> {
    let order = sqlx::query_as::<_, Order>(
        r#"
        SELECT id, status, payment_method, payment_status, subtotal_cents,
               shipping_cents, total_cents, recipient_name, phone, shipping_address,
               TO_CHAR(created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM orders WHERE id = $1 AND user_id = $2
    "#,
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = sqlx::query_as::<_, OrderItem>(
        r#"
        SELECT product_id, product_name, unit_price_cents, quantity, subtotal_cents
        FROM order_items WHERE order_id = $1 ORDER BY id
    "#,
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?;
    Ok(OrderDetail { order, items })
}

pub async fn order(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderDetail>, ApiError> {
    Ok(Json(load_order(&state.db, auth.id, order_id).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_validation_rejects_weak_input() {
        assert!(validate_account("A", "invalid", "short").is_err());
        assert!(validate_account("Claudia User", "user@example.com", "password123").is_ok());
    }
}
