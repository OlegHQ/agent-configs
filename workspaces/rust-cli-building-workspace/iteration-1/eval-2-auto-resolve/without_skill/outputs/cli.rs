//! Clap integration showing how resolve_workspace / resolve_project plug into
//! a real CLI with global --workspace and --project flags.
//!
//! Depends on `resolve.rs` (the `resolve` module) for the actual logic.

use clap::{Parser, Subcommand};

// Assuming `resolve` is a sibling module:
// mod resolve;
use crate::resolve::{resolve_project, resolve_workspace, ResolveError, WorkspaceApi};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "pm", about = "Project management CLI")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Command,
}

/// Global flags available to every subcommand.
///
/// Both `--workspace` and `--project` are optional.  When omitted, the CLI
/// auto-resolves by fetching from the API and selecting the sole entry.
/// The user can also set env vars for a persistent default.
#[derive(clap::Args, Clone)]
pub struct GlobalOpts {
    /// Workspace ID or name.  Auto-detected when you belong to exactly one.
    #[arg(long, short = 'w', global = true, env = "NOTION_WORKSPACE")]
    pub workspace: Option<String>,

    /// Project ID or name.  Auto-detected when the workspace has exactly one.
    #[arg(long, short = 'p', global = true, env = "NOTION_PROJECT")]
    pub project: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List events in a project.
    Events,
    /// Show project info.
    Info,
}

// ---------------------------------------------------------------------------
// Resolved context — passed to subcommand handlers
// ---------------------------------------------------------------------------

/// After resolution, every handler receives concrete IDs (never Option).
pub struct ResolvedContext {
    pub workspace_id: String,
    pub workspace_name: String,
    pub project_id: String,
    pub project_name: String,
}

impl ResolvedContext {
    /// Resolve workspace and project from the global CLI flags.
    /// Prints a user-friendly error to stderr and returns the appropriate
    /// exit code on failure.
    pub async fn from_opts(
        api: &dyn WorkspaceApi,
        opts: &GlobalOpts,
    ) -> Result<Self, ResolveError> {
        let ws = resolve_workspace(api, opts.workspace.as_deref()).await?;
        let proj = resolve_project(api, &ws, opts.project.as_deref()).await?;
        Ok(ResolvedContext {
            workspace_id: ws.id,
            workspace_name: ws.name,
            project_id: proj.id,
            project_name: proj.name,
        })
    }
}

// ---------------------------------------------------------------------------
// Entry point sketch
// ---------------------------------------------------------------------------

/// Example main showing how errors surface to the user.
///
/// ```ignore
/// #[tokio::main]
/// async fn main() {
///     let cli = Cli::parse();
///     let api = RealHttpClient::new(/* auth token */);
///
///     let ctx = match ResolvedContext::from_opts(&api, &cli.global).await {
///         Ok(ctx) => ctx,
///         Err(e) => {
///             eprintln!("{e}");
///             std::process::exit(2); // exit 2 = resolution failure
///         }
///     };
///
///     match cli.command {
///         Command::Events => { /* use ctx.project_id */ }
///         Command::Info   => { /* use ctx.workspace_id */ }
///     }
/// }
/// ```
const _: () = ();
