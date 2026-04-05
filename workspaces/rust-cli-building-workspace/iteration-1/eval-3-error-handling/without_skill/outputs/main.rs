//! `deployr` CLI entry point with structured error display.
//!
//! Errors are rendered as plain-English text to stderr by default, or as
//! single-line JSON to stdout when `--json` is active. The process exit
//! code is always deterministic (see `error::exit_code`).

mod error;

use error::CliError;
use std::process;

// ---------------------------------------------------------------------------
// Global flags (simplified — in practice these come from clap)
// ---------------------------------------------------------------------------

/// Top-level CLI arguments parsed before subcommand dispatch.
struct GlobalFlags {
    /// When true, all output (success *and* error) is JSON.
    json: bool,
}

impl GlobalFlags {
    fn parse() -> Self {
        // Minimal hand-parse for illustration.  In production, use clap:
        //   #[arg(long, global = true)]
        //   json: bool,
        let json = std::env::args().any(|a| a == "--json");
        Self { json }
    }
}

// ---------------------------------------------------------------------------
// Subcommand dispatch (placeholder)
// ---------------------------------------------------------------------------

/// The real implementation would match on clap subcommands here.
fn run() -> Result<(), CliError> {
    // Example: dispatch to whichever subcommand was requested.
    // Each subcommand returns Result<(), CliError>.
    //
    //   match cli.command {
    //       Command::Auth(args) => cmd::auth::run(args),
    //       Command::Events(args) => cmd::events::run(args),
    //       ...
    //   }
    //
    // For demonstration, return Ok.
    Ok(())
}

// ---------------------------------------------------------------------------
// Error rendering
// ---------------------------------------------------------------------------

/// Render `CliError` according to the active output mode and exit.
fn render_error_and_exit(err: CliError, flags: &GlobalFlags) -> ! {
    let code = err.exit_code();

    if flags.json {
        // JSON errors go to stdout so that piped consumers (`jq`, agents)
        // see a uniform stream.  stderr gets nothing.
        println!("{}", err.to_json());
    } else {
        // Human-readable errors go to stderr so stdout stays clean for
        // piped data.
        eprintln!("{err}");
    }

    process::exit(code);
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let flags = GlobalFlags::parse();

    // Install a panic hook that respects --json. Panics should never happen
    // in release builds, but if they do the user gets a useful message
    // instead of a raw backtrace.
    let json_mode = flags.json;
    std::panic::set_hook(Box::new(move |info| {
        let message = format!(
            "Internal error (this is a bug): {}",
            info.payload()
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
                .unwrap_or("unknown"),
        );
        let err = CliError::Api {
            status: None,
            message,
            hint: "Please report this at https://github.com/you/deployr/issues".into(),
        };
        if json_mode {
            // Use println, not eprintln, to keep JSON on stdout.
            println!("{}", err.to_json());
        } else {
            eprintln!("{err}");
        }
        // Exit with GENERAL_ERROR for panics — they don't fit a category.
        std::process::exit(error::exit_code::GENERAL_ERROR);
    }));

    // Run the CLI.  On error, render and exit with the right code.
    if let Err(err) = run() {
        render_error_and_exit(err, &flags);
    }

    // Explicit success exit.
    process::exit(error::exit_code::SUCCESS);
}

// ---------------------------------------------------------------------------
// Example: how library errors convert transparently
// ---------------------------------------------------------------------------
//
// Because we implement `From<std::io::Error>` and (optionally)
// `From<reqwest::Error>`, any function returning `Result<_, CliError>` can
// use `?` on standard library and HTTP calls:
//
// ```rust
// fn read_config(path: &Path) -> Result<Config, CliError> {
//     let bytes = std::fs::read(path)       // io::Error → CliError::Io via `?`
//         .map_err(|e| CliError::io(format!("reading {}", path.display()), e))?;
//     let cfg: Config = toml::from_slice(&bytes)
//         .map_err(|e| CliError::config_parse(path.display(), e))?;
//     Ok(cfg)
// }
//
// async fn fetch_events(client: &Client, token: &str) -> Result<Events, CliError> {
//     let resp = client.get(URL)
//         .bearer_auth(token)
//         .send()
//         .await?;                          // reqwest::Error → CliError via From
//     if resp.status() == 401 {
//         return Err(CliError::auth_expired());
//     }
//     let events: Events = resp.json()
//         .await
//         .map_err(|e| CliError::api_decode(e))?;
//     Ok(events)
// }
// ```
