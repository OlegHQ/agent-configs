use crate::types::Workspace;

/// Errors that can occur during resource resolution.
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("no workspaces found for your account")]
    NoWorkspaces,

    #[error("multiple workspaces found; specify --workspace <NAME-OR-ID>")]
    AmbiguousWorkspace {
        /// The workspaces available for the user to choose from.
        candidates: Vec<Workspace>,
    },

    #[error("no workspace matching {query:?}")]
    WorkspaceNotFound {
        query: String,
        candidates: Vec<Workspace>,
    },

    #[error("no projects found in workspace {workspace_name:?}")]
    NoProjects { workspace_name: String },

    #[error("multiple projects found in workspace {workspace_name:?}; specify --project <NAME-OR-ID>")]
    AmbiguousProject {
        workspace_name: String,
        candidates: Vec<crate::types::Project>,
    },

    #[error("no project matching {query:?} in workspace {workspace_name:?}")]
    ProjectNotFound {
        query: String,
        workspace_name: String,
        candidates: Vec<crate::types::Project>,
    },

    #[error("API request failed: {0}")]
    Api(#[from] ApiError),
}

/// Upstream API errors.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("network error: {0}")]
    Network(String),

    #[error("authentication failed (HTTP 401)")]
    Unauthorized,

    #[error("unexpected status {status}: {body}")]
    UnexpectedStatus { status: u16, body: String },
}

/// Top-level CLI error wrapping all error categories.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{0}")]
    Resolve(#[from] ResolveError),

    #[error("{0}")]
    Api(#[from] ApiError),

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl CliError {
    /// Exit code following the convention: 2=usage, 3=auth, 4=upstream.
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Resolve(ResolveError::Api(ApiError::Unauthorized)) => 3,
            CliError::Api(ApiError::Unauthorized) => 3,
            CliError::Resolve(
                ResolveError::AmbiguousWorkspace { .. }
                | ResolveError::WorkspaceNotFound { .. }
                | ResolveError::AmbiguousProject { .. }
                | ResolveError::ProjectNotFound { .. }
                | ResolveError::NoWorkspaces
                | ResolveError::NoProjects { .. },
            ) => 2,
            CliError::Resolve(ResolveError::Api(_)) | CliError::Api(_) => 4,
            CliError::Io(_) => 4,
        }
    }
}
