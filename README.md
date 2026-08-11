<img width="1710" height="979" alt="Screenshot 2026-08-12 at 06 01 14" src="https://github.com/user-attachments/assets/740f854a-6a30-4426-ad3d-262882ef9362" />


# Claudia Commerce API

Claudia is a small, production-minded ecommerce application built with Rust,
Axum, SQLx, PostgreSQL, React, and Tailwind CSS. It includes a responsive storefront, account
sessions, a searchable seeded catalog, persistent carts, transactional
stock-aware checkout, and order history.

All monetary values are integer Indonesian rupiah amounts in `price_cents` and
`total_cents` fields. Integers avoid floating-point rounding errors.

## Run locally

Requirements: Rust, Node.js, npm, and Docker.

```bash
cp .env.example .env
docker compose up -d
npm install --prefix apps/web
npm run build --prefix apps/web
cargo run -p api
```

Database migrations and demo product seeding run automatically at startup. The
storefront opens at `http://127.0.0.1:5001` and the API listens under
`http://127.0.0.1:5001/api/v1` by default. Axum serves the production React
build from `apps/web/dist` and falls back to `index.html` for client routes.

For frontend development with hot module replacement, run the API and Vite in
separate terminals. Vite proxies `/api` requests to Axum:

```bash
cargo run -p api
npm run dev --prefix apps/web
```

Customer accounts are available at `/account`. The administrator dashboard is
available at `/admin` and includes revenue metrics, product and stock editing,
customer activity, orders, and fulfillment status controls.

New registrations always receive the `user` role. Promote an existing account
from PostgreSQL when assigning the first trusted administrator:

```sql
UPDATE users SET role = 'admin' WHERE email = 'owner@example.com';
```

Role checks are enforced by the API on every `/api/v1/admin/*` request; changing
frontend storage or navigating directly to `/admin` does not grant access.

## API

| Method | Endpoint | Authentication | Description |
| --- | --- | --- | --- |
| GET | `/health` | No | Liveness check |
| POST | `/auth/register` | No | Create account and session |
| POST | `/auth/login` | No | Create session |
| POST | `/auth/logout` | Bearer | Revoke account sessions |
| GET | `/me` | Bearer | Current profile |
| PATCH | `/me` | Bearer | Update customer profile |
| POST | `/me/password` | Bearer | Change password and rotate sessions |
| GET | `/products` | No | Catalog with optional filters |
| GET | `/products/{slug}` | No | Product detail |
| GET | `/wishlist` | Bearer | Saved products |
| POST | `/wishlist/{product_id}` | Bearer | Save product |
| DELETE | `/wishlist/{product_id}` | Bearer | Remove saved product |
| GET | `/addresses` | Bearer | Customer address book |
| POST | `/addresses` | Bearer | Save address |
| PUT | `/addresses/{address_id}` | Bearer | Update or set default address |
| DELETE | `/addresses/{address_id}` | Bearer | Remove address |
| GET | `/cart` | Bearer | Cart and calculated totals |
| POST | `/cart/items` | Bearer | Add product to cart |
| PUT | `/cart/items/{product_id}` | Bearer | Set quantity; zero removes it |
| DELETE | `/cart/items/{product_id}` | Bearer | Remove cart item |
| POST | `/checkout` | Bearer | Atomically create confirmed order |
| GET | `/orders` | Bearer | Order history |
| GET | `/orders/{order_id}` | Bearer | Order with line items |

Product filters are `search`, `category`, and `featured`, for example:

```bash
curl 'http://127.0.0.1:5001/api/v1/products?category=Home&featured=true'
```

Register and retain the returned session token:

```bash
curl -X POST http://127.0.0.1:5001/api/v1/auth/register \
  -H 'content-type: application/json' \
  -d '{"full_name":"Claudia User","email":"user@example.com","password":"password123"}'
```

Use it on protected endpoints as `Authorization: Bearer <token>`.

Add an item, then checkout:

```bash
curl -X POST http://127.0.0.1:5001/api/v1/cart/items \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer <token>' \
  -d '{"product_id":"<product-id>","quantity":2}'

curl -X POST http://127.0.0.1:5001/api/v1/checkout \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer <token>' \
  -d '{"recipient_name":"Claudia User","phone":"081234567890","shipping_address":"Jl. Merdeka No. 10, Jakarta"}'
```

Checkout locks product rows and executes stock validation, immutable line-item
snapshots, stock deduction, order creation, and cart clearing in one transaction.

## Quality checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
npm run lint --prefix apps/web
npm run build --prefix apps/web
```
