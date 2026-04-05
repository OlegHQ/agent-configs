//! `deployr` CLI entry point.
//!
//! Responsibilities of main.rs:
//! - Parse CLI args (clap).
//! - Dispatch to command handlers.
//! - Render errors in human or JSON mode.
//! - Exit with the correct code.

mod error;
// mod commands;
// mod output;
// mod config;

use clap::Parser;
use error::{CliError, error_hint};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "deployr", version, about = "Deploy and manage your services")]
pub struct Cli {
    /// Output results as JSON (one object per line for lists, object for
    /// single items, structured error on stderr for failures).
    #[arg(long, global = true)]
    pub json: bool,

    /// Increase log verbosity (repeat for more: -v, -vv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Authenticate with the deployr service.
    Auth,
    /// Deploy a service.
    Deploy,
    /// Show deployment status.
    Status,
    // ... other commands ...
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();

    // Initialise tracing to stderr (never pollute stdout).
    init_tracing(cli.verbose);

    // Run the async command dispatcher; capture the result.
    let result: Result<(), CliError> = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(run(&cli));

    // Handle errors.
    if let Err(ref e) = result {
        render_error(&cli, e);
        std::process::exit(e.exit_code());
    }
}

// ---------------------------------------------------------------------------
// Dispatcher (placeholder — fill in with real command handlers)
// ---------------------------------------------------------------------------

async fn run(cli: &Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Auth => {
            // commands::auth::run(cli).await
            todo!("auth command")
        }
        Command::Deploy => {
            // commands::deploy::run(cli).await
            todo!("deploy command")
        }
        Command::Status => {
            // commands::status::run(cli).await
            todo!("status command")
        }
    }
}

// ---------------------------------------------------------------------------
// Error rendering
// ---------------------------------------------------------------------------

/// Render an error to stderr in human-readable or JSON format.
///
/// Stream discipline: errors always go to **stderr**. In JSON mode the error
/// is a single JSON object; in human mode it is a plain "error:" line plus an
/// optional "hint:" line.
fn render_error(cli: &Cli, err: &CliError) {
    if cli.json {
        render_error_json(err);
    } else {
        render_error_human(err);
    }
}

/// Human-readable error display:
///
/// ```text
/// error: access token has expired
/// hint:  run `deployr auth refresh` to get a new token
/// ```
fn render_error_human(err: &CliError) {
    eprintln!("error: {err}");
    if let Some(hint) = error_hint(err) {
        eprintln!("hint:  {hint}");
    }
}

/// JSON error display (single object on stderr):
///
/// ```json
/// {"error":"access token has expired","code":3,"code_name":"auth_error","hint":"run `deployr auth refresh` to get a new token"}
/// ```
fn render_error_json(err: &CliError) {
    // Build the JSON manually to avoid pulling serde into the error path.
    // If serde_json is already linked (it will be), you can use it instead.
    let message = err.to_string().replace('\\', "\\\\").replace('"', "\\\"");
    let code = err.exit_code();
    let code_name = err.code_name();

    let json = if let Some(hint) = error_hint(err) {
        let hint_escaped = hint.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            r#"{{"error":"{message}","code":{code},"code_name":"{code_name}","hint":"{hint_escaped}"}}"#
        )
    } else {
        format!(r#"{{"error":"{message}","code":{code},"code_name":"{code_name}"}}"#)
    };

    eprintln!("{json}");
}

// ---------------------------------------------------------------------------
// Tracing setup
// ---------------------------------------------------------------------------

fn init_tracing(verbosity: u8) {
    use tracing_subscriber::EnvFilter;

    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .with_writer(std::io::stderr) // never to stdout
        .init();
}
