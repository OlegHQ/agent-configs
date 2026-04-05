# Restructuring a Flat 18-File Container CLI in Rust

Your current layout has a classic "drawer full of unrelated socks" problem: every concept at the same level means nothing has meaningful gravity. The fix is not to apply a framework — it's to let the natural domain clusters you already have determine the folder boundaries.

## Identifying the clusters

Before touching any files, map what you have into coherent groups by asking one question per file: "What does this file *know about*, and what is the smallest set of concepts it can't live without?"

Working through your 18 files:

| Cluster | Files | Rationale |
|---|---|---|
| **types** | `manifest.rs`, `digest.rs`, `platform.rs`, `layer.rs` | Pure data representations with no I/O. Everything else depends on these. |
| **auth** | `auth.rs` | Registry credential handling — a coherent boundary, but small enough to stay flat inside `registry/`. |
| **registry** | `registry.rs`, `auth.rs` | The network boundary: speaks OCI Distribution Spec HTTP. `auth.rs` only makes sense in this context. |
| **image** | `oci.rs`, `manifest.rs` (types), `layer.rs` (types) | OCI spec parsing and the image object model. |
| **cache** | `cache.rs` | Local storage of layers. Depends on types, not on registry. |
| **ops** | `pull.rs`, `push.rs`, `build.rs`, `copy.rs` | High-level orchestration. Depend on registry + cache + image, but not on each other. |
| **cli** | `cli.rs`, `main.rs` | Clap definitions and the entry point. Depends on everything else but nothing depends on it. |
| **infra** | `error.rs`, `progress.rs`, `config.rs` | Cross-cutting infrastructure. No domain knowledge. |

## Proposed directory structure

```
src/
├── main.rs                  # binary entry point only: parse args, dispatch
├── lib.rs                   # re-exports what external tests/integration need
├── error.rs                 # crate-wide Error + Result types
│
├── cli/
│   ├── mod.rs               # clap App + top-level Cli struct
│   ├── args.rs              # subcommand arg structs (pull, push, build, copy)
│   └── dispatch.rs          # match arms that call into ops
│
├── types/
│   ├── mod.rs
│   ├── digest.rs            # Sha256Digest newtype + Display/FromStr/serde
│   ├── manifest.rs          # ImageManifest, ManifestList, MediaType enums
│   ├── layer.rs             # LayerDescriptor, DiffID, tar helpers
│   └── platform.rs          # Platform, Os, Arch + OCI filtering logic
│
├── registry/
│   ├── mod.rs               # public RegistryClient facade
│   ├── auth.rs              # token refresh, credential sources
│   ├── client.rs            # raw HTTP: GET blob, HEAD, POST initiate-upload
│   └── errors.rs            # registry-specific error variants (rate limit, 401, etc.)
│
├── image/
│   ├── mod.rs               # Image, ImageRef parsing ("ubuntu:22.04", digests)
│   └── oci.rs               # OCI spec parsing: config blob, layer ordering
│
├── cache/
│   ├── mod.rs               # LayerCache trait
│   └── fs.rs                # filesystem-backed implementation
│
├── ops/
│   ├── mod.rs               # re-exports the four ops
│   ├── pull.rs              # orchestrate: resolve manifest → fetch layers → populate cache
│   ├── push.rs              # orchestrate: read cache → push missing blobs → push manifest
│   ├── build.rs             # Dockerfile parse → layer construction → local image
│   └── copy.rs              # pull from src registry → push to dst registry
│
└── progress.rs              # indicatif wrappers — still top-level, referenced everywhere
    config.rs                # AppConfig, constants, env-var loading
```

Two files stay top-level because they are genuinely cross-cutting and short: `error.rs` (everything depends on it; nesting it forces awkward `crate::some_module::error` paths) and `progress.rs` / `config.rs` (infrastructure used by multiple clusters with no natural home).

## The dependency rule

The structure only works if you enforce one rule: **dependencies point inward, never outward**.

```
cli → ops → registry / cache / image → types → (std, serde, etc.)
                                ↑
                           error, config, progress (cross-cutting, any layer may use)
```

`ops` may not import `cli`. `registry` may not import `ops`. `types` may not import anything in your crate. If you find yourself needing to import "upward," that is a signal that a type or trait belongs one level lower.

## lib.rs after the restructuring

Stop declaring 16 `pub mod` entries in `lib.rs`. Instead, only re-export what integration tests and potential future library users need:

```rust
// lib.rs
pub mod error;
pub mod types;
pub mod registry;
pub mod image;
pub mod cache;
pub mod ops;
pub mod config;
pub mod progress;

// cli is binary-only; do not pub-export it from the library
```

Keeping `cli` out of `lib.rs` means your crate can be used as a library without pulling in Clap.

## Splitting lib.rs vs keeping a binary-only crate

If this tool will never be consumed as a library, consider a cleaner split using Cargo workspaces:

```
Cargo.toml              # workspace root
container-core/         # lib crate: types, registry, image, cache, ops
  Cargo.toml
  src/
    lib.rs
    ...
container-cli/          # bin crate: cli, main, progress, config
  Cargo.toml
  src/
    main.rs
    cli/
    ...
```

This gives you a hard compile-time boundary: `container-cli` depends on `container-core`, but not vice versa. You get faster incremental builds (core recompiles only when core changes) and the ability to write integration tests against `container-core` without the CLI layer in scope. For a team tool or something you might wrap with a different frontend (a TUI, a daemon), the workspace split pays for itself immediately.

For a personal tool that will stay CLI-only, a single crate with the directory structure above is fine.

## Handling the types that appear in multiple modules

`manifest.rs`, `layer.rs`, and `digest.rs` currently exist as files at the crate root. After the restructuring they live under `types/`. The temptation is to scatter them: put `manifest.rs` under `image/` because it feels like an "image concept." Resist this. A manifest is referenced by `registry` (when fetching), by `image` (when parsing), by `ops` (when orchestrating), and by `cache` (when storing metadata). Putting it under any one of those creates import noise everywhere else. A dedicated `types` module solves this cleanly — it is the only module every other module imports.

## Migrating incrementally

You do not need to do this all at once. A safe sequence:

1. Create `src/types/` and move `digest.rs`, `platform.rs` into it. Update imports. Compile.
2. Move `manifest.rs` and `layer.rs` into `types/`. Compile.
3. Create `src/registry/` and move `registry.rs` → `registry/client.rs`, `auth.rs` → `registry/auth.rs`. Compile.
4. Create `src/image/` and move `oci.rs`. Compile.
5. Create `src/ops/` and move the four orchestrators. Compile.
6. Create `src/cli/` and split `cli.rs`. Compile.
7. Clean up `lib.rs`.

Each step is a pure rename + import-path update with no logic changes. Git history stays readable.

## What not to do

- Do not create a `utils/` or `common/` folder. This is a magnet for unrelated code and tells you nothing about what lives inside. If something is genuinely shared, it either belongs in `types/` (if it is a data type) or deserves its own named module.
- Do not mirror the dependency tree mechanically with one folder per crate dependency. Your modules should reflect your domain, not your `Cargo.toml`.
- Do not over-nest. Three levels deep (`src/registry/auth.rs`) is the practical maximum for a single-binary CLI. Going deeper creates import paths that are more painful than the flat layout you started with.
- Do not put `error.rs` inside a subdirectory. Crate-wide error types used by every module should live at the crate root so the import is always `crate::error::Error`, not `crate::some_submodule::error::Error`.
