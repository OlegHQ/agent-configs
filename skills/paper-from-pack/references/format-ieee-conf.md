# ieee-conf (IEEEtran conference)

Template: `assets/templates/ieee-conf.tex` (`\documentclass[conference]{IEEEtran}`, `cite`, `booktabs`, bibliography style `IEEEtran`). Two columns, 10 pt. Use `table*` for wide tables.
Limits: page count from `venue.json` (for example 1-4 pages). Check whether references count: `pages_include_refs` true means the whole PDF must fit; `4 + 1` means content 4, references on the extra page. Run `check_limits.py --pdf`.
Double-blind flag: keep the anonymous author block, no acknowledgements, no funding, third-person tool references, anonymised links, set no PDF metadata (the template does not). Write `identity.txt` (author, handles, employer, repository owner, tool owner names) and run `check_identity.py` on .tex, .bib, SUBMIT.md and the PDF.
Skeleton (position paper): Abstract (about 150 words) / Introduction (problem, position, contribution in one paragraph each) / Method (what was measured, versions, fixed labels) / Results (table + one paragraph per notable behaviour) / Requirements or argument (each tied to a result row) / Limitations / Related work / References. Follow the pack's structure when it gives one.
Fit: figures and tables are cheap in words but expensive in space; use `\small` tables via `tabular` with `\footnotesize` only if still legible; never `\vspace` hacks, never negative spacing, never a font or margin change. Cut order in drafting.md.
Bibliography: `\bibliographystyle{IEEEtran}`, `\bibliography{refs}`. Key style `author2026word`.
IEEE metadata: the template ships no copyright block; do not add one unless the call asks.
