---
name: smdctl-deploy
description: Deploy applications as systemd user services using smdctl. Use when the user asks to "deploy a service", "run as a systemd service", "create a background service", "start on boot", "daemonize", mentions "smdctl", or wants to run an application persistently in the background on Linux.
---

# smdctl Deploy Skill

Deploy applications as persistent systemd user services. Zero sudo required for ports >= 1024.

## Quick Start

1. Create `smdctl.yml` in project root
2. Run `smdctl run -f smdctl.yml`
3. Verify with `smdctl status <name>`

If `tasks:` are present in `smdctl.yml`, `smdctl run -f smdctl.yml` will also create/enable/start the corresponding systemd timers.

## Configuration File (smdctl.yml)

### Minimal Example

```yaml
name: myapp
command: /path/to/binary
```

### Full Configuration Reference

```yaml
# Required
name: myapp                    # Service name (alphanumeric, -, _)
command: /usr/bin/node         # Executable path

# Optional - Common
args:                          # Command arguments
  - server.js
  - --port
  - "3000"
description: My Application    # Service description
workdir: /opt/myapp           # Working directory (default: /)
environment:                   # Environment variables
  PORT: "3000"
  NODE_ENV: production
  GITHUB_TOKEN: ${GITHUB_TOKEN}  # Reads from current shell env
  API_KEY: ${API_KEY}            # Missing vars become empty with warning
restart: always               # no | on-failure | always (default: always)

# Optional - Timeouts
timeout_start: 90             # Startup timeout seconds (default: 90)
timeout_stop: 30              # Stop timeout seconds (default: 30)

# Optional - Security (user mode)
private_tmp: true             # Isolate /tmp
no_new_privileges: true       # Prevent privilege escalation

# Optional - Resource Limits
limit_nofile: 65536           # File descriptor limit
tasks_max: 4096               # Max number of tasks

# Optional - Scheduled tasks (systemd timers)
# Each task becomes a oneshot .service + a .timer.
# Tasks inherit service workdir + env by default.
tasks:
  - name: cleanup                 # Task name (alphanumeric, -, _)
    description: Daily cleanup
    command: /usr/bin/bash        # Binary or script runner
    args: [/opt/myapp/cleanup.sh] # Script path or args
    schedule:
      on_calendar: "daily 03:30"  # Run daily at 03:30 local time
      persistent: true            # Default true

  - name: report
    command: /opt/myapp/bin/report
    schedule:
      on_unit_active_sec: 6h

# Optional - Dependencies
after:                        # Start after these targets
  - network-online.target
wants:                        # Soft dependencies
  - network-online.target

# Optional - System Mode (avoid unless needed)
system_mode: false            # Force system mode (requires sudo)
user: appuser                 # Run as user (system mode only)
protect_system: full          # no | strict | full (system mode)
kill_mode: control-group      # control-group | process | mixed
```

## Service Examples

### Go Binary

```yaml
name: mygoapp
command: /opt/mygoapp/server
workdir: /opt/mygoapp
environment:
  PORT: "8080"
  GIN_MODE: release
restart: always
```

### Node.js

```yaml
name: nodeapi
command: /usr/bin/node
args: [server.js]
workdir: /home/user/projects/api
environment:
  NODE_ENV: production
  PORT: "3000"
restart: always
timeout_start: 60
```

### Python

```yaml
name: flaskapp
command: /home/user/.venv/bin/python
args: [-m, gunicorn, -b, "0.0.0.0:5000", app:app]
workdir: /home/user/projects/flask
environment:
  FLASK_ENV: production
restart: always
```

### Background Worker

```yaml
name: worker
command: /opt/worker/bin/worker
args: [--queue, default, --concurrency, "4"]
workdir: /opt/worker
environment:
  REDIS_URL: redis://localhost:6379
restart: always
timeout_stop: 60
kill_mode: mixed
```

## Deployment Workflow

### Deploy New Service

```bash
# 1. Create config
# Write smdctl.yml in project directory

# 2. Deploy and start
smdctl run -f smdctl.yml

# 3. Verify
smdctl status myapp
smdctl logs -f myapp
```

### Update Existing Service

```bash
# Redeploy from YAML config (updates service in place, preserves env file)
smdctl run -f smdctl.yml

# Or just restart after code-only changes
smdctl restart myapp
```

**Note**: `smdctl run` updates the service file in place without removing the env file. Your customized environment variables are preserved.

### Environment Variable Updates

```bash
# Interactive editor for runtime env vars
smdctl env myapp
# Then restart to apply
smdctl restart myapp
```

### Environment Variable Substitution

Use `${VAR}` syntax in smdctl.yml to read from your current shell environment at deploy time:

```yaml
environment:
  GITHUB_TOKEN: ${GITHUB_TOKEN}
  DATABASE_URL: ${DATABASE_URL}
  PORT: "3000"  # Literal value
```

When you run `smdctl run -f smdctl.yml`, the current values of `$GITHUB_TOKEN` and `$DATABASE_URL` are read from your shell and written to the service's env file. Missing variables become empty strings with a warning.

## Essential Commands

| Command | Description |
|---------|-------------|
| `smdctl run -f smdctl.yml` | Deploy from config |
| `smdctl ps` | List running services |
| `smdctl ps -a` | List all services |
| `smdctl tasks` | List scheduled tasks (timers) |
| `smdctl tasks -a` | List all tasks (including inactive) |
| `smdctl tasks NAME` | List tasks for a service |
| `smdctl status NAME` | Detailed status |
| `smdctl logs NAME` | View logs |
| `smdctl logs -f NAME` | Follow logs |
| `smdctl restart NAME` | Restart service |
| `smdctl stop NAME` | Stop service |
| `smdctl rm NAME` | Remove service |
| `smdctl inspect NAME` | Show config (YAML) |
| `smdctl explain -f smdctl.yml` | Dry-run preview |

## File Locations (User Mode)

- Service files: `~/.config/systemd/user/smdctl-{name}.service`
- Env files: `~/.config/smdctl/env/{name}.env`
- Log files: `~/.config/smdctl/logs/{name}.log`

Task timers (from `tasks:` in YAML):

- Task timer units: `~/.config/systemd/user/smdctl-<svc>-task-<task>.timer`
- Task oneshot units: `~/.config/systemd/user/smdctl-<svc>-task-<task>.service`
- Task env files: `~/.config/smdctl/env/<svc>-task-<task>.env`
- Task log files: `~/.config/smdctl/logs/<svc>-task-<task>.log`

## Troubleshooting

### Service fails to start

```bash
# 1. Check status
smdctl status myapp

# 2. View logs
smdctl logs -n 100 myapp

# 3. Inspect config
smdctl inspect myapp

# 4. Dry-run to preview
smdctl explain -f smdctl.yml
```

### Common Issues

| Issue | Solution |
|-------|----------|
| Command not found | Use absolute path in `command` |
| Permission denied | Check file permissions, use user mode |
| Port already in use | Check with `ss -tlnp`, stop conflicting service |
| Service keeps restarting | Check logs, fix application errors |
| Env vars not loaded | Restart service after `smdctl env` changes |
