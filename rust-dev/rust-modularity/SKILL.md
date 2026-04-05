---
name: rust-modularity
description: |
  Guides Rust module structure, folder organization, and dependency layering for clean separation of concerns. Use this skill whenever structuring a new Rust project, adding modules to an existing crate, refactoring a flat or tangled module tree, splitting a growing file, or reorganizing code that has accumulated modularity debt. Triggers on ANY Rust structural/organizational task — not just explicit "refactor" requests. If the user says "create a new module", "split this file", "organize this code", "set up project structure", "this file is too big", or you notice a Rust crate with a crowded lib.rs, parallel implementations without shared traits, or god-modules with too many responsibilities, this skill applies. Complements rust-design-patterns (which covers individual code patterns like Builder/Strategy/State) — this skill focuses on the module graph, folder tree, and dependency direction between modules.
---

# Rust Modularity

Structure modules so that adding a new feature means adding a new file in the right place —
not touching ten existing ones. This skill is about the shape of your `src/` tree: which
modules exist, what depends on what, and what's visible where.

## The Dependency Rule

Every well-structured Rust crate follows one principle: **dependencies point inward and
downward, never sideways or upward.**

Visualize your crate as layers. Each layer may only `use` items from layers below it:

```
Layer 4: bin / main.rs          ─── entry point, wiring only
Layer 3: protocol / transport   ─── JSON-RPC, HTTP, CLI (how you talk to the world)
Layer 2: service / orchestration ─── business logic, coordinates lower layers
Layer 1: domain / storage       ─── persistence, indexing, domain operations
Layer 0: model / types / config ─── pure data, no side effects, no I/O
```

**Why this matters:** If `model.rs` imports from `service.rs`, then changing business logic
forces recompilation (and potentially redesign) of your data types. That coupling is the
root cause of "I changed one thing and everything broke." Keep the bottom layers ignorant
of the top — they define *what* things are, not *how* they're used.

**How to check:** After organizing, mentally trace each `use crate::` import. If it points
upward (a lower layer importing a higher one), that's a dependency violation. Fix it by
extracting the shared type downward or introducing a trait that the lower layer defines and
the upper layer implements.

## Module Tree Anatomy

A healthy Rust crate groups files by **domain responsibility**, not by technical kind:

```
src/
├── main.rs                    # Entry point — wiring, arg parsing, nothing else
├── lib.rs                     # Module declarations only — no logic here
│
├── model/                     # Layer 0: Pure data types
│   ├── mod.rs                 #   Re-exports public types
│   ├── symbol.rs              #   Code symbol types
│   ├── section.rs             #   Doc section types
│   └── meta.rs                #   Response metadata types
│
├── config.rs                  # Layer 0: Constants, paths, feature flags
│
├── storage/                   # Layer 1: Persistence (depends on: model, config)
│   ├── mod.rs                 #   Shared trait + re-exports
│   ├── traits.rs              #   IndexStore<T> trait — shared interface
│   ├── code.rs                #   Code index (implements IndexStore)
│   └── doc.rs                 #   Doc index (implements IndexStore)
│
├── extract/                   # Layer 1: Parsing/extraction (depends on: model)
│   ├── mod.rs                 #   Registry + re-exports
│   ├── common.rs              #   Shared tree-sitter utilities
│   ├── python.rs              #   Language-specific extractors
│   ├── rust.rs
│   └── ...
│
├── service/                   # Layer 2: Business logic (depends on: storage, extract, model)
│   ├── mod.rs                 #   Public facade type
│   ├── code.rs                #   Code operations
│   └── doc.rs                 #   Doc operations
│
└── transport/                 # Layer 3: Protocol handling (depends on: service)
    ├── mod.rs
    └── mcp.rs                 #   MCP JSON-RPC handler
```

The key properties of this tree:

1. **Each directory is one layer** — you can tell the dependency direction from the folder name
2. **mod.rs is a table of contents** — it declares submodules and re-exports, nothing more
3. **Parallel implementations share a trait** — `storage/traits.rs` defines `IndexStore<T>`,
   both `code.rs` and `doc.rs` implement it
4. **No file exceeds ~500 lines** — if it does, it's doing too much and should split

## When to Split a File

Split when a file has accumulated **multiple responsibilities**, not just when it's long.
A 400-line file with one focused job (e.g., a complex parser) is fine. A 200-line file
with three unrelated sections should split.

**Signals to split:**

- The file has `// --- Section ---` comments separating unrelated blocks
- Two structs/impls in the same file that don't reference each other
- You want to make something `pub(super)` but it's in the wrong module
- A `match` dispatches to logic that should live in separate files (god-module pattern)
- Different team members frequently have merge conflicts in the same file

**How to split:** Move the extracted code to a sibling file in the same directory. Update
`mod.rs` to declare it. Use `pub(super)` for items that only the parent module needs.
Re-export anything that was previously public from `mod.rs` so external callers don't break.

## Visibility: Say What You Mean

Rust's visibility system is your enforcement mechanism for the dependency rule. Use the
narrowest visibility that works:

| Visibility | Meaning | Use when |
|---|---|---|
| `pub` | Anyone can use this | It's part of your crate's API surface |
| `pub(crate)` | Crate-internal but cross-module | Shared infrastructure (DB pools, config) |
| `pub(super)` | Parent module only | Helper used by siblings in the same directory |
| `pub(in path)` | Specific ancestor module | Rare — usually `pub(super)` suffices |
| (private) | Same module only | Implementation details |

**Common mistake:** Making everything `pub` "because it's easier." This throws away the
compiler's ability to catch coupling violations. If a type is only used within `storage/`,
make it `pub(super)` — now the compiler prevents `transport/` from reaching into storage
internals.

**Practical rule:** Start with private. Widen to `pub(super)` when a sibling needs it.
Widen to `pub(crate)` when another top-level module needs it. Widen to `pub` only for
your crate's external API.

## mod.rs: The Table of Contents Pattern

Every directory module needs a `mod.rs`. Keep it lean — it has three jobs:

```rust
// 1. Declare submodules (controls what exists in this namespace)
mod code;
mod doc;
mod traits;

// 2. Re-export the public interface (controls what's visible outside)
pub use traits::IndexStore;
pub use code::CodeIndex;
pub use doc::DocIndex;

// 3. Optionally: a small amount of shared glue (< 30 lines)
//    If the glue grows, extract it to its own submodule.
```

**Why this matters:** When `mod.rs` contains significant logic, it becomes a bottleneck.
Every change to the directory's logic means editing `mod.rs`, which creates merge conflicts
and makes it hard to understand what the module *contains* vs what it *does*.

## lib.rs: Keep It Skeletal

`lib.rs` should be a flat list of `pub mod` declarations — nothing else. No functions,
no type definitions, no `use` statements. It's the root of your module tree, not a
place to put code.

```rust
// Good: lib.rs is just a manifest
pub mod config;
pub mod extract;
pub mod model;
pub mod service;
pub mod storage;
pub mod transport;

// Bad: lib.rs has logic, re-exports, helper functions
pub mod config;
pub use config::APP_NAME;  // Don't — let callers use config::APP_NAME
fn setup_logging() { ... }  // Don't — put this in a module
```

**When lib.rs grows beyond ~15 mod declarations,** that's a signal your top-level is too
flat. Group related modules into directories. `model.rs` + `config.rs` + `types.rs` →
`model/` directory with submodules.

## Anti-Patterns and Fixes

### The Flat Crate

**Symptom:** `lib.rs` has 10+ `pub mod` declarations, all at the root level.

```
src/
├── lib.rs      # pub mod a; pub mod b; pub mod c; ... pub mod k;
├── a.rs
├── b.rs
├── ...
└── k.rs
```

**Problem:** No hierarchy means no enforced layering. Any file can import any other.
Dependency violations creep in silently because there's no structure to prevent them.

**Fix:** Group by layer/domain. Move related `.rs` files into directories with a `mod.rs`.
The number of top-level `pub mod` declarations in `lib.rs` should be 4-7 for a
medium-sized crate.

### The God Module

**Symptom:** One module (often `service/mod.rs`) has 40+ methods, handles all dispatch,
and touches every other module in the crate.

**Problem:** Every new feature adds another method to the god module. Merge conflicts.
Hard to test one concern without loading the entire module.

**Fix:** Split by domain. If `service/mod.rs` dispatches both code and doc operations,
extract `service/code.rs` and `service/doc.rs`. The `mod.rs` keeps a thin dispatch
method that delegates. Each sub-module gets its own impl block on the service type, or
better yet, each becomes its own type behind a shared trait.

See `references/splitting-god-modules.md` for a worked example.

### Parallel Implementations Without Shared Trait

**Symptom:** `storage.rs` and `doc_storage.rs` (or `code_index.rs` and `doc_index.rs`)
implement the same operations — open, index, search, retrieve — with nearly identical
signatures but no shared interface.

**Problem:** Bug fixes must be applied twice. Behavior drifts between the two
implementations. Testing requires separate test suites that check the same invariants.

**Fix:** Extract a shared trait into the parent module, then implement it for both:

```rust
// storage/traits.rs
pub trait IndexStore {
    type Item;
    type Update;

    fn open(path: &Path) -> Result<Self> where Self: Sized;
    fn index(&mut self, updates: Vec<Self::Update>) -> Result<()>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit<Self::Item>>>;
    fn retrieve(&self, id: &str) -> Result<Option<Self::Item>>;
}

// storage/code.rs
impl IndexStore for CodeIndex { type Item = Symbol; ... }

// storage/doc.rs
impl IndexStore for DocIndex { type Item = Section; ... }
```

Now generic code can operate on `impl IndexStore` without knowing which backend it uses.

### Scattered Constants

**Symptom:** Configuration values spread across `config.rs`, `db.rs`, `savings.rs`, and
other modules. To understand "what can I configure?" you must grep the entire crate.

**Fix:** Centralize in `config.rs` (or `config/` directory for larger crates). Each module
imports from `config` — never defines its own constants that belong to the global
configuration surface.

## When to Use Workspaces vs Modules vs Crates

| Scale | Mechanism | Signal |
|---|---|---|
| < 5 files | Flat modules in `lib.rs` | Small utility or prototype |
| 5-30 files | Directory modules with `mod.rs` | Most applications |
| 30-100 files | Directory modules + consider workspace | Large application |
| Independent versioning needed | Separate crate in workspace | Library consumed by multiple binaries |
| Different dependency sets | Separate crate in workspace | One part needs `tokio`, another doesn't |
| Different teams own different parts | Workspace with per-team crates | Organizational boundary |

**The workspace question:** If two parts of your code have **different dependency trees**
(one needs heavy ML crates, another is a thin CLI), splitting into workspace members
reduces compile times and clarifies ownership. But don't split prematurely — a single
crate with good module structure is simpler than a workspace with tangled cross-crate
dependencies.

## Refactoring Checklist

Before finalizing module structure:

- [ ] `lib.rs` is a flat list of `pub mod` declarations — no logic, no re-exports
- [ ] Top-level module count in `lib.rs` is 4-7 for a medium crate
- [ ] Each directory's `mod.rs` is a table of contents — declarations + re-exports only
- [ ] No file exceeds ~500 lines of logic
- [ ] Dependencies point downward only — lower layers don't import upper layers
- [ ] Parallel implementations share a trait defined in their parent module
- [ ] Visibility is as narrow as possible — `pub(super)` before `pub(crate)` before `pub`
- [ ] Constants are centralized in `config`, not scattered across modules

See `references/dependency-layers.md` for a detailed layer validation walkthrough.
See `references/splitting-god-modules.md` for step-by-step god-module decomposition.
