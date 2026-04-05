# deployr exit codes

Exit codes are a stable API contract. Scripts and agents should branch on these values.

| Code | Name | Meaning | When |
|------|------|---------|------|
| 0 | `success` | Command completed successfully | Normal completion |
| 2 | `usage_error` | Bad flags, missing required input, invalid config | Clap parse failure, missing config key, invalid config value, validation error |
| 3 | `auth_error` | Authentication or authorization failure | No credentials found, expired token, refresh failed, corrupt credential file |
| 4 | `upstream_error` | API, network, or I/O failure | HTTP 4xx/5xx from server, DNS failure, timeout, rate-limiting, file read/write errors |

## Mapping from error types

| Error type | Variants | Exit code |
|------------|----------|-----------|
| `CliError::Usage` | (any) | 2 |
| `CliError::Config` | `NotFound`, `ReadFailed`, `ParseFailed`, `MissingKey`, `InvalidValue` | 2 |
| `CliError::Auth` | `NoCredentials`, `TokenExpired`, `RefreshFailed`, `CredentialRead`, `CredentialParse` | 3 |
| `CliError::Api` | `Http`, `Timeout`, `Network`, `ResponseParse`, `RateLimited` | 4 |
| `CliError::Io` | `ReadFailed`, `WriteFailed`, `DirNotFound`, `PermissionDenied` | 4 |

## JSON error format (when `--json` is set)

Errors are written to **stderr** as a single JSON object:

```json
{"error": "access token has expired", "code": 3, "code_name": "auth_error", "hint": "run `deployr auth refresh` to get a new token"}
```

| Field | Type | Description |
|-------|------|-------------|
| `error` | string | Human-readable error message |
| `code` | integer | Exit code (see table above) |
| `code_name` | string | Stable tag: `usage_error`, `auth_error`, `api_error`, `config_error`, `io_error` |
| `hint` | string or absent | Actionable suggestion for resolving the error |

## Human error format (default)

```
error: access token has expired
hint:  run `deployr auth refresh` to get a new token
```

The `hint:` line is omitted when no suggestion applies.
