# agent-configs

Shared [Cursor](https://cursor.com/docs/plugins.md) / Claude-style plugin trees, **agentpack** packages, and standalone skills for AI-assisted development.

Repository: [github.com/OlegHQ/agent-configs](https://github.com/OlegHQ/agent-configs)

## Layout

| Path | What it is |
| --- | --- |
| [`plugins/sample-plugin/`](plugins/sample-plugin/) | Reference plugin: rules, skills, agents, commands, hooks, MCP stub, dual Cursor/Claude manifests, `agentpack.toml`. See its [README](plugins/sample-plugin/README.md) for local install and debugging. |
| [`plugins/go-skills-plugin/`](plugins/go-skills-plugin/) | Go-focused skill bundle (architecture, testing, HTMX, Templ, SSE, etc.). Versioned via `agentpack.toml` (`name = "go-skills"`). |
| [`plugins/rust-dev/`](plugins/rust-dev/) | Rust-focused skills (design patterns, modularity, performance). Versioned via `agentpack.toml` (`name = "rust-dev"`). |
| [`plugins/simplifier/`](plugins/simplifier/) | Compact plugin (agents and agentpack stub). |
| [`skills/`](skills/) | Standalone skills you can copy or wire into a plugin. Each skill lives in `<skill-id>/SKILL.md` with YAML frontmatter. |
| [`skills/nudge-cli/`](skills/nudge-cli/) | Public Nudge CLI skill covering typed arguments, issue/project workflows, documents and databases, agents, workspace configuration, and imports. Copy the entire directory to retain its references. |
| [`commands/`](commands/) | Command definitions (e.g. session reflection) with frontmatter for tool agents. |
| [`workspaces/`](workspaces/) | Eval and benchmark workspaces for skills; not part of consumer plugin payloads. |

## Where to put a new skill

- **Go** → `plugins/go-skills-plugin/skills/<skill-id>/SKILL.md`
- **Rust** (this bundle) → `plugins/rust-dev/skills/<skill-id>/SKILL.md`
- **Cross-cutting** → `skills/<skill-id>/SKILL.md`
- **Reference / demo** → `plugins/sample-plugin/skills/<skill-id>/SKILL.md`

Details and conventions: [AGENTS.md](AGENTS.md).

## Using this repo

- **As a plugin:** Symlink or copy a directory under `plugins/` (for example `plugins/sample-plugin`) into your editor’s local plugins path and reload the window. Details are in [`plugins/sample-plugin/README.md`](plugins/sample-plugin/README.md).
- **With agentpack:** Point `agentpack.toml` in another project at a path or published key for `plugins/sample-plugin`, `plugins/go-skills-plugin`, `plugins/rust-dev`, etc., then run your usual `agentpack lock` / `agentpack sync` workflow.

## Contributing

When adding skills or plugin assets, match the patterns already in this tree (frontmatter, folder names, and manifest fields). Keep `plugins/sample-plugin` suitable as a minimal reference layout unless the goal is explicitly to extend that reference.

## See also

- [AGENTS.md](AGENTS.md) — guidance for AI/people working in this repository.
- [Cursor plugins reference](https://cursor.com/docs/reference/plugins.md)
