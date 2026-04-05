# Should You Reorganize into model/ / service/ / io/?

**Short answer: No — not as described. The proposed split has a real insight buried inside a problematic grouping. Reorganize, but do it along dependency layers, not technical kinds.**

---

## What the Proposal Gets Right

The suggestion recognizes that the flat `src/` tree has no enforced structure. With 12+ files all at the same level, any file can import any other. Dependency violations creep in silently. The instinct to add hierarchy is correct.

It also correctly identifies that `types.rs` (shared types used everywhere) belongs at the bottom of the hierarchy — that's the `model/` insight.

---

## What the Proposal Gets Wrong

The `model / service / io` grouping is organized by **technical kind**, not **dependency direction**. That distinction matters enormously in Rust.

Consider what ends up in `io/`:

- `cache.rs` — content-addressed cache: stores and retrieves package tarballs by hash
- `index.rs` — registry index: metadata lookups, version listings
- "registry HTTP" — raw HTTP client for the registry

These three have very different consumers and roles:
- `cache` is used by `fetch` (download, then cache) and by `staging` (read from cache to unpack)
- `index` is used by `resolve` (what versions exist?) and by `fetch` (what's the download URL?)
- The HTTP layer is used by `fetch` and `index`

If you put them all in `io/`, you haven't reduced coupling — you've just renamed the flat structure with a folder. `service/resolve.rs` still reaches into `io/index.rs`; `service/staging.rs` still reaches into `io/cache.rs`. The imports cross the `service/` ↔ `io/` boundary in both directions, which is exactly the problem you were trying to solve.

The `service/` bucket has the same issue. `resolve` and `fetch` are not peers — `fetch` depends on `resolve` (you resolve first, then fetch what resolution decided). Lumping them as siblings in `service/` hides this ordering.

---

## The Right Lens: Dependency Layers

Draw the actual dependency graph of the current flat files:

```
types.rs, error.rs, config.rs          ← no local imports; pure data
        ↑
manifest.rs, lockfile.rs               ← parse files, produce types
        ↑
index.rs                               ← queries registry; depends on types
        ↑
resolve.rs                             ← pubgrub; depends on index + types
        ↑
fetch.rs                               ← downloads; depends on resolve + index
        ↑
cache.rs                               ← stores fetched content; depends on types
        ↑  (cache ← fetch, but also cache ← staging)
staging.rs                             ← unpacks from cache; depends on cache + types
        ↑
install.rs                             ← moves staged to final location; depends on staging
        ↑
cli.rs                                 ← top-level dispatch; depends on everything
```

This graph has a natural layered shape. That's your reorganization target — not technical kind buckets.

---

## A Better Structure

```
src/
├── lib.rs                 # pub mod declarations only — no logic
├── main.rs                # arg parsing + wiring, delegates to cli
│
├── model/                 # Layer 0: pure data, no I/O, no side effects
│   ├── mod.rs             #   re-exports
│   ├── types.rs           #   ModuleId, PackageSpec, VersionReq, etc.
│   ├── manifest.rs        #   parsed manifest types + deserialization
│   └── lockfile.rs        #   lockfile types + serialization
│
├── config.rs              # Layer 0: paths, constants, env-var reading
├── error.rs               # Layer 0: crate-wide error type (depends on model)
│
├── registry/              # Layer 1: registry knowledge (depends on: model, config)
│   ├── mod.rs
│   ├── index.rs           #   version listings, metadata queries
│   └── http.rs            #   raw HTTP; everything else uses index, not http directly
│
├── cache.rs               # Layer 1: content-addressed store (depends on: model, config)
│
├── resolve.rs             # Layer 2: pubgrub resolution (depends on: registry, model)
│
├── fetch.rs               # Layer 2: download + cache (depends on: resolve, registry, cache)
│
├── staging.rs             # Layer 3: unpack + prepare (depends on: cache, model)
│
├── install.rs             # Layer 3: finalize install (depends on: staging, model)
│
└── cli.rs                 # Layer 4: CLI commands (depends on: resolve, fetch, staging, install)
```

Key decisions explained:

**`model/` is the only real subdirectory.** `manifest.rs` and `lockfile.rs` belong there because they are pure data definitions with serialization — no network, no filesystem side effects. They're naturally grouped: both describe the package manager's on-disk state format. `types.rs` moves in with them.

**`registry/` groups `index.rs` and `http.rs`.** These two are tightly coupled — `http.rs` is the transport backing `index.rs`. Nothing outside `registry/` should import `http.rs` directly; they should go through `index.rs`. Make `http.rs` private to the module: `mod http;` in `registry/mod.rs` without re-exporting it. This enforces the abstraction.

**`cache.rs` stays a single file at the top level** (or moves into `registry/` only if it's tightly coupled to registry HTTP — for a content-addressed cache it usually isn't). It's a single responsibility: given a hash, store or retrieve bytes on disk.

**`resolve.rs`, `fetch.rs`, `staging.rs`, `install.rs` stay as single files** — they each represent one well-defined stage of the pipeline. There's no reason to group them into a `service/` directory unless they share types or a trait interface. Adding a folder with `mod.rs` just for organizational aesthetics creates noise.

**`cli.rs` is the only consumer of everything.** It sits at the top and is the one file that is allowed to import broadly.

---

## Visibility Enforcement

Once you have this shape, use visibility to prevent layer violations:

```rust
// registry/mod.rs
mod http;           // private — only index.rs uses the HTTP client
pub mod index;      // pub(crate) is fine if only used within the crate

// cache.rs
pub(crate) struct Cache { ... }    // not part of external API
pub(crate) fn store(...) { ... }
pub(crate) fn retrieve(...) { ... }
```

If `staging.rs` tries to import `registry::http`, the compiler refuses. That's the enforcement you want — the structure prevents the violation, not code review.

---

## When the `service/` Layer Is Actually Useful

The `service/` abstraction earns its keep when:

1. You have a **shared trait** that multiple implementations satisfy. For example, if you support multiple registries (crates.io, a private registry, a local file registry), a `RegistryClient` trait in a `registry/` module with multiple backends (`crates_io.rs`, `private.rs`, `local.rs`) is a clean use of directory + trait grouping.

2. You have a **facade type** that combines several lower-layer operations and is injected into multiple upper-layer consumers. A `PackageManager` struct that holds a `Cache`, `Resolver`, and `Fetcher` and coordinates them is worth putting in `service/mod.rs`.

For the codebase as described, neither condition is clearly present. The pipeline stages are sequential, not parallel implementations of the same interface. A flat sequence of well-named files with clear dependency direction is more readable than wrapping them in a `service/` directory that adds a layer of indirection without adding a layer of abstraction.

---

## Summary

| Concern | Flat src/ | Proposed model/service/io | Layered structure |
|---|---|---|---|
| Dependency direction enforced | No | Partially (model layer ok, others muddled) | Yes |
| Easy to find where a concept lives | Moderate | Moderate | Yes |
| Prevents cross-layer coupling | No | No | Yes (via visibility) |
| Matches actual dependency graph | No | No | Yes |
| Avoids over-engineering | Yes | No (premature grouping) | Yes |

Reorganize the flat tree — but draw the dependency graph first, let that graph determine the layers, and only create a subdirectory when you have either multiple files that share a parent abstraction (trait + implementations) or two files that are so tightly coupled they should be one module with a private submodule (like `registry/` + `http.rs`).
