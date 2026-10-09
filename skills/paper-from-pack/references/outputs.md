# Outputs (in the working folder named by the pack, normally drafts/<piece>/)

| File | Content |
|---|---|
| `DRAFT.tex` + `DRAFT.pdf`, or `DRAFT.md` | the piece in the venue's format; `outline-only`: `OUTLINE.md` instead |
| `refs.bib` | only pack-named works |
| `CLAIMS.md` | claims ledger (see claims.md) |
| `citations.tsv` | per citation: sentence supported, read depth |
| `OPEN.md` | see below |
| `SUBMIT.md` | title, abstract, keywords, bio (the pack's approved bio only), the venue's exact form fields (copy field names from the live page), the AI-use statement in the venue's required form using the pack's facts, conflict-of-interest and anonymity notes, supplementary files, declarations. Mark fields you could not confirm `TO VERIFY` |
| `REVIEW.md` | three reviewers, findings by type, automatic fixes |
| `network.log` | every request made |
| `venue.json`, `PACK.md`, `identity.txt`, `allow.txt`, `citations/` | working files |

`OPEN.md` sections, in this order:
1. Venue check (accessed date; differences; unconfirmed rules).
2. Every `TO VERIFY` in the draft, with the collection task.
3. Questions for the author.
4. Rules not confirmed.
5. How identity was handled (double-blind: what was anonymised and how).
6. Check results: the commands run and PASS/FAIL, including "not run" with a reason; class versions (IEEEtran, acmart) against the call's template; "numbers unchanged by the prose pass".
7. Cuts made to fit the limit.
8. The three sentences most likely to be challenged, each with its source and why it will be challenged.
9. **What the author must read and re-run before this is sent**: his first reads (the three sentences), the commands to re-run (data harness, scripts), the interview statements to confirm, anything cited at abstract depth.

Summary to the user: the files, the checks (pass/fail), number of TO VERIFY, anything unconfirmed. Never say it is ready to submit; say what remains for the author.
