# Image Pipeline Restructuring

## Problem

The original code uses string-based dispatch (`match *filter { "resize" => ... }`) with a parallel `params` slice. This has three compounding issues:

1. **Runtime panics from typos** -- misspelling `"sharpen"` as `"sharpn"` compiles fine but panics at runtime.
2. **Growing match arms** -- every new filter requires editing the central `process_image` function, creating merge conflicts when multiple team members add filters simultaneously.
3. **Loose parameter coupling** -- filter names and their parameters travel in separate slices indexed by position. Nothing prevents passing the wrong number of parameters or mismatching a filter with its parameter.

## Solution

Two patterns, both identified from the skill's "When to Apply" table:

- **Strategy** (trait-based variant) -- each filter is a separate struct implementing a shared `ImageFilter` trait. This eliminates the match statement entirely. Filter names become type names checked at compile time.
- **Chain of Responsibility** (Vec pipeline variant) -- filters are collected into a `Vec<Box<dyn ImageFilter>>` and executed in sequence. Dynamic dispatch is the right choice here because the pipeline is heterogeneous (mixes different filter types determined at runtime).

### Key design decisions

| Decision | Rationale |
|---|---|
| `Box<dyn ImageFilter>` (dynamic dispatch) | The pipeline stores different concrete types in one Vec. Static dispatch via generics cannot express this. |
| Parameters live inside each filter struct | `Resize { scale: 0.5 }` binds the parameter to its filter at construction time. No more positional indexing into a parallel array. |
| `Result<(), FilterError>` instead of `panic!` | Unknown filters are now compile errors (no string dispatch), and invalid parameters return structured errors instead of panicking. |
| `Pipeline::add_filter(self, ...) -> Self` | Builder-style chaining for ergonomic pipeline construction. |
| `Debug` trait bound on `ImageFilter` | Enables printing/logging the pipeline contents for debugging. |

## How to add a new filter

1. Define a struct with the filter's parameters.
2. Implement `ImageFilter` for it (two methods: `apply` and `name`).
3. Add it to a pipeline with `add_filter(Box::new(YourFilter { ... }))`.

No existing code is modified. No central match statement to update. No risk of merge conflicts with other team members adding their own filters.
