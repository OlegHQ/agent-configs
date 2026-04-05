//! Auto-resolution of workspace (and project) identifiers.
//!
//! When the user omits `--workspace`, we fetch their workspaces from the API
//! and auto-select if there is exactly one.  When they supply a value it may
//! be either a UUID **or** a human-readable name; we accept both.
//!
//! The same pattern applies to `--project` (scoped within a workspace).

use std::fmt;

// ---------------------------------------------------------------------------
// Domain types (stand-ins — replace with your real API types)
// ---------------------------------------------------------------------------

/// Minimal workspace record returned by the list-workspaces endpoint.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
}

/// Minimal project record returned by the list-projects endpoint.
#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced during auto-resolution of workspace / project.
#[derive(Debug)]
pub enum ResolveError {
    /// The user omitted the flag and the API returned zero workspaces.
    NoWorkspaces,

    /// The user omitted the flag and the API returned more than one workspace.
    MultipleWorkspaces(Vec<Workspace>),

    /// The user supplied a name or ID that matched nothing.
    WorkspaceNotFound {
        query: String,
        available: Vec<Workspace>,
    },

    /// Ambiguous name matched more than one workspace.
    AmbiguousWorkspace {
        query: String,
        matches: Vec<Workspace>,
    },

    /// Same family of errors, but for projects within a workspace.
    NoProjects {
        workspace: Workspace,
    },
    MultipleProjects {
        workspace: Workspace,
        projects: Vec<Project>,
    },
    ProjectNotFound {
        query: String,
        workspace: Workspace,
        available: Vec<Project>,
    },
    AmbiguousProject {
        query: String,
        matches: Vec<Project>,
    },

    /// Catch-all for transport / auth failures when calling the API.
    Api(String),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // -- workspace errors ------------------------------------------------
            ResolveError::NoWorkspaces => {
                write!(
                    f,
                    "error: no workspaces found for your account\n\n\
                     Hint: create a workspace first, or check that your auth token is correct."
                )
            }

            ResolveError::MultipleWorkspaces(ws) => {
                writeln!(f, "error: --workspace is required (you belong to {} workspaces)\n", ws.len())?;
                write_workspace_table(f, ws)?;
                write!(
                    f,
                    "\nPass --workspace <ID or NAME> to select one.\n\
                     Tip: if you always use the same workspace, set NOTION_WORKSPACE in your environment."
                )
            }

            ResolveError::WorkspaceNotFound { query, available } => {
                writeln!(f, "error: workspace {:?} not found\n", query)?;
                if available.is_empty() {
                    write!(f, "You have no workspaces.")
                } else {
                    writeln!(f, "Available workspaces:")?;
                    write_workspace_table(f, available)?;
                    write!(f, "\nPass the exact ID or name shown above.")
                }
            }

            ResolveError::AmbiguousWorkspace { query, matches } => {
                writeln!(f, "error: {:?} matches {} workspaces\n", query, matches.len())?;
                write_workspace_table(f, matches)?;
                write!(f, "\nPass the full workspace ID to disambiguate.")
            }

            // -- project errors --------------------------------------------------
            ResolveError::NoProjects { workspace } => {
                write!(
                    f,
                    "error: workspace {:?} ({}) has no projects\n\n\
                     Hint: create a project first.",
                    workspace.name, workspace.id
                )
            }

            ResolveError::MultipleProjects { workspace, projects } => {
                writeln!(
                    f,
                    "error: --project is required (workspace {:?} has {} projects)\n",
                    workspace.name,
                    projects.len()
                )?;
                write_project_table(f, projects)?;
                write!(
                    f,
                    "\nPass --project <ID or NAME> to select one.\n\
                     Tip: set NOTION_PROJECT in your environment for a default."
                )
            }

            ResolveError::ProjectNotFound { query, workspace, available } => {
                writeln!(
                    f,
                    "error: project {:?} not found in workspace {:?}\n",
                    query, workspace.name
                )?;
                if available.is_empty() {
                    write!(f, "This workspace has no projects.")
                } else {
                    writeln!(f, "Available projects:")?;
                    write_project_table(f, available)?;
                    write!(f, "\nPass the exact ID or name shown above.")
                }
            }

            ResolveError::AmbiguousProject { query, matches } => {
                writeln!(f, "error: {:?} matches {} projects\n", query, matches.len())?;
                write_project_table(f, matches)?;
                write!(f, "\nPass the full project ID to disambiguate.")
            }

            ResolveError::Api(msg) => {
                write!(f, "error: API call failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for ResolveError {}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn write_workspace_table(f: &mut fmt::Formatter<'_>, ws: &[Workspace]) -> fmt::Result {
    let max_name = ws.iter().map(|w| w.name.len()).max().unwrap_or(4).max(4);
    writeln!(f, "  {:<width$}  {}", "NAME", "ID", width = max_name)?;
    writeln!(f, "  {:<width$}  {}", "----", "--", width = max_name)?;
    for w in ws {
        writeln!(f, "  {:<width$}  {}", w.name, w.id, width = max_name)?;
    }
    Ok(())
}

fn write_project_table(f: &mut fmt::Formatter<'_>, ps: &[Project]) -> fmt::Result {
    let max_name = ps.iter().map(|p| p.name.len()).max().unwrap_or(4).max(4);
    writeln!(f, "  {:<width$}  {}", "NAME", "ID", width = max_name)?;
    writeln!(f, "  {:<width$}  {}", "----", "--", width = max_name)?;
    for p in ps {
        writeln!(f, "  {:<width$}  {}", p.name, p.id, width = max_name)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// API trait (abstract over real HTTP client for testing)
// ---------------------------------------------------------------------------

/// Trait abstracting the two API calls needed for resolution.
/// Implement this on your real HTTP client.
#[async_trait::async_trait]
pub trait WorkspaceApi {
    async fn list_workspaces(&self) -> Result<Vec<Workspace>, ResolveError>;
    async fn list_projects(&self, workspace_id: &str) -> Result<Vec<Project>, ResolveError>;
}

// ---------------------------------------------------------------------------
// Core resolution logic
// ---------------------------------------------------------------------------

/// Resolve a workspace from an optional user-supplied flag value.
///
/// Rules:
///   1. If `flag` is `None`, fetch all workspaces.
///      - Exactly one  -> return it.
///      - Zero         -> `NoWorkspaces` error.
///      - Multiple     -> `MultipleWorkspaces` error listing them all.
///   2. If `flag` is `Some(value)`:
///      - Try exact match on ID first (case-sensitive).
///      - Then case-insensitive match on name.
///      - Zero matches -> `WorkspaceNotFound`.
///      - Multiple name matches -> `AmbiguousWorkspace`.
pub async fn resolve_workspace(
    api: &dyn WorkspaceApi,
    flag: Option<&str>,
) -> Result<Workspace, ResolveError> {
    let workspaces = api.list_workspaces().await?;

    match flag {
        None => match workspaces.len() {
            0 => Err(ResolveError::NoWorkspaces),
            1 => Ok(workspaces.into_iter().next().unwrap()),
            _ => Err(ResolveError::MultipleWorkspaces(workspaces)),
        },
        Some(query) => {
            // Exact ID match first (case-sensitive).
            if let Some(ws) = workspaces.iter().find(|w| w.id == query) {
                return Ok(ws.clone());
            }

            // Case-insensitive name match.
            let query_lower = query.to_lowercase();
            let matches: Vec<_> = workspaces
                .iter()
                .filter(|w| w.name.to_lowercase() == query_lower)
                .cloned()
                .collect();

            match matches.len() {
                0 => Err(ResolveError::WorkspaceNotFound {
                    query: query.to_string(),
                    available: workspaces,
                }),
                1 => Ok(matches.into_iter().next().unwrap()),
                _ => Err(ResolveError::AmbiguousWorkspace {
                    query: query.to_string(),
                    matches,
                }),
            }
        }
    }
}

/// Resolve a project within an already-resolved workspace.
/// Same rules as `resolve_workspace` but scoped to projects.
pub async fn resolve_project(
    api: &dyn WorkspaceApi,
    workspace: &Workspace,
    flag: Option<&str>,
) -> Result<Project, ResolveError> {
    let projects = api.list_projects(&workspace.id).await?;

    match flag {
        None => match projects.len() {
            0 => Err(ResolveError::NoProjects {
                workspace: workspace.clone(),
            }),
            1 => Ok(projects.into_iter().next().unwrap()),
            _ => Err(ResolveError::MultipleProjects {
                workspace: workspace.clone(),
                projects,
            }),
        },
        Some(query) => {
            if let Some(p) = projects.iter().find(|p| p.id == query) {
                return Ok(p.clone());
            }

            let query_lower = query.to_lowercase();
            let matches: Vec<_> = projects
                .iter()
                .filter(|p| p.name.to_lowercase() == query_lower)
                .cloned()
                .collect();

            match matches.len() {
                0 => Err(ResolveError::ProjectNotFound {
                    query: query.to_string(),
                    workspace: workspace.clone(),
                    available: projects,
                }),
                1 => Ok(matches.into_iter().next().unwrap()),
                _ => Err(ResolveError::AmbiguousProject {
                    query: query.to_string(),
                    matches,
                }),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Integration with clap: how this plugs into a real CLI
// ---------------------------------------------------------------------------

/// Example showing how `resolve_workspace` fits into a clap command handler.
///
/// ```ignore
/// #[derive(clap::Args)]
/// pub struct GlobalOpts {
///     /// Workspace ID or name (auto-detected when you belong to exactly one).
///     #[arg(long, global = true, env = "NOTION_WORKSPACE")]
///     workspace: Option<String>,
///
///     /// Project ID or name (auto-detected when workspace has exactly one).
///     #[arg(long, global = true, env = "NOTION_PROJECT")]
///     project: Option<String>,
/// }
///
/// async fn run(api: &impl WorkspaceApi, opts: &GlobalOpts) -> anyhow::Result<()> {
///     let ws = resolve_workspace(api, opts.workspace.as_deref()).await?;
///     let proj = resolve_project(api, &ws, opts.project.as_deref()).await?;
///     // ... proceed with ws.id and proj.id
///     Ok(())
/// }
/// ```
const _: () = ();

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// In-memory mock for testing.
    struct MockApi {
        workspaces: Vec<Workspace>,
        projects: Vec<Project>,
    }

    #[async_trait::async_trait]
    impl WorkspaceApi for MockApi {
        async fn list_workspaces(&self) -> Result<Vec<Workspace>, ResolveError> {
            Ok(self.workspaces.clone())
        }
        async fn list_projects(&self, ws_id: &str) -> Result<Vec<Project>, ResolveError> {
            Ok(self
                .projects
                .iter()
                .filter(|p| p.workspace_id == ws_id)
                .cloned()
                .collect())
        }
    }

    fn ws(id: &str, name: &str) -> Workspace {
        Workspace {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    fn proj(id: &str, name: &str, ws_id: &str) -> Project {
        Project {
            id: id.to_string(),
            name: name.to_string(),
            workspace_id: ws_id.to_string(),
        }
    }

    // -- workspace tests -----------------------------------------------------

    #[tokio::test]
    async fn single_workspace_auto_resolves() {
        let api = MockApi {
            workspaces: vec![ws("ws-1", "Acme Corp")],
            projects: vec![],
        };
        let result = resolve_workspace(&api, None).await.unwrap();
        assert_eq!(result.id, "ws-1");
    }

    #[tokio::test]
    async fn multiple_workspaces_without_flag_errors() {
        let api = MockApi {
            workspaces: vec![ws("ws-1", "Acme Corp"), ws("ws-2", "Side Project")],
            projects: vec![],
        };
        let err = resolve_workspace(&api, None).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--workspace is required"));
        assert!(msg.contains("Acme Corp"));
        assert!(msg.contains("Side Project"));
        assert!(msg.contains("ws-1"));
        assert!(msg.contains("ws-2"));
    }

    #[tokio::test]
    async fn resolve_by_id() {
        let api = MockApi {
            workspaces: vec![ws("ws-1", "Acme Corp"), ws("ws-2", "Side Project")],
            projects: vec![],
        };
        let result = resolve_workspace(&api, Some("ws-2")).await.unwrap();
        assert_eq!(result.name, "Side Project");
    }

    #[tokio::test]
    async fn resolve_by_name_case_insensitive() {
        let api = MockApi {
            workspaces: vec![ws("ws-1", "Acme Corp"), ws("ws-2", "Side Project")],
            projects: vec![],
        };
        let result = resolve_workspace(&api, Some("acme corp")).await.unwrap();
        assert_eq!(result.id, "ws-1");
    }

    #[tokio::test]
    async fn unknown_workspace_lists_available() {
        let api = MockApi {
            workspaces: vec![ws("ws-1", "Acme Corp")],
            projects: vec![],
        };
        let err = resolve_workspace(&api, Some("Nope")).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("\"Nope\" not found"));
        assert!(msg.contains("Acme Corp"));
    }

    #[tokio::test]
    async fn no_workspaces_gives_clear_error() {
        let api = MockApi {
            workspaces: vec![],
            projects: vec![],
        };
        let err = resolve_workspace(&api, None).await.unwrap_err();
        assert!(err.to_string().contains("no workspaces found"));
    }

    // -- project tests -------------------------------------------------------

    #[tokio::test]
    async fn single_project_auto_resolves() {
        let w = ws("ws-1", "Acme Corp");
        let api = MockApi {
            workspaces: vec![w.clone()],
            projects: vec![proj("p-1", "Backend", "ws-1")],
        };
        let result = resolve_project(&api, &w, None).await.unwrap();
        assert_eq!(result.id, "p-1");
    }

    #[tokio::test]
    async fn multiple_projects_without_flag_errors() {
        let w = ws("ws-1", "Acme Corp");
        let api = MockApi {
            workspaces: vec![w.clone()],
            projects: vec![
                proj("p-1", "Backend", "ws-1"),
                proj("p-2", "Frontend", "ws-1"),
            ],
        };
        let err = resolve_project(&api, &w, None).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--project is required"));
        assert!(msg.contains("Backend"));
        assert!(msg.contains("Frontend"));
    }

    // -- error formatting snapshot -------------------------------------------

    #[test]
    fn error_message_snapshot_multiple_workspaces() {
        let err = ResolveError::MultipleWorkspaces(vec![
            ws("a1b2c3d4", "Acme Corp"),
            ws("e5f6g7h8", "Side Project"),
            ws("i9j0k1l2", "Acme Corp"),
        ]);
        let msg = err.to_string();
        // Verify the message is human-readable and contains all workspaces.
        println!("--- error output ---\n{}\n--- end ---", msg);
        assert!(msg.contains("3 workspaces"));
        assert!(msg.contains("NOTION_WORKSPACE"));
    }

    #[test]
    fn error_message_snapshot_not_found() {
        let err = ResolveError::WorkspaceNotFound {
            query: "typo-name".to_string(),
            available: vec![
                ws("a1b2c3d4", "Acme Corp"),
                ws("e5f6g7h8", "Side Project"),
            ],
        };
        let msg = err.to_string();
        println!("--- error output ---\n{}\n--- end ---", msg);
        assert!(msg.contains("\"typo-name\" not found"));
        assert!(msg.contains("Acme Corp"));
    }
}
