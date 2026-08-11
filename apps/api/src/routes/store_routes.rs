use crate::{
    AppState,
    handlers::{admin_handler as admin, store_handler as handler},
};
use axum::{
    Router,
    routing::{get, patch, post, put},
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/register", post(handler::register))
        .route("/login", post(handler::login))
        .route("/auth/register", post(handler::register))
        .route("/auth/login", post(handler::login))
        .route("/auth/logout", post(handler::logout))
        .route("/me", get(handler::me).patch(handler::update_profile))
        .route("/me/password", post(handler::change_password))
        .route("/products", get(handler::products))
        .route("/products/{slug}", get(handler::product))
        .route("/wishlist", get(handler::wishlist))
        .route(
            "/wishlist/{product_id}",
            post(handler::add_wishlist_item).delete(handler::remove_wishlist_item),
        )
        .route(
            "/addresses",
            get(handler::addresses).post(handler::create_address),
        )
        .route(
            "/addresses/{address_id}",
            put(handler::update_address).delete(handler::delete_address),
        )
        .route("/cart", get(handler::cart))
        .route("/cart/items", post(handler::add_cart_item))
        .route(
            "/cart/items/{product_id}",
            put(handler::update_cart_item).delete(handler::remove_cart_item),
        )
        .route("/checkout", post(handler::checkout))
        .route("/orders", get(handler::orders))
        .route("/orders/{order_id}", get(handler::order))
        .route("/admin/metrics", get(admin::metrics))
        .route(
            "/admin/products",
            get(admin::products).post(admin::create_product),
        )
        .route(
            "/admin/products/{product_id}",
            put(admin::update_product).delete(admin::archive_product),
        )
        .route("/admin/orders", get(admin::orders))
        .route("/admin/orders/{order_id}", get(admin::order))
        .route(
            "/admin/orders/{order_id}/status",
            patch(admin::update_order_status),
        )
        .route(
            "/admin/orders/{order_id}/payment",
            patch(admin::update_payment_status),
        )
        .route("/admin/customers", get(admin::customers))
        .route(
            "/admin/customers/{customer_id}/role",
            patch(admin::update_customer_role),
        )
}
