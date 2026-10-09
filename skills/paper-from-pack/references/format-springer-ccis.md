# springer-ccis (Springer LNCS/CCIS, pattern papers)

Template: `assets/templates/springer-llncs.tex` (`llncs`, `runningheads`, style `splncs04`). Pattern workshops (PLoP family) are often non-anonymous and shepherded; check the call. The 2026-style limit is about 10 pages; read the live call.
Pattern form: for each pattern: name, context, problem, forces, solution, consequences, known uses (real and checkable in code or published sources), related patterns. A pattern paper reports no new measurements. "Known uses" without a source go to `TO VERIFY`.
Skeleton: Introduction and audience / pattern language overview (relationships) / one section per pattern / related patterns and prior work / acknowledgements (shepherd) / references.
Limits: `check_limits.py --pdf`. Keep Springer fonts; no spacing changes.
Fallback: if `llncs` cannot be built by either engine, produce `DRAFT.md` in pattern form and say the template step is stubbed.
