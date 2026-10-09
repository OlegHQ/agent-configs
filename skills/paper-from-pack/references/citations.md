# Citations

Rules
- Only works the pack names (prior-work block, data blocks). Not from memory, not from a search. A work not in the pack goes to OPEN.md as a question for the author ("consider adding X"), not into the draft.
- Each entry carries `eprint` (arXiv id) or `doi` when it has one, plus `packref = {phrase copied from the pack}` when the pack names it without an id (for example a tool or incident). `check_citations.py` rejects entries that the pack does not contain.
- Software tools, incident reports and blog posts without a DOI or arXiv id: cite by name, version and where the pack gives a location; if no URL is in the pack, put `TO VERIFY: URL` in the entry's `note` and OPEN.md. Such entries show as UNRESOLVED in the check; list them as unverified in OPEN.md.

When you copy title and authors into `refs.bib` from the saved index metadata, the title match is true by construction; what the check then proves is that the identifier exists and that the pack named it. Say so in OPEN.md.

Run `check_citations.py` (see SKILL.md). Statuses: VERIFIED, MISMATCH (title, first author or year differ), NOT_FOUND (identifier does not exist; treat as fabricated until proven otherwise), NOT_IN_PACK, UNRESOLVED (no identifier, no dblp hit, or lookup failed). It never edits the bib file. If the index disagrees with the bib entry or with the pack's description of the paper, tell the author in OPEN.md.

Manual step, `citations.tsv` (tab-separated, header `key<TAB>sentence_supported<TAB>read_depth<TAB>note`): for each cited key, the exact draft sentence it supports, and whether you read the source `full`, `abstract` (the saved metadata), `software` (a tool cited by name and version, no claim drawn from its content; only for entries without arXiv id or DOI), or `not-read`. A row is required for each key; `not-read` fails the check. Anything cited at abstract depth is listed in OPEN.md. Do not draw a claim from a paper beyond what the part you read says; if the pack quotes a number from a paper and you only read the abstract, check the number is in the abstract, otherwise say "reported in the pack, not verified in the abstract".

Prior-work text: distinguish, do not inflate. Say what the closest works do and what this piece adds, in the pack's gap wording. Never write "no other tool exists" or "first".
