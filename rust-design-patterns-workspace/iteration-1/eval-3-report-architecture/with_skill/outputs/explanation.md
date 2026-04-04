# Report Architecture -- Design Pattern Analysis

## Problem

A report generation system where multiple report types (PDF, CSV, HTML, JSON, and future types) share identical data-fetching and transformation logic but differ in formatting and output writing.

## Pattern Selection: Template Method

Following the skill's evaluation workflow:

**Signal identified:** "Processing pipeline with varying steps" maps directly to the Template Method pattern in the "When to Apply" table.

**Why Template Method over alternatives:**

- **vs. Strategy:** Strategy would work if *only* a single step varied. Here, two steps (format + write) vary together and are tightly coupled to each other (an HTML formatter must produce HTML; its writer must handle HTML output). Template Method groups these naturally as required trait methods on one type.
- **vs. Chain of Responsibility:** The steps are not independent handlers that might short-circuit. They form a fixed sequence. Template Method is the right fit for a fixed algorithm skeleton.
- **vs. Enum dispatch:** The requirement to add new types *without modifying existing code* rules out enums, which require exhaustive matching and force changes at every match site.

## Architecture Decisions

1. **Trait with default methods** for the template method (`generate`) and shared steps (`fetch_data`, `transform_data`). Required methods (`format`, `write_output`) are the varying "hooks" that each report type must implement.

2. **Dynamic dispatch** (`Box<dyn ReportGenerator>`) for the `ReportEngine` registry. This enables runtime selection of report type and heterogeneous storage. Since report generation is I/O-bound, the vtable cost is negligible.

3. **Registry pattern** (`ReportEngine` with a `HashMap`) for open/closed extensibility. New report types are registered without touching any existing code.

4. **Context passed down, not stored.** Following the skill's core principle, `ReportRequest` is passed as a parameter -- no struct stores mutable references to shared state.

5. **`Result<T, ReportError>`** for all fallible operations, with a domain-specific error enum. No panics, no sentinel values.

## How to Add a New Report Type

Implement `ReportGenerator` on a new struct (e.g., `ExcelReportGenerator`) with `format` and `write_output`. Register it:

```rust
engine.register("excel", Box::new(ExcelReportGenerator));
```

Zero changes to existing types or the engine.

## File Structure (Production)

```
src/
  report/
    mod.rs          -- ReportGenerator trait, domain types, ReportEngine
    csv.rs          -- CsvReportGenerator
    html.rs         -- HtmlReportGenerator
    pdf.rs          -- PdfReportGenerator
    json.rs         -- JsonReportGenerator
  main.rs           -- wires up the engine
```
