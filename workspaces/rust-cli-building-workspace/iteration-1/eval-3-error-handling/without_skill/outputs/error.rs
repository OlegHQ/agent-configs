//! Structured, user-friendly error types for the `deployr` CLI.
//!
//! Design goals:
//! - Every variant carries a plain-English message and an actionable hint.
//! - `Display` renders the human-readable form; `.to_json()` renders the
//!   machine-readable form.
//! - Each variant maps to a deterministic exit code (see `exit_code()`).

use std::fmt;

// ---------------------------------------------------------------------------
// Exit codes
// ---------------------------------------------------------------------------

/// Deterministic process exit codes.
///
/// Callers (scripts, CI, agents) can branch on these without parsing stderr.
pub mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const AUTH_ERROR: i32 = 2;
    pub const API_ERROR: i32 = 3;
    pub const CONFIG_ERROR: i32 = 4;
    pub const IO_ERROR: i32 = 5;
    pub const TIMEOUT_ERROR: i32 = 6;
    pub const RATE_LIMIT_ERROR: i32 = 7;
    pub const NOT_FOUND_ERROR: i32 = 8;
    pub const INVALID_INPUT_ERROR: i32 = 9;
}

// ---------------------------------------------------------------------------
// Core error type
// ---------------------------------------------------------------------------

/// Top-level error returned by every CLI command.
#[derive(Debug)]
pub enum CliError {
    /// Authentication or authorization failure (expired token, bad creds, etc.)
    Auth {
        message: String,
        hint: String,
    },

    /// Remote API returned a non-success response.
    Api {
        status: Option<u16>,
        message: String,
        hint: String,
    },

    /// Configuration file missing, malformed, or contains invalid values.
    Config {
        message: String,
        hint: String,
    },

    /// Filesystem / network I/O failure.
    Io {
        message: String,
        hint: String,
    },

    /// HTTP request timed out.
    Timeout {
        message: String,
        hint: String,
    },

    /// Server indicated rate limiting (HTTP 429).
    RateLimit {
        retry_after_secs: Option<u64>,
        message: String,
        hint: String,
    },

    /// Requested resource does not exist (HTTP 404 or equivalent).
    NotFound {
        resource: String,
        message: String,
        hint: String,
    },

    /// User supplied bad input (invalid flag combo, bad ID format, etc.)
    InvalidInput {
        message: String,
        hint: String,
    },
}

// ---------------------------------------------------------------------------
// Constructors — keep call sites terse
// ---------------------------------------------------------------------------

impl CliError {
    // -- Auth ---------------------------------------------------------------

    pub fn auth_expired() -> Self {
        Self::Auth {
            message: "Your session has expired.".into(),
            hint: "Run `deployr auth login` to re-authenticate.".into(),
        }
    }

    pub fn auth_missing() -> Self {
        Self::Auth {
            message: "No credentials found.".into(),
            hint: "Run `deployr auth login` or set DEPLOYR_TOKEN.".into(),
        }
    }

    pub fn auth_forbidden(detail: impl Into<String>) -> Self {
        Self::Auth {
            message: format!("Permission denied: {}", detail.into()),
            hint: "Check that your account has access to this resource.".into(),
        }
    }

    // -- API ----------------------------------------------------------------

    pub fn api(status: u16, body: impl Into<String>) -> Self {
        let body = body.into();
        Self::Api {
            status: Some(status),
            message: format!("API request failed (HTTP {status}): {body}"),
            hint: "Retry the command. If the problem persists, check https://status.notion.so or run with DEPLOYR_LOG=debug.".into(),
        }
    }

    pub fn api_decode(detail: impl Into<String>) -> Self {
        Self::Api {
            status: None,
            message: format!(
                "Could not understand the server response: {}",
                detail.into()
            ),
            hint: "The API may have changed. Update deployr (`deployr self-update`) or report a bug.".into(),
        }
    }

    // -- Config -------------------------------------------------------------

    pub fn config_missing(path: impl Into<String>) -> Self {
        let path = path.into();
        Self::Config {
            message: format!("Configuration file not found: {path}"),
            hint: format!("Run `deployr init` to create a default config at {path}."),
        }
    }

    pub fn config_parse(path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Config {
            message: format!(
                "Configuration file is malformed ({}): {}",
                path.into(),
                detail.into()
            ),
            hint: "Fix the syntax error, or delete the file and run `deployr init` to regenerate it.".into(),
        }
    }

    pub fn config_invalid(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Config {
            message: format!("Invalid config value for `{}`: {}", field.into(), reason.into()),
            hint: "Edit your config file or re-run `deployr init`.".into(),
        }
    }

    // -- I/O ----------------------------------------------------------------

    pub fn io(action: impl Into<String>, inner: std::io::Error) -> Self {
        Self::Io {
            message: format!("{}: {}", action.into(), inner),
            hint: "Check file permissions and available disk space.".into(),
        }
    }

    pub fn io_network(detail: impl Into<String>) -> Self {
        Self::Io {
            message: format!("Network error: {}", detail.into()),
            hint: "Check your internet connection and any proxy settings (HTTPS_PROXY).".into(),
        }
    }

    // -- Timeout ------------------------------------------------------------

    pub fn timeout(url: impl Into<String>) -> Self {
        Self::Timeout {
            message: format!("Request timed out: {}", url.into()),
            hint: "Retry the command. You can increase the timeout with --timeout <seconds>.".into(),
        }
    }

    // -- Rate limit ---------------------------------------------------------

    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        let hint = match retry_after {
            Some(secs) => format!("Wait {secs} seconds and retry."),
            None => "Wait a moment and retry. Reduce request frequency if this recurs.".into(),
        };
        Self::RateLimit {
            retry_after_secs: retry_after,
            message: "Rate limit exceeded.".into(),
            hint,
        }
    }

    // -- Not found ----------------------------------------------------------

    pub fn not_found(resource: impl Into<String>) -> Self {
        let resource = resource.into();
        Self::NotFound {
            message: format!("{resource} not found."),
            hint: "Verify the ID or name is correct. Use `deployr list` to see available resources.".into(),
            resource,
        }
    }

    // -- Invalid input ------------------------------------------------------

    pub fn invalid_input(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
            hint: hint.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Exit code mapping
// ---------------------------------------------------------------------------

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Auth { .. } => exit_code::AUTH_ERROR,
            Self::Api { .. } => exit_code::API_ERROR,
            Self::Config { .. } => exit_code::CONFIG_ERROR,
            Self::Io { .. } => exit_code::IO_ERROR,
            Self::Timeout { .. } => exit_code::TIMEOUT_ERROR,
            Self::RateLimit { .. } => exit_code::RATE_LIMIT_ERROR,
            Self::NotFound { .. } => exit_code::NOT_FOUND_ERROR,
            Self::InvalidInput { .. } => exit_code::INVALID_INPUT_ERROR,
        }
    }

    /// Short category tag used in JSON output and logs.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Auth { .. } => "auth_error",
            Self::Api { .. } => "api_error",
            Self::Config { .. } => "config_error",
            Self::Io { .. } => "io_error",
            Self::Timeout { .. } => "timeout_error",
            Self::RateLimit { .. } => "rate_limit_error",
            Self::NotFound { .. } => "not_found_error",
            Self::InvalidInput { .. } => "invalid_input_error",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Auth { message, .. }
            | Self::Api { message, .. }
            | Self::Config { message, .. }
            | Self::Io { message, .. }
            | Self::Timeout { message, .. }
            | Self::RateLimit { message, .. }
            | Self::NotFound { message, .. }
            | Self::InvalidInput { message, .. } => message,
        }
    }

    fn hint(&self) -> &str {
        match self {
            Self::Auth { hint, .. }
            | Self::Api { hint, .. }
            | Self::Config { hint, .. }
            | Self::Io { hint, .. }
            | Self::Timeout { hint, .. }
            | Self::RateLimit { hint, .. }
            | Self::NotFound { hint, .. }
            | Self::InvalidInput { hint, .. } => hint,
        }
    }
}

// ---------------------------------------------------------------------------
// Human display  (stderr)
// ---------------------------------------------------------------------------

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}\nhint: {}", self.message(), self.hint())
    }
}

impl std::error::Error for CliError {}

// ---------------------------------------------------------------------------
// JSON display  (--json mode)
// ---------------------------------------------------------------------------

impl CliError {
    /// Render as a single-line JSON object. No serde dependency required for
    /// this small fixed schema — we format manually to keep the error crate
    /// dependency-free. If serde_json is already in your tree, feel free to
    /// swap this for a `Serialize` derive.
    pub fn to_json(&self) -> String {
        // Escape backslashes and double-quotes in dynamic strings.
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\").replace('"', "\\\"")
        }

        let mut obj = format!(
            r#"{{"error":true,"kind":"{}","exit_code":{},"message":"{}","hint":"{}""#,
            self.kind(),
            self.exit_code(),
            esc(self.message()),
            esc(self.hint()),
        );

        // Append variant-specific fields.
        if let Self::Api {
            status: Some(code), ..
        } = self
        {
            obj.push_str(&format!(r#","http_status":{code}"#));
        }
        if let Self::RateLimit {
            retry_after_secs: Some(secs),
            ..
        } = self
        {
            obj.push_str(&format!(r#","retry_after_secs":{secs}"#));
        }
        if let Self::NotFound { resource, .. } = self {
            obj.push_str(&format!(r#","resource":"{}""#, esc(resource)));
        }

        obj.push('}');
        obj
    }
}

// ---------------------------------------------------------------------------
// From impls — wrap common library errors automatically
// ---------------------------------------------------------------------------

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            message: format!("I/O error: {e}"),
            hint: "Check file permissions and available disk space.".into(),
        }
    }
}

/// Convert a `reqwest::Error` into the most specific `CliError` variant we
/// can infer from the error metadata.
#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            return Self::Timeout {
                message: format!("Request timed out: {}", e.url().map_or("-".into(), |u| u.to_string())),
                hint: "Retry the command. You can increase the timeout with --timeout <seconds>.".into(),
            };
        }
        if e.is_connect() {
            return Self::io_network(e.to_string());
        }
        if let Some(status) = e.status() {
            let code = status.as_u16();
            return match code {
                401 | 403 => Self::Auth {
                    message: format!("Authentication failed (HTTP {code})."),
                    hint: "Run `deployr auth login` to re-authenticate.".into(),
                },
                404 => Self::NotFound {
                    resource: e.url().map_or("unknown".into(), |u| u.path().to_string()),
                    message: format!("Resource not found (HTTP 404)."),
                    hint: "Verify the ID or name. Use `deployr list` to see available resources.".into(),
                },
                429 => Self::rate_limited(None),
                _ => Self::api(code, e.to_string()),
            };
        }
        if e.is_decode() {
            return Self::api_decode(e.to_string());
        }
        // Fallback
        Self::Io {
            message: format!("HTTP client error: {e}"),
            hint: "Check your network connection and retry.".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_stable() {
        assert_eq!(CliError::auth_expired().exit_code(), 2);
        assert_eq!(CliError::api(500, "oops").exit_code(), 3);
        assert_eq!(CliError::config_missing("/a").exit_code(), 4);
        assert_eq!(CliError::timeout("https://x").exit_code(), 6);
        assert_eq!(CliError::rate_limited(Some(30)).exit_code(), 7);
        assert_eq!(CliError::not_found("event/abc").exit_code(), 8);
    }

    #[test]
    fn display_includes_hint() {
        let e = CliError::auth_expired();
        let s = e.to_string();
        assert!(s.contains("error:"));
        assert!(s.contains("hint:"));
    }

    #[test]
    fn json_output_is_valid() {
        let e = CliError::api(502, "bad gateway");
        let j = e.to_json();
        assert!(j.starts_with('{'));
        assert!(j.ends_with('}'));
        assert!(j.contains(r#""error":true"#));
        assert!(j.contains(r#""http_status":502"#));
    }

    #[test]
    fn json_escapes_quotes() {
        let e = CliError::invalid_input(
            r#"field "name" is required"#,
            r#"pass --name "value""#,
        );
        let j = e.to_json();
        // Should not break JSON structure.
        assert!(j.contains(r#"\"name\""#));
    }
}
