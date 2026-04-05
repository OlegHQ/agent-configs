# Research Sources

This skill synthesizes CLI design best practices from these sources:

## Design Guidelines
- [Command Line Interface Guidelines (clig.dev)](https://clig.dev/) — comprehensive community-driven CLI design guide
- [12 Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46) — Jeff Dickey's principles for well-behaved CLIs
- [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide) — human-first CLI design from Heroku
- [10 Design Principles for Delightful CLIs (Atlassian)](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)

## CLI Exemplars
- **kubectl**: Resource-oriented verbs, `-o json/yaml/jsonpath`, `--dry-run`, consistent output modes
- **gh (GitHub CLI)**: Auto TTY detection, `--json field1,field2` with field selection, `--jq` inline filtering
- **docker**: Go template `--format`, consistent across all list/inspect commands
- **Heroku CLI**: `topic:command` pattern, `cli.action()` feedback, grep-parseable tables

## Rust Ecosystem
- [Rust CLI Book](https://rust-cli.github.io/book/) — official guide to building CLIs in Rust
- [clap documentation](https://docs.rs/clap/) — derive macro patterns, arg groups, subcommands
- [assert_cmd](https://docs.rs/assert_cmd/) — integration testing for CLI binaries
- [insta](https://docs.rs/insta/) — snapshot testing for output format stability
- [trycmd](https://docs.rs/trycmd/) — file-driven CLI test cases

## Error Handling
- [Effective Error Handling in Rust CLI Apps](https://technorely.com/insights/effective-error-handling-in-rust-cli-apps-best-practices-examples-and-advanced-techniques)
- [Error Handling in CLI Tools (Chloe Zhou)](https://medium.com/@czhoudev/error-handling-in-cli-tools-a-practical-pattern-thats-worked-for-me-6c658a9141a9)

## Key Patterns Extracted

### From clig.dev
- Humans first, machines second
- Show help when run without args
- Use color with intention, respect NO_COLOR
- Place important info at the end of output
- Suggest corrections for typos

### From 12 Factor CLI Apps
- Prefer flags over args
- Omit table borders
- Support --json for machine output
- Errors should include: code, title, description, fix instructions

### From kubectl
- Resource-oriented command structure
- Multiple output formats (-o json, -o yaml, -o wide)
- --dry-run for safe previews
- Be explicit in scripts, smart defaults in interactive use

### From gh CLI
- Auto-detect TTY vs pipe
- Tab-delimited when piped, tables when interactive
- --json with field selection
- Exit codes distinguish "not found" from "error"
