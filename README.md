<img width="1710" height="979" alt="Claudia ecommerce storefront" src="https://github.com/user-attachments/assets/740f854a-6a30-4426-ad3d-262882ef9362" />

# Claudia - E-Commerce Platform

Claudia is a full-stack ecommerce application with a Rust backend and React
frontend. It includes a storefront, customer accounts, wishlists, an address
book, persistent carts, transactional checkout, simulated payments, order
history, role-based access control, and an admin operations dashboard.

## Tech Stack

- Backend: Rust 2024, Axum 0.8, Tokio, SQLx 0.9
- Database: PostgreSQL
- Frontend: React, React Router, Tailwind CSS 4, Vite
- Authentication: opaque bearer sessions stored in PostgreSQL
- Testing: Cargo test, Clippy, ESLint, Playwright
- CI: GitHub Actions

## Features

### Customers

- Registration, login, logout, and 30-day sessions
- Searchable and filterable product catalog
- Product details, featured products, and stock visibility
- Persistent wishlist and cart
- Address book with one default address
- Transactional checkout with stock locking
- Bank transfer, simulated credit card, and cash-on-delivery payments
- Free delivery for subtotals of at least Rp1,000,000
- Rp30,000 delivery fee below the free-delivery threshold
- Order history and line-item details
- Profile editing and password changes with session rotation

### Administrators

- Revenue, order, customer, active-product, and low-stock metrics
- Product creation, editing, featuring, archiving, pricing, images, and stock
- Customer list and `user` / `admin` role management
- Full order list and line-item details
- Fulfillment states: `confirmed`, `processing`, `shipped`, `delivered`, `cancelled`
- Payment states: `pending`, `paid`, `refunded`
- Order cancellation returns stock exactly once
- Revenue only includes `paid`, non-cancelled orders

## Project Structure

```text
claudia/
├── apps/
│   ├── api/                 # Axum binary and HTTP handlers
│   └── web/                 # React, Tailwind CSS, and Vite
├── crates/
│   ├── database/            # PostgreSQL connection pool
│   ├── models/              # Shared Rust models
│   ├── service/             # Domain services
│   ├── repository/          # Repository crate
│   └── auth/                # Authentication crate
├── migrations/              # Versioned SQLx migrations and product seed data
├── postman/                 # Postman collection
├── docker-compose.yml       # Development PostgreSQL service
└── .github/workflows/ci.yml # Backend and frontend CI
```

## Requirements

- Rust stable
- Node.js 22 or the latest LTS release
- npm
- PostgreSQL 14+ or Docker

The application uses port `5001` by default. Port `5000` is avoided because it
is commonly occupied by Control Center / AirPlay Receiver on macOS.

## Running Locally

### 1. Configure the environment

```bash
cp .env.example .env
```

Default values:

```dotenv
DATABASE_URL=postgres://postgres:postgres@localhost:5432/claudia
HOST=127.0.0.1
PORT=5001
FRONTEND_DIR=apps/web/dist
```

### 2. Start PostgreSQL

With Docker:

```bash
docker compose up -d --wait
```

Alternatively, use a local PostgreSQL instance and update `DATABASE_URL` in
`.env`.

### 3. Install and build the frontend

```bash
npm install --prefix apps/web
npm run build --prefix apps/web
```

### 4. Start the API

```bash
cargo run -p api
```

SQLx automatically runs migrations and seeds demo products during startup.

Application URLs:

- Storefront: http://127.0.0.1:5001
- Customer account: http://127.0.0.1:5001/account
- Admin dashboard: http://127.0.0.1:5001/admin
- API: http://127.0.0.1:5001/api/v1

## Frontend Development

Run Axum and Vite in separate terminals:

```bash
# Terminal 1
cargo run -p api

# Terminal 2
npm run dev --prefix apps/web
```

Vite runs at `http://127.0.0.1:5173` and proxies `/api` requests to Axum on
port `5001`.

For a local production build, run `npm run build --prefix apps/web` and open
the Axum server on port `5001`. Axum serves `apps/web/dist` and falls back to
`index.html` for the `/account` and `/admin` client routes.

## Creating the First Administrator

Public registration always creates a `user` account. After registering through
the UI, promote a trusted account in PostgreSQL:

```sql
UPDATE users
SET role = 'admin'
WHERE email = 'owner@example.com';
```

When PostgreSQL is running through Docker:

```bash
docker compose exec postgres psql -U postgres -d claudia \
  -c "UPDATE users SET role = 'admin' WHERE email = 'owner@example.com';"
```

The API verifies the role from the database on every `/api/v1/admin/*` request.
Changing browser storage or navigating directly to `/admin` does not grant
administrator access.

## Authentication

Registration and login return an opaque UUID session token:

```json
{
  "token": "session-uuid",
  "user": {
    "id": "user-uuid",
    "full_name": "Claudia User",
    "email": "user@example.com",
    "role": "user"
  }
}
```

Send the token to protected endpoints:

```http
Authorization: Bearer <token>
```

Changing a password verifies the previous password, revokes all old sessions,
and issues a new token for the active device.

## API Reference

All endpoints use the `/api/v1` prefix.

### Public and Authentication

| Method | Endpoint | Authentication | Description |
| --- | --- | --- | --- |
| GET | `/health` | No | Liveness check |
| POST | `/auth/register` | No | Create an account and session |
| POST | `/auth/login` | No | Create a session |
| POST | `/auth/logout` | Bearer | Revoke account sessions |
| GET | `/products` | No | List and filter products |
| GET | `/products/{slug}` | No | Product details |

Product filters are `search`, `category`, and `featured`:

```bash
curl 'http://127.0.0.1:5001/api/v1/products?category=Home&featured=true'
```

### Customer Account

| Method | Endpoint | Authentication | Description |
| --- | --- | --- | --- |
| GET | `/me` | Bearer | Current profile |
| PATCH | `/me` | Bearer | Update the customer name |
| POST | `/me/password` | Bearer | Change password and rotate sessions |
| GET | `/wishlist` | Bearer | List saved products |
| POST | `/wishlist/{product_id}` | Bearer | Save a product |
| DELETE | `/wishlist/{product_id}` | Bearer | Remove a saved product |
| GET | `/addresses` | Bearer | List saved addresses |
| POST | `/addresses` | Bearer | Create an address |
| PUT | `/addresses/{address_id}` | Bearer | Update or set the default address |
| DELETE | `/addresses/{address_id}` | Bearer | Delete an address |

### Cart and Orders

| Method | Endpoint | Authentication | Description |
| --- | --- | --- | --- |
| GET | `/cart` | Bearer | Cart items and subtotal |
| POST | `/cart/items` | Bearer | Add an item |
| PUT | `/cart/items/{product_id}` | Bearer | Set quantity; `0` removes the item |
| DELETE | `/cart/items/{product_id}` | Bearer | Remove an item |
| POST | `/checkout` | Bearer | Atomically create an order |
| GET | `/orders` | Bearer | Order history |
| GET | `/orders/{order_id}` | Bearer | Order and line-item details |

Example checkout payload:

```json
{
  "recipient_name": "Claudia User",
  "phone": "081234567890",
  "shipping_address": "Jl. Merdeka No. 10, Jakarta",
  "payment_method": "bank_transfer"
}
```

Supported payment methods:

- `bank_transfer`, initially `pending`
- `credit_card`, simulated as immediately `paid`
- `cash_on_delivery`, initially `pending`

Checkout locks product rows and performs stock validation, immutable product
and price snapshots, stock deduction, delivery calculation, order creation,
and cart clearing in one PostgreSQL transaction.

### Administration

Every admin endpoint requires a bearer session with the `admin` role.

| Method | Endpoint | Description |
| --- | --- | --- |
| GET | `/admin/metrics` | Revenue and operational metrics |
| GET | `/admin/products` | All products, including archived products |
| POST | `/admin/products` | Create a product |
| PUT | `/admin/products/{product_id}` | Update a product |
| DELETE | `/admin/products/{product_id}` | Soft-archive a product |
| GET | `/admin/orders` | List all orders |
| GET | `/admin/orders/{order_id}` | Order and line-item details |
| PATCH | `/admin/orders/{order_id}/status` | Update fulfillment status |
| PATCH | `/admin/orders/{order_id}/payment` | Update payment status |
| GET | `/admin/customers` | Customer list and activity |
| PATCH | `/admin/customers/{customer_id}/role` | Update a role |

Administrators cannot remove their own administrator access or demote the last
remaining administrator.

## Money Representation

Prices are stored as integers to avoid floating-point rounding. Field names use
the `_cents` suffix for internal consistency, but values represent whole
Indonesian rupiah. For example, `499000` means Rp499,000.

Main order fields:

- `subtotal_cents`: line-item total before delivery
- `shipping_cents`: delivery fee
- `total_cents`: subtotal plus delivery

## Quality Checks

Backend:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Frontend:

```bash
npm run lint --prefix apps/web
npm run build --prefix apps/web
```

The admin end-to-end test requires a running API, a migrated database, and the
development `admin@claudia.local` account:

```bash
npm run test:e2e --prefix apps/web
```

GitHub Actions runs formatting, unit tests, Clippy, ESLint, and a production
frontend build on every push and pull request.

## Troubleshooting

### Port 5000 returns 403

macOS Control Center may occupy port `5000`. Claudia uses port `5001` by
default. Ensure the Vite proxy and browser URL both use `5001`.

### Direct navigation to `/account` or `/admin` returns 404

Build the frontend first:

```bash
npm run build --prefix apps/web
```

Axum uses `ServeDir` with a fallback to `dist/index.html` for client-side
routing.

### Docker cannot find `docker-credential-desktop`

The Docker CLI may be installed without the Docker Desktop credential helper.
Start Docker Desktop or use a local PostgreSQL instance and update
`DATABASE_URL`.

### A legacy database already contains the `users` table

The initial migration uses `IF NOT EXISTS` and can adopt a compatible legacy
`users` table. Later migrations add roles, wishlists, addresses, payments, and
the remaining ecommerce schema.

### The admin dashboard shows a blank page

An older API version returned timezone offsets as `+00`, which browsers could
not parse consistently. Timestamps now use valid ISO-8601 offsets such as
`+00:00`. Perform a hard refresh if the browser still has an older bundle.
