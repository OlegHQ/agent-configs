use crate::error::{ApiError, ResolveError};
use crate::types::{Project, Workspace};

/// Minimal API client stub. In production this wraps reqwest with auth headers,
/// token refresh, and retry logic.
pub struct Client {
    base_url: String,
    token: String,
}

impl Client {
    /// Build a client from environment variables.
    pub fn from_env() -> Result<Self, crate::error::CliError> {
        let token = std::env::var("PMTOOL_TOKEN").map_err(|_| {
            crate::error::CliError::Api(ApiError::Unauthorized)
        })?;
        let base_url = std::env::var("PMTOOL_API_URL")
            .unwrap_or_else(|_| "https://api.pmtool.example.com".to_string());
        Ok(Self { base_url, token })
    }

    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>, ResolveError> {
        // Real implementation: GET /v1/workspaces with Bearer token.
        // Stub returns empty; tests use MockClient.
        let _url = format!("{}/v1/workspaces", self.base_url);
        let _auth = &self.token;
        todo!("call the API")
    }

    pub async fn list_projects(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<Project>, ResolveError> {
        let _url = format!(
            "{}/v1/workspaces/{}/projects",
            self.base_url, workspace_id
        );
        todo!("call the API")
    }

    pub async fn list_tasks(
        &self,
        _workspace_id: &str,
        _project_id: &str,
        _limit: usize,
    ) -> Result<Vec<serde_json::Value>, ApiError> {
        todo!("call the API")
    }

    pub async fn get_task(
        &self,
        _workspace_id: &str,
        _project_id: &str,
        _task_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        todo!("call the API")
    }
}

// ---------------------------------------------------------------------------
// Mock client for tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub struct MockClient {
    workspaces: Vec<Workspace>,
    projects: Vec<Project>,
}

#[cfg(test)]
impl MockClient {
    pub fn new(workspaces: Vec<Workspace>, projects: Vec<Project>) -> Self {
        Self {
            workspaces,
            projects,
        }
    }
}

#[cfg(test)]
impl From<MockClient> for Client {
    fn from(_mock: MockClient) -> Self {
        // In a real codebase you'd use a trait (ApiClient) so the mock
        // replaces the real HTTP calls. This stub shows the pattern.
        todo!("trait-based mock; see resolve.rs tests for the intended API")
    }
}
