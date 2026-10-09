# CLAIMS.md

Written before prose. One Markdown table, columns exactly: `ID | Type | Claim | Source | Anchor`.

| Type | Meaning | Source must be |
|---|---|---|
| fact | something the data or history shows | pack block heading, results file with line (`RESULTS.md:57`), commit hash, or file and line |
| number | a figure copied from data | file:line where the figure appears, with n |
| citation | what a cited work says | key, and the sentence of that work or its abstract; read depth |
| limitation | a limit the data file states | file:line; anchor mandatory |
| interview | the author's words or experience | the pack's interview block, date |
| inference | what you conclude from facts | the fact IDs it rests on |
| strategy | a recommendation or position | the inference IDs it rests on; label as the author's position |

Anchor: a short verbatim phrase (3-8 words) that appears in the draft. `check_claims.py` fails when it is missing, so a prose pass that drops a limitation or a claim is caught. Re-anchor after a pass only if the meaning is unchanged; record it.

Rules:
- No row for a sentence you cannot source; that sentence becomes `TO VERIFY` or goes.
- Keep fact, inference, strategy in different rows. A position paper's requirements are `strategy`, each tied to the `fact` rows that motivate them.
- Numbers: copy with n ("51 of 192"). No averages without a spread. Derived numbers (sums, ratios) are allowed only as an `inference` row that shows the arithmetic; add them to the allowlist with that row's ID as the reason.
- Contradictions between pack and results file: follow the results file, add an `OPEN.md` entry.
