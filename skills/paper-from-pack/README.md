# paper-from-pack

An agent skill that turns one generated "writer pack" (venue rules, angle, structure, data with numbers and file paths, prior work, interview material, bio, writing rules) into a submission-ready piece in the venue's format, then checks it mechanically and leaves every open point for the author.

It is plain `SKILL.md` plus scripts and reference files. It uses no hooks, no settings edits and no agent-specific features, so it works with Claude Code, Codex, Cursor Agent, OpenCode, Grok CLI and Antigravity.

## What it does

1. Intake from a tracker issue (`nudge issue get KEY -o json`) or a file; refuses packs marked WAITS, HOLD, IDEA or VERIFY unless the user overrides.
2. Compares the live call with the pack; stops if a hard limit changed.
3. Picks a format class from the pack's Format line.
4. Writes a claims ledger (`CLAIMS.md`) before prose.
5. Verifies citations against arXiv, Crossref and dblp using identifiers or titles only.
6. Drafts, builds, checks numbers, limits and anonymity.
7. Runs a prose cleanup (the `unslop` skill, then a paper-specific list) and proves numbers, citations and limitations did not change.
8. Runs a three-reviewer hostile review in the session.
9. Writes DRAFT, CLAIMS, OPEN, SUBMIT, REVIEW, `network.log`. It never submits and never contacts a venue.

## Format classes

| class | status |
|---|---|
| `ieee-conf` | implemented: template, compile (system TeX or Tectonic), page limits, double-blind check |
| `acm` | implemented: acmart sigconf template, compile, page limits (content + references) |
| `ieee-cs-magazine` | implemented as Markdown with word budget (cost per figure/table), abstract, reference count; no IEEE Word template |
| `springer-ccis` | implemented: llncs template and pattern form; compile tested on system TeX |
| `prose-md` | implemented: word limits |
| `proposal` | implemented: Markdown (or the IEEE/ACM class when the call needs it) |
| `outline-only` | implemented: outline, fact sheet, sources, no prose |

## Scripts (all run with `uv run`, inline metadata, no global installs)

| script | job |
|---|---|
| `scripts/pack.py` | intake, readiness gate, class and limits parse, working folder |
| `scripts/check_numbers.py` | every number in the draft must be in the pack or a results file it names; `--compare` for the prose pass |
| `scripts/check_claims.py` | every claim row sourced, anchors still in the draft |
| `scripts/check_citations.py` | existence and metadata check; pack membership; manual log; writes `network.log` |
| `scripts/check_limits.py` | PDF pages (content versus references), words with float cost, abstract, references |
| `scripts/check_identity.py` | double-blind identity strings, e-mails, code-hosting URLs, PDF metadata |
| `scripts/ensure_tex.sh` | system TeX first; pinned, SHA-256 verified Tectonic as fallback; Docker as documented alternative |

## Setup

- `uv` (https://docs.astral.sh/uv/).
- TeX for LaTeX classes. Recommended on Debian/Ubuntu:
  `sudo apt install texlive-latex-base texlive-latex-recommended texlive-latex-extra texlive-publishers texlive-fonts-recommended texlive-fonts-extra texlive-bibtex-extra texlive-science latexmk`
  Without TeX the skill downloads Tectonic 0.17.0 into `skills/paper-from-pack/.cache/` after a checksum check.
- The `unslop` skill (github.com/cursor/plugins, `pstack/skills/unslop`) is a dependency of the prose pass; the skill falls back to its own pattern list when it is absent.

## Install with agentpack, pinned to this branch

```toml
"github.com/OlegHQ/agent-configs/skills/paper-from-pack" = { ref = "feat/paper-from-pack" }
```
(After the merge use `ref = "dev"` or a tag.) Check the exact `ref` key spelling against your agentpack version.

## Tests

```
uv run --with pytest --with pypdf pytest skills/paper-from-pack/tests
```
They cover: a number not in the pack is caught; a number changed by the prose pass is caught; a fabricated DOI is flagged; an over-length PDF and an over-length abstract fail; an identity string in a double-blind draft is caught; plus intake, classification, limit parsing and the claims gate. Network calls are mocked.

## Evals

`evals/` holds synthetic packs, `evals.json`, a mechanical grader (`grade.py`) and a runner (`run_evals.py`) that launches a fresh agent session per case with and without the skill. See `evals/README.md` and `evals/RESULTS.md`.

## Confidentiality

Draft and pack text go only to the agent's own model. The only network use is the venue's call page, citation APIs with an identifier or title, and the pinned Tectonic download. See `references/confidentiality.md`.
