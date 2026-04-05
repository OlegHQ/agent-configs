use clap::Subcommand;

use crate::output;
use crate::types::{Priority, Status, Task};

/// Manage tasks.
#[derive(Debug, Subcommand)]
pub enum TasksCmd {
    /// List tasks, optionally filtered by status or priority.
    List {
        /// Filter by status (pending, done, archived).
        #[arg(long)]
        status: Option<Status>,

        /// Filter by priority (low, medium, high).
        #[arg(long)]
        priority: Option<Priority>,

        /// Maximum number of tasks to return.
        #[arg(long, default_value_t = 25)]
        limit: usize,

        /// Include archived tasks (excluded by default).
        #[arg(long)]
        all: bool,
    },

    /// Show details for a single task.
    Get {
        /// Task ID (UUID or short prefix).
        task_id: String,
    },

    /// Create a new task.
    Create {
        /// Task title.
        #[arg(long)]
        title: String,

        /// Priority (low, medium, high). Defaults to medium.
        #[arg(long, default_value = "medium")]
        priority: Priority,

        /// Due date in RFC 3339 format (e.g. 2026-04-10T14:00:00Z).
        #[arg(long)]
        due: Option<String>,
    },

    /// Update an existing task.
    Update {
        /// Task ID.
        task_id: String,

        /// New title.
        #[arg(long)]
        title: Option<String>,

        /// New status.
        #[arg(long)]
        status: Option<Status>,

        /// New priority.
        #[arg(long)]
        priority: Option<Priority>,

        /// New due date (RFC 3339), or "none" to clear.
        #[arg(long)]
        due: Option<String>,
    },

    /// Delete a task.
    Delete {
        /// Task ID.
        task_id: String,
    },
}

/// Execute the `tasks list` subcommand.
///
/// Fetches tasks from the API, applies client-side filters, and renders
/// output as a table (default) or JSON Lines (`--json`).
pub async fn handle_list(
    client: &reqwest::Client,
    base_url: &str,
    json: bool,
    status_filter: Option<Status>,
    priority_filter: Option<Priority>,
    limit: usize,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build the request URL with query parameters.
    let mut url = format!("{base_url}/tasks?limit={limit}");
    if let Some(s) = &status_filter {
        url.push_str(&format!("&status={s}"));
    }
    if let Some(p) = &priority_filter {
        url.push_str(&format!("&priority={p}"));
    }
    if all {
        url.push_str("&include_archived=true");
    }

    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if json {
            let err = serde_json::json!({
                "error": format!("API returned {status}"),
                "code": 4,
                "hint": "check `taskr tasks list --help` for valid filters"
            });
            eprintln!("{}", serde_json::to_string(&err)?);
        } else {
            eprintln!("error: API returned {status}");
            if !body.is_empty() {
                eprintln!("  {body}");
            }
        }
        std::process::exit(4);
    }

    let tasks: Vec<Task> = resp.json().await?;

    // Client-side filter for archived tasks when not using --all.
    // The API may or may not support this, so we double-check.
    let tasks: Vec<Task> = if all {
        tasks
    } else {
        tasks.into_iter().filter(|t| t.status != Status::Archived).collect()
    };

    output::print_tasks(json, &tasks)?;
    Ok(())
}

/// Dispatch a `TasksCmd` variant to the appropriate handler.
pub async fn dispatch(
    cmd: TasksCmd,
    client: &reqwest::Client,
    base_url: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        TasksCmd::List {
            status,
            priority,
            limit,
            all,
        } => {
            handle_list(client, base_url, json, status, priority, limit, all).await
        }

        TasksCmd::Get { task_id } => {
            let url = format!("{base_url}/tasks/{task_id}");
            let resp = client.get(&url).send().await?;
            if !resp.status().is_success() {
                let status = resp.status();
                if json {
                    let err = serde_json::json!({
                        "error": format!("API returned {status}"),
                        "code": 4
                    });
                    eprintln!("{}", serde_json::to_string(&err)?);
                } else {
                    eprintln!("error: task {task_id:?} not found ({status})");
                }
                std::process::exit(4);
            }
            let task: Task = resp.json().await?;
            output::print_task_detail(json, &task)?;
            Ok(())
        }

        TasksCmd::Create { title, priority, due } => {
            let due_date: Option<chrono::DateTime<chrono::Utc>> = match due {
                Some(ref s) => Some(s.parse().map_err(|e| {
                    format!("invalid --due date (expected RFC 3339): {e}")
                })?),
                None => None,
            };
            let body = serde_json::json!({
                "title": title,
                "priority": priority,
                "due_date": due_date,
            });
            let resp = client
                .post(&format!("{base_url}/tasks"))
                .json(&body)
                .send()
                .await?;
            if !resp.status().is_success() {
                let status = resp.status();
                eprintln!("error: failed to create task ({status})");
                std::process::exit(4);
            }
            let task: Task = resp.json().await?;
            output::print_task_detail(json, &task)?;
            Ok(())
        }

        TasksCmd::Update {
            task_id,
            title,
            status,
            priority,
            due,
        } => {
            let mut body = serde_json::Map::new();
            if let Some(t) = title {
                body.insert("title".into(), serde_json::Value::String(t));
            }
            if let Some(s) = status {
                body.insert("status".into(), serde_json::to_value(s)?);
            }
            if let Some(p) = priority {
                body.insert("priority".into(), serde_json::to_value(p)?);
            }
            if let Some(ref d) = due {
                if d == "none" {
                    body.insert("due_date".into(), serde_json::Value::Null);
                } else {
                    let dt: chrono::DateTime<chrono::Utc> = d.parse().map_err(|e| {
                        format!("invalid --due date (expected RFC 3339 or \"none\"): {e}")
                    })?;
                    body.insert("due_date".into(), serde_json::to_value(dt)?);
                }
            }
            let resp = client
                .patch(&format!("{base_url}/tasks/{task_id}"))
                .json(&body)
                .send()
                .await?;
            if !resp.status().is_success() {
                let status_code = resp.status();
                eprintln!("error: failed to update task ({status_code})");
                std::process::exit(4);
            }
            let task: Task = resp.json().await?;
            output::print_task_detail(json, &task)?;
            Ok(())
        }

        TasksCmd::Delete { task_id } => {
            let resp = client
                .delete(&format!("{base_url}/tasks/{task_id}"))
                .send()
                .await?;
            if !resp.status().is_success() {
                let status = resp.status();
                eprintln!("error: failed to delete task ({status})");
                std::process::exit(4);
            }
            if json {
                println!(r#"{{"deleted":"{}"}}"#, task_id);
            } else {
                println!("Deleted task {task_id}");
                output::hint(&["taskr tasks list  -- view remaining tasks"]);
            }
            Ok(())
        }
    }
}
