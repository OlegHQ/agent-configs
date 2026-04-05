# Restructuring a Container Image CLI: 18 Flat Files to a Domain-Component Layout

## Step 0: Evaluate Architecture (Required First Step)

Work through the decision tree before proposing any structure.

**Q1: Are components independently publishable or reusable as libraries?**

No. `registry.rs`, `auth.rs`, `oci.rs` are not intended for publication — they exist only to serve this CLI. Workspace crates (Pattern C) are premature here. One binary, everything depends on everything else today.

**Q2: Does the code flow through clear sequential phases?**

Partially. Pull and push look like phases: auth → registry → manifest → layer → cache. But this is not the dominant shape of the whole codebase. `build.rs`, `copy.rs`, and the OCI/manifest/digest/platform types don't fit cleanly into a single sequential pipeline. If you forced phase names on top of everything, you'd end up scattering related logic across multiple directories.

**Q3: Are there distinct domains that own their own types and logic?**

Yes. This is the right answer. The 18 files fall into distinct clusters that share types within a cluster but are largely independent across clusters:

- Registry communication cluster: `registry.rs`, `auth.rs`
- Image data cluster: `oci.rs`, `manifest.rs`, `layer.rs`, `digest.rs`, `platform.rs`
- Local storage cluster: `cache.rs`
- Operations cluster: `pull.rs`, `push.rs`, `build.rs`, `copy.rs`
- Infrastructure cluster: `error.rs`, `progress.rs`, `config.rs`
- CLI wiring: `cli.rs`, `main.rs`, `lib.rs`

**Verdict: Pattern A — Domain Components**, with a small shared infrastructure module.

---

## The Proposed Structure

```
src/
├── lib.rs                  # Flat mod declarations only, no logic, no re-exports
├── main.rs                 # Calls cli::run()
│
├── image/                  # Domain: OCI image data model
│   ├── mod.rs              # pub use types, pub use digest::Digest, etc.
│   ├── types.rs            # ImageConfig, MediaType, RootFs, ImageIndex
│   ├── manifest.rs         # Manifest, ManifestList — parsing + serialization
│   ├── layer.rs            # Layer, LayerDescriptor, tar handling
│   ├── digest.rs           # Digest newtype, sha256 computation + verification
│   └── platform.rs         # Platform, os/arch filtering logic
│
├── registry/               # Domain: Registry communication
│   ├── mod.rs              # pub use client::RegistryClient, pub use auth::Credential
│   ├── client.rs           # HTTP client: manifest fetch/push, layer upload/download
│   ├── auth.rs             # Token auth, credential resolution, challenge handling
│   └── refs.rs             # Reference parsing: "ubuntu:22.04", digest refs, tags
│
├── cache/                  # Domain: Local layer cache
│   ├── mod.rs              # pub use Cache
│   ├── layout.rs           # Directory structure: blobs/sha256/<digest>
│   └── store.rs            # Read/write/verify cached layers and manifests
│
├── ops/                    # Domain: High-level image operations (orchestrators)
│   ├── mod.rs
│   ├── pull.rs             # Pull: resolve ref → fetch manifest → pull layers → cache
│   ├── push.rs             # Push: read cache → check existing → upload layers + manifest
│   ├── build.rs            # Build: parse Dockerfile → layer creation → manifest assembly
│   └── copy.rs             # Copy: pull from src registry → push to dst registry
│
├── cli/                    # Wiring only: clap args + dispatch
│   ├── mod.rs              # pub fn run() — top-level entry point
│   ├── args.rs             # Clap structs: Cli, Commands, PullArgs, PushArgs, etc.
│   └── dispatch.rs         # Match on Commands, call ops::*, handle top-level errors
│
└── infra/                  # Shared infrastructure (imported by 4+ domains)
    ├── mod.rs
    ├── error.rs            # Error enum (or anyhow wrapper), AppError
    ├── progress.rs         # Progress bars, spinners — thin wrapper over indicatif
    └── config.rs           # CLI config, constants, env var reading
```

---

## Rationale for Each Group

### `image/` — the OCI data model

`manifest.rs`, `oci.rs`, `layer.rs`, `digest.rs`, and `platform.rs` all define or manipulate OCI image data. They are peers: `manifest.rs` references `digest.rs` types, `layer.rs` references `digest.rs`, and `platform.rs` filters `image/` types. They should live together.

Rename `oci.rs` → `image/types.rs`. It was a catch-all for OCI spec types that properly belong alongside the other image data files. The old `manifest.rs` (image manifest types) becomes `image/manifest.rs`.

`digest.rs` becomes `image/digest.rs` — the `Digest` type is part of the image data model, not a standalone utility.

`platform.rs` → `image/platform.rs`. Platform filtering is image-domain logic; it operates on `image::types::Platform` and filters `image::manifest::ManifestList` entries.

### `registry/` — registry communication

`registry.rs` (Docker Hub API calls) and `auth.rs` (registry auth) are tightly coupled: every registry request depends on auth, and auth state is registry-specific. They belong in the same domain.

Split old `registry.rs` into `registry/client.rs` (HTTP calls) and `registry/refs.rs` (reference string parsing — `ubuntu:22.04` → host + name + tag). Auth challenge handling lives in `registry/auth.rs`.

### `cache/` — local layer cache

`cache.rs` is a distinct domain. It owns the on-disk layout and read/write/verify operations. If it has both layout logic and I/O, split into `cache/layout.rs` + `cache/store.rs`. If it's small and focused, a single `cache/store.rs` (with `mod.rs` for the public interface) is fine — don't split prematurely.

### `ops/` — orchestrators

`pull.rs`, `push.rs`, `build.rs`, `copy.rs` are operations: they import from `registry/`, `image/`, and `cache/` and coordinate them. They don't own types — they own workflows. Grouping them in `ops/` makes the dependency direction explicit: ops depend on the domains below, not the other way around.

The old flat layout allowed `pull.rs` to accidentally import from `push.rs` (if they shared logic). In `ops/`, that would be visible and discouraged — shared logic gets extracted to `ops/mod.rs` or down into the appropriate domain.

### `cli/` — thin wiring only

`cli.rs` and `main.rs` become `cli/args.rs` and `cli/dispatch.rs`. The `cli/` module should have zero business logic. It parses args and calls `ops::*`. If dispatch logic starts growing (retry loops, progress setup, config loading), move that into `ops/` or `infra/`.

### `infra/` — shared infrastructure

`error.rs`, `progress.rs`, and `config.rs` are imported by everything. They are not a domain — they are cross-cutting infrastructure. A dedicated `infra/` module (rather than leaving them flat at the root) signals "these are shared utilities, not domain logic."

Alternative: keep `error.rs` at the crate root as a single file if it's small. The signal for moving it into `infra/` is when it needs to grow or split. For now, `infra/error.rs` is fine.

---

## Dependency Direction

After restructuring, the import graph should look like this:

```
cli/          → ops/, infra/
ops/          → registry/, image/, cache/, infra/
registry/     → image/ (for Digest, Manifest types), infra/
cache/        → image/ (for Digest, Layer types), infra/
image/        → infra/ (for Error only)
infra/        → (nothing in this crate)
```

**Violations to watch for:**

- `image/` must NOT import from `registry/` or `cache/`. If it does, you have a domain-inversion: the data model depends on its consumers.
- `registry/` must NOT import from `ops/`. Registry makes HTTP calls; it doesn't know about pull/push workflows.
- `cache/` must NOT import from `registry/`. Cache is local storage; it doesn't know about remote registries.
- `ops/copy.rs` may import from both `ops/pull.rs` and `ops/push.rs` internal helpers — but only if those helpers are promoted to `pub(crate)` in `ops/mod.rs`. If copy.rs has enough unique logic, it should not call pull/push directly; it should call the same `registry/` and `cache/` APIs directly.

---

## The `lib.rs` After Restructuring

```rust
// lib.rs — declarations only, no logic, no re-exports
pub mod image;
pub mod registry;
pub mod cache;
pub mod ops;
pub(crate) mod cli;
pub(crate) mod infra;
```

If this is a binary-only crate (no external library consumers), `main.rs` can call `cli::run()` directly and `lib.rs` can use `pub(crate)` for everything.

---

## Migration Order

Do this incrementally. Each step compiles and passes tests before the next step.

1. **Create the directories** (empty `mod.rs` stubs). Add them to `lib.rs`. `cargo check` passes.
2. **Move `infra/`** first — `error.rs`, `progress.rs`, `config.rs`. Everything else already imports these; now they import from `infra::error`, etc. Update all `use crate::error` → `use crate::infra::error`. One commit.
3. **Move `image/`** — `digest.rs`, `platform.rs`, `layer.rs`, `manifest.rs`, old `oci.rs` → `image/types.rs`. Update imports. One commit.
4. **Move `registry/`** — split `registry.rs` into `client.rs` + `refs.rs`, move `auth.rs`. One commit.
5. **Move `cache/`**. One commit.
6. **Move `ops/`** — `pull.rs`, `push.rs`, `build.rs`, `copy.rs`. One commit.
7. **Move `cli/`** — split `cli.rs` into `args.rs` + `dispatch.rs`. One commit.
8. **Audit the dependency graph** — trace all `use crate::` imports, verify no upward arrows. Fix any violations found.

---

## Refactoring Checklist (Completed)

- [x] Architectural pattern chosen via decision tree (Pattern A: Domain Components)
- [x] `lib.rs` will be a flat list of `mod` declarations, no logic
- [x] Each directory answers "what domain is this?" in one sentence
- [x] No model bag — types live with the domain that owns them (`Digest` in `image/`, not a top-level `types.rs`)
- [x] Dependencies point inward — `ops/` → `registry/` → `image/` → `infra/`
- [x] `mod.rs` files serve as tables of contents and public facades only
- [x] Migration is incremental — compile + test after each step
- [x] Visibility is as narrow as possible — `pub(crate)` for cross-domain, `pub(super)` within a domain
