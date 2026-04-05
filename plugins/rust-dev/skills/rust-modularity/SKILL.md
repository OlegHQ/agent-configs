---
name: rust-modularity
description: |
  Guides Rust module structure, folder organization, and dependency direction for clean
  separation of concerns. CRITICAL FIRST STEP: evaluates the codebase to determine which
  architectural pattern fits (component-based, phase-pipeline, layered, workspace) before
  prescribing any structure. Use this skill whenever structuring a new Rust project, adding
  modules, refactoring a tangled module tree, splitting a growing file, or reorganizing code
  with modularity debt. Triggers on ANY Rust structural task. Complements rust-design-patterns
  (which covers individual code patterns like Builder/Strategy/enum-vs-trait decisions) — this
  skill focuses on the module graph, folder tree, and dependency direction between modules.
---

# Rust Modularity

Structure modules so that adding a new feature means adding a new file in the right
place — not touching ten existing ones.

**Before restructuring anything, evaluate the codebase to determine which architectural
pattern fits.** Forcing the wrong pattern creates artificial groupings that obscure domain
boundaries — the cure is worse than the disease.

## Step 0: Evaluate Architecture

Every restructuring starts here. Skip this and you'll impose a structure that fights the
code's natural shape.

### The Decision Tree

Answer these questions about the codebase in order:

```
1. Are components independently publishable or reusable as libraries?
   ├── Yes → Workspace crates (Pattern C)
   └── No ↓

2. Does the code flow through clear sequential phases?
   (e.g., parse → resolve → fetch → cache → stage → launch)
   ├── Yes → Phase-pipeline modules (Pattern B)
   └── No ↓

3. Are there distinct domains that own their own types and logic?
   (e.g., "auth", "billing", "inventory" that share little)
   ├── Yes → Domain component modules (Pattern A)
   └── No ↓

4. Is there a clear request/response or input/output boundary?
   (e.g., HTTP handler → service → storage)
   ├── Yes → Layered modules (Pattern D)
   └── No → Start flat, group when patterns emerge
```

### Pattern A: Domain Components

**When it fits:** Distinct domains that each own their types, logic, and I/O. Domains
share some types but are otherwise independent. Most Rust CLIs and applications.

**Shape:** Each top-level module is a self-contained domain. Shared types live with
the domain that owns them, or in a small shared module when genuinely cross-cutting.

```
src/
├── lib.rs
├── github/          # GitHub API: URL parsing, downloads, ref resolution, types
├── cache/           # Package caching: layout, materialization, restore, indexing
├── lockfile/        # pack.lock: data model, serialization, init
├── manifest/        # agentpack.toml: parsing, editing, types
├── staging/         # Harness staging: merge cached trees into plugin layouts
├── launcher/        # Execute harnesses: Claude, Codex, Cursor, OpenCode
└── cli.rs           # Arg parsing + dispatch (thin wiring)
```

**Key property:** Each directory answers "what domain is this?" not "what technical
layer is this?" A type like `GitHubSource` lives in `github/`, not in a generic
`model/` bag — because `github/` is the domain that defines, creates, and validates it.

**Real-world examples:** mise (`cli/`, `backend/`, `config/`, `toolset/`, `shell/`),
bat (`assets/`, `controller.rs`, `printer.rs`), starship (`modules/`, `configs/`,
`formatter/`).

### Pattern B: Phase-Pipeline

**When it fits:** Code flows through a clear sequence of processing phases. Each phase
transforms data and passes it to the next. Common in compilers, build tools, and
package managers.

**Shape:** Modules named after what they *do* in the pipeline, not what technical
kind they are.

```
src/
├── lib.rs
├── parse/           # Phase 1: read config files, parse syntax
├── resolve/         # Phase 2: dependency resolution, constraint solving
├── fetch/           # Phase 3: download/cache from registries
├── build/           # Phase 4: compile, link, transform artifacts
├── install/         # Phase 5: place artifacts in target locations
├── types.rs         # Shared types that flow between phases (small!)
└── cli.rs           # Entry point, wiring
```

**Key property:** Types flow *forward* through the pipeline. `resolve/` produces a
lockfile that `fetch/` consumes. Backward dependencies (fetch importing from build)
are violations. A small shared `types.rs` is acceptable for types that genuinely
cross 3+ phases — but resist making it a dumping ground.

**Real-world examples:** cargo (resolve → download → compile → install), rustc
(parse → expand → lower → codegen), uv (resolve → fetch → build → install).

### Pattern C: Workspace Crates

**When it fits:** Components are independently useful libraries, have different
dependency trees, or the team is large enough that compile-time isolation matters.

**Shape:** Multiple `Cargo.toml` files, one per crate, orchestrated by a root workspace.

```
crates/
├── core/            # Thin CLI shell, wiring only
├── resolver/        # Dependency resolution (publishable library)
├── registry/        # Registry API client
├── cache/           # Content-addressed storage
└── types/           # Shared types (keep minimal!)
```

**Warning signs you're splitting too early:**
- Every crate depends on `*-types` (you just moved the coupling, not removed it)
- Cross-crate refactoring is painful because of coordinated `Cargo.toml` changes
- Only one binary consumes all the crates

**Real-world examples:** ripgrep (10 crates — `grep`, `globset`, `ignore` are
independently published), uv (67+ crates — justified by team size and reuse).

### Pattern D: Layered

**When it fits:** Clear request/response boundary where handlers delegate to
services that delegate to storage. Common in web servers and RPC services.

**Shape:** Modules named by their distance from the boundary.

```
src/
├── lib.rs
├── transport/       # HTTP/gRPC handlers, serialization
├── service/         # Business logic, orchestration
├── storage/         # Database, filesystem, external APIs
└── types/           # Shared data model
```

**Warning signs you're forcing layers:**
- Your "model" bag mixes `HttpRequest`, `DbRow`, `ConfigEntry`, and `DomainEvent`
- Your "service" layer is a pass-through that adds no logic
- Pipeline code gets scattered across three directories per phase

### Hybrid: Domains With Internal Layers

Real codebases often combine patterns. A domain component can have internal layering:

```
src/
├── github/          # Domain component
│   ├── mod.rs       # Public API (facade)
│   ├── types.rs     # GitHubSource, TagEntry (domain types)
│   ├── api.rs       # HTTP calls (I/O layer)
│   ├── cache.rs     # Metadata caching (storage layer)
│   └── parse.rs     # URL parsing (pure logic)
├── cache/           # Another domain component
│   ├── mod.rs
│   ├── layout.rs    # Directory structure logic
│   └── restore.rs   # Download + verify (I/O)
```

Each component is self-contained. Internal layering within a component is fine — it's
forced *top-level* layering across unrelated domains that creates problems.

## The Dependency Rule

Regardless of which pattern you chose, one rule is universal:

**Dependencies point inward — never from infrastructure toward the code that uses it.**

In practice:
- A domain component should not import from the CLI/transport layer
- A pipeline phase should not import from a later phase
- A storage module should not import from the service that calls it

**How to check:** Trace each `use crate::` import. Draw an arrow. If any arrow points
"upward" (toward the entry point / orchestration layer), that's a violation.

**How to fix violations:**
1. **Extract downward:** Move the shared type/function to the module that should own it
2. **Trait inversion:** Lower module defines a trait, upper module implements it
3. **Small shared module:** For types genuinely used by 3+ components, a focused `types.rs`
   (not a catch-all `model/`)

See `references/dependency-analysis.md` for the full audit process.

## Module Mechanics

These rules apply regardless of architectural pattern.

### lib.rs: Skeletal

`lib.rs` is a flat list of `mod` declarations. No functions, no type definitions, no
`use` re-exports. If it has logic, extract it to a named module.

```rust
// Good
pub mod cache;
pub mod github;
mod cli;
mod staging;

// Bad
pub mod cache;
pub use cache::CacheKey;  // Let callers use cache::CacheKey
fn setup() { ... }        // Put this in a module
```

### mod.rs: Table of Contents

Three jobs only: declare submodules, re-export the public interface, optionally < 30
lines of glue. If glue grows, extract it to a submodule.

```rust
mod api;
mod parse;
mod types;

pub use types::GitHubSource;
pub use parse::parse_github_url;
pub use api::{download_tarball, resolve_ref};
```

### Visibility: Narrowest First

| Visibility | Use when |
|---|---|
| private | Default. Same module only. |
| `pub(super)` | Sibling in same directory needs it. |
| `pub(crate)` | Another top-level module needs it. |
| `pub` | Part of the crate's external API. |

Start private. Widen only when the compiler tells you to.

### When to Split a File

Split on **multiple responsibilities**, not on line count. A 400-line file with one
focused job is fine. A 200-line file with three unrelated sections should split.

**Signals:**
- `// --- Section ---` comments separating unrelated blocks
- Two structs/impls that don't reference each other
- You need `pub(super)` but the item is in the wrong module
- A match dispatches to logic that should live in separate files

**How:** Move code to a sibling file. Declare in `mod.rs`. Use `pub(super)` for
parent-only visibility. Re-export anything that was previously public.

### When to Use Workspaces

| Signal | Action |
|---|---|
| Components are independently publishable | Workspace crates |
| Different parts need different heavy dependencies | Workspace crates |
| Different teams own different parts | Workspace crates |
| Everything depends on everything else | Stay single crate |
| Only one binary consumes the code | Stay single crate |

## Anti-Patterns

### The Model Bag

**Symptom:** A `model/` or `types/` directory that mixes `GitHubSource`, `CacheKey`,
`Lockfile`, `Manifest`, `StagingConfig`, and `CliArgs` because they're all "data types."

**Problem:** These types belong to different domains. Changes to GitHub types shouldn't
require navigating past lockfile types. The bag grows without bound and creates false
coupling — everything depends on `model/` so nothing can move independently.

**Fix:** Each type lives with the domain that owns it. `GitHubSource` in `github/`,
`PackLock` in `lockfile/`, `Manifest` in `manifest/`. When a type is genuinely shared
across 3+ domains, place it in a small, focused `types.rs` at the crate root — but
resist the urge to expand it.

### Layer-Forcing on Pipeline Code

**Symptom:** A pipeline tool (parse → resolve → fetch → stage) organized as
`model/`, `service/`, `io/` layers. Each pipeline phase is scattered across three
directories.

**Problem:** To understand "how does resolution work?" you must read files in three
different directories. Adding a new phase means touching every layer directory.

**Fix:** Name modules after phases (`resolve/`, `fetch/`, `staging/`), not after
technical layers. Each phase module contains its own types, logic, and I/O.

### The Flat Crate

**Symptom:** `lib.rs` has 10+ root-level `pub mod` declarations with no hierarchy.

**Problem:** No structure means no enforced dependency direction. Any file can import
any other. Violations creep in silently.

**Fix:** Group related modules into directories. Ask: "which modules share a domain?"
and "which modules are part of the same pipeline phase?" Group accordingly.

### The God Module

**Symptom:** One module has 300+ lines, 3+ unrelated impl blocks, and is the most
frequently edited file.

**Fix:** Split by responsibility. Each responsibility becomes a submodule. The parent
`mod.rs` keeps a thin dispatch/facade. See `references/splitting-god-modules.md`.

### Premature Crate Splitting

**Symptom:** Extracting `github/` into its own crate when it still needs `ModuleId`,
`CacheKey`, `Error`, and `Ui` from the parent.

**Problem:** You create a `*-types` crate that is really "everything shared" — the same
model bag, now with Cargo.toml overhead and painful cross-crate refactoring.

**Fix:** Stay single-crate until a component is genuinely independent or the team
needs compile-time isolation.

## Integration with rust-design-patterns

This skill decides **where code lives** (module tree, folder structure, dependency
direction). The companion `rust-design-patterns` skill decides **how code is shaped**
(enum vs trait, Strategy, Builder, pipeline composition).

Use them together:

1. **rust-modularity** first: evaluate architecture, determine module structure
2. **rust-design-patterns** second: within each module, apply appropriate patterns
   (e.g., the harness stager uses Strategy pattern inside `staging/`)

A common workflow: rust-modularity identifies a god module that needs splitting.
rust-design-patterns determines whether the split pieces should be trait impls,
enum variants, or independent structs.

## Refactoring Checklist

Before finalizing module structure:

- [ ] Architectural pattern is chosen based on the decision tree, not assumed
- [ ] `lib.rs` is a flat list of `mod` declarations — no logic, no re-exports
- [ ] Each directory answers "what domain/phase is this?" in one sentence
- [ ] No "model bag" — types live with the domain that owns them
- [ ] Dependencies point inward — trace `use crate::` imports to verify
- [ ] `mod.rs` files are tables of contents (< 30 lines of logic)
- [ ] No file exceeds ~500 lines of logic
- [ ] Visibility is as narrow as possible
- [ ] Parallel implementations share a trait (coordinate with rust-design-patterns)

See `references/dependency-analysis.md` for a step-by-step dependency audit.
See `references/splitting-god-modules.md` for god-module decomposition.
See `references/architecture-examples.md` for real-world Rust project structures.
