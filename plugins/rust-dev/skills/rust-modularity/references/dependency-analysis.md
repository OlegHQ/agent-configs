# Dependency Analysis

A step-by-step process for auditing dependencies and identifying the right module
structure for a Rust crate. Run this BEFORE restructuring.

## 1. Map the Dependency Graph

For every `.rs` file, list its `use crate::` imports. Build an adjacency list:

```
cli.rs         → lockfile, manifest, paths, sync, launcher, ui
lockfile.rs    → error, paths
manifest.rs    → error, paths
module_id.rs   → error, github (GitHubSource only)
github/        → error, paths, cache (repo_dir_is_package_root), ui
cache/         → error, github, lockfile, manifest, paths, ui
staging/       → artifacts, cache, error, fs_util, lockfile, manifest, paths
sync/          → cache, error, github, index, lockfile, manifest, module_id,
                 paths, resolve, staging, ui
resolve.rs     → cache, error, github, lockfile, manifest, module_id, ui
launcher/      → paths, staging, sync, ui
```

## 2. Identify Natural Clusters

Look for groups of modules that import each other heavily vs groups that are
loosely connected:

**Cluster detection heuristics:**
- Modules that import from each other form a cluster
- Modules that share the same set of dependencies are peers
- A module imported by everything is infrastructure (error, paths, ui)

**Infrastructure modules** (imported by 5+ others): These are legitimate shared
utilities. Keep them at the root level or in a small shared module.

**Domain clusters** (import each other, share types):
- `github/` + its types → GitHub domain
- `cache/` + its submodules → caching domain
- `lockfile` + `manifest` → configuration domain (or keep separate if independent)
- `staging/` + its submodules → staging domain
- `sync/` → orchestrator (imports many domains)

## 3. Detect the Architectural Pattern

From the dependency graph, determine which pattern emerges:

### Test for Phase-Pipeline (Pattern B)
Draw the data flow: does data move sequentially through phases?
```
manifest → resolve → fetch/cache → stage → launch
```
If yes, the phases are your modules. Dependencies should only point forward
(or to shared infrastructure).

### Test for Domain Components (Pattern A)
Can you draw circles around clusters that are mostly self-contained?
```
[github/]  [cache/]  [staging/]  [launcher/]
    ↓          ↓          ↓           ↓
         [shared: error, paths, ui]
```
If yes, each circle is a domain component.

### Test for Layers (Pattern D)
Is there a clear boundary between "interface" and "business logic" and "storage"?
If the same module does API calls AND business logic AND storage, layers don't
fit naturally.

### Mixed Pattern
Most real codebases show a mix. That's fine — label each module with its role
and ensure dependencies are consistent.

## 4. Find Violations

A violation is any import where:
- A lower-level module imports from a higher-level module
- An infrastructure module imports from a domain module
- A pipeline phase imports from a later phase

```
VIOLATION: github/download.rs imports cache::repo_dir_is_package_root
           (cross-domain dependency — extract the shared function)

VIOLATION: module_id.rs imports github::parse_github_url
           (a type module importing parsing logic from a domain)

OK:        staging/ imports cache::cache_entry_dir
           (staging consumes cached data — correct direction)
```

## 5. Fix Violations

### Downward Extraction
Move shared functions/types to the module that should own them:

```rust
// Before: module_id.rs imports github::parse_github_url (violation)
// After: remove the dependency, or move the function to module_id's level
```

### Trait Inversion
Lower module defines a trait, upper module implements it:

```rust
// cache/ needs to check if a directory is a "package root"
// Instead of importing from github/, define the check in cache/
pub fn cache_dir_is_package_root(path: &Path) -> bool { ... }
```

### Small Shared Module
For types genuinely used across 3+ domains, create a focused module:

```rust
// types.rs — only cross-cutting types, NOT a dump
pub struct ModuleId(pub String);
pub struct CacheKey(pub String);
```

Keep this small. If it grows past 100 lines, types are probably domain-specific
and should move to their domains.

## 6. Validate the Result

After restructuring:

```
cargo check             # Must compile
cargo test              # All tests pass
```

Then trace imports again. The graph should be cleaner:
- No upward arrows
- Clusters are visible
- Infrastructure is clearly separated
- Each module's responsibility is obvious from its name
