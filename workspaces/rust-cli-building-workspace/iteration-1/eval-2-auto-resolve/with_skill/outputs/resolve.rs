use crate::client::Client;
use crate::error::ResolveError;
use crate::types::{Project, Workspace};

/// Resolved context that commands need to operate.
#[derive(Debug, Clone)]
pub struct ResolvedContext {
    pub workspace: Workspace,
    pub project: Project,
}

// ---------------------------------------------------------------------------
// Workspace resolution
// ---------------------------------------------------------------------------

/// Resolve a workspace from an optional user-provided hint.
///
/// Resolution strategy (highest priority wins):
/// 1. If `hint` is provided, match by ID or name (case-insensitive).
/// 2. If no hint, fetch workspaces from the API:
///    - Exactly one  -> use it automatically (log to stderr).
///    - Zero         -> error with actionable message.
///    - Multiple     -> error listing all options with copy-pasteable flags.
pub async fn resolve_workspace(
    client: &Client,
    hint: Option<&str>,
) -> Result<Workspace, ResolveError> {
    let workspaces = client.list_workspaces().await?;

    if workspaces.is_empty() {
        return Err(ResolveError::NoWorkspaces);
    }

    // --- User gave an explicit --workspace value ---
    if let Some(query) = hint {
        let q_lower = query.to_ascii_lowercase();

        // Try exact ID match first, then case-insensitive name match.
        if let Some(found) = workspaces.iter().find(|w| {
            w.id == query || w.name.to_ascii_lowercase() == q_lower
        }) {
            return Ok(found.clone());
        }

        // No match — build a helpful error with all available options.
        return Err(ResolveError::WorkspaceNotFound {
            query: query.to_string(),
            candidates: workspaces,
        });
    }

    // --- No hint: auto-resolve ---
    if workspaces.len() == 1 {
        let ws = workspaces.into_iter().next().unwrap();
        // Auto-resolution notice goes to stderr (stream discipline).
        eprintln!("auto: using workspace {:?} ({})", ws.name, ws.id);
        return Ok(ws);
    }

    // Multiple workspaces, no way to pick automatically.
    Err(ResolveError::AmbiguousWorkspace {
        candidates: workspaces,
    })
}

// ---------------------------------------------------------------------------
// Project resolution (same pattern)
// ---------------------------------------------------------------------------

/// Resolve a project within a workspace from an optional user-provided hint.
///
/// Same strategy as `resolve_workspace`: match by ID or name, auto-pick if
/// there is exactly one, error with options if ambiguous.
pub async fn resolve_project(
    client: &Client,
    workspace: &Workspace,
    hint: Option<&str>,
) -> Result<Project, ResolveError> {
    let projects = client.list_projects(&workspace.id).await?;

    if projects.is_empty() {
        return Err(ResolveError::NoProjects {
            workspace_name: workspace.name.clone(),
        });
    }

    if let Some(query) = hint {
        let q_lower = query.to_ascii_lowercase();

        if let Some(found) = projects.iter().find(|p| {
            p.id == query || p.name.to_ascii_lowercase() == q_lower
        }) {
            return Ok(found.clone());
        }

        return Err(ResolveError::ProjectNotFound {
            query: query.to_string(),
            workspace_name: workspace.name.clone(),
            candidates: projects,
        });
    }

    if projects.len() == 1 {
        let proj = projects.into_iter().next().unwrap();
        eprintln!("auto: using project {:?} ({})", proj.name, proj.id);
        return Ok(proj);
    }

    Err(ResolveError::AmbiguousProject {
        workspace_name: workspace.name.clone(),
        candidates: projects,
    })
}

// ---------------------------------------------------------------------------
// Combined resolution
// ---------------------------------------------------------------------------

/// Resolve both workspace and project in one call.
///
/// This is the main entry point that command handlers use:
///
/// ```rust,ignore
/// let ctx = resolve_context(&client, cli.workspace.as_deref(), cli.project.as_deref()).await?;
/// ```
pub async fn resolve_context(
    client: &Client,
    workspace_hint: Option<&str>,
    project_hint: Option<&str>,
) -> Result<ResolvedContext, ResolveError> {
    let workspace = resolve_workspace(client, workspace_hint).await?;
    let project = resolve_project(client, &workspace, project_hint).await?;
    Ok(ResolvedContext { workspace, project })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockClient;

    fn ws(id: &str, name: &str) -> Workspace {
        Workspace {
            id: id.to_string(),
            name: name.to_string(),
            owner_email: None,
        }
    }

    #[tokio::test]
    async fn single_workspace_auto_resolves() {
        let client = MockClient::new(vec![ws("ws-1", "Acme Corp")], vec![]);
        let result = resolve_workspace(&client.into(), None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "ws-1");
    }

    #[tokio::test]
    async fn multiple_workspaces_without_hint_is_error() {
        let client = MockClient::new(
            vec![ws("ws-1", "Acme Corp"), ws("ws-2", "Side Project")],
            vec![],
        );
        let result = resolve_workspace(&client.into(), None).await;
        assert!(matches!(
            result,
            Err(ResolveError::AmbiguousWorkspace { .. })
        ));
    }

    #[tokio::test]
    async fn resolve_by_name_case_insensitive() {
        let client = MockClient::new(
            vec![ws("ws-1", "Acme Corp"), ws("ws-2", "Side Project")],
            vec![],
        );
        let result = resolve_workspace(&client.into(), Some("acme corp")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "ws-1");
    }

    #[tokio::test]
    async fn resolve_by_id() {
        let client = MockClient::new(
            vec![ws("ws-1", "Acme Corp"), ws("ws-2", "Side Project")],
            vec![],
        );
        let result = resolve_workspace(&client.into(), Some("ws-2")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Side Project");
    }

    #[tokio::test]
    async fn no_match_returns_candidates() {
        let client = MockClient::new(
            vec![ws("ws-1", "Acme Corp"), ws("ws-2", "Side Project")],
            vec![],
        );
        let result = resolve_workspace(&client.into(), Some("nonexistent")).await;
        match result {
            Err(ResolveError::WorkspaceNotFound { query, candidates }) => {
                assert_eq!(query, "nonexistent");
                assert_eq!(candidates.len(), 2);
            }
            other => panic!("expected WorkspaceNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn no_workspaces_at_all() {
        let client = MockClient::new(vec![], vec![]);
        let result = resolve_workspace(&client.into(), None).await;
        assert!(matches!(result, Err(ResolveError::NoWorkspaces)));
    }
}
