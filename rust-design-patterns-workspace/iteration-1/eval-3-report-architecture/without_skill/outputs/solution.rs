// =============================================================================
// Report Generation Architecture
//
// Pattern: Template Method via shared pipeline + Strategy for format/write steps
//
// The pipeline has four stages:
//   1. Fetch data     (shared)
//   2. Transform data (shared)
//   3. Format output  (varies by report type)
//   4. Write output   (varies by report type)
//
// Stages 1-2 are identical for every report, so they live in free functions.
// Stages 3-4 differ per format, so they sit behind a trait (`ReportFormatter`).
// Adding a new format means implementing one trait -- nothing else changes.
// =============================================================================

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A single row coming out of the database.
#[derive(Debug, Clone)]
pub struct RawRecord {
    pub fields: HashMap<String, String>,
}

/// A row after business-rule transformation.
#[derive(Debug, Clone)]
pub struct TransformedRecord {
    pub id: u64,
    pub label: String,
    pub value: f64,
}

/// Metadata that every report carries.
#[derive(Debug, Clone)]
pub struct ReportMeta {
    pub title: String,
    pub generated_at: String, // ISO-8601 in real code
}

// ---------------------------------------------------------------------------
// Shared pipeline stages (steps 1 & 2)
// ---------------------------------------------------------------------------

/// Stage 1 -- fetch raw data.  In production this would take a `&dyn DbPool`
/// or similar; here we keep it simple.
fn fetch_data(query: &str) -> Result<Vec<RawRecord>, ReportError> {
    // Placeholder: imagine a real DB call here.
    println!("[pipeline] fetching data with query: {query}");
    Ok(vec![
        RawRecord {
            fields: HashMap::from([
                ("id".into(), "1".into()),
                ("label".into(), "Alpha".into()),
                ("value".into(), "42.5".into()),
            ]),
        },
        RawRecord {
            fields: HashMap::from([
                ("id".into(), "2".into()),
                ("label".into(), "Beta".into()),
                ("value".into(), "17.3".into()),
            ]),
        },
    ])
}

/// Stage 2 -- apply shared business-rule transformations.
fn transform_data(raw: Vec<RawRecord>) -> Result<Vec<TransformedRecord>, ReportError> {
    println!("[pipeline] transforming {} raw records", raw.len());
    raw.into_iter()
        .map(|r| {
            let id = r.fields.get("id")
                .ok_or(ReportError::Transform("missing id".into()))?
                .parse::<u64>()
                .map_err(|e| ReportError::Transform(e.to_string()))?;
            let label = r.fields.get("label")
                .ok_or(ReportError::Transform("missing label".into()))?
                .clone();
            let value = r.fields.get("value")
                .ok_or(ReportError::Transform("missing value".into()))?
                .parse::<f64>()
                .map_err(|e| ReportError::Transform(e.to_string()))?;
            Ok(TransformedRecord { id, label, value })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Trait for format-specific behaviour (steps 3 & 4)
// ---------------------------------------------------------------------------

/// Implement this trait for each output format.  Only the parts that actually
/// differ between formats live here.
pub trait ReportFormatter {
    /// Human-readable name of the format (used in logs / error messages).
    fn format_name(&self) -> &str;

    /// Stage 3 -- turn transformed records into the final byte payload.
    fn format(&self, meta: &ReportMeta, records: &[TransformedRecord]) -> Result<Vec<u8>, ReportError>;

    /// Stage 4 -- write the payload to its destination (file, HTTP response, etc.).
    fn write(&self, payload: &[u8], destination: &Path) -> Result<(), ReportError>;
}

// ---------------------------------------------------------------------------
// Concrete formatters
// ---------------------------------------------------------------------------

// -- PDF ---------------------------------------------------------------------

pub struct PdfFormatter;

impl ReportFormatter for PdfFormatter {
    fn format_name(&self) -> &str { "PDF" }

    fn format(&self, meta: &ReportMeta, records: &[TransformedRecord]) -> Result<Vec<u8>, ReportError> {
        // In production you would use a crate like `printpdf` or `genpdf`.
        let mut buf = String::new();
        buf.push_str(&format!("%PDF-STUB\nTitle: {}\nDate: {}\n", meta.title, meta.generated_at));
        for r in records {
            buf.push_str(&format!("  {} | {} | {:.2}\n", r.id, r.label, r.value));
        }
        Ok(buf.into_bytes())
    }

    fn write(&self, payload: &[u8], dest: &Path) -> Result<(), ReportError> {
        std::fs::write(dest, payload)
            .map_err(|e| ReportError::Write(e.to_string()))?;
        println!("[pdf] wrote {} bytes to {}", payload.len(), dest.display());
        Ok(())
    }
}

// -- CSV ---------------------------------------------------------------------

pub struct CsvFormatter {
    pub delimiter: u8,
}

impl Default for CsvFormatter {
    fn default() -> Self { Self { delimiter: b',' } }
}

impl ReportFormatter for CsvFormatter {
    fn format_name(&self) -> &str { "CSV" }

    fn format(&self, _meta: &ReportMeta, records: &[TransformedRecord]) -> Result<Vec<u8>, ReportError> {
        let sep = self.delimiter as char;
        let mut buf = format!("id{sep}label{sep}value\n");
        for r in records {
            buf.push_str(&format!("{}{sep}{}{sep}{:.2}\n", r.id, r.label, r.value));
        }
        Ok(buf.into_bytes())
    }

    fn write(&self, payload: &[u8], dest: &Path) -> Result<(), ReportError> {
        std::fs::write(dest, payload)
            .map_err(|e| ReportError::Write(e.to_string()))?;
        println!("[csv] wrote {} bytes to {}", payload.len(), dest.display());
        Ok(())
    }
}

// -- HTML --------------------------------------------------------------------

pub struct HtmlFormatter;

impl ReportFormatter for HtmlFormatter {
    fn format_name(&self) -> &str { "HTML" }

    fn format(&self, meta: &ReportMeta, records: &[TransformedRecord]) -> Result<Vec<u8>, ReportError> {
        let mut buf = String::new();
        buf.push_str("<!DOCTYPE html><html><head><meta charset=\"utf-8\">");
        buf.push_str(&format!("<title>{}</title></head><body>", meta.title));
        buf.push_str(&format!("<h1>{}</h1>", meta.title));
        buf.push_str(&format!("<p>Generated: {}</p>", meta.generated_at));
        buf.push_str("<table><tr><th>ID</th><th>Label</th><th>Value</th></tr>");
        for r in records {
            buf.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{:.2}</td></tr>",
                r.id, r.label, r.value,
            ));
        }
        buf.push_str("</table></body></html>");
        Ok(buf.into_bytes())
    }

    fn write(&self, payload: &[u8], dest: &Path) -> Result<(), ReportError> {
        std::fs::write(dest, payload)
            .map_err(|e| ReportError::Write(e.to_string()))?;
        println!("[html] wrote {} bytes to {}", payload.len(), dest.display());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// The report engine (orchestrates the full pipeline)
// ---------------------------------------------------------------------------

pub struct ReportEngine<F: ReportFormatter> {
    formatter: F,
    meta: ReportMeta,
}

impl<F: ReportFormatter> ReportEngine<F> {
    pub fn new(formatter: F, meta: ReportMeta) -> Self {
        Self { formatter, meta }
    }

    /// Run the full four-stage pipeline.
    pub fn generate(&self, query: &str, destination: &Path) -> Result<(), ReportError> {
        // Stage 1 -- shared
        let raw = fetch_data(query)?;

        // Stage 2 -- shared
        let records = transform_data(raw)?;

        // Stage 3 -- format-specific
        let payload = self.formatter.format(&self.meta, &records)?;

        // Stage 4 -- format-specific
        self.formatter.write(&payload, destination)?;

        println!(
            "[engine] {} report written to {}",
            self.formatter.format_name(),
            destination.display(),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Using trait objects for runtime polymorphism (optional alternative)
// ---------------------------------------------------------------------------

/// If you need to choose the format at runtime (e.g. from user input or a
/// config file), use a boxed trait object instead of generics.
pub fn generate_report_dynamic(
    formatter: &dyn ReportFormatter,
    meta: &ReportMeta,
    query: &str,
    destination: &Path,
) -> Result<(), ReportError> {
    let raw = fetch_data(query)?;
    let records = transform_data(raw)?;
    let payload = formatter.format(meta, &records)?;
    formatter.write(&payload, destination)?;
    println!(
        "[engine] {} report written to {}",
        formatter.format_name(),
        destination.display(),
    );
    Ok(())
}

/// Convenience: build a formatter from a string tag.
pub fn formatter_for(kind: &str) -> Result<Box<dyn ReportFormatter>, ReportError> {
    match kind {
        "pdf"  => Ok(Box::new(PdfFormatter)),
        "csv"  => Ok(Box::new(CsvFormatter::default())),
        "html" => Ok(Box::new(HtmlFormatter)),
        other  => Err(ReportError::UnknownFormat(other.into())),
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ReportError {
    Fetch(String),
    Transform(String),
    Format(String),
    Write(String),
    UnknownFormat(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fetch(msg)         => write!(f, "fetch error: {msg}"),
            Self::Transform(msg)     => write!(f, "transform error: {msg}"),
            Self::Format(msg)        => write!(f, "format error: {msg}"),
            Self::Write(msg)         => write!(f, "write error: {msg}"),
            Self::UnknownFormat(msg) => write!(f, "unknown format: {msg}"),
        }
    }
}

impl std::error::Error for ReportError {}

// ---------------------------------------------------------------------------
// Example: adding a new format (Excel / JSON) is just one impl block
// ---------------------------------------------------------------------------

pub struct JsonFormatter;

impl ReportFormatter for JsonFormatter {
    fn format_name(&self) -> &str { "JSON" }

    fn format(&self, meta: &ReportMeta, records: &[TransformedRecord]) -> Result<Vec<u8>, ReportError> {
        // Hand-rolled for zero extra deps; use `serde_json` in real code.
        let mut buf = String::from("{\n");
        buf.push_str(&format!("  \"title\": \"{}\",\n", meta.title));
        buf.push_str(&format!("  \"generated_at\": \"{}\",\n", meta.generated_at));
        buf.push_str("  \"records\": [\n");
        for (i, r) in records.iter().enumerate() {
            let comma = if i + 1 < records.len() { "," } else { "" };
            buf.push_str(&format!(
                "    {{\"id\": {}, \"label\": \"{}\", \"value\": {:.2}}}{comma}\n",
                r.id, r.label, r.value,
            ));
        }
        buf.push_str("  ]\n}");
        Ok(buf.into_bytes())
    }

    fn write(&self, payload: &[u8], dest: &Path) -> Result<(), ReportError> {
        std::fs::write(dest, payload)
            .map_err(|e| ReportError::Write(e.to_string()))?;
        println!("[json] wrote {} bytes to {}", payload.len(), dest.display());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// main -- demonstrates both static-dispatch and dynamic-dispatch usage
// ---------------------------------------------------------------------------

fn main() -> Result<(), ReportError> {
    let meta = ReportMeta {
        title: "Q1 Sales Report".into(),
        generated_at: "2026-04-04T12:00:00Z".into(),
    };

    // --- Static dispatch (generic) ------------------------------------------
    println!("=== Static dispatch ===\n");

    let pdf_engine = ReportEngine::new(PdfFormatter, meta.clone());
    pdf_engine.generate("SELECT * FROM sales", Path::new("/tmp/report.pdf"))?;

    let csv_engine = ReportEngine::new(CsvFormatter::default(), meta.clone());
    csv_engine.generate("SELECT * FROM sales", Path::new("/tmp/report.csv"))?;

    let html_engine = ReportEngine::new(HtmlFormatter, meta.clone());
    html_engine.generate("SELECT * FROM sales", Path::new("/tmp/report.html"))?;

    // --- Dynamic dispatch (trait object) ------------------------------------
    println!("\n=== Dynamic dispatch ===\n");

    let formats = ["pdf", "csv", "html", "json"];
    for fmt in formats {
        let formatter = formatter_for(fmt)?;
        let dest = format!("/tmp/report_dyn.{fmt}");
        generate_report_dynamic(formatter.as_ref(), &meta, "SELECT * FROM sales", Path::new(&dest))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_meta() -> ReportMeta {
        ReportMeta {
            title: "Test".into(),
            generated_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn csv_format_produces_valid_output() {
        let formatter = CsvFormatter::default();
        let records = vec![
            TransformedRecord { id: 1, label: "A".into(), value: 10.0 },
        ];
        let payload = formatter.format(&test_meta(), &records).unwrap();
        let text = String::from_utf8(payload).unwrap();
        assert!(text.starts_with("id,label,value\n"));
        assert!(text.contains("1,A,10.00"));
    }

    #[test]
    fn html_format_contains_table() {
        let formatter = HtmlFormatter;
        let records = vec![
            TransformedRecord { id: 1, label: "X".into(), value: 5.0 },
        ];
        let payload = formatter.format(&test_meta(), &records).unwrap();
        let text = String::from_utf8(payload).unwrap();
        assert!(text.contains("<table>"));
        assert!(text.contains("<td>X</td>"));
    }

    #[test]
    fn json_format_is_valid() {
        let formatter = JsonFormatter;
        let records = vec![
            TransformedRecord { id: 1, label: "Z".into(), value: 3.14 },
        ];
        let payload = formatter.format(&test_meta(), &records).unwrap();
        let text = String::from_utf8(payload).unwrap();
        assert!(text.contains("\"title\": \"Test\""));
        assert!(text.contains("\"label\": \"Z\""));
    }

    #[test]
    fn formatter_for_unknown_returns_error() {
        assert!(formatter_for("xlsx").is_err());
    }

    #[test]
    fn full_pipeline_writes_file() {
        let dir = std::env::temp_dir();
        let dest = dir.join("test_report.csv");
        let engine = ReportEngine::new(CsvFormatter::default(), test_meta());
        engine.generate("SELECT 1", &dest).unwrap();
        assert!(dest.exists());
        let content = std::fs::read_to_string(&dest).unwrap();
        assert!(content.contains("id,label,value"));
        std::fs::remove_file(&dest).ok();
    }
}
