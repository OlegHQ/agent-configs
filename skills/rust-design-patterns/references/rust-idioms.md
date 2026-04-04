# Rust-Specific Patterns and Idioms

These patterns have no direct equivalent in traditional OOP design pattern catalogs. They emerge
from Rust's type system, ownership model, and trait system. Apply them alongside or instead of
GoF patterns when they better fit the problem.

---

## Newtype Pattern

**Problem:** Add type safety, restrict APIs, or implement external traits on external types.

```rust
/// Wrapping a primitive to prevent mixing up IDs
struct UserId(u64);
struct OrderId(u64);

// These are different types — can't accidentally pass UserId where OrderId is expected
fn get_order(id: OrderId) -> Order { /* ... */ }

// Implement traits on the newtype (orphan rule workaround)
struct Meters(f64);
impl std::fmt::Display for Meters {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.2}m", self.0)
    }
}
```

**When to use:** Preventing unit/ID mixups, implementing foreign traits on foreign types,
restricting the API surface of a wrapped type. Zero runtime cost — the newtype is erased at
compile time.

---

## Typestate Pattern

**Problem:** Enforce valid state transitions at compile time.

```rust
/// Marker types — no runtime data, exist only in the type system
struct Draft;
struct Published;
struct Archived;

struct Post<State> {
    title: String,
    content: String,
    _state: std::marker::PhantomData<State>,
}

impl Post<Draft> {
    fn new(title: String) -> Self {
        Post { title, content: String::new(), _state: std::marker::PhantomData }
    }

    fn set_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    /// Only drafts can be published — consumes Draft, produces Published
    fn publish(self) -> Post<Published> {
        Post { title: self.title, content: self.content, _state: std::marker::PhantomData }
    }
}

impl Post<Published> {
    /// Only published posts can be archived
    fn archive(self) -> Post<Archived> {
        Post { title: self.title, content: self.content, _state: std::marker::PhantomData }
    }
}

// Compile-time enforcement:
// Post::new("Hi".into()).publish().archive();  // OK
// Post::new("Hi".into()).archive();            // ERROR: no method `archive` on Post<Draft>
```

**When to use:** Protocol states (TCP connection: Connecting -> Connected -> Closed), file handles
(Open -> Closed), API builders where certain operations only make sense in certain states.
Stronger than the State pattern because invalid transitions are compile errors, not runtime errors.
But less flexible — the set of states must be known at compile time.

---

## RAII Guards

**Problem:** Ensure cleanup happens when a scope exits, regardless of how (return, panic, etc.).

```rust
struct MutexGuard<'a, T> {
    data: &'a mut T,
    // When dropped, automatically releases the lock
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        // Release the lock — guaranteed to run
    }
}

// Custom RAII guard example
struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(path: PathBuf) -> std::io::Result<Self> {
        std::fs::File::create(&path)?;
        Ok(TempFile { path })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);  // Cleanup on scope exit
    }
}
```

**Ubiquitous in Rust:** `MutexGuard`, `RwLockReadGuard`, `File` (closes on drop), `Vec` (frees
on drop). Use the `Drop` trait for any resource that needs deterministic cleanup. This is
Rust's replacement for try-finally / destructors.

---

## Extension Traits

**Problem:** Add methods to types you don't own without the Newtype wrapper overhead.

```rust
trait IteratorExt: Iterator {
    /// Add a `tap` method to all iterators
    fn tap<F: FnMut(&Self::Item)>(self, f: F) -> Tap<Self, F>
    where Self: Sized {
        Tap { iter: self, f }
    }
}

// Blanket implementation — applies to ALL iterators
impl<I: Iterator> IteratorExt for I {}

// Now any iterator has .tap()
vec![1, 2, 3].into_iter()
    .tap(|x| println!("Processing: {x}"))
    .map(|x| x * 2)
    .collect::<Vec<_>>();
```

**When to use:** Adding utility methods to standard library types, creating domain-specific
fluent APIs, implementing traits across type families via blanket impls.

---

## From/Into Conversions

**Problem:** Type conversions that are total (always succeed) or fallible.

```rust
struct Celsius(f64);
struct Fahrenheit(f64);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}

// Into is auto-derived from From
let temp: Fahrenheit = Celsius(100.0).into();

// TryFrom for fallible conversions
impl TryFrom<&str> for Email {
    type Error = EmailError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.contains('@') { Ok(Email(s.to_string())) }
        else { Err(EmailError::MissingAt) }
    }
}
```

**Idiomatic Rust:** Implement `From<T>` for infallible conversions (you get `Into` free).
Implement `TryFrom<T>` for fallible ones. This replaces many uses of Factory Method and is
the standard Rust approach to type conversion.

---

## Error Handling Patterns

**Problem:** Propagate and handle errors without exceptions.

**Custom error type with thiserror:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation failed: {field} — {message}")]
    Validation { field: String, message: String },

    #[error("Not found: {0}")]
    NotFound(String),
}
```

**The `?` operator** propagates errors up the call stack, replacing try-catch patterns.
`From` implementations enable automatic error type conversion.

**When to use:** Always. Rust's error handling is not a "pattern" — it's the standard approach.
Use `Result<T, E>` for all fallible operations. Use `thiserror` for library errors (typed),
`anyhow` for application errors (boxed).

---

## Enum-Based State Machines

**Problem:** Small, fixed set of states with simple transitions — lighter than the State pattern.

```rust
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Error { message: String },
}

impl ConnectionState {
    fn handle_event(self, event: Event) -> Self {
        match (self, event) {
            (ConnectionState::Disconnected, Event::Connect) =>
                ConnectionState::Connecting { attempt: 1 },

            (ConnectionState::Connecting { attempt }, Event::Success(id)) =>
                ConnectionState::Connected { session_id: id },

            (ConnectionState::Connecting { attempt }, Event::Failure(msg)) if attempt < 3 =>
                ConnectionState::Connecting { attempt: attempt + 1 },

            (ConnectionState::Connecting { .. }, Event::Failure(msg)) =>
                ConnectionState::Error { message: msg },

            (state, _) => state,  // Ignore irrelevant events
        }
    }
}
```

**When to use instead of State pattern:** When the set of states is small (< 6), fixed (no new
states expected), and the behavior per state is simple (a few match arms, not pages of logic).
The State pattern wins when each state has complex behavior, when states should be independently
testable, or when new states are added frequently.

---

## Summary: Choosing the Right Abstraction

| Need | OOP Pattern | Rust Idiom | Prefer Idiom When |
|---|---|---|---|
| Type safety for primitives | — | **Newtype** | Always (zero cost) |
| Compile-time state transitions | State | **Typestate** | States known at compile time |
| Resource cleanup | — | **RAII / Drop** | Always (deterministic) |
| Add methods to foreign types | Decorator | **Extension Trait** | No wrapping overhead needed |
| Type conversion | Factory Method | **From/Into** | Conversion is the primary operation |
| Error propagation | Chain of Responsibility | **Result + ?** | Always in Rust |
| Small fixed state machines | State | **Enum match** | < 6 states, simple behavior |
