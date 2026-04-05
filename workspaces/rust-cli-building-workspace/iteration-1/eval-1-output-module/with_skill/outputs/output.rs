use crate::types::Task;
use chrono::{DateTime, Utc};

// ---------------------------------------------------------------------------
// Core table printer (no external crate needed)
// ---------------------------------------------------------------------------

/// Print a borderless, column-aligned table to stdout.
///
/// Columns are separated by two spaces. The last column is not padded
/// (avoids trailing whitespace). Empty `rows` prints "(none)".
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("(none)");
        return;
    }
    let n = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(n) {
            widths[i] = widths[i].max(cell.len());
        }
    }

    let fmt_row = |cells: &[&str]| -> String {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| {
                if i == n - 1 {
                    // Last column: no trailing padding.
                    c.to_string()
                } else {
                    format!("{:<w$}", c, w = widths[i])
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    };

    let header_refs: Vec<&str> = headers.iter().copied().collect();
    println!("{}", fmt_row(&header_refs));
    for row in rows {
        let refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        println!("{}", fmt_row(&refs));
    }
}

// ---------------------------------------------------------------------------
// JSON helpers
// ---------------------------------------------------------------------------

/// Print a value as compact single-line JSON to stdout.
pub fn print_json<T: serde::Serialize>(value: &T) -> Result<(), std::io::Error> {
    let out = serde_json::to_string(value).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;
    println!("{out}");
    Ok(())
}

/// Print a slice as JSON Lines (one object per line) to stdout.
pub fn print_json_lines<T: serde::Serialize>(items: &[T]) -> Result<(), std::io::Error> {
    for item in items {
        print_json(item)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Hints (stderr, suppressed in --json mode)
// ---------------------------------------------------------------------------

/// Print next-action hints to stderr. Pass an empty slice to skip.
pub fn hint(items: &[&str]) {
    if items.is_empty() {
        return;
    }
    eprintln!();
    for (i, item) in items.iter().enumerate() {
        if i == 0 {
            eprintln!("hint: {item}");
        } else {
            eprintln!("      {item}");
        }
    }
}

// ---------------------------------------------------------------------------
// Date formatting
// ---------------------------------------------------------------------------

/// Format a `DateTime<Utc>` for human display (e.g. "Apr 05, 2026 14:30").
fn fmt_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%b %d, %Y %H:%M").to_string()
}

/// Format an optional date, returning "-" when absent.
fn fmt_optional_date(dt: &Option<DateTime<Utc>>) -> String {
    match dt {
        Some(d) => fmt_datetime(d),
        None => "-".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Task-specific renderers
// ---------------------------------------------------------------------------

/// Print a list of tasks as a human-readable table (default) or JSON Lines.
///
/// Columns: ID (short), TITLE, STATUS, PRIORITY, DUE
///
/// In `--json` mode, emits one JSON object per line (JSON Lines).
pub fn print_tasks(json: bool, tasks: &[Task]) -> Result<(), std::io::Error> {
    if json {
        return print_json_lines(tasks);
    }

    let rows: Vec<Vec<String>> = tasks
        .iter()
        .map(|t| {
            vec![
                short_id(&t.id),
                t.title.clone(),
                t.status.to_string(),
                t.priority.to_string(),
                fmt_optional_date(&t.due_date),
            ]
        })
        .collect();

    print_table(&["ID", "TITLE", "STATUS", "PRIORITY", "DUE"], &rows);

    if !tasks.is_empty() {
        hint(&[
            "taskr tasks get <ID>  -- show task details",
            "taskr tasks create    -- create a new task",
        ]);
    } else {
        hint(&[
            "taskr tasks create  -- create your first task",
        ]);
    }

    Ok(())
}

/// Print a single task as key-value pairs (human) or JSON (agent).
pub fn print_task_detail(json: bool, task: &Task) -> Result<(), std::io::Error> {
    if json {
        return print_json(task);
    }

    let pairs: Vec<(&str, String)> = vec![
        ("ID:", task.id.to_string()),
        ("Title:", task.title.clone()),
        ("Status:", task.status.to_string()),
        ("Priority:", task.priority.to_string()),
        ("Due:", fmt_optional_date(&task.due_date)),
    ];

    let max_label = pairs.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    for (label, value) in &pairs {
        println!("{:<w$}  {}", label, value, w = max_label);
    }

    hint(&[
        "taskr tasks update <ID> --status done  -- mark complete",
        "taskr tasks delete <ID>                 -- remove task",
        "taskr tasks list                        -- back to list",
    ]);

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the first 8 hex chars of a UUID for compact display.
fn short_id(id: &uuid::Uuid) -> String {
    id.to_string()[..8].to_string()
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
                title: "Write output module".to_string(),
                status: Status::Pending,
                due_date: Some(Utc.with_ymd_and_hms(2026, 4, 10, 14, 0, 0).unwrap()),
                priority: Priority::High,
            },
            Task {
                id: Uuid::parse_str("b2c3d4e5-f6a7-8901-bcde-f12345678901").unwrap(),
                title: "Add tests".to_string(),
                status: Status::Done,
                due_date: None,
                priority: Priority::Low,
            },
        ]
    }

    #[test]
    fn table_empty_shows_none() {
        // Capture by redirecting -- just ensure no panic.
        print_table(&["A", "B"], &[]);
    }

    #[test]
    fn short_id_returns_eight_chars() {
        let id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        assert_eq!(short_id(&id), "a1b2c3d4");
    }

    #[test]
    fn fmt_optional_date_none() {
        assert_eq!(fmt_optional_date(&None), "-");
    }

    #[test]
    fn fmt_optional_date_some() {
        let dt = Utc.with_ymd_and_hms(2026, 4, 5, 9, 30, 0).unwrap();
        assert_eq!(fmt_optional_date(&Some(dt)), "Apr 05, 2026 09:30");
    }

    #[test]
    fn print_tasks_json_does_not_panic() {
        let tasks = sample_tasks();
        // Just verify it doesn't panic; real tests would capture stdout.
        print_tasks(true, &tasks).unwrap();
    }

    #[test]
    fn print_tasks_table_does_not_panic() {
        let tasks = sample_tasks();
        print_tasks(false, &tasks).unwrap();
    }
}
