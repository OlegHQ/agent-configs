// =============================================================================
// Report Generation Architecture
//
// Pattern: Template Method (via trait default methods) + Strategy (for format/write)
//
// The algorithm skeleton (fetch -> transform -> format -> write) lives in a
// trait's default method. Shared steps (fetch, transform) have default
// implementations. Varying steps (format, write) are required methods that
// each report type must implement.
//
// Adding a new report type = implement one struct + the ReportFormatter trait.
// No existing code changes.
// =============================================================================

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Raw rows coming from the database.
#[derive(Debug, Clone)]
pub struct RawRecord {
    pub fields: HashMap<String, String>,
}

/// Cleaned, transformed data ready for formatting.
#[derive(Debug, Clone)]
pub struct ReportData {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// The final rendered output, ready to be written.
#[derive(Debug, Clone)]
pub struct RenderedReport {
    pub content: Vec<u8>,
    pub file_extension: String,
}

/// Configuration for a report run.
#[derive(Debug, Clone)]
pub struct ReportRequest {
    pub query: String,
    pub title: String,
    pub output_path: String,
}

/// Errors that can occur during report generation.
#[derive(Debug)]
pub enum ReportError {
    FetchError(String),
    TransformError(String),
    FormatError(String),
    WriteError(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportError::FetchError(msg) => write!(f, "Fetch error: {msg}"),
            ReportError::TransformError(msg) => write!(f, "Transform error: {msg}"),
            ReportError::FormatError(msg) => write!(f, "Format error: {msg}"),
            ReportError::WriteError(msg) => write!(f, "Write error: {msg}"),
        }
    }
}

impl std::error::Error for ReportError {}

// ---------------------------------------------------------------------------
// Template Method trait
// ---------------------------------------------------------------------------

/// The core abstraction. The `generate` method is the **template method** --
/// it defines the fixed algorithm skeleton. Shared steps (`fetch_data`,
/// `transform_data`) have default implementations. Varying steps (`format`,
/// `write_output`) must be provided by each report type.
///
/// This follows the skill's guidance: "Rust's default trait methods make
/// [Template Method] natural. Required methods (no body) are the 'abstract'
/// steps; methods with bodies are the optional hooks."
pub trait ReportGenerator {
    /// Human-readable name for this report type (e.g., "PDF", "CSV").
    fn name(&self) -> &str;

    // -- Template method (algorithm skeleton) --------------------------------

    /// Generate a complete report. This is the template method.
    /// Override only if you need a fundamentally different pipeline.
    fn generate(&self, request: &ReportRequest) -> Result<(), ReportError> {
        let raw = self.fetch_data(&request.query)?;
        let data = self.transform_data(raw, &request.title)?;
        let rendered = self.format(&data)?;
        self.write_output(&rendered, &request.output_path)?;
        Ok(())
    }

    // -- Shared steps (default implementations) ------------------------------

    /// Fetch raw records from the database. Same logic for all report types.
    fn fetch_data(&self, query: &str) -> Result<Vec<RawRecord>, ReportError> {
        // In production, this would execute `query` against a real database.
        // Placeholder: simulate fetching rows.
        println!("[{}] Fetching data with query: {}", self.name(), query);

        let mut row1 = HashMap::new();
        row1.insert("id".into(), "1".into());
        row1.insert("name".into(), "Alice".into());
        row1.insert("amount".into(), "1500.00".into());

        let mut row2 = HashMap::new();
        row2.insert("id".into(), "2".into());
        row2.insert("name".into(), "Bob".into());
        row2.insert("amount".into(), "2300.50".into());

        Ok(vec![
            RawRecord { fields: row1 },
            RawRecord { fields: row2 },
        ])
    }

    /// Transform raw records into structured report data. Same for all types.
    fn transform_data(
        &self,
        raw: Vec<RawRecord>,
        title: &str,
    ) -> Result<ReportData, ReportError> {
        println!("[{}] Transforming {} records", self.name(), raw.len());

        let headers = vec!["ID".into(), "Name".into(), "Amount".into()];
        let rows: Vec<Vec<String>> = raw
            .iter()
            .map(|r| {
                vec![
                    r.fields.get("id").cloned().unwrap_or_default(),
                    r.fields.get("name").cloned().unwrap_or_default(),
                    r.fields.get("amount").cloned().unwrap_or_default(),
                ]
            })
            .collect();

        Ok(ReportData {
            title: title.to_string(),
            headers,
            rows,
        })
    }

    // -- Varying steps (must be implemented per report type) -----------------

    /// Format the structured data into a rendered output.
    fn format(&self, data: &ReportData) -> Result<RenderedReport, ReportError>;

    /// Write the rendered output to its destination.
    fn write_output(&self, report: &RenderedReport, path: &str) -> Result<(), ReportError>;
}

// ---------------------------------------------------------------------------
// Concrete report types
// ---------------------------------------------------------------------------

/// CSV report generator.
pub struct CsvReportGenerator;

impl ReportGenerator for CsvReportGenerator {
    fn name(&self) -> &str {
        "CSV"
    }

    fn format(&self, data: &ReportData) -> Result<RenderedReport, ReportError> {
        let mut output = String::new();

        // Header row
        output.push_str(&data.headers.join(","));
        output.push('\n');

        // Data rows
        for row in &data.rows {
            // Escape fields that contain commas
            let escaped: Vec<String> = row
                .iter()
                .map(|field| {
                    if field.contains(',') || field.contains('"') {
                        format!("\"{}\"", field.replace('"', "\"\""))
                    } else {
                        field.clone()
                    }
                })
                .collect();
            output.push_str(&escaped.join(","));
            output.push('\n');
        }

        Ok(RenderedReport {
            content: output.into_bytes(),
            file_extension: "csv".into(),
        })
    }

    fn write_output(&self, report: &RenderedReport, path: &str) -> Result<(), ReportError> {
        let full_path = format!("{}.{}", path, report.file_extension);
        std::fs::write(&full_path, &report.content)
            .map_err(|e| ReportError::WriteError(format!("Failed to write {full_path}: {e}")))?;
        println!("[CSV] Wrote report to {full_path}");
        Ok(())
    }
}

/// HTML report generator.
pub struct HtmlReportGenerator;

impl ReportGenerator for HtmlReportGenerator {
    fn name(&self) -> &str {
        "HTML"
    }

    fn format(&self, data: &ReportData) -> Result<RenderedReport, ReportError> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\">\n");
        html.push_str(&format!("<title>{}</title>\n", data.title));
        html.push_str("<style>\n");
        html.push_str("  table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("  th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("  th { background-color: #4CAF50; color: white; }\n");
        html.push_str("</style>\n</head><body>\n");
        html.push_str(&format!("<h1>{}</h1>\n<table>\n<tr>", data.title));

        for header in &data.headers {
            html.push_str(&format!("<th>{header}</th>"));
        }
        html.push_str("</tr>\n");

        for row in &data.rows {
            html.push_str("<tr>");
            for cell in row {
                html.push_str(&format!("<td>{cell}</td>"));
            }
            html.push_str("</tr>\n");
        }

        html.push_str("</table>\n</body></html>");

        Ok(RenderedReport {
            content: html.into_bytes(),
            file_extension: "html".into(),
        })
    }

    fn write_output(&self, report: &RenderedReport, path: &str) -> Result<(), ReportError> {
        let full_path = format!("{}.{}", path, report.file_extension);
        std::fs::write(&full_path, &report.content)
            .map_err(|e| ReportError::WriteError(format!("Failed to write {full_path}: {e}")))?;
        println!("[HTML] Wrote report to {full_path}");
        Ok(())
    }
}

/// PDF report generator.
/// In production, this would use a crate like `printpdf` or `genpdf`.
pub struct PdfReportGenerator;

impl ReportGenerator for PdfReportGenerator {
    fn name(&self) -> &str {
        "PDF"
    }

    fn format(&self, data: &ReportData) -> Result<RenderedReport, ReportError> {
        // Placeholder: real implementation would use a PDF library.
        // This demonstrates the structure -- the actual PDF bytes would be
        // generated here.
        let mut content = format!("%%PDF-PLACEHOLDER\nTitle: {}\n", data.title);
        content.push_str(&format!("Columns: {}\n", data.headers.join(" | ")));
        for row in &data.rows {
            content.push_str(&format!("Row: {}\n", row.join(" | ")));
        }

        Ok(RenderedReport {
            content: content.into_bytes(),
            file_extension: "pdf".into(),
        })
    }

    fn write_output(&self, report: &RenderedReport, path: &str) -> Result<(), ReportError> {
        let full_path = format!("{}.{}", path, report.file_extension);
        std::fs::write(&full_path, &report.content)
            .map_err(|e| ReportError::WriteError(format!("Failed to write {full_path}: {e}")))?;
        println!("[PDF] Wrote report to {full_path}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Adding a new report type is this simple -- no existing code changes needed.
// ---------------------------------------------------------------------------

/// JSON report generator -- demonstrates how easy it is to extend.
pub struct JsonReportGenerator;

impl ReportGenerator for JsonReportGenerator {
    fn name(&self) -> &str {
        "JSON"
    }

    fn format(&self, data: &ReportData) -> Result<RenderedReport, ReportError> {
        // Manual JSON construction to avoid serde dependency in this example.
        // In production, use serde_json::to_string_pretty.
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"title\": \"{}\",\n", data.title));
        json.push_str("  \"records\": [\n");

        for (i, row) in data.rows.iter().enumerate() {
            json.push_str("    {");
            let fields: Vec<String> = data
                .headers
                .iter()
                .zip(row.iter())
                .map(|(h, v)| format!("\"{}\": \"{}\"", h, v))
                .collect();
            json.push_str(&fields.join(", "));
            json.push('}');
            if i < data.rows.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }

        json.push_str("  ]\n}");

        Ok(RenderedReport {
            content: json.into_bytes(),
            file_extension: "json".into(),
        })
    }

    fn write_output(&self, report: &RenderedReport, path: &str) -> Result<(), ReportError> {
        let full_path = format!("{}.{}", path, report.file_extension);
        std::fs::write(&full_path, &report.content)
            .map_err(|e| ReportError::WriteError(format!("Failed to write {full_path}: {e}")))?;
        println!("[JSON] Wrote report to {full_path}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Report engine -- orchestrates report generation
// ---------------------------------------------------------------------------

/// Registry of available report generators. Uses dynamic dispatch to hold
/// heterogeneous generator types in a single collection.
pub struct ReportEngine {
    generators: HashMap<String, Box<dyn ReportGenerator>>,
}

impl ReportEngine {
    pub fn new() -> Self {
        Self {
            generators: HashMap::new(),
        }
    }

    /// Register a new report type. This is how you extend the system at
    /// runtime without modifying any existing code (Open/Closed Principle).
    pub fn register(&mut self, key: &str, generator: Box<dyn ReportGenerator>) {
        self.generators.insert(key.to_lowercase(), generator);
    }

    /// Generate a report of the given type.
    pub fn generate(&self, report_type: &str, request: &ReportRequest) -> Result<(), ReportError> {
        let generator = self
            .generators
            .get(&report_type.to_lowercase())
            .ok_or_else(|| {
                ReportError::FormatError(format!("Unknown report type: {report_type}"))
            })?;

        generator.generate(request)
    }

    /// List all registered report types.
    pub fn available_types(&self) -> Vec<&str> {
        self.generators.values().map(|g| g.name()).collect()
    }
}

// ---------------------------------------------------------------------------
// Convenience constructor -- pre-registers built-in types
// ---------------------------------------------------------------------------

impl Default for ReportEngine {
    fn default() -> Self {
        let mut engine = Self::new();
        engine.register("csv", Box::new(CsvReportGenerator));
        engine.register("html", Box::new(HtmlReportGenerator));
        engine.register("pdf", Box::new(PdfReportGenerator));
        engine.register("json", Box::new(JsonReportGenerator));
        engine
    }
}

// ---------------------------------------------------------------------------
// Usage example
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = ReportEngine::default();

    println!("Available report types: {:?}", engine.available_types());

    let request = ReportRequest {
        query: "SELECT * FROM sales WHERE quarter = 'Q1'".into(),
        title: "Q1 Sales Report".into(),
        output_path: "/tmp/q1_sales".into(),
    };

    // Generate all report formats from the same request
    for report_type in &["csv", "html", "pdf", "json"] {
        engine.generate(report_type, &request)?;
        println!();
    }

    // Demonstrate runtime extensibility: add a new type without touching
    // any existing code.
    //
    // let mut engine = ReportEngine::default();
    // engine.register("excel", Box::new(ExcelReportGenerator));
    // engine.generate("excel", &request)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_format_produces_valid_output() {
        let gen = CsvReportGenerator;
        let data = ReportData {
            title: "Test".into(),
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "2".into()]],
        };
        let rendered = gen.format(&data).unwrap();
        let output = String::from_utf8(rendered.content).unwrap();
        assert!(output.contains("A,B"));
        assert!(output.contains("1,2"));
        assert_eq!(rendered.file_extension, "csv");
    }

    #[test]
    fn html_format_produces_table() {
        let gen = HtmlReportGenerator;
        let data = ReportData {
            title: "Test".into(),
            headers: vec!["Name".into()],
            rows: vec![vec!["Alice".into()]],
        };
        let rendered = gen.format(&data).unwrap();
        let output = String::from_utf8(rendered.content).unwrap();
        assert!(output.contains("<table>"));
        assert!(output.contains("<th>Name</th>"));
        assert!(output.contains("<td>Alice</td>"));
    }

    #[test]
    fn json_format_produces_valid_structure() {
        let gen = JsonReportGenerator;
        let data = ReportData {
            title: "Test".into(),
            headers: vec!["ID".into(), "Val".into()],
            rows: vec![vec!["1".into(), "X".into()]],
        };
        let rendered = gen.format(&data).unwrap();
        let output = String::from_utf8(rendered.content).unwrap();
        assert!(output.contains("\"title\": \"Test\""));
        assert!(output.contains("\"ID\": \"1\""));
    }

    #[test]
    fn engine_rejects_unknown_type() {
        let engine = ReportEngine::default();
        let request = ReportRequest {
            query: "SELECT 1".into(),
            title: "Test".into(),
            output_path: "/tmp/test".into(),
        };
        let result = engine.generate("xlsx", &request);
        assert!(result.is_err());
    }

    #[test]
    fn engine_lists_registered_types() {
        let engine = ReportEngine::default();
        let types = engine.available_types();
        assert_eq!(types.len(), 4);
    }
}
