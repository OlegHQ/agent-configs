// CLI Data Tool — `dtool`
//
// Architecture:
// - **Strategy pattern** for file format readers/writers: trait DataReader / DataWriter
//   with one impl per format (CsvFormat, JsonFormat, YamlFormat). Adding a new format
//   (e.g. Parquet) means adding one struct + impls — zero changes to existing code.
// - **Strategy pattern** for transforms: trait Transform with one impl per operation
//   (FilterTransform, SortTransform, RenameTransform, ComputedColumnTransform).
//   Adding a new transform means one new struct + impl.
// - **Pipeline composition**: transforms collected into Vec<Box<dyn Transform>> and
//   applied sequentially. Order matches CLI argument order.

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Core data model
// ---------------------------------------------------------------------------

/// A simple columnar data frame: column names + rows of string values.
#[derive(Debug, Clone)]
struct DataFrame {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl DataFrame {
    fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c == name)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ToolError(String);

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ToolError {}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

// ---------------------------------------------------------------------------
// Strategy: DataReader — one impl per file format
// ---------------------------------------------------------------------------

trait DataReader {
    fn read(&self, path: &Path) -> Result<DataFrame>;
}

// ---------------------------------------------------------------------------
// Strategy: DataWriter — one impl per file format
// ---------------------------------------------------------------------------

trait DataWriter {
    fn write(&self, data: &DataFrame, path: &Path) -> Result<()>;
}

// ---------------------------------------------------------------------------
// CSV format
// ---------------------------------------------------------------------------

struct CsvFormat;

impl DataReader for CsvFormat {
    fn read(&self, path: &Path) -> Result<DataFrame> {
        let content = fs::read_to_string(path)?;
        let mut lines = content.lines();

        let header_line = lines.next().ok_or_else(|| ToolError("CSV file is empty".into()))?;
        let columns: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();

        let rows: Vec<Vec<String>> = lines
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
            .collect();

        Ok(DataFrame { columns, rows })
    }
}

impl DataWriter for CsvFormat {
    fn write(&self, data: &DataFrame, path: &Path) -> Result<()> {
        let mut out = String::new();
        out.push_str(&data.columns.join(","));
        out.push('\n');
        for row in &data.rows {
            out.push_str(&row.join(","));
            out.push('\n');
        }
        fs::write(path, out)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// JSON format
// ---------------------------------------------------------------------------

struct JsonFormat;

impl DataReader for JsonFormat {
    fn read(&self, path: &Path) -> Result<DataFrame> {
        let content = fs::read_to_string(path)?;
        // Expect an array of objects: [{"col": "val", ...}, ...]
        let records: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(&content)?;

        if records.is_empty() {
            return Ok(DataFrame {
                columns: vec![],
                rows: vec![],
            });
        }

        // Collect all unique keys preserving first-seen order
        let mut columns: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for record in &records {
            for key in record.keys() {
                if seen.insert(key.clone()) {
                    columns.push(key.clone());
                }
            }
        }
        columns.sort(); // deterministic output

        let rows: Vec<Vec<String>> = records
            .iter()
            .map(|rec| {
                columns
                    .iter()
                    .map(|col| {
                        rec.get(col)
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .collect();

        Ok(DataFrame { columns, rows })
    }
}

impl DataWriter for JsonFormat {
    fn write(&self, data: &DataFrame, path: &Path) -> Result<()> {
        let records: Vec<serde_json::Value> = data
            .rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, col) in data.columns.iter().enumerate() {
                    let val = row.get(i).cloned().unwrap_or_default();
                    // Try to preserve numeric types in JSON output
                    if let Ok(n) = val.parse::<i64>() {
                        map.insert(col.clone(), serde_json::Value::Number(n.into()));
                    } else if let Ok(n) = val.parse::<f64>() {
                        if let Some(num) = serde_json::Number::from_f64(n) {
                            map.insert(col.clone(), serde_json::Value::Number(num));
                        } else {
                            map.insert(col.clone(), serde_json::Value::String(val));
                        }
                    } else {
                        map.insert(col.clone(), serde_json::Value::String(val));
                    }
                }
                serde_json::Value::Object(map)
            })
            .collect();

        let json = serde_json::to_string_pretty(&records)?;
        fs::write(path, json)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// YAML format
// ---------------------------------------------------------------------------

struct YamlFormat;

impl DataReader for YamlFormat {
    fn read(&self, path: &Path) -> Result<DataFrame> {
        let content = fs::read_to_string(path)?;
        let values: Vec<HashMap<String, serde_yaml::Value>> = serde_yaml::from_str(&content)?;

        if values.is_empty() {
            return Ok(DataFrame {
                columns: vec![],
                rows: vec![],
            });
        }

        let mut columns: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for rec in &values {
            for key in rec.keys() {
                if seen.insert(key.clone()) {
                    columns.push(key.clone());
                }
            }
        }
        columns.sort();

        let rows: Vec<Vec<String>> = values
            .iter()
            .map(|rec| {
                columns
                    .iter()
                    .map(|col| {
                        rec.get(col)
                            .map(|v| yaml_value_to_string(v))
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .collect();

        Ok(DataFrame { columns, rows })
    }
}

fn yaml_value_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => String::new(),
        other => format!("{:?}", other),
    }
}

impl DataWriter for YamlFormat {
    fn write(&self, data: &DataFrame, path: &Path) -> Result<()> {
        let records: Vec<serde_yaml::Value> = data
            .rows
            .iter()
            .map(|row| {
                let mut map = serde_yaml::Mapping::new();
                for (i, col) in data.columns.iter().enumerate() {
                    let val = row.get(i).cloned().unwrap_or_default();
                    let yaml_val = if let Ok(n) = val.parse::<i64>() {
                        serde_yaml::Value::Number(serde_yaml::Number::from(n))
                    } else if let Ok(n) = val.parse::<f64>() {
                        serde_yaml::Value::Number(serde_yaml::Number::from(n))
                    } else {
                        serde_yaml::Value::String(val)
                    };
                    map.insert(serde_yaml::Value::String(col.clone()), yaml_val);
                }
                serde_yaml::Value::Mapping(map)
            })
            .collect();

        let yaml = serde_yaml::to_string(&records)?;
        fs::write(path, yaml)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Format resolution — small closed enum just for extension-based dispatch
// ---------------------------------------------------------------------------

enum FormatKind {
    Csv,
    Json,
    Yaml,
}

fn detect_format(path: &str) -> Result<FormatKind> {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("csv") => Ok(FormatKind::Csv),
        Some("json") => Ok(FormatKind::Json),
        Some("yaml" | "yml") => Ok(FormatKind::Yaml),
        Some(ext) => Err(Box::new(ToolError(format!("Unsupported format: .{}", ext)))),
        None => Err(Box::new(ToolError("Cannot detect format: no file extension".into()))),
    }
}

fn make_reader(kind: &FormatKind) -> Box<dyn DataReader> {
    match kind {
        FormatKind::Csv => Box::new(CsvFormat),
        FormatKind::Json => Box::new(JsonFormat),
        FormatKind::Yaml => Box::new(YamlFormat),
    }
}

fn make_writer(kind: &FormatKind) -> Box<dyn DataWriter> {
    match kind {
        FormatKind::Csv => Box::new(CsvFormat),
        FormatKind::Json => Box::new(JsonFormat),
        FormatKind::Yaml => Box::new(YamlFormat),
    }
}

// ---------------------------------------------------------------------------
// Strategy: Transform — one impl per transformation kind
// ---------------------------------------------------------------------------

trait Transform {
    /// Apply this transformation, consuming and returning the data frame.
    fn apply(&self, data: DataFrame) -> Result<DataFrame>;

    /// Human-readable description for debug / dry-run output.
    fn describe(&self) -> String;
}

// --- FilterTransform -------------------------------------------------------

#[derive(Debug)]
enum FilterOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    Contains,
}

struct FilterTransform {
    column: String,
    op: FilterOp,
    value: String,
}

impl FilterTransform {
    /// Parse a filter expression like "age>30", "name=Alice", "city!=London",
    /// "name~Ali" (contains).
    fn parse(expr: &str) -> Result<Self> {
        // Order matters: check two-char operators before single-char
        let ops: &[(&str, FilterOp)] = &[
            (">=", FilterOp::Ge),
            ("<=", FilterOp::Le),
            ("!=", FilterOp::Ne),
            (">", FilterOp::Gt),
            ("<", FilterOp::Lt),
            ("=", FilterOp::Eq),
            ("~", FilterOp::Contains),
        ];

        for (token, op) in ops {
            if let Some(pos) = expr.find(token) {
                let column = expr[..pos].trim().to_string();
                let value = expr[pos + token.len()..].trim().to_string();
                if column.is_empty() {
                    return Err(Box::new(ToolError(format!(
                        "Filter expression missing column name: '{}'",
                        expr
                    ))));
                }
                return Ok(FilterTransform { column, op: op.clone(), value });
            }
        }

        Err(Box::new(ToolError(format!(
            "Cannot parse filter expression: '{}'. Use col>val, col=val, col!=val, col~val, etc.",
            expr
        ))))
    }

    fn compare(&self, cell: &str) -> bool {
        // Try numeric comparison first, fall back to string comparison
        let numeric = cell.parse::<f64>().ok().zip(self.value.parse::<f64>().ok());

        match self.op {
            FilterOp::Eq => cell == self.value,
            FilterOp::Ne => cell != self.value,
            FilterOp::Contains => cell.contains(&self.value),
            FilterOp::Gt => numeric
                .map(|(a, b)| a > b)
                .unwrap_or_else(|| cell > self.value.as_str()),
            FilterOp::Lt => numeric
                .map(|(a, b)| a < b)
                .unwrap_or_else(|| cell < self.value.as_str()),
            FilterOp::Ge => numeric
                .map(|(a, b)| a >= b)
                .unwrap_or_else(|| cell >= self.value.as_str()),
            FilterOp::Le => numeric
                .map(|(a, b)| a <= b)
                .unwrap_or_else(|| cell <= self.value.as_str()),
        }
    }
}

impl Clone for FilterOp {
    fn clone(&self) -> Self {
        match self {
            FilterOp::Eq => FilterOp::Eq,
            FilterOp::Ne => FilterOp::Ne,
            FilterOp::Gt => FilterOp::Gt,
            FilterOp::Lt => FilterOp::Lt,
            FilterOp::Ge => FilterOp::Ge,
            FilterOp::Le => FilterOp::Le,
            FilterOp::Contains => FilterOp::Contains,
        }
    }
}

impl Transform for FilterTransform {
    fn apply(&self, data: DataFrame) -> Result<DataFrame> {
        let col_idx = data
            .column_index(&self.column)
            .ok_or_else(|| ToolError(format!("Filter: column '{}' not found", self.column)))?;

        let rows: Vec<Vec<String>> = data
            .rows
            .into_iter()
            .filter(|row| {
                row.get(col_idx)
                    .map(|cell| self.compare(cell))
                    .unwrap_or(false)
            })
            .collect();

        Ok(DataFrame {
            columns: data.columns,
            rows,
        })
    }

    fn describe(&self) -> String {
        format!("filter: {} {:?} {}", self.column, self.op, self.value)
    }
}

// --- SortTransform ---------------------------------------------------------

struct SortTransform {
    column: String,
    descending: bool,
}

impl SortTransform {
    fn parse(spec: &str) -> Self {
        if let Some(col) = spec.strip_prefix('-') {
            SortTransform {
                column: col.to_string(),
                descending: true,
            }
        } else {
            SortTransform {
                column: spec.to_string(),
                descending: false,
            }
        }
    }
}

impl Transform for SortTransform {
    fn apply(&self, mut data: DataFrame) -> Result<DataFrame> {
        let col_idx = data
            .column_index(&self.column)
            .ok_or_else(|| ToolError(format!("Sort: column '{}' not found", self.column)))?;

        data.rows.sort_by(|a, b| {
            let va = a.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            let vb = b.get(col_idx).map(|s| s.as_str()).unwrap_or("");

            // Try numeric comparison first
            let cmp = match (va.parse::<f64>(), vb.parse::<f64>()) {
                (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
                _ => va.cmp(vb),
            };

            if self.descending {
                cmp.reverse()
            } else {
                cmp
            }
        });

        Ok(data)
    }

    fn describe(&self) -> String {
        let dir = if self.descending { "desc" } else { "asc" };
        format!("sort: {} ({})", self.column, dir)
    }
}

// --- RenameTransform -------------------------------------------------------

struct RenameTransform {
    old_name: String,
    new_name: String,
}

impl RenameTransform {
    fn parse(spec: &str) -> Result<Self> {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(Box::new(ToolError(format!(
                "Rename spec must be 'old_name:new_name', got '{}'",
                spec
            ))));
        }
        Ok(RenameTransform {
            old_name: parts[0].trim().to_string(),
            new_name: parts[1].trim().to_string(),
        })
    }
}

impl Transform for RenameTransform {
    fn apply(&self, mut data: DataFrame) -> Result<DataFrame> {
        let col_idx = data.column_index(&self.old_name).ok_or_else(|| {
            ToolError(format!("Rename: column '{}' not found", self.old_name))
        })?;
        data.columns[col_idx] = self.new_name.clone();
        Ok(data)
    }

    fn describe(&self) -> String {
        format!("rename: {} -> {}", self.old_name, self.new_name)
    }
}

// --- ComputedColumnTransform -----------------------------------------------

/// Adds a new column computed from existing columns.
/// Syntax: "new_col=col1+col2" (numeric addition) or "new_col=col1&col2" (string concat).
struct ComputedColumnTransform {
    new_column: String,
    expression: ComputedExpr,
}

enum ComputedExpr {
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    Concat(String, String),
}

impl ComputedColumnTransform {
    fn parse(spec: &str) -> Result<Self> {
        let parts: Vec<&str> = spec.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(Box::new(ToolError(format!(
                "Computed column must be 'new_col=expr', got '{}'",
                spec
            ))));
        }
        let new_column = parts[0].trim().to_string();
        let expr_str = parts[1].trim();

        let expression = if let Some(pos) = expr_str.find('&') {
            let left = expr_str[..pos].trim().to_string();
            let right = expr_str[pos + 1..].trim().to_string();
            ComputedExpr::Concat(left, right)
        } else if let Some(pos) = expr_str.find('+') {
            let left = expr_str[..pos].trim().to_string();
            let right = expr_str[pos + 1..].trim().to_string();
            ComputedExpr::Add(left, right)
        } else if let Some(pos) = expr_str.find('*') {
            let left = expr_str[..pos].trim().to_string();
            let right = expr_str[pos + 1..].trim().to_string();
            ComputedExpr::Mul(left, right)
        } else if let Some(pos) = expr_str.find('-') {
            let left = expr_str[..pos].trim().to_string();
            let right = expr_str[pos + 1..].trim().to_string();
            ComputedExpr::Sub(left, right)
        } else {
            return Err(Box::new(ToolError(format!(
                "Computed expression must use +, -, *, or & operator: '{}'",
                expr_str
            ))));
        };

        Ok(ComputedColumnTransform {
            new_column,
            expression,
        })
    }

    fn evaluate_row(&self, row: &[String], columns: &[String]) -> Result<String> {
        let get = |name: &str| -> Result<String> {
            let idx = columns
                .iter()
                .position(|c| c == name)
                .ok_or_else(|| ToolError(format!("Computed: column '{}' not found", name)))?;
            Ok(row.get(idx).cloned().unwrap_or_default())
        };

        match &self.expression {
            ComputedExpr::Add(left, right) => {
                let a: f64 = get(left)?.parse().unwrap_or(0.0);
                let b: f64 = get(right)?.parse().unwrap_or(0.0);
                Ok(format!("{}", a + b))
            }
            ComputedExpr::Sub(left, right) => {
                let a: f64 = get(left)?.parse().unwrap_or(0.0);
                let b: f64 = get(right)?.parse().unwrap_or(0.0);
                Ok(format!("{}", a - b))
            }
            ComputedExpr::Mul(left, right) => {
                let a: f64 = get(left)?.parse().unwrap_or(0.0);
                let b: f64 = get(right)?.parse().unwrap_or(0.0);
                Ok(format!("{}", a * b))
            }
            ComputedExpr::Concat(left, right) => {
                let a = get(left)?;
                let b = get(right)?;
                Ok(format!("{}{}", a, b))
            }
        }
    }
}

impl Transform for ComputedColumnTransform {
    fn apply(&self, mut data: DataFrame) -> Result<DataFrame> {
        let mut new_col_values: Vec<String> = Vec::with_capacity(data.rows.len());
        for row in &data.rows {
            new_col_values.push(self.evaluate_row(row, &data.columns)?);
        }

        data.columns.push(self.new_column.clone());
        for (row, val) in data.rows.iter_mut().zip(new_col_values) {
            row.push(val);
        }

        Ok(data)
    }

    fn describe(&self) -> String {
        format!("computed column: {}", self.new_column)
    }
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

struct CliArgs {
    input_path: String,
    output_path: Option<String>,
    transforms: Vec<Box<dyn Transform>>,
}

fn parse_args(args: Vec<String>) -> Result<CliArgs> {
    if args.len() < 2 {
        return Err(Box::new(ToolError(
            "Usage: dtool <input> [--filter 'col>val'] [--sort col] \
             [--rename 'old:new'] [--compute 'new=a+b'] [--output out.json]"
                .into(),
        )));
    }

    let input_path = args[1].clone();
    let mut output_path: Option<String> = None;
    let mut transforms: Vec<Box<dyn Transform>> = Vec::new();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--filter" | "-f" => {
                i += 1;
                let expr = args
                    .get(i)
                    .ok_or_else(|| ToolError("--filter requires a value".into()))?;
                transforms.push(Box::new(FilterTransform::parse(expr)?));
            }
            "--sort" | "-s" => {
                i += 1;
                let spec = args
                    .get(i)
                    .ok_or_else(|| ToolError("--sort requires a column name".into()))?;
                transforms.push(Box::new(SortTransform::parse(spec)));
            }
            "--rename" | "-r" => {
                i += 1;
                let spec = args
                    .get(i)
                    .ok_or_else(|| ToolError("--rename requires 'old:new'".into()))?;
                transforms.push(Box::new(RenameTransform::parse(spec)?));
            }
            "--compute" | "-c" => {
                i += 1;
                let spec = args
                    .get(i)
                    .ok_or_else(|| ToolError("--compute requires 'new_col=expr'".into()))?;
                transforms.push(Box::new(ComputedColumnTransform::parse(spec)?));
            }
            "--output" | "-o" => {
                i += 1;
                output_path = Some(
                    args.get(i)
                        .ok_or_else(|| ToolError("--output requires a file path".into()))?
                        .clone(),
                );
            }
            other => {
                return Err(Box::new(ToolError(format!("Unknown argument: {}", other))));
            }
        }
        i += 1;
    }

    Ok(CliArgs {
        input_path,
        output_path,
        transforms,
    })
}

// ---------------------------------------------------------------------------
// Pipeline execution
// ---------------------------------------------------------------------------

fn run(args: CliArgs) -> Result<()> {
    // 1. Read input
    let input_format = detect_format(&args.input_path)?;
    let reader = make_reader(&input_format);
    let mut data = reader.read(Path::new(&args.input_path))?;

    eprintln!(
        "Read {} rows x {} columns from '{}'",
        data.rows.len(),
        data.columns.len(),
        args.input_path
    );

    // 2. Apply transform pipeline (Strategy pattern — Vec<Box<dyn Transform>>)
    for transform in &args.transforms {
        eprintln!("  Applying: {}", transform.describe());
        data = transform.apply(data)?;
        eprintln!("    -> {} rows x {} columns", data.rows.len(), data.columns.len());
    }

    // 3. Write output
    if let Some(ref out_path) = args.output_path {
        let output_format = detect_format(out_path)?;
        let writer = make_writer(&output_format);
        writer.write(&data, Path::new(out_path))?;
        eprintln!("Wrote output to '{}'", out_path);
    } else {
        // Default: print as CSV to stdout
        println!("{}", data.columns.join(","));
        for row in &data.rows {
            println!("{}", row.join(","));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().collect();
    let cli_args = match parse_args(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = run(cli_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> DataFrame {
        DataFrame {
            columns: vec![
                "name".to_string(),
                "age".to_string(),
                "city".to_string(),
            ],
            rows: vec![
                vec!["Alice".to_string(), "30".to_string(), "London".to_string()],
                vec!["Bob".to_string(), "25".to_string(), "Paris".to_string()],
                vec!["Charlie".to_string(), "35".to_string(), "London".to_string()],
                vec!["Diana".to_string(), "28".to_string(), "Berlin".to_string()],
            ],
        }
    }

    #[test]
    fn test_filter_greater_than() {
        let data = sample_data();
        let filter = FilterTransform::parse("age>28").unwrap();
        let result = filter.apply(data).unwrap();
        assert_eq!(result.rows.len(), 2); // Alice (30) and Charlie (35)
        assert_eq!(result.rows[0][0], "Alice");
        assert_eq!(result.rows[1][0], "Charlie");
    }

    #[test]
    fn test_filter_equals() {
        let data = sample_data();
        let filter = FilterTransform::parse("city=London").unwrap();
        let result = filter.apply(data).unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_filter_not_equals() {
        let data = sample_data();
        let filter = FilterTransform::parse("city!=London").unwrap();
        let result = filter.apply(data).unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][2], "Paris");
        assert_eq!(result.rows[1][2], "Berlin");
    }

    #[test]
    fn test_filter_contains() {
        let data = sample_data();
        let filter = FilterTransform::parse("name~li").unwrap();
        let result = filter.apply(data).unwrap();
        assert_eq!(result.rows.len(), 2); // Alice, Charlie
    }

    #[test]
    fn test_sort_ascending_string() {
        let data = sample_data();
        let sort = SortTransform::parse("name");
        let result = sort.apply(data).unwrap();
        assert_eq!(result.rows[0][0], "Alice");
        assert_eq!(result.rows[1][0], "Bob");
        assert_eq!(result.rows[2][0], "Charlie");
        assert_eq!(result.rows[3][0], "Diana");
    }

    #[test]
    fn test_sort_descending_numeric() {
        let data = sample_data();
        let sort = SortTransform::parse("-age");
        let result = sort.apply(data).unwrap();
        assert_eq!(result.rows[0][0], "Charlie"); // 35
        assert_eq!(result.rows[1][0], "Alice"); // 30
    }

    #[test]
    fn test_rename() {
        let data = sample_data();
        let rename = RenameTransform::parse("name:full_name").unwrap();
        let result = rename.apply(data).unwrap();
        assert_eq!(result.columns[0], "full_name");
    }

    #[test]
    fn test_computed_add() {
        let mut data = sample_data();
        // Add a "score" column to test addition
        data.columns.push("score".to_string());
        for row in &mut data.rows {
            row.push("10".to_string());
        }

        let compute = ComputedColumnTransform::parse("total=age+score").unwrap();
        let result = compute.apply(data).unwrap();
        assert_eq!(result.columns.last().unwrap(), "total");
        assert_eq!(result.rows[0].last().unwrap(), "40"); // 30 + 10
    }

    #[test]
    fn test_computed_concat() {
        let data = sample_data();
        let compute = ComputedColumnTransform::parse("label=name&city").unwrap();
        let result = compute.apply(data).unwrap();
        assert_eq!(result.rows[0].last().unwrap(), "AliceLondon");
    }

    #[test]
    fn test_pipeline_composition() {
        let data = sample_data();

        // Build a pipeline: filter age>27, then sort by name
        let pipeline: Vec<Box<dyn Transform>> = vec![
            Box::new(FilterTransform::parse("age>27").unwrap()),
            Box::new(SortTransform::parse("name")),
        ];

        let mut result = data;
        for step in &pipeline {
            result = step.apply(result).unwrap();
        }

        // Should have Alice(30), Charlie(35), Diana(28) sorted by name
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.rows[0][0], "Alice");
        assert_eq!(result.rows[1][0], "Charlie");
        assert_eq!(result.rows[2][0], "Diana");
    }

    #[test]
    fn test_detect_format() {
        assert!(matches!(detect_format("data.csv"), Ok(FormatKind::Csv)));
        assert!(matches!(detect_format("data.json"), Ok(FormatKind::Json)));
        assert!(matches!(detect_format("data.yaml"), Ok(FormatKind::Yaml)));
        assert!(matches!(detect_format("data.yml"), Ok(FormatKind::Yaml)));
        assert!(detect_format("data.xml").is_err());
    }

    #[test]
    fn test_parse_args_minimal() {
        let args = vec!["dtool".to_string(), "input.csv".to_string()];
        let cli = parse_args(args).unwrap();
        assert_eq!(cli.input_path, "input.csv");
        assert!(cli.output_path.is_none());
        assert!(cli.transforms.is_empty());
    }

    #[test]
    fn test_parse_args_full() {
        let args = vec![
            "dtool".to_string(),
            "input.csv".to_string(),
            "--filter".to_string(),
            "age>30".to_string(),
            "--sort".to_string(),
            "name".to_string(),
            "--rename".to_string(),
            "old:new".to_string(),
            "--output".to_string(),
            "result.json".to_string(),
        ];
        let cli = parse_args(args).unwrap();
        assert_eq!(cli.input_path, "input.csv");
        assert_eq!(cli.output_path.as_deref(), Some("result.json"));
        assert_eq!(cli.transforms.len(), 3);
    }

    #[test]
    fn test_filter_unknown_column() {
        let data = sample_data();
        let filter = FilterTransform::parse("nonexistent>5").unwrap();
        let result = filter.apply(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_bad_spec() {
        assert!(RenameTransform::parse("no_colon_here").is_err());
    }
}
