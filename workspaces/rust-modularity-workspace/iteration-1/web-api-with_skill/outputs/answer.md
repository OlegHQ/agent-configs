# Organizing a Rust Axum Web API

## Step 0: Evaluate Architecture

Working through the decision tree:

1. **Are components independently publishable?** No — users, products, orders, and admin all share your DB pool, auth state, and error types. Stay single-crate.

2. **Does the code flow through clear sequential phases?** No — HTTP request handling is not a pipeline. Ruling out phase-pipeline.

3. **Are there distinct domains that own their own types and logic?** Yes, partially — users, products, orders, admin are distinct domains, but they share a request/response boundary and a storage layer.

4. **Is there a clear request/response → logic → storage boundary?** Yes. Each request goes through: HTTP handler → business logic → SQL query.

**Verdict: Hybrid — Domain Components (Pattern A) with internal layering per domain.**

The critical mistake to avoid here is applying *pure* layered architecture (Pattern D) across all domains at the top level. That would give you a `handlers/` bag with 4 domains mixed together, a `service/` bag with 4 domains, and a `storage/` bag with 4 domains — the god-module problem recreated at the directory level. Instead, split by domain first, then layer within each domain.

---

## Target Structure

```
src/
├── main.rs              # Server setup, router assembly, state wiring
├── error.rs             # AppError, IntoResponse impl — shared across all domains
├── db.rs                # DB pool type, connection helpers — shared infrastructure
├── state.rs             # AppState (pool, config, auth keys) — passed to handlers
│
├── users/
│   ├── mod.rs           # pub use re-exports; router() fn
│   ├── routes.rs        # axum handlers: create_user, get_user, list_users, ...
│   ├── service.rs       # Business logic: validate, hash password, send email
│   ├── queries.rs       # SQL: INSERT INTO users, SELECT FROM users
│   └── types.rs         # User, CreateUserRequest, UserResponse
│
├── products/
│   ├── mod.rs
│   ├── routes.rs        # axum handlers
│   ├── queries.rs       # SQL queries
│   └── types.rs         # Product, CreateProductRequest, ProductResponse
│
├── orders/
│   ├── mod.rs
│   ├── routes.rs        # axum handlers
│   ├── service.rs       # Order fulfillment logic, inventory checks
│   ├── queries.rs       # SQL queries (orders + order_items)
│   └── types.rs         # Order, OrderItem, CreateOrderRequest
│
├── admin/
│   ├── mod.rs
│   ├── routes.rs        # axum handlers (guarded by admin middleware)
│   ├── queries.rs       # Admin-specific SQL
│   └── types.rs
│
└── auth.rs              # JWT validation, middleware, AuthUser extractor
```

---

## What Goes Where

### `main.rs` — wiring only

```rust
mod auth;
mod db;
mod error;
mod orders;
mod products;
mod state;
mod users;
mod admin;

#[tokio::main]
async fn main() {
    let pool = db::create_pool(&config).await?;
    let state = AppState { pool, config };

    let app = Router::new()
        .nest("/users",    users::router())
        .nest("/products", products::router())
        .nest("/orders",   orders::router())
        .nest("/admin",    admin::router())
        .with_state(state);

    axum::serve(listener, app).await?;
}
```

`main.rs` has no handlers, no SQL, no business logic. It assembles.

### `users/mod.rs` — table of contents + router

```rust
mod queries;
mod routes;
mod service;
pub mod types;

pub use types::{User, CreateUserRequest};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",    post(routes::create_user).get(routes::list_users))
        .route("/:id", get(routes::get_user).put(routes::update_user).delete(routes::delete_user))
}
```

### `users/routes.rs` — handlers only

Handlers extract from the request, call service or queries, return responses. No SQL inline, no business logic inline.

```rust
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = service::create_user(&state.pool, req).await?;
    Ok((StatusCode::CREATED, Json(user)))
}
```

### `users/service.rs` — business logic

Validation, password hashing, calling external services, composing multiple queries into a transaction. No axum types (`State`, `Json`, `StatusCode`) in here.

```rust
pub async fn create_user(pool: &PgPool, req: CreateUserRequest) -> Result<User, AppError> {
    validate_email(&req.email)?;
    let hash = hash_password(&req.password)?;
    queries::insert_user(pool, &req.email, &hash).await
}
```

### `users/queries.rs` — SQL only

```rust
pub async fn insert_user(pool: &PgPool, email: &str, hash: &str) -> Result<User, AppError> {
    sqlx::query_as!(User,
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
        email, hash
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}
```

No axum types, no business logic.

### `users/types.rs` — domain types for this domain

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
}
```

---

## What to Do With Your Current Files

| Current file | New home |
|---|---|
| `routes.rs` (users handlers) | `users/routes.rs` |
| `routes.rs` (products handlers) | `products/routes.rs` |
| `routes.rs` (orders handlers) | `orders/routes.rs` |
| `routes.rs` (admin handlers) | `admin/routes.rs` |
| `db.rs` (user SQL) | `users/queries.rs` |
| `db.rs` (product SQL) | `products/queries.rs` |
| `db.rs` (order SQL) | `orders/queries.rs` |
| `db.rs` (pool setup) | `db.rs` (keep, it's infrastructure) |
| `models.rs` (User + related) | `users/types.rs` |
| `models.rs` (Product + related) | `products/types.rs` |
| `models.rs` (Order + related) | `orders/types.rs` |
| `auth.rs` | `auth.rs` (keep at root — cross-cutting) |
| `error.rs` | `error.rs` (keep at root — cross-cutting) |

---

## Dependency Direction

Dependencies must point inward — never from infrastructure back toward handlers:

```
routes.rs → service.rs → queries.rs → db.rs
routes.rs → types.rs
service.rs → types.rs
queries.rs → types.rs

auth.rs    → (no domain imports)
error.rs   → (no domain imports)
db.rs      → (no domain imports)
```

**Check:** trace every `use crate::` in your query files. If `queries.rs` imports from `routes.rs` or `service.rs`, that's a violation. Fix by moving the shared type down to `types.rs` or `error.rs`.

---

## Splitting the 800-line `routes.rs` — Execution Order

Do not move everything at once. One domain, one commit:

1. Create empty `users/mod.rs`, `users/routes.rs`, `users/queries.rs`, `users/types.rs`
2. Declare `mod users;` in `main.rs`
3. Move User types from `models.rs` → `users/types.rs`. `cargo check`.
4. Move user SQL from `db.rs` → `users/queries.rs`. `cargo check`.
5. Move user handlers from `routes.rs` → `users/routes.rs`. `cargo check`.
6. Wire `users::router()` into the axum `Router` in `main.rs`. `cargo test`.
7. Commit.
8. Repeat for `products/`, `orders/`, `admin/`.

After all domains are moved, `routes.rs` and the domain-specific sections of `db.rs` and `models.rs` should be empty and can be deleted.

---

## When Service Layer Is Optional

Not every domain needs a `service.rs`. Products with no business logic (just CRUD) can skip it:

```
products/
├── mod.rs
├── routes.rs    # handlers call queries directly
├── queries.rs
└── types.rs
```

Add `service.rs` only when handlers start doing more than one query or non-trivial validation. The signal: if you'd want to test it without an HTTP request, extract it to `service.rs`.

---

## Checklist Before Finalizing

- [ ] Architecture pattern chosen based on decision tree (hybrid domain + internal layers)
- [ ] Each directory answers "which domain?" in one word
- [ ] No `models.rs` bag — types live with their domain
- [ ] No SQL in `routes.rs`, no axum types in `queries.rs`
- [ ] `main.rs` has no handlers or queries — wiring only
- [ ] `mod.rs` files are tables of contents (< 30 lines of logic)
- [ ] Dependencies point inward: routes → service → queries → db
- [ ] `error.rs` and `auth.rs` import from no domain module
- [ ] No file exceeds ~500 lines
