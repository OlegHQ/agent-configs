mod client;
mod error;
mod output;
mod resolve;
mod types;

use clap::{Parser, Subcommand};
use error::CliError;

#[derive(Parser)]
#[command(name = "pmtool", about = "Project management CLI for humans and agents")]
pub struct Cli {
    /// Output JSON instead of human-readable tables.
    #[arg(long, global = true)]
    json: bool,

    /// Workspace name or ID. Auto-resolved if you have exactly one.
    #[arg(long, global = true)]
    workspace: Option<String>,

    /// Project name or ID. Auto-resolved if there's exactly one in the workspace.
    #[arg(long, global = true)]
    project: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List workspaces accessible to your account.
    Workspaces,

    /// Manage tasks within a project.
    #[command(subcommand)]
    Tasks(TasksCmd),
}

#[derive(Subcommand)]
pub enum TasksCmd {
    /// List tasks (requires workspace + project context).
    List {
        /// Maximum number of tasks to return.
        #[arg(long, default_value = "25")]
        limit: usize,
    },
    /// Get a single task by ID.
    Get {
        /// Task ID.
        task_id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli).await {
        print_error(&cli, &e);
        std::process::exit(e.exit_code());
    }
}

async fn run(cli: &Cli) -> Result<(), CliError> {
    let client = client::Client::from_env()?;

    match &cli.command {
        Command::Workspaces => {
            let workspaces = client.list_workspaces().await?;
            output::print_workspaces(cli.json, &workspaces)?;
            if !cli.json {
                output::hint(&[
                    "pmtool tasks list --workspace <NAME>  -- list tasks in a workspace",
                ]);
            }
        }

        Command::Tasks(sub) => {
            // Auto-resolve workspace + project for all task commands.
            let ctx = resolve::resolve_context(
                &client,
                cli.workspace.as_deref(),
                cli.project.as_deref(),
            )
            .await?;

            match sub {
                TasksCmd::List { limit } => {
                    let tasks = client
                        .list_tasks(&ctx.workspace.id, &ctx.project.id, *limit)
                        .await?;
                    // ... print tasks (omitted for brevity) ...
                    let _ = tasks;
                }
                TasksCmd::Get { task_id } => {
                    let task = client
                        .get_task(&ctx.workspace.id, &ctx.project.id, task_id)
                        .await?;
                    let _ = task;
                }
            }
        }
    }
    Ok(())
}

/// Print errors to stderr. In --json mode, emit structured JSON.
fn print_error(cli: &Cli, err: &CliError) {
    match err {
        CliError::Resolve(resolve_err) => {
            if cli.json {
                let json = output::format_resolve_error_json(resolve_err, err.exit_code());
                eprintln!("{json}");
            } else {
                eprintln!("{}", output::format_resolve_error(resolve_err));
            }
        }
        other => {
            if cli.json {
                let obj = serde_json::json!({
                    "error": format!("{other}"),
                    "code": other.exit_code(),
                });
                eprintln!("{}", serde_json::to_string(&obj).unwrap());
            } else {
                eprintln!("error: {other}");
                if let Some(h) = error_hint(other) {
                    eprintln!("\nhint: {h}");
                }
            }
        }
    }
}

fn error_hint(e: &CliError) -> Option<&'static str> {
    match e {
        CliError::Api(error::ApiError::Unauthorized) => {
            Some("run `pmtool auth login` to authenticate")
        }
        CliError::Api(error::ApiError::Network(_)) => Some("check your internet connection"),
        _ => None,
    }
}
