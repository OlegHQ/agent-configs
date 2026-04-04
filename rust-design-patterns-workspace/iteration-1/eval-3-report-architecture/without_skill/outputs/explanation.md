# Report Architecture -- Explanation

## Problem

Multiple report formats (PDF, CSV, HTML, and future ones) share identical data-fetching and transformation logic but differ in how they format and write output. The design must make adding new formats trivial.

## Chosen pattern: Strategy (via trait) + shared pipeline

The architecture splits the four-stage pipeline into two groups:

| Stage | Ownership | Why |
|-------|-----------|-----|
| 1. Fetch data | Free function (`fetch_data`) | Identical for every format |
| 2. Transform data | Free function (`transform_data`) | Identical for every format |
| 3. Format output | `ReportFormatter` trait | Differs per format |
| 4. Write output | `ReportFormatter` trait | Differs per format |

### Key components

- **`ReportFormatter` trait** -- the single extension point. Each output format implements `format()` and `write()`. Adding Excel support means writing one `impl` block; nothing else in the codebase changes.
- **`ReportEngine<F: ReportFormatter>`** -- orchestrates the pipeline with static dispatch (monomorphised, zero-cost). Good when the format is known at compile time.
- **`generate_report_dynamic(&dyn ReportFormatter, ...)`** -- the same pipeline but using trait objects for runtime polymorphism. Good when the format comes from user input or configuration.
- **`formatter_for(kind: &str)`** -- simple factory that maps a string tag to a boxed formatter, centralising the one place that needs updating when a new format is registered.

### Why this works well

1. **Open/Closed Principle** -- new formats are added by implementing the trait; existing code is untouched.
2. **DRY** -- fetch and transform logic exists exactly once.
3. **Testable** -- each formatter can be unit-tested in isolation (pass records in, assert bytes out). The full pipeline can be integration-tested by writing to a temp file.
4. **Flexible dispatch** -- callers choose between generics (zero overhead) and trait objects (runtime flexibility) depending on their needs.

### Adding a new format (checklist)

1. Create a struct (e.g., `ExcelFormatter`).
2. Implement `ReportFormatter` for it.
3. Add a branch in `formatter_for()` if you need dynamic dispatch.

No other files or functions need to change.
