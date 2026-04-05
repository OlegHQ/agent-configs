# Architecture Examples

Real-world Rust projects and the architectural patterns they chose.

## Pattern A: Domain Components

### mise (dev tool manager)

```
src/
├── cli/         # CLI subcommands (one file per command)
├── backend/     # Tool backends: node, python, ruby, go, etc.
├── config/      # Configuration parsing and resolution
├── task/        # Task runner (scripts, dependencies)
├── toolset/     # Installed tool management
├── shell/       # Shell integration (bash, zsh, fish completions)
├── ui/          # Terminal output, colors, progress
└── plugins/     # Plugin system
```

**Why it works:** Each directory is a self-contained domain. `backend/` owns
everything about tool backends. `task/` owns everything about task running.
Adding a new backend means adding a file in `backend/`, not touching `cli/`
or `config/`.

### bat (cat clone)

```
src/
├── assets/          # Syntax definitions, themes
├── controller.rs    # Main orchestration
├── printer.rs       # Output formatting
├── input.rs         # File/stdin reading
├── output.rs        # Pager integration
├── config.rs        # CLI config
├── diff.rs          # Git diff integration
├── error.rs         # Error types
└── pager.rs         # Pager subprocess
```

**Why it works:** Small enough for mostly-flat files. Each file is one
responsibility. `assets/` is the only directory because it has substructure.

### starship (prompt)

```
src/
├── modules/     # One file per prompt module (git, node, rust, python, ...)
├── configs/     # Configuration structs for each module
├── formatter/   # Prompt string formatting
├── init/        # Shell initialization scripts
└── utils/       # Shared utilities
```

**Why it works:** The `modules/` directory is the extension point. Adding a
new prompt segment = one new file in `modules/` + one in `configs/`. This
is the Strategy pattern at the file level.

## Pattern B: Phase-Pipeline

### Typical package manager flow

```
src/
├── manifest.rs      # Phase 0: Parse project manifest
├── resolve/         # Phase 1: Dependency resolution
├── fetch/           # Phase 2: Download packages
├── cache/           # Phase 2.5: Content-addressed storage
├── build/           # Phase 3: Compile/transform
├── install/         # Phase 4: Place in target
├── lockfile.rs      # Shared: Lock file format
└── cli.rs           # Entry: Wire phases together
```

**Why it works:** The pipeline is the architecture. Each phase has clear
inputs and outputs. Dependencies flow forward through the pipeline.

## Pattern C: Workspace Crates

### ripgrep

```
crates/
├── core/        # Thin CLI: arg parsing, orchestration (< 10 files)
├── grep/        # Grep library: Matcher trait + implementations
├── globset/     # Glob pattern matching (independently published)
├── ignore/      # .gitignore-style filtering (independently published)
├── searcher/    # File searching with memory mapping
├── printer/     # Result formatting
└── regex/       # Regex engine wrapper
```

**Why it works:** `globset` and `ignore` are used by other projects — genuine
reuse justifies crate boundaries. Each crate has a focused public API.

### uv (Python package manager)

```
crates/
├── uv/              # CLI entry point
├── uv-resolver/     # PubGrub-based dependency resolution
├── uv-git/          # Git source handling
├── uv-cache/        # Content-addressed cache
├── uv-types/        # Shared types across crates
├── uv-distribution-types/  # Package distribution types
├── uv-build-frontend/      # Build system frontend
└── ... (67+ crates total)
```

**Why it works (at this scale):** Large team, independent testing per crate,
significant compile-time savings. The `*-types` crates are a cost they accept
for the compile-time benefits.

**Why it wouldn't work for smaller projects:** The `*-types` crate explosion
creates coupling through the back door. With a small team, the coordination
cost exceeds the benefit.

## Pattern D: Layered

### Typical web service

```
src/
├── handlers/    # HTTP handlers (deserialize, call service, serialize)
├── service/     # Business logic (no HTTP knowledge)
├── storage/     # Database queries (no business logic)
├── types/       # Shared request/response/domain types
└── config.rs    # Application configuration
```

**Why it works:** Clear request/response boundary. Each layer has one job.
Testing the service layer doesn't need HTTP. Testing storage doesn't need
business logic.

**Where it breaks down:** When "business logic" is really just pass-through
to storage, the service layer becomes ceremony. When the domain is complex
(e.g., a compiler), the three layers don't capture the real architecture.

## Choosing: Summary Table

| Codebase shape | Pattern | Module naming |
|---|---|---|
| Distinct features/plugins/backends | A: Domain components | Named after the domain |
| Sequential processing phases | B: Phase-pipeline | Named after the phase |
| Independently reusable libraries | C: Workspace crates | Named after the library |
| Request → process → store | D: Layered | Named after the layer |
| Mixed / unclear | Hybrid | Dominant pattern + exceptions |
