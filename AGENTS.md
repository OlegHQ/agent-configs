# AGENTS.md

Instructions for AI coding agents (Cursor, Codex, Claude Code, etc.) when editing **agent-configs**.

## Repository purpose

This repo holds **reusable agent configuration**: Cursor/Claude plugin layouts, **agentpack**-ready packages, optional standalone **skills**, and **commands**. Changes should stay consistent with how each subtree is consumed (editor discovery, agentpack staging, or copy-paste).

## Directory roles

- **`sample-plugin/`** — Reference layout. Preserve it as a small, documented exemplar unless the task is to change that reference. It documents symlink install, hooks, and MCP placeholders in its own README.
- **`go-skills-plugin/`** — Go ecosystem skills only. New Go-related skills belong under `go-skills-plugin/skills/<skill-id>/SKILL.md`. Bump or align `go-skills-plugin/agentpack.toml` when versioning matters for consumers.
- **`skills/`** — Cross-cutting or non-Go skills with `SKILL.md` + YAML frontmatter (`name`, `description`; follow existing files).
- **`commands/`** — Command markdown with frontmatter (`description`, optional `allowed-tools`, `argument-hint`). Mirror the style of `commands/reflect.md`.

## Conventions

1. **Skills:** Use `---` frontmatter at the top of `SKILL.md`. The `description` should state when to load the skill (triggers, keywords, “use when…”).
2. **Plugins:** Cursor manifest is under `.cursor-plugin/plugin.json`; Claude under `.claude-plugin/plugin.json` where both exist. Respect documented `logo` paths and default discovery folders unless explicitly overriding paths in the manifest.
3. **Hooks/scripts:** If you add hook scripts, they must be executable where the repo expects that; prefer no-op or clearly safe behavior for templates.
4. **MCP:** Do not turn placeholders into real credentials or production servers without explicit user direction.

## Scope of edits

- Prefer **minimal diffs** that match surrounding style.
- Do not rename plugin package directories or manifest `name` fields casually — that can break users’ symlinks and agentpack keys.
- Avoid adding repo-wide docs the user did not ask for; root `README.md` and this file are the primary navigation.

## Verification

After structural changes to a plugin, sanity-check that paths referenced from `plugin.json` (e.g. `logo`, custom rule/skill dirs) still exist.
