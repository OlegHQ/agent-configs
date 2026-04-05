//! Structured error types for the `deployr` CLI.
//!
//! Design principles (from the rust-cli-building skill):
//! - Every error answers three questions: **what** failed, **why**, and **what to do**.
//! - Variants carry enough context to produce a plain-English message — never
//!   expose raw `reqwest::Error` or `serde_json::Error` directly.
//! - Exit codes are a stable API contract (see `CliError::exit_code`).

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Auth errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("no credentials found (checked env vars, config file, and keychain)")]
    NoCredentials,

    #[error("access token has expired")]
    TokenExpired,

    #[error("token refresh failed: {reason}")]
    RefreshFailed { reason: String },

    #[error("cannot read credentials from {path}: {reason}")]
    CredentialRead { path: String, reason: String },

    #[error("cannot parse credential data ({context}): {detail}")]
    CredentialParse { context: String, detail: String },
}

// ---------------------------------------------------------------------------
// API / upstream errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("server returned HTTP {status} for {endpoint}: {body}")]
    Http {
        status: u16,
        endpoint: String,
        body: String,
    },

    #[error("request to {endpoint} timed out after {seconds}s")]
    Timeout { endpoint: String, seconds: u64 },

    #[error("cannot reach {host}: {reason}")]
    Network { host: String, reason: String },

    #[error("unexpected response from {endpoint}: {detail}")]
    ResponseParse { endpoint: String, detail: String },

    #[error("rate-limited by server (retry after {retry_after_secs}s)")]
    RateLimited { retry_after_secs: u64 },
}

// ---------------------------------------------------------------------------
// Config errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found at {}", path.display())]
    NotFound { path: PathBuf },

    #[error("cannot read config file {}: {reason}", path.display())]
    ReadFailed { path: PathBuf, reason: String },

    #[error("invalid config in {}: {detail}", path.display())]
    ParseFailed { path: PathBuf, detail: String },

    #[error("missing required config key \"{key}\"")]
    MissingKey { key: String },

    #[error("invalid value for \"{key}\": {detail}")]
    InvalidValue { key: String, detail: String },
}

// ---------------------------------------------------------------------------
// I/O errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("cannot read {}: {reason}", path.display())]
    ReadFailed { path: PathBuf, reason: String },

    #[error("cannot write {}: {reason}", path.display())]
    WriteFailed { path: PathBuf, reason: String },

    #[error("directory {} does not exist", path.display())]
    DirNotFound { path: PathBuf },

    #[error("permission denied on {}", path.display())]
    PermissionDenied { path: PathBuf },
}

// ---------------------------------------------------------------------------
// Top-level CLI error — the only type `main` works with
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{0}")]
    Auth(#[from] AuthError),

    #[error("{0}")]
    Api(#[from] ApiError),

    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("{0}")]
    Io(#[from] IoError),

    #[error("{0}")]
    Usage(String),
}

impl CliError {
    /// Stable exit code for this error category.
    ///
    /// | Code | Meaning       |
    /// |------|---------------|
    /// |  0   | Success       |
    /// |  2   | Usage error   |
    /// |  3   | Auth error    |
    /// |  4   | Upstream/IO   |
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Usage(_) => 2,
            CliError::Auth(_) => 3,
            CliError::Api(_) | CliError::Io(_) => 4,
            CliError::Config(_) => 2, // config problems are a form of usage/setup error
        }
    }

    /// Short stable tag for JSON output (`"code_name"` field).
    pub fn code_name(&self) -> &'static str {
        match self {
            CliError::Usage(_) => "usage_error",
            CliError::Auth(_) => "auth_error",
            CliError::Api(_) => "api_error",
            CliError::Config(_) => "config_error",
            CliError::Io(_) => "io_error",
        }
    }
}

// ---------------------------------------------------------------------------
// Hints — maps each error variant to an actionable suggestion
// ---------------------------------------------------------------------------

/// Returns a user-facing hint that tells them what to do about the error.
pub fn error_hint(e: &CliError) -> Option<&'static str> {
    match e {
        // Auth
        CliError::Auth(AuthError::NoCredentials) => {
            Some("run `deployr auth login` to authenticate, or set DEPLOYR_TOKEN")
        }
        CliError::Auth(AuthError::TokenExpired) => {
            Some("run `deployr auth refresh` to get a new token")
        }
        CliError::Auth(AuthError::RefreshFailed { .. }) => {
            Some("run `deployr auth login` to re-authenticate from scratch")
        }
        CliError::Auth(AuthError::CredentialRead { .. }) => {
            Some("check file permissions, or run `deployr auth login` to set up credentials again")
        }
        CliError::Auth(AuthError::CredentialParse { .. }) => {
            Some("credentials may be corrupted; run `deployr auth login` to reset")
        }

        // API
        CliError::Api(ApiError::Http { status, .. }) if *status == 401 || *status == 403 => {
            Some("your token may be invalid; run `deployr auth refresh` or `deployr auth login`")
        }
        CliError::Api(ApiError::Http { status, .. }) if *status >= 500 => {
            Some("the server is having issues; retry in a few minutes")
        }
        CliError::Api(ApiError::Timeout { .. }) => {
            Some("the server did not respond in time; check your connection and retry")
        }
        CliError::Api(ApiError::Network { .. }) => {
            Some("check your internet connection and DNS settings")
        }
        CliError::Api(ApiError::RateLimited { retry_after_secs }) => {
            Some(if *retry_after_secs > 0 {
                "wait for the retry period and try again"
            } else {
                "you are being rate-limited; wait a moment and retry"
            })
        }
        CliError::Api(ApiError::ResponseParse { .. }) => {
            Some("the server returned an unexpected format; check if deployr needs updating")
        }
        CliError::Api(ApiError::Http { .. }) => None,

        // Config
        CliError::Config(ConfigError::NotFound { .. }) => {
            Some("run `deployr init` to create a config file, or pass options via flags")
        }
        CliError::Config(ConfigError::ParseFailed { .. }) => {
            Some("check your config file syntax (TOML format)")
        }
        CliError::Config(ConfigError::MissingKey { key }) => {
            // The hint text is static so we give a generic pointer
            Some("add the missing key to your config file or pass it as a CLI flag")
        }
        CliError::Config(_) => None,

        // I/O
        CliError::Io(IoError::PermissionDenied { .. }) => {
            Some("check file/directory permissions, or run with appropriate access")
        }
        CliError::Io(IoError::DirNotFound { .. }) => {
            Some("create the directory first, or check the path for typos")
        }
        CliError::Io(_) => None,

        // Usage
        CliError::Usage(_) => Some("run `deployr --help` for usage information"),
    }
}
