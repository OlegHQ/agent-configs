# Image Pipeline Restructuring

## Problem

The original code uses string-based filter dispatch (`match *filter { "resize" => ... }`), which causes:

- **Runtime panics** from typos in filter names (e.g., `"blurr"` hits the `_ => panic!` arm).
- **Monolithic function** that every team member must edit when adding a filter.
- **Fragile parameter mapping** where `params[i]` silently grabs the wrong value if indices are misaligned, and filters that need no parameter (like `grayscale`) still occupy an index.

## Solution: Trait Objects + Builder Pipeline

Two patterns work together:

1. **Command pattern via a `Filter` trait.** Each filter is a self-contained struct that owns its own parameters and implements `fn apply(&self, image: &mut Image)`. This replaces the stringly-typed dispatch with compile-time type checking — a typo like `Blurr::new(2.0)` is a compiler error, not a runtime panic.

2. **Builder-style `Pipeline`** that stores `Vec<Box<dyn Filter>>` and chains `.add()` calls. This keeps pipeline construction readable while allowing any combination and ordering of filters.

## Key benefits

| Before | After |
|---|---|
| Filter names are unchecked strings | Filter names are types checked at compile time |
| All parameters share one flat `&[f64]` array | Each filter struct owns its own typed parameters |
| Adding a filter means editing `process_image` | Adding a filter means adding a new struct in a new file |
| One team member's change can break another's filter | Filters are fully independent units |

## How to add a new filter

Create a struct, implement `Filter`, and use it — no existing code changes required:

```rust
struct Sepia { intensity: f64 }

impl Filter for Sepia {
    fn name(&self) -> &str { "sepia" }
    fn apply(&self, image: &mut Image) { /* ... */ }
}

// Usage:
let pipeline = Pipeline::new()
    .add(Resize::new(0.5))
    .add(Sepia { intensity: 0.8 });
```

## Design patterns used

- **Command** — each filter encapsulates an action and its parameters.
- **Builder** — `Pipeline::new().add(...).add(...)` for fluent construction.
- **Open/Closed Principle** — the pipeline is open for extension (new filters) but closed for modification (no edits to `Pipeline` or other filters).
