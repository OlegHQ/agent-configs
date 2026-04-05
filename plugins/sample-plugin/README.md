# sample-plugin

Reference [Cursor plugin](https://cursor.com/docs/reference/plugins.md) layout plus **`agentpack.toml`** for nested-package debugging. Use to validate discovery, hooks, MCP file presence, and **agentpack** staging.

## Contents

| Path | Purpose |
| --- | --- |
| `.cursor-plugin/plugin.json` | Cursor manifest (`logo` → `assets/logo.svg`) |
| `.claude-plugin/plugin.json` | Claude manifest (agentpack normalizes both) |
| `agentpack.toml` | Nested package stub (`[dependencies]` optional) |
| `rules/*.mdc` | `alwaysApply` + `globs` samples |
| `skills/sample-skill/SKILL.md` | Skill discovery |
| `agents/sample-agent.md` | Agent (default `agents/`) |
| `commands/` | `.md` with frontmatter + `.txt` (discovery) |
| `hooks/hooks.json` | Hook events → `scripts/*.sh` |
| `scripts/*.sh` | **No-op** hooks (safe for debugging) |
| `assets/logo.svg` | Logo for manifest `logo` field |
| `mcp.json` | **`mcpServers`** placeholder (replace before real MCP use) |
| `.claude/agents/` | Extra agent path for **agentpack** merges |
| `.opencode/commands/` | OpenCode-style command for **agentpack** merges |

### Custom manifest paths

This plugin relies on **default folder discovery** (no `rules` / `skills` paths in `plugin.json`). To test manifest overrides, add e.g. `"rules": "./rules/"` in `.cursor-plugin/plugin.json` (it replaces default scanning for that component per Cursor docs).

## Try in Cursor (local)

1. Symlink or copy this directory:

   ```bash
   cd /path/to/agent-configs/plugins/sample-plugin
   ln -sf "$(pwd)" ~/.cursor/plugins/local/sample-plugin
   ```

2. Restart Cursor or **Developer: Reload Window**.

3. Check Settings → Rules / Skills for entries from this plugin.

### Hooks and scripts

Hook scripts must be executable:

```bash
chmod +x scripts/*.sh
```

Hooks are **no-ops** (`exit 0`) for safe debugging.

### MCP placeholder

`mcp.json` defines **`sample-placeholder`** with `command: /bin/true` so the file is valid JSON and parseable. It is **not** a real MCP server. Replace with a real [`mcpServers`](https://cursor.com/docs/mcp.md) entry before relying on MCP.

## Try with agentpack

From another project with `agentpack.toml`, add this tree as a **path** dependency per your agentpack docs (or copy into `AGENTPACK_HOME/local/...` and use a `github.com/...` key if mirrored). Then run **`agentpack lock`** and **`agentpack sync`**, and inspect staged output under your staging root (`$AGENTPACK_STAGING_ROOT`).

## Reference

- [Plugins reference (structure)](https://cursor.com/docs/reference/plugins.md)
- [Plugins overview](https://cursor.com/docs/plugins.md)
