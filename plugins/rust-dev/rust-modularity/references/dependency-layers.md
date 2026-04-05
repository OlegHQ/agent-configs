# Dependency Layer Validation

A step-by-step process for auditing and enforcing the dependency rule in a Rust crate.

## Table of Contents

1. [Mapping the current graph](#mapping-the-current-graph)
2. [Assigning layers](#assigning-layers)
3. [Finding violations](#finding-violations)
4. [Fixing violations](#fixing-violations)
5. [Preventing regression](#preventing-regression)

---

## Mapping the current graph

For every `.rs` file, list its `use crate::` imports. Build an adjacency list:

```
model.rs       → (none)
config.rs      → (none)
db.rs          → config
storage.rs     → config, db, model
doc_storage.rs → config, db, model, sections
sections.rs    → (none)
savings.rs     → config, model
extractors/    → model
service/       → config, extractors, savings, storage, doc_storage, model
mcp.rs         → service
main.rs        → config, mcp, service
```

## Assigning layers

Group modules by their position in the dependency hierarchy:

**Layer 0 — Pure types and config (no crate imports):**
Modules with zero `use crate::` imports. These are leaves. They define data types,
constants, and pure functions.

Examples: `model.rs`, `config.rs`, `sections.rs` (pure parsing)

**Layer 1 — Domain operations (import only Layer 0):**
Modules that import from Layer 0 but never from Layer 2+. They implement domain
logic, storage backends, and extraction.

Examples: `storage.rs`, `doc_storage.rs`, `extractors/`, `savings.rs`, `db.rs`

**Layer 2 — Orchestration (import Layer 0 + Layer 1):**
Modules that coordinate multiple Layer 1 modules. Business logic lives here.

Examples: `service/`

**Layer 3 — Protocol/transport (import Layer 2, maybe Layer 0):**
The outermost layer that translates external requests into service calls.

Examples: `mcp.rs`, `main.rs`

## Finding violations

A violation is any import where a lower layer reaches into a higher layer:

```
VIOLATION: storage.rs imports service::SomeHelper
           (Layer 1 importing Layer 2 — breaks the rule)

VIOLATION: model.rs imports storage::IndexEntry
           (Layer 0 importing Layer 1 — breaks the rule)

OK:        service.rs imports storage::CodeIndex
           (Layer 2 importing Layer 1 — correct direction)
```

**Lateral imports** (same layer) are a gray area:
- Within the same directory module: usually fine (siblings share a parent)
- Across different top-level modules at the same layer: acceptable if they don't form
  a cycle, but consider whether the shared dependency should be in a lower layer

## Fixing violations

### Downward extraction

If `storage.rs` needs a helper defined in `service.rs`, the helper is in the wrong
place. Move it to a shared module at Layer 0 or Layer 1.

Before:
```rust
// service/helpers.rs (Layer 2)
pub fn normalize_path(p: &str) -> String { ... }

// storage.rs (Layer 1) — VIOLATION
use crate::service::helpers::normalize_path;
```

After:
```rust
// util.rs or model/paths.rs (Layer 0)
pub fn normalize_path(p: &str) -> String { ... }

// Both service/ and storage/ import from Layer 0 — no violation
```

### Trait inversion

If a lower layer needs to call behavior defined in a higher layer, define a trait in
the lower layer and implement it in the higher layer (dependency inversion):

Before:
```rust
// storage.rs (Layer 1) — VIOLATION
use crate::service::Notifier;
fn index(&mut self) { self.notifier.notify("done"); }
```

After:
```rust
// storage.rs (Layer 1) — defines trait
pub trait IndexCallback {
    fn on_complete(&self, msg: &str);
}
fn index(&mut self, callback: &dyn IndexCallback) { callback.on_complete("done"); }

// service.rs (Layer 2) — implements trait
impl IndexCallback for Notifier { ... }
```

### Shared types

When two same-layer modules both need a type, don't put it in either — put it in a
module one layer below. The `model/` directory is the natural home for shared types.

## Preventing regression

### Compile-time enforcement

Rust's module visibility system can prevent violations:

```rust
// storage/mod.rs
mod internal {
    // Types here are invisible outside storage/
    pub(super) struct CacheEntry { ... }
}

pub use internal::CacheEntry; // Only if you want to expose it
```

### Code review heuristic

When reviewing a PR that adds a `use crate::` import:
1. Identify the layer of the importing module
2. Identify the layer of the imported module
3. If the import points upward → request extraction to a lower layer

### CI check (optional)

For larger projects, consider a `cargo` script or custom lint that parses `use crate::`
statements and validates them against a layer definition file:

```toml
# layers.toml
[layers]
model = 0
config = 0
db = 1
storage = 1
extract = 1
service = 2
transport = 3
```

Any import from a higher layer number to a lower layer number is flagged.
