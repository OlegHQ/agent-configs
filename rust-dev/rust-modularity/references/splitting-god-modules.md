# Splitting God Modules

A step-by-step guide for decomposing oversized Rust modules that have accumulated too
many responsibilities.

## Table of Contents

1. [Identifying a god module](#identifying-a-god-module)
2. [Planning the split](#planning-the-split)
3. [Executing the split](#executing-the-split)
4. [Worked example: service dispatcher](#worked-example-service-dispatcher)
5. [Post-split validation](#post-split-validation)

---

## Identifying a god module

A module is a "god module" when it:

- Has **300+ lines** of logic (not counting blank lines and comments)
- Contains **3+ unrelated impl blocks** or function groups
- Is the **most frequently edited file** in git history (`git log --format=%H -- file | wc -l`)
- Has **merge conflicts** regularly because multiple features touch it
- Imports from **many** other modules (high fan-in suggests it's orchestrating too much)

The most common god module in Rust projects is the service/dispatch module — the one
with a big `match` statement routing requests to handlers.

## Planning the split

Before moving code, plan the target structure on paper:

### Step 1: List responsibilities

Read through the god module and tag each function/impl block with its domain:

```
fn handle_index_repo()     → code
fn handle_search_symbols() → code
fn handle_index_docs()     → docs
fn handle_search_sections()→ docs
fn handle_list_repos()     → discovery
fn handle_metadata()       → discovery
fn dispatch()              → routing (keep in mod.rs)
```

### Step 2: Group by domain

```
code:      handle_index_repo, handle_search_symbols, handle_retrieve_symbol
docs:      handle_index_docs, handle_search_sections, handle_retrieve_section
discovery: handle_list_repos, handle_metadata
routing:   dispatch (thin dispatcher — stays in mod.rs)
```

### Step 3: Plan the file structure

```
service/
├── mod.rs        # dispatch() + re-exports (< 80 lines)
├── code.rs       # code domain handlers
├── doc.rs        # doc domain handlers
└── discovery.rs  # repo listing and metadata
```

### Step 4: Decide the split strategy

**Strategy A: Split impl blocks (same struct, separate files)**

Each file adds methods to the same service struct via `impl ServiceName`:

```rust
// service/code.rs
impl CodeService {
    pub(super) fn handle_index_repo(&self, ...) -> Result<Value> { ... }
    pub(super) fn handle_search_symbols(&self, ...) -> Result<Value> { ... }
}
```

Pro: Simplest migration. No API changes.
Con: All methods still share the same struct's state — doesn't reduce coupling.

**Strategy B: Split into domain types (separate structs)**

Each domain gets its own struct, composed by the parent:

```rust
// service/code.rs
pub(crate) struct CodeHandler {
    store: CodeIndex,
    registry: ExtractorRegistry,
}
impl CodeHandler {
    pub fn index_repo(&self, ...) -> Result<Value> { ... }
    pub fn search_symbols(&self, ...) -> Result<Value> { ... }
}

// service/mod.rs
pub struct Service {
    code: CodeHandler,
    doc: DocHandler,
    discovery: DiscoveryHandler,
}
impl Service {
    pub fn dispatch(&self, method: &str, params: Value) -> Result<Value> {
        match method {
            "index_repo" | "search_symbols" => self.code.dispatch(method, params),
            "index_docs" | "search_sections" => self.doc.dispatch(method, params),
            "list_repos" | "metadata" => self.discovery.dispatch(method, params),
            _ => Err(MethodNotFound(method)),
        }
    }
}
```

Pro: Each handler owns only the state it needs. Testable in isolation.
Con: Requires refactoring state ownership. More changes to existing code.

**When to use which:**
- Strategy A for quick wins and when the module is growing but manageable
- Strategy B when the god module is causing real pain (test coupling, merge conflicts)

## Executing the split

Follow this exact sequence to avoid breaking things mid-refactor:

### 1. Create the new files (empty)

```
touch src/service/code.rs
touch src/service/doc.rs
```

### 2. Declare them in mod.rs

```rust
mod code;
mod doc;
```

### 3. Move functions one group at a time

Move the smallest, most independent group first. After each move:
- `cargo check` — verify compilation
- `cargo test` — verify behavior

Don't move everything at once. One group, one commit.

### 4. Update visibility

Functions that were private in the god module but are now in sub-modules need
`pub(super)` so `mod.rs` can still call them.

### 5. Update the dispatcher

Thin out the dispatch match in `mod.rs` to delegate to the new sub-modules.

### 6. Clean up imports

After all moves are done, review `use` statements in mod.rs. It should import from
its sub-modules, not from the entire crate.

## Worked example: service dispatcher

Starting state — god module with 800 lines:

```rust
// service/mod.rs (BEFORE — 800 lines)
pub struct CodeService {
    extractors: ExtractorRegistry,
    savings: SavingsTracker,
}

impl CodeService {
    pub fn dispatch(&mut self, method: &str, params: Value) -> Result<Value> {
        match method {
            "index_repo" => self.handle_index_repo(params),
            "search_symbols" => self.handle_search_symbols(params),
            "retrieve_symbol" => self.handle_retrieve_symbol(params),
            "index_docs" => self.handle_index_docs(params),
            "search_sections" => self.handle_search_sections(params),
            "retrieve_section" => self.handle_retrieve_section(params),
            "list_repos" => self.handle_list_repos(params),
            _ => Err(anyhow!("unknown method")),
        }
    }

    // ... 700 lines of handler implementations ...
}
```

After split — 4 files, none over 250 lines:

```rust
// service/mod.rs (AFTER — 60 lines)
mod args;
mod code;
mod doc;
mod helpers;

use code::CodeOps;
use doc::DocOps;

pub struct CodeService {
    code: CodeOps,
    doc: DocOps,
}

impl CodeService {
    pub fn new() -> Self {
        Self {
            code: CodeOps::new(),
            doc: DocOps::new(),
        }
    }

    pub fn dispatch(&mut self, method: &str, params: Value) -> Result<Value> {
        match method {
            "index_repo" | "search_symbols" | "retrieve_symbol" | "list_repos"
                => self.code.dispatch(method, params),
            "index_docs" | "search_sections" | "retrieve_section"
                => self.doc.dispatch(method, params),
            _ => Err(anyhow!("unknown method: {method}")),
        }
    }
}
```

```rust
// service/code.rs (AFTER — 250 lines)
pub(super) struct CodeOps {
    extractors: ExtractorRegistry,
    savings: SavingsTracker,
}

impl CodeOps {
    pub(super) fn dispatch(&mut self, method: &str, params: Value) -> Result<Value> {
        match method {
            "index_repo" => self.handle_index_repo(params),
            "search_symbols" => self.handle_search_symbols(params),
            "retrieve_symbol" => self.handle_retrieve_symbol(params),
            "list_repos" => self.handle_list_repos(params),
            _ => unreachable!(),
        }
    }

    fn handle_index_repo(&mut self, params: Value) -> Result<Value> { ... }
    fn handle_search_symbols(&self, params: Value) -> Result<Value> { ... }
    fn handle_retrieve_symbol(&self, params: Value) -> Result<Value> { ... }
    fn handle_list_repos(&self, params: Value) -> Result<Value> { ... }
}
```

## Post-split validation

After completing the split:

1. **Compile check:** `cargo check` — the refactor is pure restructuring, no logic changes
2. **Test:** `cargo test` — all existing tests must pass unchanged
3. **Line count:** No file should exceed ~500 lines. If one does, split further.
4. **Responsibility check:** Can you describe each file's job in one sentence? If not,
   it's still doing too much.
5. **Import direction:** Each sub-module should import from lower layers (model, storage)
   and not from sibling service sub-modules unless they share a helper extracted to
   `helpers.rs`.
6. **Git blame preservation:** Use `git mv` where possible so blame history follows the code.
