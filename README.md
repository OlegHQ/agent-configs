# agent-configs

Shared [Cursor](https://cursor.com/docs/plugins.md) / Claude-style plugin trees, **agentpack** packages, and standalone skills for AI-assisted development.

Repository: [github.com/OlegHQ/agent-configs](https://github.com/OlegHQ/agent-configs)

## Layout

| Path | What it is |
| --- | --- |
| [`sample-plugin/`](sample-plugin/) | Reference plugin: rules, skills, agents, commands, hooks, MCP stub, dual Cursor/Claude manifests, `agentpack.toml`. See its [README](sample-plugin/README.md) for local install and debugging. |
| [`go-skills-plugin/`](go-skills-plugin/) | Go-focused skill bundle (architecture, testing, HTMX, Templ, SSE, etc.). Versioned via `agentpack.toml` (`name = "go-skills"`). |
| [`skills/`](skills/) | Standalone skills (e.g. deployment, tutoring) you can copy or wire into a plugin. Each skill lives in `<name>/SKILL.md` with YAML frontmatter. |
| [`commands/`](commands/) | Command definitions (e.g. session reflection) with frontmatter for tool agents. |

## Using this repo

- **As a plugin:** Symlink or copy a plugin directory (for example `sample-plugin`) into your editor’s local plugins path and reload the window. Details are in [`sample-plugin/README.md`](sample-plugin/README.md).
- **With agentpack:** Point `agentpack.toml` in another project at a path or published key for `sample-plugin` or `go-skills-plugin`, then run your usual `agentpack lock` / `agentpack sync` workflow.

## Contributing

When adding skills or plugin assets, match the patterns already in this tree (frontmatter, folder names, and manifest fields). Keep `sample-plugin` suitable as a minimal reference layout unless the goal is explicitly to extend that reference.

## See also

- [AGENTS.md](AGENTS.md) — guidance for AI agents working in this repository.
- [Cursor plugins reference](https://cursor.com/docs/reference/plugins.md)
