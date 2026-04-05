use crate::error::{CliError, ResolveError};
use crate::types::{Project, Workspace};

// ---------------------------------------------------------------------------
// Table printer (no external crate needed)
// ---------------------------------------------------------------------------

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
            .map(|(i, c)| format!("{:<w$}", c, w = widths[i]))
            .collect::<Vec<_>>()
            .join("  ")
    };
    println!(
        "{}",
        fmt_row(&headers.iter().copied().collect::<Vec<_>>())
    );
    for row in rows {
        let refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        println!("{}", fmt_row(&refs));
    }
}

// ---------------------------------------------------------------------------
// Workspace table
// ---------------------------------------------------------------------------

pub fn print_workspaces(json: bool, workspaces: &[Workspace]) -> Result<(), CliError> {
    if json {
        for ws in workspaces {
            println!("{}", serde_json::to_string(ws).unwrap());
        }
        return Ok(());
    }
    let rows: Vec<Vec<String>> = workspaces
        .iter()
        .map(|w| {
            vec![
                w.name.clone(),
                w.owner_email.clone().unwrap_or_default(),
                w.id.clone(),
            ]
        })
        .collect();
    print_table(&["NAME", "OWNER", "ID"], &rows);
    Ok(())
}

// ---------------------------------------------------------------------------
// Hints (stderr, suppressed in --json mode)
// ---------------------------------------------------------------------------

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
// Error display with copy-pasteable suggestions
// ---------------------------------------------------------------------------

/// Format a resolve error into a human-readable message with actionable
/// suggestions. Returns the formatted string (caller prints to stderr).
pub fn format_resolve_error(err: &ResolveError) -> String {
    match err {
        ResolveError::NoWorkspaces => {
            "error: no workspaces found for your account\n\n\
             hint: check that your account has access to at least one workspace"
                .to_string()
        }

        ResolveError::AmbiguousWorkspace { candidates } => {
            let mut msg =
                "error: multiple workspaces found; specify --workspace <NAME-OR-ID>:\n"
                    .to_string();
            for ws in candidates {
                let label = ws
                    .owner_email
                    .as_deref()
                    .map(|e| format!(" ({})", e))
                    .unwrap_or_default();
                msg.push_str(&format!(
                    "  --workspace {:?}  # {}{}\n",
                    ws.name, ws.id, label
                ));
            }
            msg.push_str(
                "\nor set defaults.workspace in ~/.config/pmtool/config.toml",
            );
            msg
        }

        ResolveError::WorkspaceNotFound { query, candidates } => {
            let mut msg = format!(
                "error: no workspace matching {:?}; available:\n",
                query
            );
            for ws in candidates {
                msg.push_str(&format!("  {:?}  ({})\n", ws.name, ws.id));
            }
            msg.push_str(
                "\nhint: use the exact name or ID from the list above",
            );
            msg
        }

        ResolveError::NoProjects { workspace_name } => {
            format!(
                "error: no projects found in workspace {:?}\n\n\
                 hint: create a project first, or check --workspace",
                workspace_name
            )
        }

        ResolveError::AmbiguousProject {
            workspace_name,
            candidates,
        } => {
            let mut msg = format!(
                "error: multiple projects in workspace {:?}; specify --project <NAME-OR-ID>:\n",
                workspace_name
            );
            for proj in candidates {
                msg.push_str(&format!("  --project {:?}  # {}\n", proj.name, proj.id));
            }
            msg.push_str(
                "\nor set defaults.project in ~/.config/pmtool/config.toml",
            );
            msg
        }

        ResolveError::ProjectNotFound {
            query,
            workspace_name,
            candidates,
        } => {
            let mut msg = format!(
                "error: no project matching {:?} in workspace {:?}; available:\n",
                query, workspace_name
            );
            for proj in candidates {
                msg.push_str(&format!("  {:?}  ({})\n", proj.name, proj.id));
            }
            msg
        }

        ResolveError::Api(api_err) => {
            format!("error: {api_err}")
        }
    }
}

/// Format a resolve error as JSON for --json mode (written to stderr).
pub fn format_resolve_error_json(err: &ResolveError, exit_code: i32) -> String {
    let (error_text, hint_text) = match err {
        ResolveError::AmbiguousWorkspace { candidates } => {
            let names: Vec<&str> = candidates.iter().map(|w| w.name.as_str()).collect();
            (
                format!("{err}"),
                Some(format!(
                    "pass --workspace with one of: {}",
                    names.join(", ")
                )),
            )
        }
        ResolveError::WorkspaceNotFound { .. } => {
            (format!("{err}"), Some("check workspace name or ID".into()))
        }
        _ => (format!("{err}"), None),
    };

    let mut obj = serde_json::json!({
        "error": error_text,
        "code": exit_code,
    });
    if let Some(h) = hint_text {
        obj["hint"] = serde_json::Value::String(h);
    }
    serde_json::to_string(&obj).unwrap()
}
