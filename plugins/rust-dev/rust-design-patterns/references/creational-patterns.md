# Creational Patterns in Rust

Creational patterns abstract the instantiation process. In Rust, almost all can use generics
(static dispatch) — the type system resolves concrete types at compile time.

---

## Builder

**Problem:** Complex objects with many optional fields, validation requirements, or multi-step
construction that a single constructor can't express cleanly.

**Rust Implementation:**

```rust
/// Builder trait with associated type — different builders can produce different products
trait Builder {
    type Output;
    fn set_seats(&mut self, seats: u16);
    fn set_engine(&mut self, engine: Engine);
    fn set_gps(&mut self, gps: bool);
    /// Consumes the builder — prevents accidental reuse after build
    fn build(self) -> Self::Output;
}

struct CarBuilder {
    seats: Option<u16>,
    engine: Option<Engine>,
    gps: bool,
}

impl Builder for CarBuilder {
    type Output = Car;

    fn set_seats(&mut self, seats: u16) { self.seats = Some(seats); }
    fn set_engine(&mut self, engine: Engine) { self.engine = Some(engine); }
    fn set_gps(&mut self, gps: bool) { self.gps = gps; }

    fn build(self) -> Car {
        Car {
            seats: self.seats.expect("Seats are required"),
            engine: self.engine.expect("Engine is required"),
            gps: self.gps,
        }
    }
}
```

**Key Rust Details:**
- `build(self)` consumes the builder via move semantics — calling `.build()` twice is a compile error
- Use `Option<T>` for optional/unset fields, `.expect()` or `Result` for required ones
- Director pattern: a free function or struct that calls builder methods in a specific sequence
- Distinguish from **fluent interface** (`Car::default().seats(5).gas(30)`) which chains on the product itself

**When to use:** 5+ fields, optional fields, validation at build time, multiple representations of the same construction process.

---

## Abstract Factory

**Problem:** Create families of related objects without coupling to concrete types.

**Static Dispatch (Generics):**

```rust
trait UIFactory {
    type Button: ButtonTrait;
    type Checkbox: CheckboxTrait;

    fn create_button(&self) -> Self::Button;
    fn create_checkbox(&self) -> Self::Checkbox;
}

struct MacFactory;
impl UIFactory for MacFactory {
    type Button = MacButton;
    type Checkbox = MacCheckbox;
    fn create_button(&self) -> MacButton { MacButton }
    fn create_checkbox(&self) -> MacCheckbox { MacCheckbox }
}

/// Generic code works with any factory — resolved at compile time
fn render_ui<F: UIFactory>(factory: &F) {
    let button = factory.create_button();
    let checkbox = factory.create_checkbox();
    button.render();
    checkbox.render();
}
```

**Dynamic Dispatch (Trait Objects):**

```rust
trait UIFactory {
    fn create_button(&self) -> Box<dyn ButtonTrait>;
    fn create_checkbox(&self) -> Box<dyn CheckboxTrait>;
}

/// When the factory type is determined at runtime (config, user input, etc.)
fn create_factory(os: &str) -> Box<dyn UIFactory> {
    match os {
        "mac" => Box::new(MacFactory),
        "win" => Box::new(WinFactory),
        _ => panic!("Unknown OS"),
    }
}
```

**When to choose which:**
- Static dispatch: when the concrete factory is known at compile time (most cases)
- Dynamic dispatch: when factory selection happens at runtime (plugin systems, config-driven)

---

## Factory Method

**Problem:** Delegate object creation to subtypes.

```rust
trait Dialog {
    /// Factory method — each dialog type creates its own button
    fn create_button(&self) -> Box<dyn Button>;

    /// Template method that uses the factory method
    fn render(&self) {
        let button = self.create_button();
        button.on_click();
        button.render();
    }
}

struct WebDialog;
impl Dialog for WebDialog {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(HtmlButton)
    }
}
```

**When to use:** When a method needs to create objects but the concrete type should be determined by the implementing type.

---

## Prototype

**Problem:** Create new objects by copying existing ones.

```rust
#[derive(Clone, Debug)]
struct Circle {
    x: f64,
    y: f64,
    radius: f64,
    color: String,
}

let original = Circle { x: 10.0, y: 15.0, radius: 5.0, color: "red".into() };
let mut copy = original.clone();
copy.color = "blue".into(); // Modify the copy
```

**Trivial in Rust** — `#[derive(Clone)]` does it. Use when deep-copying is the right creation
strategy, especially for objects with complex internal state.

---

## Singleton

**Problem:** Ensure exactly one instance of a type exists globally.

**Warning:** Singleton fundamentally conflicts with Rust's ownership model. Global mutable state
requires synchronization primitives. Consider these approaches in order of preference:

**1. Dependency Injection (preferred — not a singleton at all):**
```rust
fn process(config: &Config, db: &mut Database) {
    // Pass shared state explicitly — no globals needed
}
```

**2. OnceLock + Mutex (Rust 1.80+, standard library):**
```rust
use std::sync::{Mutex, OnceLock};

fn global_config() -> &'static Mutex<Config> {
    static INSTANCE: OnceLock<Mutex<Config>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(Config::load()))
}

// Usage
let config = global_config().lock().unwrap();
```

**3. LazyLock (Rust 1.80+):**
```rust
use std::sync::LazyLock;

static LOGGER: LazyLock<Mutex<Logger>> = LazyLock::new(|| {
    Mutex::new(Logger::new())
});
```

**When to use:** Truly global resources (logging, configuration loaded once). But first ask: can
this be passed as a parameter instead?

---

## Static Creation Method

**Problem:** Named constructors that express intent better than `new()`.

```rust
impl User {
    fn new(name: &str) -> Self { /* default user */ }
    fn admin(name: &str) -> Self { /* admin user */ }
    fn from_config(config: &Config) -> Self { /* load from config */ }
    fn guest() -> Self { /* anonymous user */ }
}
```

**Idiomatic Rust** — this is standard practice. Use `new()` for the primary constructor,
named methods for variants. Implement `From<T>` / `TryFrom<T>` for type conversions.
