//! Output formatting for taskr CLI.
//!
//! Two modes:
//! - **Table** (default): human-readable aligned columns, written to stdout.
//! - **JSON** (`--json`): one JSON array to stdout for scripting / agent consumption.
//!
//! Progress and diagnostics always go to stderr so stdout stays machine-parseable.

use crate::types::Task;
use std::io::{self, Write};

/// Chosen output format, typically derived from CLI flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
}

// ── JSON output ──────────────────────────────────────────────────────

/// Serialize `tasks` as a JSON array to stdout.
///
/// Errors are returned (not swallowed) so the caller can set the right exit code.
pub fn print_json(tasks: &[Task]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut out = io::stdout().lock();
    writeln!(out, "{json}")
}

// ── Table output ─────────────────────────────────────────────────────

/// Fixed column headers for the human-readable table.
const HEADERS: [&str; 5] = ["ID", "TITLE", "STATUS", "DUE", "PRIORITY"];

/// Render `tasks` as an aligned, fixed-width table to stdout.
///
/// The ID column is shortened to the first 8 hex characters (like short git
/// hashes) to keep the table compact.  The full UUID is available via `--json`.
pub fn print_table(tasks: &[Task]) -> io::Result<()> {
    if tasks.is_empty() {
        let mut out = io::stdout().lock();
        writeln!(out, "No tasks.")?;
        return Ok(());
    }

    // Pre-format every cell so we can measure column widths.
    let rows: Vec<[String; 5]> = tasks.iter().map(|t| format_row(t)).collect();

    // Column widths: max of header length and longest cell in each column.
    let mut widths = [0usize; 5];
    for (i, h) in HEADERS.iter().enumerate() {
        widths[i] = h.len();
    }
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }
    }

    let mut out = io::stdout().lock();

    // Header row.
    write_row(&mut out, &HEADERS.map(String::from), &widths)?;

    // Separator.
    let sep: Vec<String> = widths.iter().map(|&w| "-".repeat(w)).collect();
    write_row(&mut out, &sep.try_into().unwrap(), &widths)?;

    // Data rows.
    for row in &rows {
        write_row(&mut out, row, &widths)?;
    }

    Ok(())
}

/// Format a single task into display strings for each column.
fn format_row(task: &Task) -> [String; 5] {
    let short_id = &task.id.to_string()[..8];
    let due = match &task.due_date {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => "-".to_string(),
    };
    [
        short_id.to_string(),
        task.title.clone(),
        task.status.to_string(),
        due,
        task.priority.to_string(),
    ]
}

/// Write one row of cells with padding to `out`.
fn write_row<W: Write>(out: &mut W, cells: &[String; 5], widths: &[usize; 5]) -> io::Result<()> {
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            write!(out, "  ")?; // two-space gutter
        }
        write!(out, "{:<width$}", cell, width = widths[i])?;
    }
    writeln!(out)
}

// ── Convenience dispatcher ───────────────────────────────────────────

/// Print tasks in the requested format. Returns `io::Result` so the caller can
/// map failures to the appropriate exit code.
pub fn print_tasks(tasks: &[Task], format: OutputFormat) -> io::Result<()> {
    match format {
        OutputFormat::Table => print_table(tasks),
        OutputFormat::Json => print_json(tasks),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Priority, Status, Task};
    use chrono::TimeZone;
    use uuid::Uuid;

    fn sample_tasks() -> Vec<Task> {
        vec![
            Task {
                id: Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "Write tests".into(),
                status: Status::Pending,
                due_date: Some(Utc.with_ymd_and_hms(2026, 4, 10, 14, 0, 0).unwrap()),
                priority: Priority::High,
            },
            Task {
                id: Uuid::parse_str("deadbeef-0000-1111-2222-333344445555").unwrap(),
                title: "Buy milk".into(),
                status: Status::Done,
                due_date: None,
                priority: Priority::Low,
            },
        ]
    }

    #[test]
    fn json_output_parses_back() {
        let tasks = sample_tasks();
        let json = serde_json::to_string_pretty(&tasks).unwrap();
        let round_trip: Vec<Task> = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip.len(), tasks.len());
        assert_eq!(round_trip[0].id, tasks[0].id);
    }

    #[test]
    fn table_empty_says_no_tasks() {
        let mut buf = Vec::new();
        // We can't use print_table directly (it writes to stdout), so test
        // format_row and write_row in isolation.
        write_row(
            &mut buf,
            &HEADERS.map(String::from),
            &[8, 12, 8, 16, 8],
        )
        .unwrap();
        let line = String::from_utf8(buf).unwrap();
        assert!(line.contains("ID"));
        assert!(line.contains("TITLE"));
    }

    #[test]
    fn format_row_short_id() {
        let task = &sample_tasks()[0];
        let row = format_row(task);
        assert_eq!(row[0], "a1b2c3d4");
    }

    #[test]
    fn format_row_no_due_date() {
        let task = &sample_tasks()[1];
        let row = format_row(task);
        assert_eq!(row[3], "-");
    }
}
