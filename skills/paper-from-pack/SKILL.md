---
name: paper-from-pack
description: Turn one generated "Writer pack" (venue rules, angle, structure, data with numbers and file paths, prior work, interview material, bio, writing rules) from a tracker issue or a file into a submission-ready paper, article, talk proposal or outline in the venue's format, without inventing anything. Checks numbers, citations, limits and anonymity with scripts, cleans the prose, runs a three-reviewer hostile review, and leaves every open point for the author. Use when the user points you at an issue or file containing a writer pack, or says "write the piece from this pack". Never submits and never contacts a venue.
---

# paper-from-pack

Plain Markdown plus portable scripts. No hooks, no settings edits, no agent-specific features. Scripts run with `uv run` (inline metadata). Every script path below is relative to this skill's directory; call them as `uv run <skill_dir>/scripts/<name>.py`.

The author reads and corrects everything before use. Your job is a draft he can defend sentence by sentence, with every gap visible. A smaller true claim beats a bigger built one.

## Hard rules

1. Invent nothing: no number, date, quote, citation, relationship, result, affiliation. A gap becomes `TO VERIFY: <what>` in the draft and a line in OPEN.md.
2. Use only the pack and the files it names. The pack's "Must not", "Owns" and "May cite" blocks bind you. Data under "May cite" is at most one or two sentences, and only if the pack says its pointer is public.
3. Confidentiality (see [references/confidentiality.md](references/confidentiality.md)): pack text and draft text go only to your own model. Allowed network: the venue's call pages, citation APIs (arXiv, Crossref, dblp) with an identifier or a title only, and the pinned Tectonic download. Log every request in `network.log`. Never read `.env` files or credentials, never install anything globally, never edit agent settings, no telemetry.
4. Never submit, never email, never post, never write to the tracker unless the user asks.
5. Do not shrink fonts, margins or spacing to fit. Cut content. If the pack's structure cannot fit, cut sections in the order given in the format reference and record each cut in OPEN.md.
6. Double-blind flag set: no author name, no repository owner, no employer, no "our earlier work", anonymised links, tool owned by the author referred to neutrally.
7. The AI-use statement stays. The prose pass is for quality, never for hiding AI use. Outlets that ban or screen AI prose get `outline-only`.

## Workflow

Create the working folder first (step 1) and keep all outputs in it. Do the steps in order; stop where a step says stop.

### 1. Intake
```
uv run scripts/pack.py intake <ISSUE-KEY or path/to/pack.md> --out-root <repo root that holds drafts/>
```
It reads the issue (`nudge issue get <KEY> -o json`; text between `<!-- writer-pack:start -->` and `<!-- writer-pack:end -->`) or the file, checks the Readiness line, creates `drafts/<piece>/`, writes `PACK.md` and a draft `venue.json`.
- Readiness WAITS, HOLD, IDEA or VERIFY: the script refuses (exit 3). Tell the user which dependency the Readiness line names and stop. Continue only if the user passes an explicit override (`--override`), and then write the override and its reason at the top of OPEN.md.
- Read `PACK.md` fully, then every file it says to read in full (results files, reports). Read them in full, not a summary. If a named file is missing, write that to OPEN.md and do not use the data.
- Open `venue.json`: it holds the class and limits the script parsed. Correct it by hand, set `"confirmed": true` after step 2.

### 2. Venue check
Follow [references/venue-check.md](references/venue-check.md). Open the live call (curl or your fetch tool) and compare deadline, length, template, anonymity, AI policy and concurrent-submission wording with the pack. Write every difference to `OPEN.md`. **Stop if a hard limit changed** (length, template, anonymity, a concurrent-submission ban that hits this piece, a closed call) and ask the user.

### 3. Format class
`venue.json` `class` is one of the classes below. Read only that class's reference.

| class | for | reference |
|---|---|---|
| `ieee-conf` | IEEEtran conference papers, page limits | [format-ieee-conf.md](references/format-ieee-conf.md) |
| `acm` | acmart sigconf/other ACM papers | [format-acm.md](references/format-acm.md) |
| `ieee-cs-magazine` | IEEE Software/Computer style, word limit, insight bullets | [format-ieee-cs-magazine.md](references/format-ieee-cs-magazine.md) |
| `springer-ccis` | Springer CCIS/LNCS, pattern papers | [format-springer-ccis.md](references/format-springer-ccis.md) |
| `prose-md` | practitioner articles, posts, tutorials (Markdown, word limits) | [format-prose-md.md](references/format-prose-md.md) |
| `proposal` | talk proposals, 1-2 page abstracts | [format-proposal.md](references/format-proposal.md) |
| `outline-only` | outlets that ban or screen AI-written prose | [format-outline-only.md](references/format-outline-only.md) |
| `unknown` | pack says "per live call" | decide after step 2; ask the user |

### 4. Claims ledger before prose
Write `CLAIMS.md` before any prose, per [references/claims.md](references/claims.md): one row per intended claim with type (`fact`, `number`, `citation`, `limitation`, `interview`, `inference`, `strategy`), exact source (pack block, results file and line, commit, or citation) and an anchor phrase. Keep fact, inference and strategy separate. Mark position-paper requirements as `inference` or `strategy`, not as findings.

### 5. Citations
Follow [references/citations.md](references/citations.md). `refs.bib` (or the reference list) contains only works named in the pack. Verify with
```
uv run scripts/check_citations.py refs.bib --pack PACK.md --log citations.tsv --netlog network.log --save-meta citations/
```
Fill `citations.tsv` by hand: key, sentence supported, read depth (`full`, `abstract`, `not-read`), note. Read the saved metadata (abstract) before you write a sentence that depends on a paper. Never cite a paper you did not at least open; never quote a number from a paper you only know from the pack without saying so in OPEN.md. Mismatches are reported, you do not auto-fix them silently: tell the user and correct the entry only to match the index, noting it.

### 6. Draft
Use the class template in `assets/templates/`. Rules: [references/drafting.md](references/drafting.md). First-person plain voice as the pack's rules say; the pack's suggested structure; short sentences and concrete nouns; every limitation the data file states. Every sentence with a fact traces to a CLAIMS row. Put `TO VERIFY:` markers in the text where evidence is missing.

### 7. Check, build, limits (loop until clean)
```
uv run scripts/check_numbers.py DRAFT --pack PACK.md --root <repo root> [--allow allow.txt]
uv run scripts/check_claims.py CLAIMS.md DRAFT
scripts/ensure_tex.sh compile DRAFT.tex            # LaTeX classes only
uv run scripts/check_limits.py --venue venue.json --pdf DRAFT.pdf --text DRAFT.tex --bib refs.bib
uv run scripts/check_identity.py --strings identity.txt DRAFT.tex refs.bib SUBMIT.md --pdf DRAFT.pdf   # double-blind only
```
- `check_numbers.py` fails on any number not found verbatim in the pack or a results file the pack names. The allowlist (`allow.txt`, lines `token  # reason`) is only for page numbers, years, section numbers, and derived counts you explain in CLAIMS.md. Do not allowlist a result.
- Build: [references/build.md](references/build.md). System TeX is used when the class exists; otherwise a pinned, SHA-256 verified Tectonic is fetched into the skill cache. If neither works, stop with the message and offer the Docker option.
- Over the limit: cut, per the class reference. Re-run everything.

### 8. Prose pass
[references/prose-pass.md](references/prose-pass.md). Copy the finished draft to `DRAFT.before-unslop.*`, apply the `unslop` skill if it is installed (it may need to be invoked by name by the user: if you cannot load it, apply the same patterns from the reference file), then the paper-specific banned-pattern list. Then prove nothing moved:
```
uv run scripts/check_numbers.py --compare DRAFT.before-unslop.tex DRAFT.tex
uv run scripts/check_claims.py CLAIMS.md DRAFT.tex
```
and re-run step 7. Record "numbers unchanged" in OPEN.md only if the compare passed.

### 9. Hostile review
[references/review.md](references/review.md). Three reviewers (methods, practitioner relevance, related work). Use subagents if your agent supports them; otherwise run them one after another in the same session, each starting with a fresh read of the compiled draft and the pack only, not your notes. Write all findings to `REVIEW.md`. You fix only factual errors and limit violations automatically; every other finding goes to the author with your recommendation.

### 10. Outputs
Exactly as [references/outputs.md](references/outputs.md): `DRAFT.*` (+ PDF), `CLAIMS.md`, `OPEN.md`, `SUBMIT.md`, `REVIEW.md`, `network.log`, `refs.bib`, `citations.tsv`, `venue.json`. OPEN.md must end with "What the author must read and re-run" and carry the three sentences most likely to be challenged, with sources. Finish with a short summary to the user: files, check results, number of TO VERIFY items, anything unconfirmed. Do not submit.

## When something fails

- A script cannot run (no `uv`, no network): say so, do the check by hand only if the user accepts, and mark it "not run" in OPEN.md. A check you could not run is never reported as passed.
- A pack contradicts itself or the results file (for example a count of cases): follow the results file, record the contradiction in OPEN.md.
- The data cannot support the claim the pack asks for: write the smaller claim the data supports, or `TO VERIFY`, and say so in OPEN.md. Do not strengthen.
