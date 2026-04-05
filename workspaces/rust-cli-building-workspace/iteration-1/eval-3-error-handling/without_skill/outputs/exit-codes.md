# deployr exit codes

Deterministic exit codes for scripting, CI, and agent automation. Branch on the
numeric code without parsing stderr.

| Code | Constant             | Meaning                                                    | Example cause                                              |
|------|----------------------|------------------------------------------------------------|------------------------------------------------------------|
| 0    | `SUCCESS`            | Command completed successfully.                            | -                                                          |
| 1    | `GENERAL_ERROR`      | Unclassified failure (bug, panic, unexpected state).       | Internal assertion, panic in release build.                 |
| 2    | `AUTH_ERROR`         | Authentication or authorization failure.                   | Expired token, missing credentials, HTTP 401/403.          |
| 3    | `API_ERROR`          | Remote API returned a non-success response.                | HTTP 500, unexpected response body, decode failure.        |
| 4    | `CONFIG_ERROR`       | Configuration missing, malformed, or has invalid values.   | Missing config file, TOML parse error, bad field value.    |
| 5    | `IO_ERROR`           | Filesystem or network I/O failure.                         | File not readable, DNS resolution failure, connection refused. |
| 6    | `TIMEOUT_ERROR`      | HTTP request timed out.                                    | Server did not respond within the deadline.                |
| 7    | `RATE_LIMIT_ERROR`   | Server indicated rate limiting (HTTP 429).                 | Too many requests; `retry_after_secs` may be present in JSON output. |
| 8    | `NOT_FOUND_ERROR`    | Requested resource does not exist.                         | HTTP 404, invalid event/calendar ID.                       |
| 9    | `INVALID_INPUT_ERROR`| User supplied bad input.                                   | Incompatible flags, malformed ID, missing required arg.    |

## Usage in scripts

```bash
deployr events list --json
rc=$?
case $rc in
  0) echo "ok" ;;
  2) echo "re-auth needed"; deployr auth login ;;
  7) echo "rate limited, backing off"; sleep 30 ;;
  *) echo "failed with exit code $rc" ;;
esac
```

## JSON error shape (when `--json` is set)

All errors produce a single JSON object on **stdout**:

```json
{
  "error": true,
  "kind": "auth_error",
  "exit_code": 2,
  "message": "Your session has expired.",
  "hint": "Run `deployr auth login` to re-authenticate."
}
```

Variant-specific fields appear when applicable:

| Field              | Present when           | Type     |
|--------------------|------------------------|----------|
| `http_status`      | `kind == "api_error"`  | integer  |
| `retry_after_secs` | `kind == "rate_limit_error"` | integer |
| `resource`         | `kind == "not_found_error"` | string  |

## Human-readable error shape (default, on stderr)

```
error: Your session has expired.
hint: Run `deployr auth login` to re-authenticate.
```

Two lines, always prefixed with `error:` and `hint:`. Scripts that parse
stderr can split on the first colon + space.
