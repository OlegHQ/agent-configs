# Eval results, first run

Date 2026-10-09. Agent: Claude Code CLI non-interactive (`claude -p`), model alias `sonnet` (Claude Sonnet 5.5), `acceptEdits` permission mode, tools Bash/Read/Write/Edit/Glob/Grep/WebFetch, one fresh session per case in a throwaway directory holding only the synthetic pack. With-skill sessions had the skill copied to `.claude/skills/` and `.agents/skills/` of that directory. Sessions run: 26 of the allowed 30 (1 pilot on e09, 13 with-skill, 10 baseline, 2 with-skill re-runs after a fix). Parallelism 5. One session per cell, so there is no variance estimate.

Baseline subset (10 of 13 cases, same pack, no skill, prompt asks for the deliverables the pack lists): e01, e03, e05, e06, e08, e09, e10, e11, e12, e13. Not run without the skill: e02, e04, e07 (budget).

## Table (all assertions mechanical; see README.md)

"Baseline (strict)" requires the files named `DRAFT.tex`/`DRAFT.md` that the skill produces. The baseline prompt did not name files, and the baseline agent used its own names (for example `paper.tex`), so strict grading charges it for naming. "Baseline (lenient)" accepts any `.tex`/`.md` as the draft (`--lenient-names`); it is the fairer comparison.

| case | with skill | baseline (strict) | baseline (lenient) | wall time s (with / baseline) |
|---|---|---|---|---|
| e01-ieee-conf | 14/14 | 11/14 | 12/14 | 159 / 62 |
| e02-acm | 15/15 | not run | not run | 151 / - |
| e03-magazine | 13/13 | 7/13 | 12/13 | 141 / 78 |
| e04-pattern | 14/14 | not run | not run | 122 / - |
| e05-prose | 10/10 | 7/10 | 9/10 | 138 / 78 |
| e06-talk | 10/10 | 6/10 | 9/10 | 80 / 37 |
| e07-abstract | 13/13 | not run | not run | 147 / - |
| e08-outline | 7/7 | 7/7 | 7/7 | 64 / 42 |
| e09-waits | 2/2 | 0/2 | 0/2 | 11 / 53 |
| e10-unsupported | 13/13 | 9/13 | 12/13 | 139 / 64 |
| e11-fakeref | 13/13 | 10/13 | 10/13 | 127 / 57 |
| e12-overlimit | 14/14 | 11/14 | 13/14 | 115 / 64 |
| e13-blind-leak | 13/13 | 10/13 | 10/13 | 132 / 59 |

Totals. With skill: 151/151 assertions (100%), 13/13 cases fully passing (after the grader fix below; first grading gave 150/151, 12/13). Baseline strict: 78/109 (72%), 1/10 cases fully passing. Baseline lenient: 94/109 (86%), 1/10 cases fully passing. The same 10 cases with the skill: 100%.

Raw per-assertion results: `results.json` (with skill and strict baseline), `results-baseline-lenient.json`.

## What the baseline failed (lenient grading)
- e09 (pack marked WAITS): wrote a paper anyway and never mentioned the WAITS status. This is the clearest difference: the skill refused in 11 s.
- e12 (hard 2-page limit): no remaining failure except missing CLAIMS/refs form (see below). The original strict failure on `no_layout_hacks` was a wrong assertion: my hack list contained `microtype`, which the baseline loaded; microtype is not a way of shrinking the layout. I removed it from the list (disclosed here; no with-skill result changed) and re-graded.
- e10 (data cannot support a cost claim): no `TO VERIFY` marker at all, although it did not state the unsupported number.
- e11 (fake reference): cited with inline bibliography and no CLAIMS.md; the fake reference itself was not cited, so that assertion passed.
- e13 (double-blind with planted identity strings): no identity leak in the draft, but no CLAIMS.md and an inline bibliography.
- e05: 699 words against the 800-1,000 range.
- Several: ordinals in reference text ("22nd", "40th") counted as numbers not in the pack, and "cites without refs.bib" for inline `thebibliography`. Both are strict readings of the assertions. They are noted, not loosened.
- The baseline passed e08 (outline only) as well as the skill: the pack itself says the outlet bans AI prose, and a plain agent obeyed it.

## Failures with the skill
None after the fixes below. In the first grading, one assertion failed (e04, `contains_count`).

## What was fixed, and what was not
1. **Grader defect (not a skill defect).** `grade.py` had no implementation for the assertion type `contains_count` used by e04, so it reported "unknown assertion type" and failed. I implemented the type with the threshold already in `evals.json` (at least 3 occurrences of "forces") and re-graded the saved e04 output: 14/14. The assertion itself was not changed.
2. **Skill defect found in transcripts, not by an assertion.** In the e11 and e12 sessions the agent reported a citation MISMATCH for a real paper, because Crossref returns that title as `D <scp>e</scp> F <scp>laker</scp>` with the remainder in a `subtitle` field. `check_citations.py` now strips markup, includes the subtitle, compares letters and digits only, and accepts containment of a title of 8+ characters. Tests added (26 pass). e11 and e12 were re-run with the skill after the first half of the fix (markup stripping): 13/13 and 14/14. The second half (subtitle and containment) was verified by running the fixed script on the saved e12 draft's `refs.bib` (all entries VERIFIED), not by another agent session.
3. **Wrong assertion corrected.** `microtype` removed from the layout-hack list (see e12 above). This raised the baseline, not the skill.
4. **Grader option added.** `--lenient-names` for fairer baseline grading (see above). The strict numbers are kept in the table.

Not fixed: nothing known. Weak points I did not test: other agents than Claude Code (the runner accepts `--agent-cmd`, untried); network-less operation; Tectonic fallback inside a session (system TeX was present); a real double-blind leak caused by the agent writing the author name in SUBMIT.md (not graded, since SUBMIT.md holds form fields).

## Caveats
One run per cell. The assertions were written by the skill's author after reading the skill, so they favour what the skill does; they do not grade prose quality, argument, or the usefulness of the hostile review. The with-skill sessions produced REVIEW.md in all but the refusal case, but its content is not graded. The baseline prompt did not name output files, which is why the strict comparison is harsher than the lenient one.
