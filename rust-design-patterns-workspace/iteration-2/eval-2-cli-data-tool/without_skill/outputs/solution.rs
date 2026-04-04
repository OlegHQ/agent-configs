// CLI Data Processing Tool
//
// Usage:
//   dtool input.csv --filter 'age>30' --sort name --rename 'old_name:new_name' --add 'new_col=col1+col2' --output result.json
//
// Cargo.toml dependencies:
// [dependencies]
// clap = { version = "4", features = ["derive"] }
// csv = "1.3"
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// serde_yaml = "0.9"
// anyhow = "1"

use anyhow::{anyhow, bail, Context, Result};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A table is a list of column names plus rows of string values.
#[derive(Debug, Clone)]
struct Table {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn column_index(&self, name: &str) -> Result<usize> {
        self.columns
            .iter()
            .position(|c| c == name)
            .ok_or_else(|| anyhow!("column '{}' not found (available: {:?})", name, self.columns))
    }
}

// ---------------------------------------------------------------------------
// Format detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Csv,
    Json,
    Yaml,
}

fn detect_format(path: &str) -> Result<Format> {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("csv") => Ok(Format::Csv),
        Some("json") => Ok(Format::Json),
        Some("yaml" | "yml") => Ok(Format::Yaml),
        Some(ext) => bail!("unsupported file extension: .{}", ext),
        None => bail!("cannot detect format: file has no extension"),
    }
}

// ---------------------------------------------------------------------------
// Readers
// ---------------------------------------------------------------------------

fn read_csv(path: &str) -> Result<Table> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open CSV file: {}", path))?;
    let columns: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();
    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result?;
        rows.push(record.iter().map(|f| f.to_string()).collect());
    }
    Ok(Table { columns, rows })
}

fn read_json(path: &str) -> Result<Table> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("failed to read JSON file: {}", path))?;
    let values: Vec<serde_json::Map<String, serde_json::Value>> =
        serde_json::from_str(&data).context("JSON must be an array of objects")?;

    if values.is_empty() {
        return Ok(Table {
            columns: Vec::new(),
            rows: Vec::new(),
        });
    }

    // Collect all keys in stable order (insertion order of first object, then any extras).
    let mut columns: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for obj in &values {
        for key in obj.keys() {
            if seen.insert(key.clone()) {
                columns.push(key.clone());
            }
        }
    }

    let rows: Vec<Vec<String>> = values
        .iter()
        .map(|obj| {
            columns
                .iter()
                .map(|col| {
                    obj.get(col)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();

    Ok(Table { columns, rows })
}

fn read_yaml(path: &str) -> Result<Table> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML file: {}", path))?;
    let values: Vec<HashMap<String, serde_yaml::Value>> =
        serde_yaml::from_str(&data).context("YAML must be a sequence of mappings")?;

    if values.is_empty() {
        return Ok(Table {
            columns: Vec::new(),
            rows: Vec::new(),
        });
    }

    let mut columns: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for obj in &values {
        for key in obj.keys() {
            if seen.insert(key.clone()) {
                columns.push(key.clone());
            }
        }
    }

    let rows: Vec<Vec<String>> = values
        .iter()
        .map(|obj| {
            columns
                .iter()
                .map(|col| {
                    obj.get(col)
                        .map(|v| yaml_value_to_string(v))
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();

    Ok(Table { columns, rows })
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

fn read_table(path: &str) -> Result<Table> {
    let fmt = detect_format(path)?;
    match fmt {
        Format::Csv => read_csv(path),
        Format::Json => read_json(path),
        Format::Yaml => read_yaml(path),
    }
}

// ---------------------------------------------------------------------------
// Writers
// ---------------------------------------------------------------------------

fn write_csv(table: &Table, path: &str) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(&table.columns)?;
    for row in &table.rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
}

fn write_json(table: &Table, path: &str) -> Result<()> {
    let objects: Vec<serde_json::Map<String, serde_json::Value>> = table
        .rows
        .iter()
        .map(|row| {
            table
                .columns
                .iter()
                .zip(row.iter())
                .map(|(col, val)| {
                    let json_val = if let Ok(n) = val.parse::<i64>() {
                        serde_json::Value::Number(n.into())
                    } else if let Ok(n) = val.parse::<f64>() {
                        serde_json::json!(n)
                    } else if val == "true" || val == "false" {
                        serde_json::Value::Bool(val == "true")
                    } else {
                        serde_json::Value::String(val.clone())
                    };
                    (col.clone(), json_val)
                })
                .collect()
        })
        .collect();
    let json = serde_json::to_string_pretty(&objects)?;
    fs::write(path, json)?;
    Ok(())
}

fn write_yaml(table: &Table, path: &str) -> Result<()> {
    let objects: Vec<HashMap<String, serde_yaml::Value>> = table
        .rows
        .iter()
        .map(|row| {
            table
                .columns
                .iter()
                .zip(row.iter())
                .map(|(col, val)| {
                    let yaml_val = if let Ok(n) = val.parse::<i64>() {
                        serde_yaml::Value::Number(serde_yaml::Number::from(n))
                    } else if let Ok(n) = val.parse::<f64>() {
                        serde_yaml::Value::Number(
                            serde_yaml::Number::from(n),
                        )
                    } else if val == "true" || val == "false" {
                        serde_yaml::Value::Bool(val == "true")
                    } else {
                        serde_yaml::Value::String(val.clone())
                    };
                    (col.clone(), yaml_val)
                })
                .collect()
        })
        .collect();
    let yaml = serde_yaml::to_string(&objects)?;
    fs::write(path, yaml)?;
    Ok(())
}

fn write_table(table: &Table, path: &str) -> Result<()> {
    let fmt = detect_format(path)?;
    // Ensure parent directory exists.
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    match fmt {
        Format::Csv => write_csv(table, path),
        Format::Json => write_json(table, path),
        Format::Yaml => write_yaml(table, path),
    }
}

// ---------------------------------------------------------------------------
// Transformations
// ---------------------------------------------------------------------------

/// Supported filter operators: =, !=, >, <, >=, <=, ~(contains)
fn apply_filter(table: &mut Table, expr: &str) -> Result<()> {
    // Parse the expression: column<op>value
    let operators = [">=", "<=", "!=", "~", ">", "<", "="];
    let (col_name, op, value) = operators
        .iter()
        .find_map(|&op| {
            expr.find(op).map(|pos| {
                let col = expr[..pos].trim();
                let val = expr[pos + op.len()..].trim();
                (col, op, val)
            })
        })
        .ok_or_else(|| anyhow!("invalid filter expression: '{}'. Use column<op>value where op is one of =, !=, >, <, >=, <=, ~", expr))?;

    let idx = table.column_index(col_name)?;

    table.rows.retain(|row| {
        let cell = &row[idx];
        match op {
            "=" => cell == value,
            "!=" => cell != value,
            "~" => cell.contains(value),
            ">" | "<" | ">=" | "<=" => {
                // Try numeric comparison first, fall back to string.
                let cmp = match (cell.parse::<f64>(), value.parse::<f64>()) {
                    (Ok(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
                    _ => cell.as_str().cmp(value),
                };
                match op {
                    ">" => cmp == Ordering::Greater,
                    "<" => cmp == Ordering::Less,
                    ">=" => cmp != Ordering::Less,
                    "<=" => cmp != Ordering::Greater,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    });

    Ok(())
}

fn apply_sort(table: &mut Table, col_name: &str) -> Result<()> {
    let idx = table.column_index(col_name)?;
    table.rows.sort_by(|a, b| {
        let va = &a[idx];
        let vb = &b[idx];
        // Numeric-aware comparison.
        match (va.parse::<f64>(), vb.parse::<f64>()) {
            (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(Ordering::Equal),
            _ => va.cmp(vb),
        }
    });
    Ok(())
}

fn apply_rename(table: &mut Table, spec: &str) -> Result<()> {
    let (old, new) = spec
        .split_once(':')
        .ok_or_else(|| anyhow!("rename format must be 'old_name:new_name', got '{}'", spec))?;
    let old = old.trim();
    let new = new.trim();
    let idx = table.column_index(old)?;
    table.columns[idx] = new.to_string();
    Ok(())
}

/// Add a computed column.  Supports:
///   - Arithmetic between two columns: `new=col1+col2`, `new=col1-col2`, `new=col1*col2`, `new=col1/col2`
///   - Arithmetic between a column and a literal: `new=col1+10`
///   - Literal assignment: `new=some_value`
fn apply_add(table: &mut Table, spec: &str) -> Result<()> {
    let (new_col, expr) = spec
        .split_once('=')
        .ok_or_else(|| anyhow!("add format must be 'new_col=expr', got '{}'", spec))?;
    let new_col = new_col.trim().to_string();
    let expr = expr.trim();

    // Try to parse as binary arithmetic: operand {+,-,*,/} operand
    let arith_ops = ['+', '-', '*', '/'];
    let parsed = arith_ops.iter().find_map(|&op_char| {
        // Find the operator (skip first char so negative numbers work).
        expr[1..].find(op_char).map(|pos| {
            let pos = pos + 1;
            let lhs = expr[..pos].trim().to_string();
            let rhs = expr[pos + 1..].trim().to_string();
            (lhs, op_char, rhs)
        })
    });

    if let Some((lhs, op_char, rhs)) = parsed {
        let lhs_idx = table.column_index(&lhs).ok();
        let rhs_idx = table.column_index(&rhs).ok();

        let new_values: Vec<String> = table
            .rows
            .iter()
            .map(|row| {
                let lval: f64 = lhs_idx
                    .and_then(|i| row[i].parse().ok())
                    .or_else(|| lhs.parse().ok())
                    .unwrap_or(0.0);
                let rval: f64 = rhs_idx
                    .and_then(|i| row[i].parse().ok())
                    .or_else(|| rhs.parse().ok())
                    .unwrap_or(0.0);
                let result = match op_char {
                    '+' => lval + rval,
                    '-' => lval - rval,
                    '*' => lval * rval,
                    '/' => {
                        if rval == 0.0 {
                            f64::NAN
                        } else {
                            lval / rval
                        }
                    }
                    _ => unreachable!(),
                };
                // Render as integer if it is one.
                if result.fract() == 0.0 && result.is_finite() {
                    format!("{}", result as i64)
                } else {
                    format!("{}", result)
                }
            })
            .collect();

        table.columns.push(new_col);
        for (row, val) in table.rows.iter_mut().zip(new_values) {
            row.push(val);
        }
    } else {
        // Constant or single-column reference.
        let col_idx = table.column_index(expr).ok();
        let new_values: Vec<String> = table
            .rows
            .iter()
            .map(|row| {
                col_idx
                    .map(|i| row[i].clone())
                    .unwrap_or_else(|| expr.to_string())
            })
            .collect();

        table.columns.push(new_col);
        for (row, val) in table.rows.iter_mut().zip(new_values) {
            row.push(val);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Transformation pipeline (command dispatch)
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Transform {
    Filter(String),
    Sort(String),
    Rename(String),
    Add(String),
}

fn apply_transform(table: &mut Table, t: &Transform) -> Result<()> {
    match t {
        Transform::Filter(expr) => apply_filter(table, expr),
        Transform::Sort(col) => apply_sort(table, col),
        Transform::Rename(spec) => apply_rename(table, spec),
        Transform::Add(spec) => apply_add(table, spec),
    }
}

// ---------------------------------------------------------------------------
// Argument parsing (lightweight, no macro-based derive)
// ---------------------------------------------------------------------------

struct Args {
    input: String,
    output: Option<String>,
    transforms: Vec<Transform>,
}

fn parse_args() -> Result<Args> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        std::process::exit(0);
    }

    let input = args[0].clone();
    let mut output: Option<String> = None;
    let mut transforms = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--filter" | "-f" => {
                i += 1;
                let val = args.get(i).ok_or_else(|| anyhow!("--filter requires a value"))?;
                transforms.push(Transform::Filter(val.clone()));
            }
            "--sort" | "-s" => {
                i += 1;
                let val = args.get(i).ok_or_else(|| anyhow!("--sort requires a value"))?;
                transforms.push(Transform::Sort(val.clone()));
            }
            "--rename" | "-r" => {
                i += 1;
                let val = args.get(i).ok_or_else(|| anyhow!("--rename requires a value"))?;
                transforms.push(Transform::Rename(val.clone()));
            }
            "--add" | "-a" => {
                i += 1;
                let val = args.get(i).ok_or_else(|| anyhow!("--add requires a value"))?;
                transforms.push(Transform::Add(val.clone()));
            }
            "--output" | "-o" => {
                i += 1;
                let val = args.get(i).ok_or_else(|| anyhow!("--output requires a value"))?;
                output = Some(val.clone());
            }
            other => {
                bail!("unknown argument: '{}'. Run with --help for usage.", other);
            }
        }
        i += 1;
    }

    Ok(Args {
        input,
        output,
        transforms,
    })
}

fn print_usage() {
    eprintln!(
        r#"dtool - CLI data file processor

USAGE:
    dtool <INPUT> [OPTIONS]

INPUT:
    Path to a CSV, JSON, or YAML file.

OPTIONS:
    -f, --filter <EXPR>       Filter rows. Operators: =, !=, >, <, >=, <=, ~ (contains)
                              Example: --filter 'age>30'
    -s, --sort <COLUMN>       Sort rows by column (numeric-aware).
    -r, --rename <OLD:NEW>    Rename a column.
                              Example: --rename 'old_name:new_name'
    -a, --add <NAME=EXPR>     Add a computed column.
                              Examples: --add 'total=price*quantity'
                                        --add 'label=hello'
    -o, --output <PATH>       Output file (CSV, JSON, or YAML). Defaults to stdout as CSV.
    -h, --help                Show this help message.

EXAMPLES:
    dtool data.csv --filter 'age>30' --sort name --output result.json
    dtool input.yaml --rename 'old:new' --add 'double=value*2' --output out.csv
    dtool records.json --filter 'status=active' --sort created_at"#
    );
}

// ---------------------------------------------------------------------------
// Pretty-print to stdout as a simple ASCII table (fallback when no --output)
// ---------------------------------------------------------------------------

fn print_table_stdout(table: &Table) {
    if table.columns.is_empty() {
        eprintln!("(empty table)");
        return;
    }

    // Compute column widths.
    let mut widths: Vec<usize> = table.columns.iter().map(|c| c.len()).collect();
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if cell.len() > widths[i] {
                widths[i] = cell.len();
            }
        }
    }

    let separator: String = widths.iter().map(|&w| "-".repeat(w + 2)).collect::<Vec<_>>().join("+");
    let fmt_row = |cells: &[String]| -> String {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| format!(" {:width$} ", c, width = widths[i]))
            .collect::<Vec<_>>()
            .join("|")
    };

    println!("{}", fmt_row(&table.columns));
    println!("{}", separator);
    for row in &table.rows {
        println!("{}", fmt_row(row));
    }
    eprintln!("\n({} rows)", table.rows.len());
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn run() -> Result<()> {
    let args = parse_args()?;

    // Read input.
    let mut table = read_table(&args.input)
        .with_context(|| format!("failed to read input file '{}'", args.input))?;

    eprintln!(
        "Loaded {} rows x {} columns from '{}'",
        table.rows.len(),
        table.columns.len(),
        args.input
    );

    // Apply transforms in order.
    for t in &args.transforms {
        apply_transform(&mut table, t)
            .with_context(|| format!("failed to apply transform {:?}", t))?;
        match t {
            Transform::Filter(_) => eprintln!("  after filter: {} rows", table.rows.len()),
            Transform::Sort(col) => eprintln!("  sorted by '{}'", col),
            Transform::Rename(spec) => eprintln!("  renamed {}", spec),
            Transform::Add(spec) => eprintln!("  added column from '{}'", spec),
        }
    }

    // Write output.
    match &args.output {
        Some(path) => {
            write_table(&table, path)
                .with_context(|| format!("failed to write output file '{}'", path))?;
            eprintln!("Wrote {} rows to '{}'", table.rows.len(), path);
        }
        None => {
            print_table_stdout(&table);
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table() -> Table {
        Table {
            columns: vec![
                "name".into(),
                "age".into(),
                "city".into(),
            ],
            rows: vec![
                vec!["Alice".into(), "30".into(), "NYC".into()],
                vec!["Bob".into(), "25".into(), "LA".into()],
                vec!["Charlie".into(), "35".into(), "NYC".into()],
                vec!["Diana".into(), "28".into(), "Chicago".into()],
            ],
        }
    }

    #[test]
    fn test_filter_greater_than() {
        let mut t = sample_table();
        apply_filter(&mut t, "age>28").unwrap();
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0][0], "Alice");
        assert_eq!(t.rows[1][0], "Charlie");
    }

    #[test]
    fn test_filter_equals() {
        let mut t = sample_table();
        apply_filter(&mut t, "city=NYC").unwrap();
        assert_eq!(t.rows.len(), 2);
    }

    #[test]
    fn test_filter_not_equals() {
        let mut t = sample_table();
        apply_filter(&mut t, "city!=NYC").unwrap();
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0][0], "Bob");
        assert_eq!(t.rows[1][0], "Diana");
    }

    #[test]
    fn test_filter_contains() {
        let mut t = sample_table();
        apply_filter(&mut t, "name~li").unwrap();
        assert_eq!(t.rows.len(), 2); // Alice, Charlie
    }

    #[test]
    fn test_sort_by_string() {
        let mut t = sample_table();
        apply_sort(&mut t, "name").unwrap();
        let names: Vec<&str> = t.rows.iter().map(|r| r[0].as_str()).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Charlie", "Diana"]);
    }

    #[test]
    fn test_sort_by_numeric() {
        let mut t = sample_table();
        apply_sort(&mut t, "age").unwrap();
        let ages: Vec<&str> = t.rows.iter().map(|r| r[1].as_str()).collect();
        assert_eq!(ages, vec!["25", "28", "30", "35"]);
    }

    #[test]
    fn test_rename() {
        let mut t = sample_table();
        apply_rename(&mut t, "name:full_name").unwrap();
        assert_eq!(t.columns[0], "full_name");
    }

    #[test]
    fn test_add_computed_column() {
        let mut t = Table {
            columns: vec!["a".into(), "b".into()],
            rows: vec![
                vec!["10".into(), "3".into()],
                vec!["20".into(), "7".into()],
            ],
        };
        apply_add(&mut t, "sum=a+b").unwrap();
        assert_eq!(t.columns.len(), 3);
        assert_eq!(t.columns[2], "sum");
        assert_eq!(t.rows[0][2], "13");
        assert_eq!(t.rows[1][2], "27");
    }

    #[test]
    fn test_add_column_with_literal() {
        let mut t = sample_table();
        apply_add(&mut t, "status=active").unwrap();
        assert_eq!(t.columns.last().unwrap(), "status");
        assert!(t.rows.iter().all(|r| r.last().unwrap() == "active"));
    }

    #[test]
    fn test_add_multiply() {
        let mut t = Table {
            columns: vec!["price".into(), "qty".into()],
            rows: vec![vec!["5".into(), "4".into()]],
        };
        apply_add(&mut t, "total=price*qty").unwrap();
        assert_eq!(t.rows[0][2], "20");
    }

    #[test]
    fn test_filter_invalid_column() {
        let mut t = sample_table();
        assert!(apply_filter(&mut t, "nonexistent>5").is_err());
    }

    #[test]
    fn test_rename_invalid_column() {
        let mut t = sample_table();
        assert!(apply_rename(&mut t, "nonexistent:new").is_err());
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(detect_format("data.csv").unwrap(), Format::Csv);
        assert_eq!(detect_format("data.json").unwrap(), Format::Json);
        assert_eq!(detect_format("data.yaml").unwrap(), Format::Yaml);
        assert_eq!(detect_format("data.yml").unwrap(), Format::Yaml);
        assert!(detect_format("data.txt").is_err());
        assert!(detect_format("noext").is_err());
    }

    #[test]
    fn test_pipeline_filter_then_sort() {
        let mut t = sample_table();
        apply_filter(&mut t, "age>25").unwrap();
        apply_sort(&mut t, "age").unwrap();
        let names: Vec<&str> = t.rows.iter().map(|r| r[0].as_str()).collect();
        assert_eq!(names, vec!["Diana", "Alice", "Charlie"]);
    }

    #[test]
    fn test_csv_roundtrip() {
        let t = sample_table();
        let path = "/tmp/dtool_test_roundtrip.csv";
        write_csv(&t, path).unwrap();
        let t2 = read_csv(path).unwrap();
        assert_eq!(t.columns, t2.columns);
        assert_eq!(t.rows, t2.rows);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_json_roundtrip() {
        let t = sample_table();
        let path = "/tmp/dtool_test_roundtrip.json";
        write_json(&t, path).unwrap();
        let t2 = read_json(path).unwrap();
        assert_eq!(t.columns, t2.columns);
        // JSON reader converts "30" to number then back to string, so values match.
        assert_eq!(t.rows, t2.rows);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_yaml_roundtrip() {
        let t = sample_table();
        let path = "/tmp/dtool_test_roundtrip.yaml";
        write_yaml(&t, path).unwrap();
        let t2 = read_yaml(path).unwrap();
        assert_eq!(t.columns, t2.columns);
        assert_eq!(t.rows, t2.rows);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_filter_less_than_or_equal() {
        let mut t = sample_table();
        apply_filter(&mut t, "age<=28").unwrap();
        assert_eq!(t.rows.len(), 2);
        let names: Vec<&str> = t.rows.iter().map(|r| r[0].as_str()).collect();
        assert_eq!(names, vec!["Bob", "Diana"]);
    }

    #[test]
    fn test_add_division() {
        let mut t = Table {
            columns: vec!["a".into(), "b".into()],
            rows: vec![vec!["10".into(), "4".into()]],
        };
        apply_add(&mut t, "ratio=a/b").unwrap();
        assert_eq!(t.rows[0][2], "2.5");
    }

    #[test]
    fn test_add_column_literal_number() {
        let mut t = Table {
            columns: vec!["a".into()],
            rows: vec![vec!["10".into()], vec!["20".into()]],
        };
        apply_add(&mut t, "bonus=a+100").unwrap();
        assert_eq!(t.rows[0][1], "110");
        assert_eq!(t.rows[1][1], "120");
    }
}
