---
name: rust-design-patterns
description: >
  Proactively applies Rust design patterns and idiomatic abstractions when writing new features,
  implementing systems, or extending existing code. Use this skill whenever writing Rust code that
  involves more than trivial logic — especially when building new features, adding to existing
  systems, implementing multi-component architectures, or any task where the user says "build me X",
  "add Y feature", "implement Z", "create a system for W". The skill prevents the common failure
  mode of writing monolithic, enum-match-heavy, or free-function-scattered code that works initially
  but becomes unmaintainable as requirements grow. Triggers on ANY Rust implementation task, not
  just explicit "design pattern" or "refactor" requests. If you're about to write Rust code, this
  skill applies.
---

# Rust Design Patterns

Before jumping to implementation, pause and think about what will change later. Structure
code so those changes are additions, not surgeries. This isn't about elegance — it's about
preventing the tech debt that compounds when someone says "now add Y."

## Before Writing Code: Identify Change Vectors

A "change vector" is a dimension along which code will grow. Spot them BEFORE writing:

- **New variants of a thing** — "We support CSV, but JSON and YAML coming later"
- **New steps in a process** — "We validate then save, but caching and logging later"
- **New behaviors for different configs** — "Email now, Slack and SMS soon"
- **Multiple backends** — "Postgres now, but might need SQLite or DynamoDB"

For each change vector, ask: **"When someone adds a new variant, do they modify existing
code or just add a new file?"** If the answer is "modify," restructure now.

## The Decision That Matters: Enum+Match vs Trait+Impl

This is the single most impactful decision in Rust architecture, and the one most often
gotten wrong. Here's when each is correct:

### Use an Enum When:
- The set of variants is **closed** — you control them all, and new ones are rare
- Behavior differences between variants are **small** (a few lines per arm)
- You want the compiler to enforce **exhaustiveness** on every match
- Examples: `Format { Csv, Json, Yaml }`, `Direction { North, South, East, West }`,
  `LogLevel { Debug, Info, Warn, Error }`

### Use a Trait When:
- The set of variants is **open** — new ones will be added by different people/teams/plugins
- Each variant has **its own data** (parameters, config, internal state)
- Each variant's behavior is **substantial** (10+ lines, not a one-liner)
- You want to add a new variant **without touching existing code at all**
- Examples: notification channels, file format readers/writers, transform pipeline steps,
  payment processors, storage backends

### The Failure Mode This Skill Prevents

The natural instinct is to reach for enums + free functions:

```rust
// BAD: Every new format = new enum variant + new match arm in BOTH functions
enum Format { Csv, Json, Yaml }

fn read_data(path: &str, fmt: Format) -> Result<Data> {
    match fmt {
        Format::Csv => { /* 30 lines of CSV parsing */ },
        Format::Json => { /* 25 lines of JSON parsing */ },
        Format::Yaml => { /* 20 lines of YAML parsing */ },
    }
}

fn write_data(data: &Data, path: &str, fmt: Format) -> Result<()> {
    match fmt {  // SECOND match on the same enum — they must stay in sync
        Format::Csv => { /* ... */ },
        Format::Json => { /* ... */ },
        Format::Yaml => { /* ... */ },
    }
}
```

```rust
// GOOD: Every new format = one new struct with impl. No existing code touched.
trait DataReader {
    fn read(&self, path: &Path) -> Result<DataFrame>;
}
trait DataWriter {
    fn write(&self, data: &DataFrame, path: &Path) -> Result<()>;
}

struct CsvFormat;
impl DataReader for CsvFormat { fn read(&self, path: &Path) -> Result<DataFrame> { /* ... */ } }
impl DataWriter for CsvFormat { fn write(&self, data: &DataFrame, path: &Path) -> Result<()> { /* ... */ } }
// Adding Parquet = new struct + impls. Zero changes to existing formats.
```

The same applies to **processing pipelines**:

```rust
// BAD: enum + single match function — every new transform edits apply_transform()
enum Transform { Filter(String), Sort(String), Rename(String, String) }
fn apply_transform(data: &mut Data, t: &Transform) { match t { ... } }

// GOOD: trait + separate types — each transform is independent
trait Transform {
    fn apply(&self, data: DataFrame) -> Result<DataFrame>;
}
struct FilterTransform { column: String, op: FilterOp, value: String }
struct SortTransform { column: String, ascending: bool }
impl Transform for FilterTransform { /* ... */ }
impl Transform for SortTransform { /* ... */ }
// Pipeline: Vec<Box<dyn Transform>> — composable, extensible, each step testable alone
```

## Calibrating: Don't Over-Engineer

**Rule of Three:**
- 1 variant → Write it directly. No trait.
- 2 variants → Enum is probably fine unless they're truly independent concerns.
- 3+ variants, or user says "more coming" → Trait-based abstraction.

**Over-engineering smells:**
- Trait with one implementor and no plans for a second
- `Box<dyn Trait>` when there are only 2 variants (enum is simpler)
- Builder for a struct with 3 fields
- Strategy trait for what should be a closure

**Context matters:** A prototype → keep it simple. A library → get abstractions right.
A feature where user said "we'll add more" → abstract now, debt compounds fast.

## Red Flags While Writing

If you catch yourself writing any of these, stop and restructure:

**The growing match.** `match kind { ... }` with 3+ arms and more coming. Every copy of
this match in the codebase must stay in sync when a variant is added.
→ Extract a trait. Each variant becomes a separate `impl`.

**Scattered free functions for the same family.** `read_csv()`, `read_json()`,
`read_yaml()` as loose functions dispatched by a match. They're a family with identical
signatures — that's what traits express.
→ `trait Reader { fn read(&self, path: &Path) -> Result<Data>; }` with impls.

**Parallel arrays.** `names[i]` corresponds to `handlers[i]`. One update that forgets
both arrays silently breaks.
→ Single struct or trait object owning both pieces.

**The god function.** 50+ lines doing validate-transform-save-notify. Each concern
will evolve independently but they're tangled.
→ Split into steps. Each step is a method or a trait impl in a pipeline.

## Dispatch Decision Guide

```
Will the set of variants grow with new types added by others?
├── Yes → Trait + impl per variant (open for extension)
│         Each variant: own struct, own data, own impl
│         Collection: Vec<Box<dyn Trait>>
│
└── No → Is the set small and fixed?
    ├── Yes → Enum + match (compiler-enforced exhaustiveness)
    │
    └── Not sure → Start with enum. Refactor to trait when the third variant lands.
```

## Naming Patterns Correctly

When you apply a pattern, name it in comments so future readers understand the architecture.
Use the standard GoF names — they're a shared vocabulary:

- Processing pipeline with varying steps → **Strategy** (trait per step) or **Chain of Responsibility** (handler pipeline)
- Algorithm with shared skeleton + varying parts → **Template Method** (trait with default methods)
- Object constructed in stages → **Builder** (consuming `build(self)`)
- Wrapping to add behavior → **Decorator** (same trait, delegates + enhances)
- Simplified interface to complex subsystem → **Facade**
- Families of related objects → **Abstract Factory** (trait with associated types)

See `references/behavioral-patterns.md`, `references/structural-patterns.md`,
`references/creational-patterns.md` for complete Rust implementations of each.
See `references/rust-idioms.md` for Rust-specific patterns: Newtype, Typestate, RAII, Extension traits.

## Implementation Checklist

Before finalizing non-trivial Rust code:

- [ ] Each "kind of thing" is its own type — not a branch in a growing match
- [ ] New variants don't require modifying existing code — just adding new impls
- [ ] Families of functions with the same signature are behind a trait, not loose free functions
- [ ] Pipeline steps are separate types composed in a `Vec<Box<dyn Step>>`, not one function
- [ ] The pattern is named in a comment so readers know the architecture
