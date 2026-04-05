# Package Manager Restructuring: Should You Use model/service/io Layers?

**Short answer: No. The proposed layering fights the natural shape of a package manager. Use phase-pipeline modules instead.**

---

## Step 0: Evaluate Architecture (Decision Tree)

Before accepting or rejecting the proposal, apply the decision tree from the skill:

**Question 1: Are components independently publishable?**
No — `resolve`, `fetch`, `cache`, and `index` are tightly coupled and only consumed by this one binary. No workspace split yet.

**Question 2: Does code flow through clear sequential phases?**
Yes. This is the defining signal:

```
manifest.rs → resolve.rs → fetch.rs → cache.rs → staging.rs → install.rs
```

Data flows forward. `resolve` produces a locked dependency set. `fetch` downloads what `resolve` identified. `cache` stores what `fetch` downloaded. `staging` uses the cache. `install` finalizes staging output. This is a textbook phase-pipeline (Pattern B).

**Conclusion: Pattern B (Phase-Pipeline).** Stop here. Do not layer.

---

## Why the Proposed model/service/io Split Fails

The proposal forces Pattern D (layered architecture) onto Pattern B (pipeline) code. The skill explicitly calls this out as an anti-pattern called **"Layer-Forcing on Pipeline Code"**:

> A pipeline tool (parse → resolve → fetch → stage) organized as `model/`, `service/`, `io/` layers. Each pipeline phase is scattered across three directories.
> **Problem:** To understand "how does resolution work?" you must read files in three different directories.

Let's trace what happens to a single concern — dependency resolution — under the proposed layout:

- `resolve.rs` logic moves to `service/resolve.rs`
- The `SolvedDependency` or equivalent output type moves to `model/` (it's a "type")
- The registry HTTP call resolve needs moves to `io/` (it's "I/O")

Now, to understand how resolution works, a reader must open `model/`, `service/`, and `io/`. To add a new resolution strategy, they must touch all three directories. The phase is scattered.

The same scattering happens to every phase: `fetch`, `staging`, and `install` each get their types stolen by `model/`, their logic kept in `service/`, and their I/O moved to `io/`.

---

## The Second Problem: The Model Bag

Grouping `types.rs`, `manifest.rs`, `lockfile.rs`, and `config.rs` into `model/` creates a **model bag** — a directory that mixes types from different domains because they are all "data types."

`Manifest` is the type that the manifest-parsing phase owns. `Lockfile` is the type the lockfile phase owns. `ModuleId`, `PackageSpec`, and `VersionReq` from `types.rs` are genuinely cross-cutting. These have different owners and different reasons to change. Lumping them together means:

- A change to lockfile serialization sits next to a change to manifest parsing — false coupling
- `model/` grows without bound because everything is "data"
- Nothing can move independently because everything imports from `model/`

---

## What to Do Instead

Organize by pipeline phase. Each module contains its own types, logic, and I/O:

```
src/
├── lib.rs           # Flat mod declarations only — no logic
├── manifest.rs      # Phase 0: parse agentpack.toml / Cargo.toml equivalent
├── resolve/         # Phase 1: PubGrub solver, constraint types, resolution output
├── fetch/           # Phase 2: registry downloads, HTTP client
├── cache/           # Phase 2.5: content-addressed storage, layout, materialization
├── index/           # Registry index: metadata, version listings (fetch dependency)
├── staging/         # Phase 3: prepare install directory from cached packages
├── install/         # Phase 4: copy staged output to final location
├── types.rs         # SMALL: ModuleId, PackageSpec, VersionReq — only if used by 3+ phases
├── config.rs        # Tool-level config (separate concern, not a "phase")
├── error.rs         # Error types (infrastructure — imported by all phases)
└── cli.rs           # Thin wiring: parse args, call phases in order
```

**Rules for this layout:**

1. **`resolve/`** contains `SolvedSet`, `DependencyGraph`, constraint types, and the solver call. It does not import from `staging/` or `install/`.
2. **`fetch/`** contains the HTTP client, retry logic, and download types. It consumes `resolve/`'s output types.
3. **`cache/`** contains the content-addressed storage types and logic. `staging/` imports from `cache/`, not the reverse.
4. **`index/`** supports `fetch/` and `resolve/` — it is a dependency of those phases, not a peer layer.
5. **`types.rs`** stays small: only `ModuleId`, `PackageSpec`, `VersionReq` if they genuinely appear in 3+ phases. If `PackageSpec` is only used in `resolve/` and `fetch/`, move it to `resolve/`.
6. **`config.rs`** and **`error.rs`** are infrastructure — imported by phases, never importing from them.

---

## Dependency Direction Check

After restructuring, trace every `use crate::` import and verify no arrows point "upward" (toward later phases or the CLI):

```
OK:   resolve/ imports types, error, manifest
OK:   fetch/   imports resolve, index, cache, error
OK:   staging/ imports cache, lockfile, error
OK:   install/ imports staging, error
OK:   cli.rs   imports all phases (orchestrator — this is its job)

VIOLATION if: cache/ imports staging/  (lower phase importing from higher phase)
VIOLATION if: index/ imports fetch/    (dependency importing from its consumer)
VIOLATION if: types.rs imports resolve/ (shared types importing from a phase)
```

---

## On `types.rs` Specifically

The existing `types.rs` with `ModuleId`, `PackageSpec`, `VersionReq` is a legitimate shared module **if and only if** those types are used across 3+ phases. The skill permits this:

> For types genuinely used by 3+ components, a focused `types.rs` is acceptable — but resist making it a dumping ground.

Keep it. But keep it focused. If it grows past ~100 lines, audit which types are actually cross-cutting and move the rest to their owning phase.

---

## When Would the Workspace Pattern Make Sense?

If `resolve/` (the PubGrub solver) becomes independently useful — e.g., you want to publish it as `pubgrub-registry` for others to use — extract it to a workspace crate then. Not before. The warning signs from the skill apply:

- Every crate would depend on a `*-types` crate (you have `ModuleId`, `PackageSpec` everywhere)
- Only one binary consumes all the crates
- Cross-crate refactoring is painful during active development

Stay single-crate until there is genuine external reuse.

---

## Summary

| Proposal | Verdict | Reason |
|---|---|---|
| `model/` (types, manifest, lockfile, config) | Reject | Model bag — mixes unrelated domains, creates false coupling |
| `service/` (resolve, fetch, staging, install) | Reject | Scatters each phase across three directories |
| `io/` (cache, index, registry HTTP) | Reject | `cache/` and `index/` belong to different pipeline positions, not the same "layer" |
| Phase-pipeline (`resolve/`, `fetch/`, `cache/`, `staging/`, `install/`) | Accept | Matches the natural data flow; each module answers "what phase is this?" |
| Small `types.rs` for `ModuleId`, `PackageSpec`, `VersionReq` | Accept | Genuinely cross-cutting; keep it focused and small |
