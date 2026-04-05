use serde::{Deserialize, Serialize};

/// A workspace returned by the project management API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    /// The owner's email, if available.
    pub owner_email: Option<String>,
}

/// A project within a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
}
