//! `taskr list` command handler.
//!
//! Fetches tasks from the REST API and prints them using the selected output
//! format. Filtering by status is supported via `--status`.

use crate::output::{print_tasks, OutputFormat};
use crate::types::{Status, Task};
use std::process;

/// CLI arguments specific to the `list` subcommand.
///
/// In a real build these would be derived via `clap::Args`; shown here as a
/// plain struct so the module is self-contained.
#[derive(Debug)]
pub struct ListArgs {
    /// Restrict to tasks with this status (None = all).
    pub status: Option<Status>,
    /// Emit JSON instead of a table.
    pub json: bool,
}

/// Exit codes following the agent-cli-ux skill conventions.
mod exit {
    pub const SUCCESS: i32 = 0;
    pub const API_ERROR: i32 = 4;
}

/// Entry point for `taskr list`. Meant to be called from the top-level
/// command dispatcher after argument parsing.
pub fn run(args: &ListArgs, api_base: &str, token: &str) {
    let format = if args.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    let tasks = match fetch_tasks(api_base, token, args.status) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: failed to fetch tasks: {e}");
            process::exit(exit::API_ERROR);
        }
    };

    if let Err(e) = print_tasks(&tasks, format) {
        eprintln!("error: writing output: {e}");
        process::exit(exit::API_ERROR);
    }

    process::exit(exit::SUCCESS);
}

// ── API client (minimal) ────────────────────────────────────────────

/// Fetch tasks from the REST API, optionally filtered server-side by status.
///
/// Uses `ureq` for a blocking HTTP call. The response is expected to be a JSON
/// array of `Task` objects.
fn fetch_tasks(
    api_base: &str,
    token: &str,
    status: Option<Status>,
) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let mut url = format!("{api_base}/v1/tasks");

    if let Some(s) = status {
        url.push_str(&format!("?status={s}"));
    }

    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/json")
        .call()?;

    let tasks: Vec<Task> = resp.into_json()?;
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_selection_json_flag() {
        let args = ListArgs {
            status: None,
            json: true,
        };
        let fmt = if args.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        };
        assert_eq!(fmt, OutputFormat::Json);
    }

    #[test]
    fn format_selection_default_table() {
        let args = ListArgs {
            status: None,
            json: false,
        };
        let fmt = if args.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        };
        assert_eq!(fmt, OutputFormat::Table);
    }
}
