// dtool - A CLI data processing pipeline tool
//
// Supports CSV, JSON, and YAML input/output with transformations:
//   --filter 'column>value'   Filter rows by column comparison
//   --sort column             Sort rows by column
//   --rename 'old:new'        Rename a column
//   --add 'new_col=col1+col2' Add a computed column
//   --output file.ext         Write result (format inferred from extension)
//
// Usage:
//   dtool input.csv --filter 'age>30' --sort name --rename 'old_name:new_name' --output result.json
//
// Cargo.toml dependencies:
// [dependencies]
// clap = { version = "4", features = ["derive"] }
// csv = "1"
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// serde_yaml = "0.9"
// anyhow = "1"
// regex = "1"

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

/// A CLI tool for reading, transforming, and writing tabular data files.
#[derive(Parser, Debug)]
#[command(name = "dtool", version = "1.0.0", about = "Process data files with a pipeline of transformations")]
struct Cli {
    /// Input file path (CSV, JSON, or YAML)
    input: String,

    /// Filter rows: 'column>value', 'column<value', 'column>=value',
    /// 'column<=value', 'column==value', 'column!=value'
    #[arg(long, num_args = 1..)]
    filter: Vec<String>,

    /// Sort rows by the given column name (ascending)
    #[arg(long)]
    sort: Option<String>,

    /// Rename a column: 'old_name:new_name' (repeatable)
    #[arg(long, num_args = 1..)]
    rename: Vec<String>,

    /// Add a computed column: 'new_col=col_a+col_b' or 'new_col=col_a-col_b'
    /// (supports +, -, *, / on numeric columns)
    #[arg(long, num_args = 1..)]
    add: Vec<String>,

    /// Output file path (format inferred from extension). Defaults to stdout as CSV.
    #[arg(long, short)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// Core data model
// ---------------------------------------------------------------------------

/// A simple in-memory table: ordered column names + rows of string values.
#[derive(Debug, Clone)]
struct Table {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn col_index(&self, name: &str) -> Result<usize> {
        self.columns
            .iter()
            .position(|c| c == name)
            .ok_or_else(|| anyhow!("Column '{}' not found. Available: {:?}", name, self.columns))
    }

    /// Return a single row as a HashMap for convenient lookup.
    fn row_map(&self, row_idx: usize) -> HashMap<&str, &str> {
        self.columns
            .iter()
            .zip(self.rows[row_idx].iter())
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
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
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "csv" | "tsv" => Ok(Format::Csv),
        "json" => Ok(Format::Json),
        "yaml" | "yml" => Ok(Format::Yaml),
        other => bail!("Unsupported file extension '.{other}'. Use .csv, .json, or .yaml"),
    }
}

// ---------------------------------------------------------------------------
// Readers
// ---------------------------------------------------------------------------

fn read_csv(path: &str) -> Result<Table> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("Failed to open CSV file: {path}"))?;

    let columns: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();
    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();
        rows.push(row);
    }
    Ok(Table { columns, rows })
}

fn read_json(path: &str) -> Result<Table> {
    let data = fs::read_to_string(path).with_context(|| format!("Failed to read JSON file: {path}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&data).with_context(|| "Failed to parse JSON")?;

    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("JSON root must be an array of objects"))?;

    if arr.is_empty() {
        return Ok(Table {
            columns: vec![],
            rows: vec![],
        });
    }

    // Collect all unique keys preserving first-seen order.
    let mut columns: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in arr {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                if seen.insert(key.clone()) {
                    columns.push(key.clone());
                }
            }
        }
    }

    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|item| {
            let obj = item.as_object();
            columns
                .iter()
                .map(|col| {
                    obj.and_then(|o| o.get(col))
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Null => String::new(),
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
    let data = fs::read_to_string(path).with_context(|| format!("Failed to read YAML file: {path}"))?;
    let value: serde_yaml::Value =
        serde_yaml::from_str(&data).with_context(|| "Failed to parse YAML")?;

    let seq = value
        .as_sequence()
        .ok_or_else(|| anyhow!("YAML root must be a sequence of mappings"))?;

    if seq.is_empty() {
        return Ok(Table {
            columns: vec![],
            rows: vec![],
        });
    }

    let mut columns: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in seq {
        if let Some(map) = item.as_mapping() {
            for key in map.keys() {
                let k = yaml_value_to_string(key);
                if seen.insert(k.clone()) {
                    columns.push(k);
                }
            }
        }
    }

    let rows: Vec<Vec<String>> = seq
        .iter()
        .map(|item| {
            let map = item.as_mapping();
            columns
                .iter()
                .map(|col| {
                    map.and_then(|m| {
                        m.get(&serde_yaml::Value::String(col.clone()))
                    })
                    .map(yaml_value_to_string)
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
        other => format!("{other:?}"),
    }
}

fn read_input(path: &str) -> Result<Table> {
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
    let mut wtr = csv::Writer::from_path(path)
        .with_context(|| format!("Failed to create CSV file: {path}"))?;
    wtr.write_record(&table.columns)?;
    for row in &table.rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
}

fn write_json(table: &Table, path: &str) -> Result<()> {
    let records: Vec<serde_json::Value> = table
        .rows
        .iter()
        .map(|row| {
            let obj: serde_json::Map<String, serde_json::Value> = table
                .columns
                .iter()
                .zip(row.iter())
                .map(|(col, val)| {
                    // Try to preserve numeric types in JSON output.
                    let json_val = if let Ok(n) = val.parse::<i64>() {
                        serde_json::Value::Number(n.into())
                    } else if let Ok(f) = val.parse::<f64>() {
                        serde_json::json!(f)
                    } else if val == "true" || val == "false" {
                        serde_json::Value::Bool(val == "true")
                    } else {
                        serde_json::Value::String(val.clone())
                    };
                    (col.clone(), json_val)
                })
                .collect();
            serde_json::Value::Object(obj)
        })
        .collect();

    let json = serde_json::to_string_pretty(&records)?;
    fs::write(path, json).with_context(|| format!("Failed to write JSON file: {path}"))?;
    Ok(())
}

fn write_yaml(table: &Table, path: &str) -> Result<()> {
    let records: Vec<serde_yaml::Value> = table
        .rows
        .iter()
        .map(|row| {
            let mapping: serde_yaml::Mapping = table
                .columns
                .iter()
                .zip(row.iter())
                .map(|(col, val)| {
                    let yaml_val = if let Ok(n) = val.parse::<i64>() {
                        serde_yaml::Value::Number(serde_yaml::Number::from(n))
                    } else if let Ok(f) = val.parse::<f64>() {
                        serde_yaml::Value::Number(
                            serde_yaml::Number::from(f),
                        )
                    } else if val == "true" || val == "false" {
                        serde_yaml::Value::Bool(val == "true")
                    } else {
                        serde_yaml::Value::String(val.clone())
                    };
                    (serde_yaml::Value::String(col.clone()), yaml_val)
                })
                .collect();
            serde_yaml::Value::Mapping(mapping)
        })
        .collect();

    let yaml = serde_yaml::to_string(&records)?;
    fs::write(path, yaml).with_context(|| format!("Failed to write YAML file: {path}"))?;
    Ok(())
}

fn write_csv_stdout(table: &Table) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(&table.columns)?;
    for row in &table.rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
}

fn write_output(table: &Table, path: Option<&str>) -> Result<()> {
    match path {
        None => write_csv_stdout(table),
        Some(p) => {
            let fmt = detect_format(p)?;
            match fmt {
                Format::Csv => write_csv(table, p),
                Format::Json => write_json(table, p),
                Format::Yaml => write_yaml(table, p),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Transformations
// ---------------------------------------------------------------------------

/// Parse a filter expression like "age>30", "name==Alice", "score<=100".
fn apply_filter(table: &mut Table, expr: &str) -> Result<()> {
    let re = Regex::new(r"^(\w+)\s*(>=|<=|!=|==|>|<)\s*(.+)$")?;
    let caps = re
        .captures(expr)
        .ok_or_else(|| anyhow!("Invalid filter expression: '{expr}'. Expected format: column>value"))?;

    let col_name = &caps[1];
    let operator = &caps[2];
    let target = caps[3].trim().to_string();

    let col_idx = table.col_index(col_name)?;

    table.rows.retain(|row| {
        let cell = &row[col_idx];
        // Try numeric comparison first, fall back to string comparison.
        let numeric_cmp = cell.parse::<f64>().ok().and_then(|cell_num| {
            target.parse::<f64>().ok().map(|target_num| {
                cell_num.partial_cmp(&target_num).unwrap_or(Ordering::Equal)
            })
        });

        let cmp = numeric_cmp.unwrap_or_else(|| cell.cmp(&target).into());

        match operator {
            ">" => cmp == Ordering::Greater,
            "<" => cmp == Ordering::Less,
            ">=" => cmp == Ordering::Greater || cmp == Ordering::Equal,
            "<=" => cmp == Ordering::Less || cmp == Ordering::Equal,
            "==" => cmp == Ordering::Equal,
            "!=" => cmp != Ordering::Equal,
            _ => true,
        }
    });

    Ok(())
}

fn apply_sort(table: &mut Table, col_name: &str) -> Result<()> {
    let col_idx = table.col_index(col_name)?;

    table.rows.sort_by(|a, b| {
        let va = &a[col_idx];
        let vb = &b[col_idx];
        // Numeric-aware sort: if both parse as f64, compare numerically.
        match (va.parse::<f64>(), vb.parse::<f64>()) {
            (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(Ordering::Equal),
            _ => va.cmp(vb),
        }
    });

    Ok(())
}

fn apply_rename(table: &mut Table, spec: &str) -> Result<()> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() != 2 {
        bail!("Invalid rename spec: '{spec}'. Expected 'old_name:new_name'");
    }
    let old_name = parts[0].trim();
    let new_name = parts[1].trim();

    let idx = table.col_index(old_name)?;
    table.columns[idx] = new_name.to_string();
    Ok(())
}

/// Add a computed column. Supports expressions like:
///   new_col=col_a+col_b
///   new_col=col_a-col_b
///   new_col=col_a*col_b
///   new_col=col_a/col_b
///   new_col=literal_value  (constant fill)
fn apply_add(table: &mut Table, spec: &str) -> Result<()> {
    let parts: Vec<&str> = spec.splitn(2, '=').collect();
    if parts.len() != 2 {
        bail!("Invalid add spec: '{spec}'. Expected 'new_col=expression'");
    }
    let new_col = parts[0].trim().to_string();
    let expression = parts[1].trim();

    // Try to parse as binary operation: colA {op} colB
    let op_re = Regex::new(r"^(\w+)\s*([+\-*/])\s*(\w+)$")?;

    if let Some(caps) = op_re.captures(expression) {
        let left_name = &caps[1];
        let op = &caps[2];
        let right_name = &caps[3];

        let left_idx = table.col_index(left_name)?;
        let right_idx = table.col_index(right_name)?;

        let new_values: Vec<String> = table
            .rows
            .iter()
            .map(|row| {
                let lv = row[left_idx].parse::<f64>().unwrap_or(0.0);
                let rv = row[right_idx].parse::<f64>().unwrap_or(0.0);
                let result = match op {
                    "+" => lv + rv,
                    "-" => lv - rv,
                    "*" => lv * rv,
                    "/" => {
                        if rv == 0.0 {
                            f64::NAN
                        } else {
                            lv / rv
                        }
                    }
                    _ => f64::NAN,
                };
                // Format nicely: drop trailing .0 for whole numbers
                if result == result.floor() && result.is_finite() {
                    format!("{}", result as i64)
                } else {
                    format!("{result}")
                }
            })
            .collect();

        table.columns.push(new_col);
        for (i, row) in table.rows.iter_mut().enumerate() {
            row.push(new_values[i].clone());
        }
    } else {
        // Constant fill — the expression is used as a literal value for every row.
        table.columns.push(new_col);
        let val = expression.to_string();
        for row in table.rows.iter_mut() {
            row.push(val.clone());
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Read input
    let mut table = read_input(&cli.input)
        .with_context(|| format!("Error reading input file '{}'", cli.input))?;

    eprintln!(
        "Loaded {} rows x {} columns from '{}'",
        table.rows.len(),
        table.columns.len(),
        cli.input
    );

    // 2. Apply transformations in the order they are specified.
    //    clap does not easily preserve cross-argument ordering, so we apply
    //    them in a fixed logical order: filter -> add -> rename -> sort.
    //    This matches typical data-pipeline semantics.

    // Filters
    for expr in &cli.filter {
        apply_filter(&mut table, expr)
            .with_context(|| format!("Error applying filter '{expr}'"))?;
        eprintln!("After filter '{}': {} rows", expr, table.rows.len());
    }

    // Add computed columns (before rename so source columns still have original names)
    for spec in &cli.add {
        apply_add(&mut table, spec)
            .with_context(|| format!("Error adding computed column '{spec}'"))?;
        eprintln!("Added column from '{}'", spec);
    }

    // Rename columns
    for spec in &cli.rename {
        apply_rename(&mut table, spec)
            .with_context(|| format!("Error renaming column '{spec}'"))?;
        eprintln!("Renamed column: {}", spec);
    }

    // Sort
    if let Some(ref col) = cli.sort {
        apply_sort(&mut table, col)
            .with_context(|| format!("Error sorting by column '{col}'"))?;
        eprintln!("Sorted by '{}'", col);
    }

    // 3. Write output
    write_output(&table, cli.output.as_deref())?;

    if let Some(ref out) = cli.output {
        eprintln!("Wrote {} rows to '{}'", table.rows.len(), out);
    }

    Ok(())
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
                "score".into(),
            ],
            rows: vec![
                vec!["Alice".into(), "30".into(), "85".into()],
                vec!["Bob".into(), "25".into(), "92".into()],
                vec!["Charlie".into(), "35".into(), "78".into()],
                vec!["Diana".into(), "28".into(), "95".into()],
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
        apply_filter(&mut t, "name==Bob").unwrap();
        assert_eq!(t.rows.len(), 1);
        assert_eq!(t.rows[0][1], "25");
    }

    #[test]
    fn test_filter_not_equals() {
        let mut t = sample_table();
        apply_filter(&mut t, "name!=Alice").unwrap();
        assert_eq!(t.rows.len(), 3);
    }

    #[test]
    fn test_sort_by_name() {
        let mut t = sample_table();
        apply_sort(&mut t, "name").unwrap();
        let names: Vec<&str> = t.rows.iter().map(|r| r[0].as_str()).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Charlie", "Diana"]);
    }

    #[test]
    fn test_sort_by_age_numeric() {
        let mut t = sample_table();
        apply_sort(&mut t, "age").unwrap();
        let ages: Vec<&str> = t.rows.iter().map(|r| r[1].as_str()).collect();
        assert_eq!(ages, vec!["25", "28", "30", "35"]);
    }

    #[test]
    fn test_rename_column() {
        let mut t = sample_table();
        apply_rename(&mut t, "name:full_name").unwrap();
        assert_eq!(t.columns[0], "full_name");
    }

    #[test]
    fn test_rename_invalid_spec() {
        let mut t = sample_table();
        assert!(apply_rename(&mut t, "no_colon").is_err());
    }

    #[test]
    fn test_add_computed_column() {
        let mut t = sample_table();
        apply_add(&mut t, "total=age+score").unwrap();
        assert_eq!(t.columns.len(), 4);
        assert_eq!(t.columns[3], "total");
        // Alice: 30 + 85 = 115
        assert_eq!(t.rows[0][3], "115");
    }

    #[test]
    fn test_add_constant_column() {
        let mut t = sample_table();
        apply_add(&mut t, "status=active").unwrap();
        assert_eq!(t.columns.last().unwrap(), "status");
        for row in &t.rows {
            assert_eq!(row.last().unwrap(), "active");
        }
    }

    #[test]
    fn test_col_index_missing() {
        let t = sample_table();
        assert!(t.col_index("nonexistent").is_err());
    }

    #[test]
    fn test_filter_less_than_or_equal() {
        let mut t = sample_table();
        apply_filter(&mut t, "score<=85").unwrap();
        assert_eq!(t.rows.len(), 2); // Alice(85), Charlie(78)
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(detect_format("data.csv").unwrap(), Format::Csv);
        assert_eq!(detect_format("data.json").unwrap(), Format::Json);
        assert_eq!(detect_format("data.yaml").unwrap(), Format::Yaml);
        assert_eq!(detect_format("data.yml").unwrap(), Format::Yaml);
        assert!(detect_format("data.txt").is_err());
    }
}
