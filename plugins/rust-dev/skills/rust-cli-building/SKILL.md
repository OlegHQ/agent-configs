---
name: rust-cli-building
description: >
  Builds production-quality Rust CLIs that work beautifully for both humans and automated agents.
  Use this skill whenever writing a new Rust CLI tool, adding commands to an existing CLI, designing
  CLI output formats, implementing error handling for command-line tools, or structuring a Rust binary
  crate with clap. Triggers on: new CLI projects, adding subcommands, designing --json output,
  structuring error messages, building tables/formatters, writing CLI tests, or any Rust binary
  that humans and scripts both need to use.
---

# Building Rust CLIs for Humans and Agents

This skill teaches you to build CLIs that feel natural to humans while being perfectly parseable by scripts and AI agents. The core insight: these goals aren't in tension — they require the same discipline (clear structure, predictable output, good errors), just rendered differently.

## The Two-Audience Problem

Every CLI command produces output that two very different consumers need to use:

**Humans** want scannable tables, readable errors with suggestions, progressive disclosure (brief by default, detailed on request), and contextual hints about what to do next.

**Agents** (scripts, CI, LLMs) want structured JSON, stable exit codes they can branch on, parseable error objects, and discoverable next-actions they can invoke without reading man pages.

The solution is **dual-mode output**: human-friendly by default, machine-friendly with `--json`. Both modes share the same data — they just render it differently.

## Architecture

Organize your Rust CLI as a workspace with clean separation:

```
crates/
  myapp-api/        # Library: HTTP client, types, errors (thiserror)
    src/
      client.rs     # API client with auth, retry, refresh
      endpoints/    # One module per API resource group
      types/        # Shared data types (serde Serialize + Deserialize)
      error.rs      # Library error enums (thiserror)
  myapp-cli/        # Binary: argument parsing, output formatting
    src/
      main.rs       # Entry point, clap parse, top-level error display
      commands/     # One module per command group (events.rs, auth.rs...)
      output.rs     # ALL output formatting: tables, JSON, hints, type renderers
      config.rs     # Layered config loading
```

The library crate knows nothing about terminals or formatting. The CLI crate owns all presentation. This lets you swap the CLI for a different frontend (TUI, web) without touching business logic.

### Dependency choices

| Purpose | Crate | Why |
|---------|-------|-----|
| Arg parsing | `clap` v4 (derive) | Structs define the CLI; derive macros generate the parser |
| Errors (lib) | `thiserror` | Custom error enums with structured fields |
| Errors (bin) | Display manually | You want full control over what humans vs agents see |
| Serialization | `serde` + `serde_json` | JSON output + API payloads |
| Config | `toml` + `directories` | TOML config + XDG-compliant paths |
| HTTP | `reqwest` + `tokio` | Async HTTP with rustls (no OpenSSL dep) |
| Logging | `tracing` + `tracing-subscriber` | Structured logging to stderr |

Avoid pulling in crates for things you can do in 20 lines (table formatting, hint printing). A simple column-aligned table printer is ~30 lines and avoids a dependency.

## Command Design

### Resource-oriented structure

Model commands as **verb-noun pairs** with consistent vocabulary:

```rust
#[derive(Subcommand)]
pub enum Command {
    /// Manage calendar events.
    #[command(subcommand)]
    Events(EventsCmd),
    /// List connected accounts.
    #[command(subcommand)]
    Accounts(AccountsCmd),
}

#[derive(Subcommand)]
pub enum EventsCmd {
    /// List upcoming events (auto-resolves account and calendar).
    List { /* flags */ },
    /// Get a single event by ID.
    Get { event_id: String, /* flags */ },
    /// Create a new event.
    Create { /* flags */ },
    /// Update fields on an existing event.
    Update { event_id: String, /* flags */ },
    /// Cancel or delete an event.
    Delete { event_id: String, /* flags */ },
}
```

Every enum variant has a `///` doc comment — this becomes the help text users and agents see. Every non-obvious flag also gets one.

### Verb consistency

Use these verbs across all resource types:

| Verb | Meaning | Returns |
|------|---------|---------|
| `list` | Enumerate resources | Table (human) / JSON array (agent) |
| `get` | Single resource detail | Key-value (human) / JSON object (agent) |
| `create` | Create new resource | Created resource detail |
| `update` | Modify existing | Updated resource detail |
| `delete` | Remove/cancel | Confirmation message |

### Flags over positional args

Flags are self-documenting and order-independent. The only positional arg should be the resource ID:

```rust
/// Get a single event by ID.
Get {
    /// Event ID.
    event_id: String,
    /// Account email or ID.
    #[arg(long)]
    account: Option<String>,
    #[arg(long)]
    calendar: Option<String>,
},
```

### Smart defaults

Commands should work with minimal flags. The user shouldn't need to know the internal ID of their only account:

- **Default to upcoming**: `events list` shows events from now, not all history. Use `--all` to override.
- **Default limits**: `--limit 25` by default, not unbounded.
- **Auto-resolve context**: See the Auto-Resolution section below.

Add `--all` as the escape hatch from smart defaults, not the other way around.

## Output System

### The output module

Centralize ALL output in `output.rs`. Every type gets a dedicated renderer:

```rust
// Dispatch based on --json flag
pub fn print_events(cli: &Cli, events: &[Event]) -> Result<(), CliError> {
    if cli.json {
        return print_json(cli, &events);
    }
    let rows: Vec<Vec<String>> = events.iter().map(|e| vec![
        fmt_datetime(&e.start),
        fmt_datetime(&e.end),
        e.summary.clone().unwrap_or_else(|| "(no title)".into()),
        e.status.map(|s| s.to_string()).unwrap_or_default(),
        e.id.clone(),
    ]).collect();
    print_table(&["START", "END", "SUMMARY", "STATUS", "ID"], &rows);
    Ok(())
}
```

This pattern (one `print_<type>` function per resource) keeps command handlers clean — they call `print_events(cli, &events)` and the output module handles everything.

### Table formatting

Tables should be borderless, column-aligned, and grep-friendly:

```
PROVIDER  EMAIL             NAME            PRI  ID
google    user@example.com  Jane Smith      *    609ede4d-f209-...
notion    user@notion.so    Jane's Notion        ccafcf53-40f7-...
```

Implementation (no external crate needed):

```rust
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
        cells.iter().enumerate()
            .map(|(i, c)| format!("{:<w$}", c, w = widths[i]))
            .collect::<Vec<_>>().join("  ")
    };
    println!("{}", fmt_row(&headers.iter().copied().collect::<Vec<_>>()));
    for row in rows {
        let refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        println!("{}", fmt_row(&refs));
    }
}
```

### Key-value output for single resources

For `get` / `describe` / `status` commands, use aligned key-value pairs:

```
Name:      Oleg Pustovit
Email:     user@example.com
User ID:   10efb0f7-6ee0-4dd3-9f13-7d01246983ed
Accounts:  3
```

### JSON output mode

When `--json` is passed:
- Single resources: compact single-line JSON to stdout
- Lists: one JSON object per line (JSON Lines) for streaming
- Errors: JSON to stderr with `{error, code, hint}` structure
- No tables, no hints to stderr, no decorative text

### Stream discipline

This is critical and non-negotiable:
- **stdout**: Data only. The artifact a pipe consumer expects.
- **stderr**: Everything else — progress, auto-resolution messages, hints, errors.

This means `ncal events list > events.json` captures clean data while the user sees `auto: using account user@example.com` on their terminal.

## Auto-Resolution

When a command requires context (account, project, calendar), resolve it automatically instead of demanding flags:

```
$ ncal events list
auto: using account user@example.com (google)
auto: using calendar "user@example.com" (primary)
START          END            SUMMARY          STATUS     ID
Apr 06, 16:00  Apr 06, 16:30  Team Sync        confirmed  5ca3d8a1...
```

### Resolution strategy

1. Check the CLI flag (`--account`)
2. Check config file (`defaults.account`)
3. Fetch from API and auto-pick:
   - If exactly one → use it (log to stderr)
   - If one is marked primary → use it (log to stderr)
   - If multiple with no primary → fail with copy-pasteable flag list

### Accept human-friendly identifiers

Users shouldn't need to memorize UUIDs. Accept both IDs and emails/names:

```rust
async fn resolve_account(client: &Client, hint: Option<&str>) -> Result<(String, Provider)> {
    let user = get_user(client).await?;
    let accounts = user.accounts.unwrap_or_default();
    if let Some(q) = hint {
        let q_lower = q.to_ascii_lowercase();
        if let Some(found) = accounts.iter().find(|a| {
            a.id == q || a.email.as_deref()
                .map(|e| e.to_ascii_lowercase()) == Some(q_lower.clone())
        }) {
            return Ok((found.id.clone(), found.provider));
        }
        return Err(format!("no account matching {:?}; available:\n{}", q, list));
    }
    // ... auto-resolve logic
}
```

### Error on ambiguity with actionable options

When auto-resolution fails, show copy-pasteable commands:

```
error: multiple accounts found; specify --account <EMAIL>:
  --account user@example.com   # Jane Smith (google)
  --account user@notion.so     # Jane's Notion (notion)

or set defaults.account in your config file
```

The user (or agent) can copy-paste the flag directly.

## Error Reporting

Every error answers three questions: **what** failed, **why**, and **what to do about it**.

### Error structure

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("no credentials found (checked env vars, keychain, and desktop app)")]
    NoCredentials,

    #[error("cannot read desktop app database at {path}: {reason}")]
    LevelDb { path: String, reason: String },

    #[error("cannot parse {context} as JSON: {source}")]
    Json { context: String, source: serde_json::Error },
}
```

Notice: every variant carries enough context to tell the user exactly what happened. `Json(serde_json::Error)` is useless — it produces "expected value at line 1 column 1". Instead, `Json { context: "desktop app credential in LevelDB (path; 7558 bytes)" }` tells the user where the problem is.

### Hints in error display

In `main.rs`, add an `error_hint()` function that maps error types to actionable suggestions:

```rust
fn error_hint(e: &CliError) -> Option<&'static str> {
    match e {
        CliError::Auth(AuthError::NoCredentials) =>
            Some("run `ncal auth from-app` to import from the desktop app"),
        CliError::Api(ApiError::InvalidToken) =>
            Some("session expired; run `ncal auth refresh`"),
        CliError::Api(ApiError::Network(_)) =>
            Some("check your internet connection"),
        _ => None,
    }
}
```

### JSON error mode

When `--json` is active, errors go to stderr as JSON:

```json
{"error": "session expired", "code": 3, "hint": "run `ncal auth refresh`"}
```

### Exit codes

Define these precisely — they're an API contract:

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | Command completed |
| 2 | Usage error | Bad flags, missing required input, validation failure |
| 3 | Auth error | No credentials, expired token, refresh failed |
| 4 | Upstream error | API error, network failure, I/O error |

## HATEOAS Hints

After every successful command, suggest what the user or agent might want to do next:

```
hint: ncal calendars list --account <ID>  — list calendars for an account
      ncal events list                    — list upcoming events
```

### Implementation

```rust
pub fn hint(items: &[&str]) {
    if items.is_empty() { return; }
    eprintln!();
    for (i, item) in items.iter().enumerate() {
        if i == 0 {
            eprintln!("hint: {item}");
        } else {
            eprintln!("      {item}");
        }
    }
}
```

### When to hint

| After command | Suggest |
|--------------|---------|
| `auth login` / `auth from-app` | `whoami`, `events list` |
| `accounts list` | `calendars list --account <ID>`, `events list` |
| `calendars list` | `events list --calendar <ID>` |
| `events create` | `events get <ID>`, `events list` |
| `events list` (empty) | Try different time range, check calendars |
| Any error | The fix command |

### Suppress in --json mode

Hints go to stderr, but in `--json` mode suppress them entirely — agents parse stdout. If you want agents to discover next actions, include them in the JSON response body under a `next_actions` key (optional, only for commands where discoverability matters).

## Configuration

### Layered precedence (highest wins)

1. **CLI flags** — `--account xyz`
2. **Environment variables** — `MYAPP_ACCOUNT=xyz`
3. **Config file** — `defaults.account = "xyz"` in `~/.config/myapp/config.toml`
4. **Auto-resolution** — fetch from API, pick the obvious one
5. **Built-in defaults** — e.g., provider defaults to "google"

### Config file structure

```toml
[auth]
keychain_service = "myapp"

[defaults]
account = "user@example.com"
calendar = "primary"
timezone = "America/New_York"
output = "json"  # override default output mode
```

### Debug support

Provide `--show-config` or similar to dump the resolved config (redacted secrets) so users can debug what's active and where it came from.

## Testing

### Integration tests with assert_cmd

Test the CLI as a black box — run the binary, assert exit code + stdout + stderr:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn missing_credentials_shows_hint() {
    Command::cargo_bin("myapp").unwrap()
        .arg("events").arg("list")
        .env_remove("MYAPP_ACCESS_TOKEN")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("ncal auth from-app"));
}
```

### Snapshot tests with insta

Capture output format and catch accidental regressions:

```rust
#[test]
fn accounts_table_format() {
    let output = run_with_mock_data("accounts", "list");
    insta::assert_snapshot!(output);
}
```

### What to test

| Test | Checks |
|------|--------|
| `--help` output | Subcommand descriptions, flag docs |
| Default mode output | Table format, column headers |
| `--json` output | Valid JSON, correct fields |
| Error messages | Hint presence, exit code |
| Auto-resolution | Single account auto-pick, multi-account error |
| Empty results | "(none)" message, helpful hint |

## Checklist: Adding a New Command

When adding a new subcommand, follow this checklist:

1. Add a `///` doc comment on the enum variant (shows in `--help`)
2. Add `///` doc comments on non-obvious flags
3. Create a `print_<type>()` function in `output.rs` that handles both table and JSON
4. Add `hint(&[...])` after output for next-step suggestions (skip if `cli.json`)
5. For commands needing account/calendar, use the auto-resolve pattern
6. Default to sensible limits and "upcoming" time ranges
7. Return appropriate exit codes (2 for usage, 3 for auth, 4 for API)
8. Add integration tests for both default and `--json` output modes

## Reference

For deeper dives, see `references/sources.md` for the research sources this skill draws from (clig.dev, 12 Factor CLI Apps, kubectl conventions, gh CLI patterns, etc.).
