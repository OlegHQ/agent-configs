# Organizing a Rust axum Web API

Your current structure has two problems: a flat crate (everything at the top level) and god modules (`routes.rs`, `db.rs`). The fix is to group by **domain** and enforce a **dependency rule**: layers only import downward.

---

## Target Structure

```
src/
├── main.rs                    # Wiring only: build router, connect DB, spawn server
├── lib.rs                     # pub mod declarations only — no logic
│
├── error.rs                   # Shared error type (Layer 0 — no imports from above)
│
├── models/                    # Layer 0: Pure data types, no I/O
│   ├── mod.rs                 # Re-exports
│   ├── user.rs
│   ├── product.rs
│   ├── order.rs
│   └── admin.rs
│
├── db/                        # Layer 1: SQL queries (depends on: models, error)
│   ├── mod.rs                 # Shared pool type + re-exports
│   ├── users.rs
│   ├── products.rs
│   ├── orders.rs
│   └── admin.rs
│
├── auth.rs                    # Layer 1: Auth logic (depends on: models, error)
│
└── routes/                    # Layer 3: HTTP handlers (depends on: db, auth, models, error)
    ├── mod.rs                 # Assembles the full Router, re-exports
    ├── users.rs
    ├── products.rs
    ├── orders.rs
    └── admin.rs
```

---

## The Dependency Rule

Dependencies point **inward and downward only**:

```
main.rs
  └── routes/          (Layer 3 — HTTP, axum extractors, responses)
        └── db/        (Layer 1 — SQL, sqlx)
        └── auth.rs    (Layer 1 — token validation, middleware)
              └── models/   (Layer 0 — structs, no side effects)
              └── error.rs  (Layer 0 — error enum)
```

`models/` must never import from `db/` or `routes/`. `db/` must never import from `routes/`. Violating this means changing a handler forces recompilation of your data types.

---

## lib.rs — Table of Contents Only

```rust
pub mod auth;
pub mod db;
pub mod error;
pub mod models;
pub mod routes;
```

No logic, no `use` statements, no re-exports here.

---

## models/ — Split by Domain

`models/user.rs`:
```rust
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
}
```

`models/mod.rs`:
```rust
mod admin;
mod order;
mod product;
mod user;

pub use admin::{AdminStats, BanUserRequest};
pub use order::{CreateOrderRequest, Order, OrderItem};
pub use product::{CreateProductRequest, Product};
pub use user::{CreateUserRequest, User};
```

---

## db/ — One File Per Domain

`db/users.rs` — only user SQL:
```rust
use sqlx::PgPool;
use crate::error::AppError;
use crate::models::User;

pub async fn get_by_id(pool: &PgPool, id: i64) -> Result<Option<User>, AppError> {
    sqlx::query_as!(User, "SELECT id, email, created_at FROM users WHERE id = $1", id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create(pool: &PgPool, email: &str, password_hash: &str) -> Result<User, AppError> {
    sqlx::query_as!(
        User,
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id, email, created_at",
        email,
        password_hash
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}
```

`db/mod.rs`:
```rust
pub mod admin;
pub mod orders;
pub mod products;
pub mod users;
```

No shared logic here — just declarations. If you later need a shared query helper, add a `db/util.rs` and keep it `pub(super)`.

---

## routes/ — One File Per Domain

`routes/users.rs` — only user handlers:
```rust
use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::{db, error::AppError, models::{CreateUserRequest, User}};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = db::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("user"))?;
    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let hash = hash_password(&req.password)?;
    let user = db::users::create(&state.pool, &req.email, &hash).await?;
    Ok((StatusCode::CREATED, Json(user)))
}
```

`routes/admin.rs` — add auth middleware here, not in shared routes:
```rust
use axum::{middleware, Router};
use crate::auth::require_admin;

pub fn router() -> Router<super::AppState> {
    Router::new()
        .route("/stats", axum::routing::get(get_stats))
        .route("/users/:id/ban", axum::routing::post(ban_user))
        .route_layer(middleware::from_fn(require_admin))  // admin-only
}
```

`routes/mod.rs` — assembles the full router:
```rust
mod admin;
mod orders;
mod products;
mod users;

pub use users::AppState;

use axum::Router;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/users",    users_router())
        .nest("/products", products_router())
        .nest("/orders",   orders_router())
        .nest("/admin",    admin::router())
        .with_state(state)
}

fn users_router() -> Router<AppState> {
    use axum::routing::{get, post};
    Router::new()
        .route("/",    post(users::create_user))
        .route("/:id", get(users::get_user))
}

fn products_router() -> Router<AppState> {
    // ...same pattern
    Router::new()
}

fn orders_router() -> Router<AppState> {
    Router::new()
}
```

---

## main.rs — Wiring Only

```rust
use myapp::routes::{build_router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let state = AppState { pool };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

`main.rs` imports nothing from `db/` directly — only `routes/` and its public types.

---

## Migration Steps (Do This Incrementally)

1. **Split `models.rs` first** — it has no dependencies, zero risk. Create `models/` directory, move structs into domain files, update `models/mod.rs` to re-export. Fix all `use crate::models::*` call sites (usually a global find-replace).

2. **Split `db.rs` next** — create `db/` directory, move query functions by domain. Each file is self-contained. Add `use crate::models::Domain` at the top of each. Compile after each file to catch import errors early.

3. **Split `routes.rs` last** — it's the most complex because it has the most imports. Create `routes/` directory one domain at a time: start with `routes/users.rs`, move user handlers, compile, repeat for products, orders, admin.

4. **Update `lib.rs`** after each step to reflect the new module paths.

---

## Visibility Rules to Enforce

| Item | Visibility | Reason |
|---|---|---|
| SQL query functions in `db/users.rs` | `pub` | Called from `routes/` |
| Internal helpers in `db/users.rs` | private | Same file only |
| `AppState` in `routes/mod.rs` | `pub` | Needed in `main.rs` |
| Sub-routers in `routes/` | `pub(super)` or private | Only `routes/mod.rs` assembles them |
| Model structs | `pub` | Used across all layers |

Don't make everything `pub`. When `db/` internals are `pub(crate)` at most, `routes/` cannot accidentally reach into `db/` implementation details — the compiler enforces the boundary.

---

## Result

After this refactor:
- Adding a new `invoices` domain means creating `models/invoice.rs`, `db/invoices.rs`, `routes/invoices.rs`, and wiring one `.nest()` call in `routes/mod.rs`. You touch **no existing files**.
- `routes.rs` goes from 800 lines to 4 files averaging ~150–200 lines each.
- `db.rs` goes from 600 lines to 4 files averaging ~100–150 lines each.
- The compiler catches any attempt to import upward through the layer stack.
