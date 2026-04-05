// dtool — A CLI data-file processor supporting CSV, JSON, and YAML with a
// composable transformation pipeline.
//
// Usage:
//   dtool input.csv --filter 'age>30' --sort name --rename 'old_name:new_name' \
//                   --add-column 'full:first_name+last_name' --output result.json
//
// Depends on: clap, csv, serde, serde_json, serde_yaml, anyhow

use anyhow::{anyhow, bail, Context, Result};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A single row: column-name -> cell-value (all values stored as strings).
type Row = HashMap<String, String>;

/// The in-memory dataset: ordered column names + rows.
#[derive(Debug, Clone)]
struct DataFrame {
    columns: Vec<String>,
    rows: Vec<Row>,
}

impl DataFrame {
    fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self { columns, rows }
    }
}

// ---------------------------------------------------------------------------
// Format detection — small closed set, so an enum is appropriate.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Csv,
    Json,
    Yaml,
}

impl Format {
    fn from_path(path: &Path) -> Result<Self> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("csv") => Ok(Format::Csv),
            Some("json") => Ok(Format::Json),
            Some("yaml" | "yml") => Ok(Format::Yaml),
            Some(other) => bail!("Unsupported file extension: .{other}"),
            None => bail!("Cannot determine format — file has no extension"),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Csv => write!(f, "CSV"),
            Format::Json => write!(f, "JSON"),
            Format::Yaml => write!(f, "YAML"),
        }
    }
}

// ---------------------------------------------------------------------------
// DataSource trait — reads a file into a DataFrame.
// Three formats today (CSV, JSON, YAML), extensible via new impls.
// ---------------------------------------------------------------------------

trait DataSource {
    fn read(&self, path: &Path) -> Result<DataFrame>;
}

struct CsvSource;
struct JsonSource;
struct YamlSource;

impl DataSource for CsvSource {
    fn read(&self, path: &Path) -> Result<DataFrame> {
        let mut reader = csv::Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file: {}", path.display()))?;

        let columns: Vec<String> = reader
            .headers()
            .context("Failed to read CSV headers")?
            .iter()
            .map(|h| h.to_string())
            .collect();

        let mut rows = Vec::new();
        for (i, record) in reader.records().enumerate() {
            let record = record.with_context(|| format!("Failed to read CSV row {}", i + 1))?;
            let mut row = Row::new();
            for (col, val) in columns.iter().zip(record.iter()) {
                row.insert(col.clone(), val.to_string());
            }
            rows.push(row);
        }

        Ok(DataFrame::new(columns, rows))
    }
}

impl DataSource for JsonSource {
    fn read(&self, path: &Path) -> Result<DataFrame> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("Failed to read JSON file: {}", path.display()))?;

        // Expect an array of objects with string values.
        let records: Vec<HashMap<String, serde_json::Value>> =
            serde_json::from_str(&text).context("Failed to parse JSON as array of objects")?;

        if records.is_empty() {
            return Ok(DataFrame::new(Vec::new(), Vec::new()));
        }

        // Derive column order from the first object's keys.
        let columns: Vec<String> = records[0].keys().cloned().collect();

        let rows: Vec<Row> = records
            .into_iter()
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        let s = match &v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (k, s)
                    })
                    .collect()
            })
            .collect();

        Ok(DataFrame::new(columns, rows))
    }
}

impl DataSource for YamlSource {
    fn read(&self, path: &Path) -> Result<DataFrame> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("Failed to read YAML file: {}", path.display()))?;

        let values: Vec<HashMap<String, serde_yaml::Value>> =
            serde_yaml::from_str(&text).context("Failed to parse YAML as sequence of mappings")?;

        if values.is_empty() {
            return Ok(DataFrame::new(Vec::new(), Vec::new()));
        }

        let columns: Vec<String> = values[0].keys().cloned().collect();

        let rows: Vec<Row> = values
            .into_iter()
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        let s = match &v {
                            serde_yaml::Value::String(s) => s.clone(),
                            serde_yaml::Value::Number(n) => n.to_string(),
                            serde_yaml::Value::Bool(b) => b.to_string(),
                            serde_yaml::Value::Null => String::new(),
                            other => format!("{:?}", other),
                        };
                        (k, s)
                    })
                    .collect()
            })
            .collect();

        Ok(DataFrame::new(columns, rows))
    }
}

fn source_for_format(format: Format) -> Box<dyn DataSource> {
    match format {
        Format::Csv => Box::new(CsvSource),
        Format::Json => Box::new(JsonSource),
        Format::Yaml => Box::new(YamlSource),
    }
}

// ---------------------------------------------------------------------------
// DataSink trait — writes a DataFrame to a file.
// ---------------------------------------------------------------------------

trait DataSink {
    fn write(&self, df: &DataFrame, path: &Path) -> Result<()>;
}

struct CsvSink;
struct JsonSink;
struct YamlSink;

impl DataSink for CsvSink {
    fn write(&self, df: &DataFrame, path: &Path) -> Result<()> {
        let mut writer = csv::Writer::from_path(path)
            .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;

        writer
            .write_record(&df.columns)
            .context("Failed to write CSV header")?;

        for row in &df.rows {
            let values: Vec<&str> = df
                .columns
                .iter()
                .map(|c| row.get(c).map(|s| s.as_str()).unwrap_or(""))
                .collect();
            writer
                .write_record(&values)
                .context("Failed to write CSV row")?;
        }

        writer.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }
}

impl DataSink for JsonSink {
    fn write(&self, df: &DataFrame, path: &Path) -> Result<()> {
        let records: Vec<serde_json::Map<String, serde_json::Value>> = df
            .rows
            .iter()
            .map(|row| {
                df.columns
                    .iter()
                    .map(|c| {
                        let val = row.get(c).cloned().unwrap_or_default();
                        (c.clone(), serde_json::Value::String(val))
                    })
                    .collect()
            })
            .collect();

        let json = serde_json::to_string_pretty(&records)
            .context("Failed to serialize DataFrame to JSON")?;

        fs::write(path, json)
            .with_context(|| format!("Failed to write JSON file: {}", path.display()))?;
        Ok(())
    }
}

impl DataSink for YamlSink {
    fn write(&self, df: &DataFrame, path: &Path) -> Result<()> {
        let records: Vec<serde_yaml::Mapping> = df
            .rows
            .iter()
            .map(|row| {
                let mut map = serde_yaml::Mapping::new();
                for c in &df.columns {
                    let val = row.get(c).cloned().unwrap_or_default();
                    map.insert(
                        serde_yaml::Value::String(c.clone()),
                        serde_yaml::Value::String(val),
                    );
                }
                map
            })
            .collect();

        let yaml = serde_yaml::to_string(&records)
            .context("Failed to serialize DataFrame to YAML")?;

        fs::write(path, yaml)
            .with_context(|| format!("Failed to write YAML file: {}", path.display()))?;
        Ok(())
    }
}

fn sink_for_format(format: Format) -> Box<dyn DataSink> {
    match format {
        Format::Csv => Box::new(CsvSink),
        Format::Json => Box::new(JsonSink),
        Format::Yaml => Box::new(YamlSink),
    }
}

// ---------------------------------------------------------------------------
// Transform trait — the pipeline's unit of work.
// Each transform takes a DataFrame by value and returns a new one.
// New transforms are added by implementing this trait; existing code
// does not change (open/closed principle).
// ---------------------------------------------------------------------------

trait Transform: fmt::Display {
    fn apply(&self, df: DataFrame) -> Result<DataFrame>;
}

// -- Filter ----------------------------------------------------------------

/// Comparison operators for filter expressions.
#[derive(Debug, Clone, Copy)]
enum FilterOp {
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
}

impl FilterOp {
    fn compare_str(&self, left: &str, right: &str) -> bool {
        // Try numeric comparison first; fall back to lexicographic.
        if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
            return self.compare_f64(l, r);
        }
        let ord = left.cmp(right);
        match self {
            FilterOp::Gt => ord == Ordering::Greater,
            FilterOp::Lt => ord == Ordering::Less,
            FilterOp::Ge => ord != Ordering::Less,
            FilterOp::Le => ord != Ordering::Greater,
            FilterOp::Eq => ord == Ordering::Equal,
            FilterOp::Ne => ord != Ordering::Equal,
        }
    }

    fn compare_f64(&self, l: f64, r: f64) -> bool {
        match self {
            FilterOp::Gt => l > r,
            FilterOp::Lt => l < r,
            FilterOp::Ge => l >= r,
            FilterOp::Le => l <= r,
            FilterOp::Eq => (l - r).abs() < f64::EPSILON,
            FilterOp::Ne => (l - r).abs() >= f64::EPSILON,
        }
    }
}

struct FilterTransform {
    column: String,
    op: FilterOp,
    value: String,
}

impl FilterTransform {
    /// Parse an expression like `age>30`, `name==Alice`, `score<=100`.
    fn parse(expr: &str) -> Result<Self> {
        // Try two-char operators first, then single-char.
        let operators: &[(&str, FilterOp)] = &[
            (">=", FilterOp::Ge),
            ("<=", FilterOp::Le),
            ("!=", FilterOp::Ne),
            ("==", FilterOp::Eq),
            (">", FilterOp::Gt),
            ("<", FilterOp::Lt),
        ];

        for (token, op) in operators {
            if let Some(idx) = expr.find(token) {
                let column = expr[..idx].trim().to_string();
                let value = expr[idx + token.len()..].trim().to_string();
                if column.is_empty() || value.is_empty() {
                    bail!("Invalid filter expression (empty column or value): '{expr}'");
                }
                return Ok(Self {
                    column,
                    op: *op,
                    value,
                });
            }
        }

        bail!(
            "Cannot parse filter expression '{expr}'. \
             Expected format: column>value, column==value, etc."
        );
    }
}

impl fmt::Display for FilterTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self.op {
            FilterOp::Gt => ">",
            FilterOp::Lt => "<",
            FilterOp::Ge => ">=",
            FilterOp::Le => "<=",
            FilterOp::Eq => "==",
            FilterOp::Ne => "!=",
        };
        write!(f, "filter({}{}{})", self.column, op_str, self.value)
    }
}

impl Transform for FilterTransform {
    fn apply(&self, df: DataFrame) -> Result<DataFrame> {
        if !df.columns.contains(&self.column) {
            bail!(
                "Filter column '{}' not found. Available columns: {}",
                self.column,
                df.columns.join(", ")
            );
        }

        let rows: Vec<Row> = df
            .rows
            .into_iter()
            .filter(|row| {
                row.get(&self.column)
                    .map(|v| self.op.compare_str(v, &self.value))
                    .unwrap_or(false)
            })
            .collect();

        Ok(DataFrame::new(df.columns, rows))
    }
}

// -- Sort ------------------------------------------------------------------

struct SortTransform {
    column: String,
}

impl fmt::Display for SortTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sort({})", self.column)
    }
}

impl Transform for SortTransform {
    fn apply(&self, mut df: DataFrame) -> Result<DataFrame> {
        if !df.columns.contains(&self.column) {
            bail!(
                "Sort column '{}' not found. Available columns: {}",
                self.column,
                df.columns.join(", ")
            );
        }

        let col = self.column.clone();
        df.rows.sort_by(|a, b| {
            let va = a.get(&col).map(|s| s.as_str()).unwrap_or("");
            let vb = b.get(&col).map(|s| s.as_str()).unwrap_or("");

            // Numeric-aware sort: if both values parse as numbers, compare numerically.
            if let (Ok(na), Ok(nb)) = (va.parse::<f64>(), vb.parse::<f64>()) {
                return na.partial_cmp(&nb).unwrap_or(Ordering::Equal);
            }
            va.cmp(vb)
        });

        Ok(df)
    }
}

// -- Rename ----------------------------------------------------------------

struct RenameTransform {
    old_name: String,
    new_name: String,
}

impl RenameTransform {
    fn parse(spec: &str) -> Result<Self> {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            bail!(
                "Invalid rename spec '{spec}'. Expected format: old_name:new_name"
            );
        }
        Ok(Self {
            old_name: parts[0].trim().to_string(),
            new_name: parts[1].trim().to_string(),
        })
    }
}

impl fmt::Display for RenameTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rename({} -> {})", self.old_name, self.new_name)
    }
}

impl Transform for RenameTransform {
    fn apply(&self, mut df: DataFrame) -> Result<DataFrame> {
        if !df.columns.contains(&self.old_name) {
            bail!(
                "Rename: column '{}' not found. Available columns: {}",
                self.old_name,
                df.columns.join(", ")
            );
        }
        if df.columns.contains(&self.new_name) {
            bail!(
                "Rename: target column '{}' already exists",
                self.new_name
            );
        }

        // Update column list.
        for col in &mut df.columns {
            if *col == self.old_name {
                *col = self.new_name.clone();
            }
        }

        // Update rows.
        for row in &mut df.rows {
            if let Some(val) = row.remove(&self.old_name) {
                row.insert(self.new_name.clone(), val);
            }
        }

        Ok(df)
    }
}

// -- AddColumn (computed) --------------------------------------------------

/// Supported computed-column expressions:
///
/// - Concatenation: `full:first_name+last_name` (string join with space)
/// - Arithmetic:    `total:price*quantity`       (supports +, -, *, /)
/// - Literal:       `status:="active"`           (constant value, prefixed with =)
struct AddColumnTransform {
    new_column: String,
    expression: ComputedExpr,
}

enum ComputedExpr {
    Concat(Vec<String>),
    Arithmetic {
        left: String,
        op: char,
        right: String,
    },
    Literal(String),
}

impl AddColumnTransform {
    fn parse(spec: &str) -> Result<Self> {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            bail!(
                "Invalid add-column spec '{spec}'. Expected format: new_col:expression"
            );
        }
        let new_column = parts[0].trim().to_string();
        let expr_str = parts[1].trim();

        let expression = if expr_str.starts_with('=') {
            // Literal value.
            ComputedExpr::Literal(expr_str[1..].to_string())
        } else if let Some(idx) = expr_str.find(|c: char| "*/".contains(c)) {
            // Arithmetic with * or / (checked before + to avoid ambiguity with concat).
            let op = expr_str.as_bytes()[idx] as char;
            let left = expr_str[..idx].trim().to_string();
            let right = expr_str[idx + 1..].trim().to_string();
            ComputedExpr::Arithmetic { left, op, right }
        } else if expr_str.contains('-')
            && !expr_str.starts_with('-')
            && expr_str.matches('-').count() == 1
        {
            // Arithmetic subtraction (a single '-' that isn't a leading negative sign).
            let idx = expr_str.find('-').unwrap();
            let left = expr_str[..idx].trim().to_string();
            let right = expr_str[idx + 1..].trim().to_string();
            ComputedExpr::Arithmetic {
                left,
                op: '-',
                right,
            }
        } else if expr_str.contains('+') {
            // Concatenation (could also be addition — we try numeric first at eval time).
            let fields: Vec<String> = expr_str.split('+').map(|s| s.trim().to_string()).collect();
            ComputedExpr::Concat(fields)
        } else {
            bail!(
                "Cannot parse computed-column expression '{expr_str}'. \
                 Use col1+col2 (concat/add), col1*col2 (math), or =\"literal\"."
            );
        };

        Ok(Self {
            new_column,
            expression,
        })
    }
}

impl fmt::Display for AddColumnTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.expression {
            ComputedExpr::Concat(fields) => {
                write!(f, "add_column({} = concat({}))", self.new_column, fields.join(", "))
            }
            ComputedExpr::Arithmetic { left, op, right } => {
                write!(f, "add_column({} = {}{}{} )", self.new_column, left, op, right)
            }
            ComputedExpr::Literal(val) => {
                write!(f, "add_column({} = \"{}\")", self.new_column, val)
            }
        }
    }
}

impl Transform for AddColumnTransform {
    fn apply(&self, mut df: DataFrame) -> Result<DataFrame> {
        if df.columns.contains(&self.new_column) {
            bail!(
                "Add-column: column '{}' already exists",
                self.new_column
            );
        }

        for row in &mut df.rows {
            let value = match &self.expression {
                ComputedExpr::Literal(v) => v.clone(),

                ComputedExpr::Concat(fields) => {
                    // If all fields parse as numbers, do numeric addition.
                    let all_numeric = fields.iter().all(|f| {
                        row.get(f)
                            .map(|v| v.parse::<f64>().is_ok())
                            .unwrap_or(false)
                    });

                    if all_numeric {
                        let sum: f64 = fields
                            .iter()
                            .filter_map(|f| row.get(f)?.parse::<f64>().ok())
                            .sum();
                        format_number(sum)
                    } else {
                        fields
                            .iter()
                            .filter_map(|f| row.get(f).map(|v| v.as_str()))
                            .collect::<Vec<_>>()
                            .join(" ")
                    }
                }

                ComputedExpr::Arithmetic { left, op, right } => {
                    let lv: f64 = row
                        .get(left)
                        .ok_or_else(|| anyhow!("Column '{}' not found", left))?
                        .parse::<f64>()
                        .with_context(|| format!("Column '{}' is not numeric", left))?;

                    let rv: f64 = row
                        .get(right)
                        .ok_or_else(|| anyhow!("Column '{}' not found", right))?
                        .parse::<f64>()
                        .with_context(|| format!("Column '{}' is not numeric", right))?;

                    let result = match op {
                        '+' => lv + rv,
                        '-' => lv - rv,
                        '*' => lv * rv,
                        '/' => {
                            if rv == 0.0 {
                                bail!("Division by zero in column '{}'", right);
                            }
                            lv / rv
                        }
                        _ => bail!("Unknown arithmetic operator: '{}'", op),
                    };
                    format_number(result)
                }
            };
            row.insert(self.new_column.clone(), value);
        }

        df.columns.push(self.new_column.clone());
        Ok(df)
    }
}

/// Format a number: drop the trailing `.0` for integers.
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < i64::MAX as f64 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

// ---------------------------------------------------------------------------
// Pipeline — collects transforms and applies them sequentially.
// ---------------------------------------------------------------------------

struct Pipeline {
    steps: Vec<Box<dyn Transform>>,
}

impl Pipeline {
    fn new() -> Self {
        Self { steps: Vec::new() }
    }

    fn add(&mut self, step: Box<dyn Transform>) {
        self.steps.push(step);
    }

    fn execute(&self, mut df: DataFrame) -> Result<DataFrame> {
        for step in &self.steps {
            df = step
                .apply(df)
                .with_context(|| format!("Transform '{}' failed", step))?;
        }
        Ok(df)
    }
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

struct CliArgs {
    input_path: PathBuf,
    output_path: Option<PathBuf>,
    pipeline: Pipeline,
}

impl CliArgs {
    fn parse_from(args: &[String]) -> Result<Self> {
        if args.len() < 2 {
            bail!(
                "Usage: dtool <input> [--filter EXPR] [--sort COL] \
                 [--rename OLD:NEW] [--add-column NAME:EXPR] [--output PATH]"
            );
        }

        let input_path = PathBuf::from(&args[1]);
        let mut output_path: Option<PathBuf> = None;
        let mut pipeline = Pipeline::new();

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--filter" => {
                    i += 1;
                    let expr = args
                        .get(i)
                        .ok_or_else(|| anyhow!("--filter requires an argument"))?;
                    pipeline.add(Box::new(FilterTransform::parse(expr)?));
                }
                "--sort" => {
                    i += 1;
                    let col = args
                        .get(i)
                        .ok_or_else(|| anyhow!("--sort requires an argument"))?;
                    pipeline.add(Box::new(SortTransform {
                        column: col.clone(),
                    }));
                }
                "--rename" => {
                    i += 1;
                    let spec = args
                        .get(i)
                        .ok_or_else(|| anyhow!("--rename requires an argument"))?;
                    pipeline.add(Box::new(RenameTransform::parse(spec)?));
                }
                "--add-column" => {
                    i += 1;
                    let spec = args
                        .get(i)
                        .ok_or_else(|| anyhow!("--add-column requires an argument"))?;
                    pipeline.add(Box::new(AddColumnTransform::parse(spec)?));
                }
                "--output" | "-o" => {
                    i += 1;
                    let path = args
                        .get(i)
                        .ok_or_else(|| anyhow!("--output requires a path"))?;
                    output_path = Some(PathBuf::from(path));
                }
                other => {
                    bail!("Unknown argument: '{other}'");
                }
            }
            i += 1;
        }

        Ok(Self {
            input_path,
            output_path,
            pipeline,
        })
    }
}

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

fn run(args: &[String]) -> Result<()> {
    let cli = CliArgs::parse_from(args)?;

    // Read input.
    let input_format = Format::from_path(&cli.input_path)?;
    let source = source_for_format(input_format);
    let df = source.read(&cli.input_path)?;
    eprintln!(
        "Read {} rows x {} columns from {} ({})",
        df.rows.len(),
        df.columns.len(),
        cli.input_path.display(),
        input_format
    );

    // Apply transforms.
    let df = cli.pipeline.execute(df)?;
    eprintln!(
        "After transforms: {} rows x {} columns",
        df.rows.len(),
        df.columns.len()
    );

    // Write output.
    match &cli.output_path {
        Some(out_path) => {
            let output_format = Format::from_path(out_path)?;
            let sink = sink_for_format(output_format);
            sink.write(&df, out_path)?;
            eprintln!("Wrote {} to {}", output_format, out_path.display());
        }
        None => {
            // Default: print as JSON to stdout.
            let records: Vec<serde_json::Map<String, serde_json::Value>> = df
                .rows
                .iter()
                .map(|row| {
                    df.columns
                        .iter()
                        .map(|c| {
                            let val = row.get(c).cloned().unwrap_or_default();
                            (c.clone(), serde_json::Value::String(val))
                        })
                        .collect()
                })
                .collect();
            let json = serde_json::to_string_pretty(&records)
                .context("Failed to serialize output to JSON")?;
            println!("{json}");
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run(&args) {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_df() -> DataFrame {
        let columns = vec![
            "name".to_string(),
            "age".to_string(),
            "city".to_string(),
        ];
        let rows = vec![
            HashMap::from([
                ("name".into(), "Alice".into()),
                ("age".into(), "30".into()),
                ("city".into(), "NYC".into()),
            ]),
            HashMap::from([
                ("name".into(), "Bob".into()),
                ("age".into(), "25".into()),
                ("city".into(), "LA".into()),
            ]),
            HashMap::from([
                ("name".into(), "Charlie".into()),
                ("age".into(), "35".into()),
                ("city".into(), "NYC".into()),
            ]),
        ];
        DataFrame::new(columns, rows)
    }

    #[test]
    fn test_filter_gt() {
        let df = sample_df();
        let t = FilterTransform::parse("age>28").unwrap();
        let result = t.apply(df).unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0]["name"], "Alice");
        assert_eq!(result.rows[1]["name"], "Charlie");
    }

    #[test]
    fn test_filter_eq() {
        let df = sample_df();
        let t = FilterTransform::parse("city==NYC").unwrap();
        let result = t.apply(df).unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_filter_ne() {
        let df = sample_df();
        let t = FilterTransform::parse("city!=NYC").unwrap();
        let result = t.apply(df).unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0]["name"], "Bob");
    }

    #[test]
    fn test_sort_by_name() {
        let df = sample_df();
        let t = SortTransform {
            column: "name".to_string(),
        };
        let result = t.apply(df).unwrap();
        let names: Vec<&str> = result.rows.iter().map(|r| r["name"].as_str()).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
    }

    #[test]
    fn test_sort_numeric() {
        let df = sample_df();
        let t = SortTransform {
            column: "age".to_string(),
        };
        let result = t.apply(df).unwrap();
        let ages: Vec<&str> = result.rows.iter().map(|r| r["age"].as_str()).collect();
        assert_eq!(ages, vec!["25", "30", "35"]);
    }

    #[test]
    fn test_rename() {
        let df = sample_df();
        let t = RenameTransform::parse("name:full_name").unwrap();
        let result = t.apply(df).unwrap();
        assert!(result.columns.contains(&"full_name".to_string()));
        assert!(!result.columns.contains(&"name".to_string()));
        assert_eq!(result.rows[0]["full_name"], "Alice");
    }

    #[test]
    fn test_add_column_literal() {
        let df = sample_df();
        let t = AddColumnTransform::parse("status:=active").unwrap();
        let result = t.apply(df).unwrap();
        assert!(result.columns.contains(&"status".to_string()));
        assert_eq!(result.rows[0]["status"], "active");
    }

    #[test]
    fn test_add_column_concat() {
        let df = sample_df();
        let t = AddColumnTransform::parse("label:name+city").unwrap();
        let result = t.apply(df).unwrap();
        assert_eq!(result.rows[0]["label"], "Alice NYC");
    }

    #[test]
    fn test_pipeline() {
        let df = sample_df();
        let mut pipeline = Pipeline::new();
        pipeline.add(Box::new(FilterTransform::parse("age>28").unwrap()));
        pipeline.add(Box::new(SortTransform {
            column: "name".to_string(),
        }));
        pipeline.add(Box::new(RenameTransform::parse("name:person").unwrap()));

        let result = pipeline.execute(df).unwrap();
        assert_eq!(result.rows.len(), 2);
        assert!(result.columns.contains(&"person".to_string()));
        let names: Vec<&str> = result.rows.iter().map(|r| r["person"].as_str()).collect();
        assert_eq!(names, vec!["Alice", "Charlie"]);
    }

    #[test]
    fn test_filter_missing_column() {
        let df = sample_df();
        let t = FilterTransform::parse("nonexistent>5").unwrap();
        let result = t.apply(df);
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_duplicate_target() {
        let df = sample_df();
        let t = RenameTransform::parse("name:age").unwrap();
        let result = t.apply(df);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(Format::from_path(Path::new("data.csv")).unwrap(), Format::Csv);
        assert_eq!(Format::from_path(Path::new("data.json")).unwrap(), Format::Json);
        assert_eq!(Format::from_path(Path::new("data.yaml")).unwrap(), Format::Yaml);
        assert_eq!(Format::from_path(Path::new("data.yml")).unwrap(), Format::Yaml);
        assert!(Format::from_path(Path::new("data.xml")).is_err());
        assert!(Format::from_path(Path::new("noext")).is_err());
    }
}
