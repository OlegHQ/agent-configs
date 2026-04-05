# Structural Patterns in Rust

Structural patterns compose types into larger structures. In Rust, most can use generics (static
dispatch) since structure is determined at compile time.

---

## Adapter

**Problem:** Make an incompatible interface work with code that expects a different interface.

```rust
/// Target interface your code expects
trait Target {
    fn request(&self) -> String;
}

/// Existing code with incompatible interface
struct LegacyPrinter;
impl LegacyPrinter {
    fn specific_request(&self) -> String { "legacy output".into() }
}

/// Adapter wraps the incompatible type and implements the expected trait
struct PrinterAdapter {
    printer: LegacyPrinter,
}

impl Target for PrinterAdapter {
    fn request(&self) -> String {
        // Translate between interfaces
        format!("Adapted: {}", self.printer.specific_request())
    }
}
```

**When to use:** Integrating third-party libraries, legacy code, or external APIs with different
interfaces than your codebase expects. The adapter is a thin wrapper — keep it minimal.

---

## Bridge

**Problem:** Two dimensions of variation that should evolve independently.

```rust
/// Implementor hierarchy — the "how"
trait Device {
    fn is_enabled(&self) -> bool;
    fn set_volume(&mut self, volume: u8);
    fn channel(&self) -> u16;
}

/// Abstraction hierarchy — the "what"  
struct RemoteControl<D: Device> {
    device: D,
}

impl<D: Device> RemoteControl<D> {
    fn toggle_power(&mut self) { /* uses self.device */ }
    fn volume_up(&mut self) { /* uses self.device */ }
}

/// Advanced remote adds features without touching Device implementations
struct AdvancedRemote<D: Device> {
    device: D,
}

impl<D: Device> AdvancedRemote<D> {
    fn mute(&mut self) { self.device.set_volume(0); }
}
```

**Key insight:** Bridge separates *what* something does from *how* it does it. Remotes (abstraction)
and devices (implementation) can vary independently. Use generics for static dispatch; trait objects
when the device type is determined at runtime.

---

## Composite

**Problem:** Treat individual objects and compositions of objects uniformly.

```rust
trait Component {
    fn search(&self, keyword: &str) -> Vec<String>;
    fn name(&self) -> &str;
}

struct File { name: String, content: String }

impl Component for File {
    fn search(&self, keyword: &str) -> Vec<String> {
        if self.content.contains(keyword) { vec![self.name.clone()] }
        else { vec![] }
    }
    fn name(&self) -> &str { &self.name }
}

struct Directory {
    name: String,
    children: Vec<Box<dyn Component>>,
}

impl Component for Directory {
    fn search(&self, keyword: &str) -> Vec<String> {
        // Recursively search all children — uniform interface
        self.children.iter().flat_map(|c| c.search(keyword)).collect()
    }
    fn name(&self) -> &str { &self.name }
}
```

**When to use:** Tree structures — file systems, UI component hierarchies, expression trees, org charts.
Dynamic dispatch (`Box<dyn Component>`) is natural here since collections contain mixed types.

---

## Decorator

**Problem:** Add behavior to objects without modifying their type.

```rust
use std::io::{BufRead, BufReader, Cursor, Read};

// BufReader decorates any Read implementor — this is Decorator in std lib
let data = Cursor::new("Hello\nWorld\n");
let reader = BufReader::new(data); // Adds buffering behavior

for line in reader.lines() {
    println!("{}", line.unwrap());
}
```

**Custom Decorator:**
```rust
trait DataSource {
    fn read_data(&self) -> String;
    fn write_data(&mut self, data: &str);
}

struct EncryptionDecorator<T: DataSource> {
    wrapped: T,
    key: String,
}

impl<T: DataSource> DataSource for EncryptionDecorator<T> {
    fn read_data(&self) -> String {
        let raw = self.wrapped.read_data();
        decrypt(&raw, &self.key)
    }
    fn write_data(&mut self, data: &str) {
        let encrypted = encrypt(data, &self.key);
        self.wrapped.write_data(&encrypted);
    }
}
```

**Key:** Implement the same trait as the wrapped type. Delegate most methods, enhance the ones
that matter. Decorators compose: `EncryptionDecorator<CompressionDecorator<FileSource>>`.

---

## Facade

**Problem:** Complex subsystem with too many entry points.

```rust
/// Multiple internal components with complex interactions
struct AccountService { /* ... */ }
struct LedgerService { /* ... */ }
struct NotificationService { /* ... */ }
struct SecurityService { /* ... */ }

/// Facade provides a simple interface to the complex subsystem
struct WalletFacade {
    account: AccountService,
    ledger: LedgerService,
    notification: NotificationService,
    security: SecurityService,
}

impl WalletFacade {
    /// One simple method orchestrates multiple internal operations
    fn add_money(&mut self, account_id: &str, amount: f64) -> Result<(), Error> {
        self.security.verify(account_id)?;
        self.account.credit(account_id, amount)?;
        self.ledger.record(account_id, amount)?;
        self.notification.send(account_id, "Deposit received")?;
        Ok(())
    }
}
```

**When to use:** When external code needs to interact with a complex subsystem but doesn't need
fine-grained control. The facade doesn't hide the subsystem — it provides a convenient default path.

---

## Flyweight

**Problem:** Many objects share common immutable state, causing excessive memory usage.

```rust
use std::collections::HashMap;

/// Shared immutable state (the flyweight)
struct TreeType {
    name: String,
    color: String,
    texture: Vec<u8>,  // Large texture data shared across instances
}

/// Unique state per instance (context)
struct Tree {
    x: f64,
    y: f64,
    tree_type: usize,  // Index into the flyweight pool
}

struct TreeFactory {
    types: Vec<TreeType>,
    index: HashMap<String, usize>,
}

impl TreeFactory {
    fn get_tree_type(&mut self, name: &str, color: &str, texture: Vec<u8>) -> usize {
        let key = format!("{name}_{color}");
        if let Some(&idx) = self.index.get(&key) {
            return idx;  // Return existing — no duplication
        }
        let idx = self.types.len();
        self.types.push(TreeType { name: name.into(), color: color.into(), texture });
        self.index.insert(key, idx);
        idx
    }
}
```

**When to use:** Rendering thousands of similar objects (trees in a forest, characters in a text
editor, particles in a game). Profile first — only optimize when memory is actually a bottleneck.

---

## Proxy

**Problem:** Control access to an object — lazy initialization, access control, logging, caching.

```rust
trait Server {
    fn handle_request(&mut self, url: &str, method: &str) -> String;
}

struct Application;
impl Server for Application {
    fn handle_request(&mut self, url: &str, method: &str) -> String {
        format!("200 OK: {method} {url}")
    }
}

/// Proxy adds rate limiting without modifying Application
struct NginxProxy {
    app: Application,
    max_requests: u32,
    request_count: u32,
}

impl Server for NginxProxy {
    fn handle_request(&mut self, url: &str, method: &str) -> String {
        self.request_count += 1;
        if self.request_count > self.max_requests {
            return "429 Too Many Requests".into();
        }
        self.app.handle_request(url, method)
    }
}
```

**Variants:** Lazy proxy (initialize real object on first use), caching proxy (cache results),
protection proxy (check permissions), logging proxy (record calls).
