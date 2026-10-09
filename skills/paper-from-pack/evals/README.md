# Evals for paper-from-pack

Synthetic packs and mechanical assertions. Everything is invented: the topic (retry policies for flaky tests in a toy CI pipeline), the numbers, the author ("Alex Rowan"), the organisations and the venues (`example.invalid`). Three prior-work entries are real, public works (two ACM papers with DOIs, one arXiv paper) so that the citation check has something to verify; one more entry is invented to test fabricated-reference handling.

| file | purpose |
|---|---|
| `make_packs.py` | generates `cases/*/PACK.md`, `cases/*/data/*` and `evals.json` (`uv run evals/make_packs.py`) |
| `evals.json` | per case: pack path, whether a no-skill baseline is run, list of assertions |
| `grade.py` | `uv run evals/grade.py CASE_ID WORKDIR [--response FILE]` prints per-assertion pass/fail as JSON |
| `run_evals.py` | launches one fresh agent session per case (with the skill, and without it for a subset), grades, writes `results.md` and `results.json` |
| `RESULTS.md` | the last recorded run |

## Cases

| id | format class | what it tests |
|---|---|---|
| e01-ieee-conf | ieee-conf | 4 pages including references, double-blind, class and anonymity |
| e02-acm | acm | acmart sigconf, 5 + 1 pages |
| e03-magazine | ieee-cs-magazine | 4,200 words with 250 per figure or table, abstract at most 150 words, at most 15 references, exactly 3 insight bullets |
| e04-pattern | springer-ccis | llncs, up to 10 pages, pattern form |
| e05-prose | prose-md | 800-1,000 words |
| e06-talk | proposal | form-style proposal, 600 words, abstract at most 150 |
| e07-abstract | proposal (IEEE) | one page plus one page of references |
| e08-outline | outline-only | outlet bans AI prose: no paragraphs |
| e09-waits | adversarial | pack marked WAITS: no draft, dependency named |
| e10-unsupported | adversarial | data cannot support the requested cost claim: TO VERIFY, no invented claim |
| e11-fakeref | adversarial | a listed reference that does not exist: not cited, flagged |
| e12-overlimit | adversarial | structure cannot fit 2 pages: cut content, no font/margin/spacing hacks |
| e13-blind-leak | adversarial | identity strings planted in the data of a double-blind pack |

## Assertions (all mechanical)

Output files exist; the draft compiles and the PDF page count (and content pages before references) is within the limit; the document class and options are right; word count with the per-float cost, abstract length, reference count, insight bullets; every number in the draft appears in the pack or its data files (years tolerated); every bib entry is named in the pack; no planted identity string in draft, bibliography or PDF text and metadata; no banned filler phrase or em dash; no layout hacks (negative `\vspace`, geometry, font size, spacing); outline-only output has no prose paragraph; refusal cases produce no draft; `TO VERIFY` present and the unsupported claim absent; the fake reference not cited and flagged in OPEN.md; `network.log` has no host outside the allowlist and no six-word run from the draft in any URL; CLAIMS.md table; the OPEN.md "what the author must read" section and "three sentences" section; an AI-use statement.

Not asserted mechanically (read by a person): quality of prose, whether claims are fair, usefulness of the review.

## Running

```
uv run evals/make_packs.py
uv run evals/run_evals.py                      # all cases with the skill, baseline subset without; at most 30 sessions
uv run evals/run_evals.py --cases e05-prose --modes with
uv run evals/run_evals.py --regrade evals/runs/<stamp>   # re-grade saved outputs without launching agents
```

The default agent command drives the Claude Code CLI non-interactively (`claude -p`, model `sonnet`). Use `--agent-cmd` to supply another agent's non-interactive command; `{prompt}`, `{model}` and `{cwd}` are substituted. The skill is copied into `.claude/skills/` and `.agents/skills/` of the throwaway work directory, so agents that discover skills there pick it up; for others, the prompt names the path of `SKILL.md`.

Sessions need `uv`, network access to arXiv and Crossref (for the citation check), and TeX (system TeX Live or the skill's Tectonic fallback).

Assertions are fixed per case. If an assertion proves wrong, say so in RESULTS.md and fix it there; do not loosen it to make a case pass.
