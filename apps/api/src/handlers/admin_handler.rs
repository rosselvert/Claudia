use crate::{AppState, auth::AdminUser, error::ApiError};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct DashboardMetrics {
    revenue_cents: i64,
    order_count: i64,
    customer_count: i64,
    active_product_count: i64,
    low_stock_count: i64,
}

#[derive(Serialize, FromRow)]
pub struct AdminProduct {
    id: Uuid,
    name: String,
    slug: String,
    description: String,
    category: String,
    price_cents: i64,
    stock: i32,
    image_url: Option<String>,
    featured: bool,
    active: bool,
}

#[derive(Deserialize)]
pub struct ProductInput {
    name: String,
    slug: String,
    description: String,
    category: String,
    price_cents: i64,
    stock: i32,
    image_url: Option<String>,
    featured: bool,
    active: bool,
}

#[derive(Serialize, FromRow)]
pub struct AdminOrder {
    id: Uuid,
    customer_name: String,
    customer_email: String,
    status: String,
    payment_method: String,
    payment_status: String,
    subtotal_cents: i64,
    shipping_cents: i64,
    total_cents: i64,
    recipient_name: String,
    shipping_address: String,
    created_at: String,
}

#[derive(Deserialize)]
pub struct OrderStatusInput {
    status: String,
}

#[derive(Deserialize)]
pub struct PaymentStatusInput {
    status: String,
}

#[derive(Serialize, FromRow)]
pub struct AdminCustomer {
    id: Uuid,
    full_name: String,
    email: String,
    role: String,
    order_count: i64,
    total_spent_cents: i64,
    created_at: String,
}

#[derive(Deserialize)]
pub struct CustomerRoleInput {
    role: String,
}

#[derive(Serialize, FromRow)]
pub struct AdminOrderItem {
    product_id: Uuid,
    product_name: String,
    unit_price_cents: i64,
    quantity: i32,
    subtotal_cents: i64,
}

#[derive(Serialize)]
pub struct AdminOrderDetail {
    order: AdminOrder,
    items: Vec<AdminOrderItem>,
}

fn validate_product(input: &ProductInput) -> Result<(), ApiError> {
    if input.name.trim().len() < 2 || input.name.len() > 160 {
        return Err(ApiError::bad_request(
            "Product name must contain 2 to 160 characters",
        ));
    }
    if input.slug.is_empty()
        || input.slug.len() > 180
        || !input.slug.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err(ApiError::bad_request(
            "Slug may only contain lowercase letters, numbers, and hyphens",
        ));
    }
    if input.category.trim().is_empty() || input.description.trim().is_empty() {
        return Err(ApiError::bad_request(
            "Category and description are required",
        ));
    }
    if input.price_cents < 0 || input.stock < 0 {
        return Err(ApiError::bad_request("Price and stock cannot be negative"));
    }
    Ok(())
}

pub async fn metrics(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<DashboardMetrics>, ApiError> {
    let revenue_cents = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(total_cents), 0)::BIGINT FROM orders WHERE status <> 'cancelled' AND payment_status = 'paid'",
    )
    .fetch_one(&state.db)
    .await?;
    let order_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders")
        .fetch_one(&state.db)
        .await?;
    let customer_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE role = 'user'")
            .fetch_one(&state.db)
            .await?;
    let active_product_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM products WHERE active = TRUE")
            .fetch_one(&state.db)
            .await?;
    let low_stock_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM products WHERE active = TRUE AND stock <= 5",
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(DashboardMetrics {
        revenue_cents,
        order_count,
        customer_count,
        active_product_count,
        low_stock_count,
    }))
}

pub async fn products(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<Vec<AdminProduct>>, ApiError> {
    let products = sqlx::query_as::<_, AdminProduct>(r#"
        SELECT id, name, slug, description, category, price_cents, stock, image_url, featured, active
        FROM products ORDER BY created_at DESC
    "#).fetch_all(&state.db).await?;
    Ok(Json(products))
}

pub async fn create_product(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Json(input): Json<ProductInput>,
) -> Result<(StatusCode, Json<AdminProduct>), ApiError> {
    validate_product(&input)?;
    let product = sqlx::query_as::<_, AdminProduct>(r#"
        INSERT INTO products(name, slug, description, category, price_cents, stock, image_url, featured, active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, name, slug, description, category, price_cents, stock, image_url, featured, active
    "#).bind(input.name.trim()).bind(&input.slug).bind(input.description.trim())
        .bind(input.category.trim()).bind(input.price_cents).bind(input.stock)
        .bind(input.image_url.filter(|url| !url.trim().is_empty())).bind(input.featured)
        .bind(input.active).fetch_one(&state.db).await.map_err(map_product_error)?;
    Ok((StatusCode::CREATED, Json(product)))
}

pub async fn update_product(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(product_id): Path<Uuid>,
    Json(input): Json<ProductInput>,
) -> Result<Json<AdminProduct>, ApiError> {
    validate_product(&input)?;
    let product = sqlx::query_as::<_, AdminProduct>(r#"
        UPDATE products SET name = $2, slug = $3, description = $4, category = $5,
            price_cents = $6, stock = $7, image_url = $8, featured = $9, active = $10
        WHERE id = $1
        RETURNING id, name, slug, description, category, price_cents, stock, image_url, featured, active
    "#).bind(product_id).bind(input.name.trim()).bind(&input.slug).bind(input.description.trim())
        .bind(input.category.trim()).bind(input.price_cents).bind(input.stock)
        .bind(input.image_url.filter(|url| !url.trim().is_empty())).bind(input.featured)
        .bind(input.active).fetch_optional(&state.db).await.map_err(map_product_error)?
        .ok_or_else(|| ApiError::not_found("Product not found"))?;
    Ok(Json(product))
}

fn map_product_error(error: sqlx::Error) -> ApiError {
    if matches!(&error, sqlx::Error::Database(database) if database.code().as_deref() == Some("23505"))
    {
        ApiError::new(
            StatusCode::CONFLICT,
            "A product with this slug already exists",
        )
    } else {
        error.into()
    }
}

pub async fn archive_product(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("UPDATE products SET active = FALSE WHERE id = $1")
        .bind(product_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Product not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn orders(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<Vec<AdminOrder>>, ApiError> {
    let orders = sqlx::query_as::<_, AdminOrder>(
        r#"
        SELECT o.id, u.full_name AS customer_name, u.email AS customer_email, o.status,
               o.payment_method, o.payment_status, o.subtotal_cents, o.shipping_cents,
               o.total_cents, o.recipient_name, o.shipping_address,
               TO_CHAR(o.created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM orders o JOIN users u ON u.id = o.user_id ORDER BY o.created_at DESC LIMIT 250
    "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(orders))
}

pub async fn order(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<AdminOrderDetail>, ApiError> {
    let order = sqlx::query_as::<_, AdminOrder>(
        r#"
        SELECT o.id, u.full_name AS customer_name, u.email AS customer_email, o.status,
               o.payment_method, o.payment_status, o.subtotal_cents, o.shipping_cents,
               o.total_cents, o.recipient_name, o.shipping_address,
               TO_CHAR(o.created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM orders o JOIN users u ON u.id = o.user_id WHERE o.id = $1
    "#,
    )
    .bind(order_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = sqlx::query_as::<_, AdminOrderItem>(
        r#"
        SELECT product_id, product_name, unit_price_cents, quantity, subtotal_cents
        FROM order_items WHERE order_id = $1 ORDER BY id
    "#,
    )
    .bind(order_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(AdminOrderDetail { order, items }))
}

pub async fn update_order_status(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(order_id): Path<Uuid>,
    Json(input): Json<OrderStatusInput>,
) -> Result<Json<AdminOrder>, ApiError> {
    const STATUSES: [&str; 5] = [
        "confirmed",
        "processing",
        "shipped",
        "delivered",
        "cancelled",
    ];
    if !STATUSES.contains(&input.status.as_str()) {
        return Err(ApiError::bad_request("Unsupported order status"));
    }
    let mut transaction = state.db.begin().await?;
    let current_status =
        sqlx::query_scalar::<_, String>("SELECT status FROM orders WHERE id = $1 FOR UPDATE")
            .bind(order_id)
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or_else(|| ApiError::not_found("Order not found"))?;
    if current_status == "cancelled" && input.status != "cancelled" {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "Cancelled orders cannot be reopened",
        ));
    }
    if input.status == "cancelled" && current_status != "cancelled" {
        sqlx::query(
            r#"
            UPDATE products p SET stock = p.stock + quantities.quantity
            FROM (
                SELECT product_id, SUM(quantity)::INTEGER AS quantity
                FROM order_items WHERE order_id = $1 GROUP BY product_id
            ) quantities
            WHERE p.id = quantities.product_id
        "#,
        )
        .bind(order_id)
        .execute(&mut *transaction)
        .await?;
    }
    let order = sqlx::query_as::<_, AdminOrder>(
        r#"
        WITH updated AS (UPDATE orders SET status = $2 WHERE id = $1 RETURNING *)
        SELECT o.id, u.full_name AS customer_name, u.email AS customer_email, o.status,
               o.payment_method, o.payment_status, o.subtotal_cents, o.shipping_cents,
               o.total_cents, o.recipient_name, o.shipping_address,
               TO_CHAR(o.created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM updated o JOIN users u ON u.id = o.user_id
    "#,
    )
    .bind(order_id)
    .bind(input.status)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::not_found("Order not found"))?;
    transaction.commit().await?;
    Ok(Json(order))
}

pub async fn update_payment_status(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(order_id): Path<Uuid>,
    Json(input): Json<PaymentStatusInput>,
) -> Result<Json<AdminOrder>, ApiError> {
    if !["pending", "paid", "refunded"].contains(&input.status.as_str()) {
        return Err(ApiError::bad_request("Unsupported payment status"));
    }
    let order = sqlx::query_as::<_, AdminOrder>(
        r#"
        WITH updated AS (
            UPDATE orders SET payment_status = $2 WHERE id = $1 RETURNING *
        )
        SELECT o.id, u.full_name AS customer_name, u.email AS customer_email, o.status,
               o.payment_method, o.payment_status, o.subtotal_cents, o.shipping_cents,
               o.total_cents, o.recipient_name, o.shipping_address,
               TO_CHAR(o.created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM updated o JOIN users u ON u.id = o.user_id
    "#,
    )
    .bind(order_id)
    .bind(input.status)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Order not found"))?;
    Ok(Json(order))
}

pub async fn customers(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<Vec<AdminCustomer>>, ApiError> {
    let customers = sqlx::query_as::<_, AdminCustomer>(
        r#"
        SELECT u.id, u.full_name, u.email, u.role, COUNT(o.id) AS order_count,
               COALESCE(SUM(o.total_cents) FILTER (
                   WHERE o.status <> 'cancelled' AND o.payment_status = 'paid'
               ), 0)::BIGINT AS total_spent_cents,
               TO_CHAR(u.created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM users u LEFT JOIN orders o ON o.user_id = u.id
        GROUP BY u.id ORDER BY u.created_at DESC
    "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(customers))
}

pub async fn update_customer_role(
    State(state): State<Arc<AppState>>,
    admin: AdminUser,
    Path(customer_id): Path<Uuid>,
    Json(input): Json<CustomerRoleInput>,
) -> Result<Json<AdminCustomer>, ApiError> {
    if !["user", "admin"].contains(&input.role.as_str()) {
        return Err(ApiError::bad_request("Role must be user or admin"));
    }
    if admin.id == customer_id && input.role != "admin" {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "You cannot remove your own administrator access",
        ));
    }
    let mut transaction = state.db.begin().await?;
    let current_role =
        sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = $1 FOR UPDATE")
            .bind(customer_id)
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or_else(|| ApiError::not_found("Customer not found"))?;
    if current_role == "admin" && input.role == "user" {
        let admin_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE role = 'admin'")
                .fetch_one(&mut *transaction)
                .await?;
        if admin_count <= 1 {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "At least one administrator must remain",
            ));
        }
    }
    sqlx::query("UPDATE users SET role = $2 WHERE id = $1")
        .bind(customer_id)
        .bind(&input.role)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;

    let customer = sqlx::query_as::<_, AdminCustomer>(
        r#"
        SELECT u.id, u.full_name, u.email, u.role, COUNT(o.id) AS order_count,
               COALESCE(SUM(o.total_cents) FILTER (
                   WHERE o.status <> 'cancelled' AND o.payment_status = 'paid'
               ), 0)::BIGINT AS total_spent_cents,
               TO_CHAR(u.created_at, 'YYYY-MM-DD"T"HH24:MI:SSTZH:TZM') AS created_at
        FROM users u LEFT JOIN orders o ON o.user_id = u.id WHERE u.id = $1
        GROUP BY u.id
    "#,
    )
    .bind(customer_id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(customer))
}
