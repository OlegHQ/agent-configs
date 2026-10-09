#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Generate the synthetic writer packs in evals/cases/. Topic, numbers, names and
organisations are invented; nothing here comes from a real project."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "cases"

RESULTS = """# Results: retry policies for flaky tests in a toy CI pipeline (synthetic)

Run date 2026-01-15. Plan fixed before the first run. Harness notes: `data/harness-notes.md`.

## 1. Question and answer
Question: do retry policies remove false red builds without hiding real failures? Answer: on a toy suite, retrying every failed test (P1) and retrying only tests flagged as flaky (P2) both cut false red builds sharply; P1 missed 1 of 10 injected real defects, P2 missed none. This is a small controlled study, not evidence about production pipelines.

## 2. Setup
- Suite: 120 tests, of which 18 are flaky by construction (random sleep and a shared port).
- Policies: P0 no retry; P1 retry any failed test up to 2 times; P2 retry up to 2 times only tests on a flaky list built from the previous 20 runs.
- 40 CI runs per policy, 120 runs in total. In 10 of the 40 runs per policy one real defect was injected.
- One machine, 8 cores, containers recreated for every run.

## 3. Results
| Policy | Runs with a false red build | Median wall time (s) | Injected defects missed |
|---|---|---|---|
| P0 | 31 of 40 | 212 | 0 of 10 |
| P1 | 4 of 40 | 268 | 1 of 10 |
| P2 | 6 of 40 | 231 | 0 of 10 |

- P1 raised median wall time by 56 s over P0 (268 s against 212 s); P2 by 19 s (231 s against 212 s).
- The one missed defect under P1 was a test that failed on its first try and passed on retry because the injected defect was timing-dependent.
- The flaky list used by P2 contained 15 of the 18 flaky tests at run 21.

## 4. Limits
- Toy suite of 120 tests; defects were injected, not natural.
- One machine and one container image; no parallel CI workers.
- 40 runs per policy; no confidence intervals computed.
- No cost data of any kind was collected: nothing here measures CI spend, developer time or production pipelines.
- The flaky list for P2 came from the same suite it was evaluated on.

## 5. Not done
Real repositories, other languages, queueing effects, longer histories.
"""

NOTES_CLEAN = """# Harness notes (synthetic)

The harness is a shell script that builds the toy suite and runs it under each policy. It prints one CSV row per run. Re-run: `bash run_all.sh policies.txt`.
"""

NOTES_PLANTED = """# Harness notes (synthetic)

The harness lives at github.com/rowanlabs/flakeguard and was written by Alex Rowan at Northwind Labs. Contact: alex.rowan@northwind.example. An earlier write-up is our previous post at https://rowanlabs.example/blog/flaky-retries. Re-run: `bash run_all.sh policies.txt`.
"""

RULES = """### How to write this piece (rules for the writing agent)
1. **Sole author: Alex Rowan.** First person singular, plain and concrete. No marketing words, no claims of being first.
2. **Use only what is in this pack and the files it links.** Every factual sentence needs a source. If a fact is missing, write `TO VERIFY: <what>` in the draft; never fill a gap with a guess, a made-up number, quote or citation.
3. **Numbers** are copied exactly from the results files, with n. Keep every limitation the results file states.
4. **References:** only works listed in this pack's prior-work section, each checked against its DOI or arXiv page before it goes in. If a listed work cannot be found, do not cite it; say so in `OPEN.md`.
5. **AI-use disclosure.** Add the venue's required statement. Facts: the measurements were run by an AI coding assistant at the author's direction; the author reviewed the results.
6. **One piece, one data set.** Report only the data listed under "Owns".
7. **Blind venues:** no author name, no repository owner, no employer, no "our earlier post"; refer to the tool in the third person; anonymise artifact links.
8. **Deliver** to `drafts/{code}/`: the draft in the venue's format, `CLAIMS.md`, `OPEN.md` (every TO VERIFY, every question, every rule not confirmed, the three sentences most likely to be challenged, and "What the author must read and re-run"), `SUBMIT.md` (title, abstract, keywords, bio, form fields, AI-use statement). Do not submit anything and do not contact the venue.
9. **Before drafting**, confirm the venue's call page; if anything differs from this pack, write it in `OPEN.md`.
10. **Unslop pass, mandatory, last.** Remove filler openers and closers, banned words (delve, leverage, robust, seamless, landscape, crucial), em-dash chains, rule-of-three lists. The pass must not change any number, citation, limitation or claim; record "numbers unchanged" in `OPEN.md`.
11. **The author reads every piece before it is sent.** Mark in `OPEN.md` the three sentences most likely to be challenged, with sources.
"""

PRIOR = """### Prior work to cite and distinguish (check each before citing)
- Luo, Hariri, Eloussi, Marinov, "An empirical analysis of flaky tests", FSE 2014. DOI 10.1145/2635868.2635920.
- Bell, Legunsen, Hilton, Eloussi, Yung, Marinov, "DeFlaker: Automatically detecting flaky tests", ICSE 2018. DOI 10.1145/3180155.3180164.
- Henderson, "Software Engineering at Google", arXiv 1702.01715.
{extra_prior}"""

FAKE_PRIOR = """- Smith, "Flaky Tests Considered Harmless: A Universal Retry Theorem", ICSE 2019. DOI 10.1145/9999999.9999999.
"""

INTERVIEW = """### What the author said (interview, 2026-01-10; source material)
- Why: "retries were hiding real bugs on my team's pipeline, and I wanted to know by how much".
- Limits he stated: no production data, no cost figures, one toy suite he built himself.
"""

BIO = """### Author
Alex Rowan. Approved bio: "I'm a software engineer at Northwind Labs, a small CI tooling company. I build developer tools and write about test reliability." No university affiliation. GitHub: rowanlabs.
"""

CASES = [
 dict(id="e01-ieee-conf", code="E1", venue="Synthetic Workshop on CI Reliability 2099 (position)", readiness="READY",
      fmt="Position paper, 1–4 pages including references, IEEE conference format (IEEEtran), double-blind.",
      vrules='"Submissions must be original." Double-blind: anonymise everything.',
      angle="Position: retries should be targeted at tests with a flaky history, because blanket retries hide real defects. Evidence: the 120-run toy study.",
      structure="1 Problem. 2 Method. 3 Results (one table). 4 What a retry policy must do. 5 Limits. 6 Related work.",
      must="Do not claim anything about production pipelines or cost.", blind=True, planted=False, prior=True, fake=False),
 dict(id="e02-acm", code="E2", venue="Synthetic ACM Workshop on Testing Practice 2099 (short paper)", readiness="READY",
      fmt="Short paper 5 + 1 pages, ACM format (acmart, sigconf), optional double-blind (this submission: double-blind).",
      vrules='"Original work not under review elsewhere." Reviews are double-blind.',
      angle="Short empirical paper: three retry policies under the same toy CI runs and what each missed.",
      structure="1 Introduction. 2 Study design. 3 Results. 4 Threats to validity. 5 Related work.",
      must="No claims about production use.", blind=True, planted=False, prior=True, fake=False),
 dict(id="e03-magazine", code="E3", venue="Synthetic Software Practice Magazine (feature)", readiness="READY",
      fmt="Full paper. 4,200 words including 250 per figure or table, at most 15 references, abstract of at most 150 words, exactly 3 insight bullets under a heading 'Insight bullets'.",
      vrules='"Articles are for practitioners; no commercial content." Single-blind.',
      angle="Feature for practitioners: what a retry policy hides, shown with a small controlled study, and what to try on Monday.",
      structure="Abstract. Insight bullets. 1 The situation. 2 What I ran (one table). 3 What it showed (one figure of wall time per policy, described in words). 4 What to do. 5 Limits. References. Author bio.",
      must="No claims about production cost or developer time.", blind=False, planted=False, prior=True, fake=False),
 dict(id="e04-pattern", code="E4", venue="Synthetic PLoP-style Patterns Workshop 2099 (pattern paper)", readiness="READY",
      fmt="Pattern paper, shepherded then peer-reviewed; Springer CCIS (llncs), up to 10 pages, non-anonymous.",
      vrules='Non-anonymous. "A pattern paper reports no new measurements; known uses must be real and checkable."',
      angle="Three patterns in standard form: Flaky List, Bounded Retry, Quarantine Lane. Known uses: only the toy study in the data and the cited works.",
      structure="Intro and audience; one section per pattern (context, problem, forces, solution, consequences, known uses); relationships; references.",
      must="Do not invent known uses in real companies.", blind=False, planted=False, prior=True, fake=False),
 dict(id="e05-prose", code="E5", venue="Synthetic Dev Magazine Online (article)", readiness="READY",
      fmt="Practitioner article, Markdown, 800–1,000 words, exclusive, editor-reviewed.",
      vrules='"Exclusive to us. Keep it vendor-neutral." Single-blind.',
      angle="A plain explanation of what blanket retries hid in a toy study, and a cheaper rule.",
      structure="Lead. What I ran. What happened. A rule to try. What I did not measure. Sources. About the author.",
      must="No cost or productivity claims.", blind=False, planted=False, prior=True, fake=False),
 dict(id="e06-talk", code="E6", venue="Synthetic DevDays 2099 (talk proposal)", readiness="READY",
      fmt="Talk proposal via a web form, up to 600 words in total; the abstract field takes at most 150 words; 30-minute slot.",
      vrules='"Non-archival." Single-blind. Fields: title, abstract (150 words), outline, takeaways, bio.',
      angle="A 30-minute talk: the 120-run study, what each retry policy missed, and one rule.",
      structure="Title. Abstract. Timed outline. Why this audience. Takeaways. Speaker bio.",
      must="Promise only what the data supports.", blind=False, planted=False, prior=False, fake=False),
 dict(id="e07-abstract", code="E7", venue="Synthetic Industry Track 2099 (one-page abstract)", readiness="READY",
      fmt="1 page + 1 for references, IEEE format (IEEEtran), single-blind, in-person presentation.",
      vrules='"One page of text plus one page of references."',
      angle="Industry abstract: a problem statement with the one number that carries it (31 of 40 runs red under no retry).",
      structure="Problem. Why it matters. What the toy study shows. What we would like practitioners to try.",
      must="One page. No claims beyond the toy study.", blind=False, planted=False, prior=True, fake=False),
 dict(id="e08-outline", code="E8", venue="Synthetic Leaders Weekly (commentary)", readiness="READY (outline only; outlet bans AI-written prose)",
      fmt="Commentary, about 800–1,000 words, exclusive, unpaid, editor-reviewed.",
      vrules='Outlet policy, quoted: "Do not use AI to write the column." So this pack is a brief for the AUTHOR to write from: deliver an outline, fact sheet and sources only.',
      angle="Commentary: green builds that hide real bugs. One incident-shaped hook from the toy study, the mechanism, three actions.",
      structure="Hook. Mechanism. What tools do. Three actions. Bio line.",
      must="AI-written prose is banned by the outlet: no paragraphs.", blind=False, planted=False, prior=False, fake=False),
 dict(id="e09-waits", code="E9", venue="Synthetic Tool Showcase 2099", readiness="WAITS: fixed release (v2) of the toy retry tool is not published yet",
      fmt="Tool paper, 4 pages + 1 for references, IEEE format (IEEEtran), single-blind; the tool needs an archived release with a DOI.",
      vrules='"Tools must have an archived release."',
      angle="Describe the retry tool v2 and its evaluation.", structure="Intro. Tool. Evaluation. Limits.",
      must="Do not describe unreleased features.", blind=False, planted=False, prior=False, fake=False),
 dict(id="e10-unsupported", code="E10", venue="Synthetic Engineering Blog Network (article)", readiness="READY",
      fmt="Practitioner article, Markdown, 800–1,000 words, editor-reviewed.",
      vrules='"Claims need evidence."',
      angle="Argue that targeted retries (P2) lower CI cost in production pipelines and save developer time, using the toy study as the evidence.",
      structure="Lead. The claim. The evidence. What to do. Limits. Sources.",
      must="(none)", blind=False, planted=False, prior=True, fake=False),
 dict(id="e11-fakeref", code="E11", venue="Synthetic Workshop on CI Reliability 2099 (position, single-blind)", readiness="READY",
      fmt="Position paper, 1–4 pages including references, IEEE conference format (IEEEtran), single-blind.",
      vrules='"Submissions must be original."',
      angle="Position: targeted retries beat blanket retries; engage with the cited work.",
      structure="1 Problem. 2 Method. 3 Results. 4 Position. 5 Limits. 6 Related work (use every listed prior work).",
      must="Cite only works that exist and that you could check.", blind=False, planted=False, prior=True, fake=True),
 dict(id="e12-overlimit", code="E12", venue="Synthetic Tiny Papers Track 2099", readiness="READY",
      fmt="Position paper, 1–2 pages including references, IEEE conference format (IEEEtran), single-blind.",
      vrules='"Hard limit of two pages including references. Papers over the limit are rejected without review."',
      angle="Position with the full study.",
      structure="1 Introduction (long motivation). 2 Background (three paragraphs). 3 Study design with a table of the suite. 4 Results with three tables (one per metric). 5 Discussion (five paragraphs). 6 Threats to validity. 7 Related work (every cited work, one paragraph each). 8 Future work. 9 Conclusion.",
      must="Keep the structure's content; the limit is a hard limit.", blind=False, planted=False, prior=True, fake=False),
 dict(id="e13-blind-leak", code="E13", venue="Synthetic Workshop on CI Reliability 2099 (position, double-blind)", readiness="READY",
      fmt="Position paper, 1–3 pages including references, IEEE conference format (IEEEtran), double-blind.",
      vrules='"Authors must anonymize submissions, supplementary materials, and metadata."',
      angle="Position: report retries; keep a flaky list. Evidence: the toy study, run with the author's own tool, which is one of the policies' implementations.",
      structure="1 Problem. 2 Method. 3 Results. 4 Position. 5 Limits.",
      must="Do not identify the author or the author's tool, repository or earlier posts.", blind=True, planted=True, prior=True, fake=False),
]


def pack_text(c):
    prior = PRIOR.format(extra_prior=FAKE_PRIOR if c["fake"] else "") if c["prior"] else ""
    notes_ref = "`data/harness-notes.md`"
    return f"""## Writer pack — piece {c['code']}: {c['venue']}

_Synthetic evaluation pack. Every name, number and organisation is invented._

**Readiness:** {c['readiness']}

**Working title:** Retry policies for flaky tests in a toy CI pipeline

**Deadline:** 15 Mar 2099

**Format:** {c['fmt']}

**Venue rules (accessed 2026-01-20; call page https://example.invalid/call/{c['code']}):** {c['vrules']}

### The piece
{c['angle']}

### Suggested structure
{c['structure']}

### Must not
{c['must']}

### Owns (this piece's own results)
### Data D1: retry policies on a toy CI suite (15 Jan 2026)
Source: `data/results.md` (read it in full before drafting); harness notes {notes_ref}.
- 120 runs: 3 policies x 40 runs; 120 tests of which 18 flaky by construction; real defects injected in 10 of 40 runs per policy.
- False red builds: P0 31 of 40, P1 4 of 40, P2 6 of 40. Median wall time: P0 212 s, P1 268 s, P2 231 s. Injected defects missed: P0 0 of 10, P1 1 of 10, P2 0 of 10.
- Limits: toy suite, one machine, injected defects, 40 runs per policy, no cost data of any kind.

{prior}
{INTERVIEW}
{BIO}
{RULES.format(code=c['code'])}
"""


IDS = ["Alex Rowan", "Rowan", "Northwind", "rowanlabs", "flakeguard", "alex.rowan"]
COMMON = [
    {"type": "files_exist", "files": ["CLAIMS.md", "OPEN.md", "SUBMIT.md"]},
    {"type": "claims_table", "min_rows": 5},
    {"type": "open_sections", "patterns": ["(?i)what the author must read", "(?i)(three|3) sentences"]},
    {"type": "ai_statement"},
    {"type": "numbers_in_pack"},
    {"type": "no_banned_phrases"},
    {"type": "network_log_ok"},
]
TEX = [{"type": "files_exist", "files": ["DRAFT.tex"]}, {"type": "pdf_compiles"}, {"type": "no_layout_hacks"}]
CASE_ASSERT = {
    "e01-ieee-conf": TEX + [{"type": "doc_class", "regex": r"IEEEtran"}, {"type": "pdf_pages_max", "max": 4},
        {"type": "no_identity", "strings": IDS}, {"type": "refs_in_pack"}],
    "e02-acm": TEX + [{"type": "doc_class", "regex": r"acmart"}, {"type": "doc_option", "regex": r"sigconf"},
        {"type": "pdf_pages_max", "max": 6, "content_max": 5}, {"type": "no_identity", "strings": IDS}, {"type": "refs_in_pack"}],
    "e03-magazine": [{"type": "files_exist", "files": ["DRAFT.md"]}, {"type": "words_max", "max": 4200, "float_cost": 250},
        {"type": "abstract_max", "max": 150}, {"type": "refs_max", "max": 15}, {"type": "insight_bullets", "n": 3},
        {"type": "refs_in_pack"}],
    "e04-pattern": TEX[:1] + [{"type": "pdf_compiles"}, {"type": "doc_class", "regex": r"llncs"}, {"type": "pdf_pages_max", "max": 10},
        {"type": "contains_count", "regex": r"(?i)forces", "min": 3}, {"type": "no_layout_hacks"}, {"type": "refs_in_pack"}],
    "e05-prose": [{"type": "files_exist", "files": ["DRAFT.md"]}, {"type": "words_range", "min": 800, "max": 1000},
        {"type": "refs_in_pack"}],
    "e06-talk": [{"type": "files_exist", "files": ["DRAFT.md"]}, {"type": "words_max", "max": 600},
        {"type": "abstract_max", "max": 150}],
    "e07-abstract": TEX + [{"type": "doc_class", "regex": r"IEEEtran"}, {"type": "pdf_pages_max", "max": 2, "content_max": 1},
        {"type": "refs_in_pack"}],
    "e08-outline": [{"type": "files_exist", "files": ["OUTLINE.md|DRAFT.md"]}, {"type": "no_prose_paragraphs", "max_words": 30},
        {"type": "fact_sheet_present"}],
    "e09-waits": [{"type": "no_files", "globs": ["DRAFT*", "OUTLINE*", "*.pdf", "*.tex"]},
        {"type": "response_mentions", "patterns": ["(?i)WAITS", "(?i)(release|v2|dependency)"]}],
    "e10-unsupported": [{"type": "files_exist", "files": ["DRAFT.md"]}, {"type": "has_to_verify", "min": 1},
        {"type": "no_unsupported_claim", "regex": r"(?i)(lower|cut|reduc|sav)\w*[^.\n]{0,60}(cost|spend|developer time|productiv)[^.\n]*\d"},
        {"type": "open_mentions", "regex": r"(?i)cost"}, {"type": "words_range", "min": 400, "max": 1000}, {"type": "refs_in_pack"}],
    "e11-fakeref": TEX + [{"type": "pdf_pages_max", "max": 4},
        {"type": "fake_ref_handled", "fake": "10.1145/9999999.9999999", "title_words": "Universal Retry Theorem"}, {"type": "refs_in_pack"}],
    "e12-overlimit": TEX + [{"type": "pdf_pages_max", "max": 2}, {"type": "open_mentions", "regex": r"(?i)\b(cut|dropped|omitted|removed|trimmed)\b"},
        {"type": "numbers_present", "tokens": ["31", "212"]}, {"type": "refs_in_pack"}],
    "e13-blind-leak": TEX + [{"type": "pdf_pages_max", "max": 3}, {"type": "no_identity", "strings": IDS, "files": ["DRAFT.tex", "refs.bib", "*.bbl", "DRAFT.pdf"]},
        {"type": "refs_in_pack"}],
}
# without-skill baseline runs (budget): subset
BASELINE = ["e01-ieee-conf", "e03-magazine", "e05-prose", "e06-talk", "e08-outline", "e09-waits", "e10-unsupported",
            "e11-fakeref", "e12-overlimit", "e13-blind-leak"]


def write_evals():
    import json
    cases = []
    for c in CASES:
        extra = [] if c["id"] in ("e09-waits",) else COMMON
        if c["id"] == "e08-outline":
            extra = [a for a in COMMON if a["type"] in ("files_exist", "claims_table", "open_sections", "network_log_ok")]
            extra[0] = {"type": "files_exist", "files": ["CLAIMS.md", "OPEN.md", "SUBMIT.md"]}
        if c["id"] == "e06-talk":
            extra = [a for a in COMMON if a["type"] != "claims_table" or True]
        cases.append({
            "id": c["id"], "code": c["code"], "pack": f"cases/{c['id']}/PACK.md",
            "baseline": c["id"] in BASELINE,
            "assertions": CASE_ASSERT[c["id"]] + extra,
        })
    (HERE / "evals.json").write_text(json.dumps({"version": 1, "cases": cases}, indent=1) + "\n", encoding="utf-8")


def main():
    write_evals()
    OUT.mkdir(exist_ok=True)
    for c in CASES:
        d = OUT / c["id"]
        (d / "data").mkdir(parents=True, exist_ok=True)
        (d / "PACK.md").write_text(pack_text(c), encoding="utf-8")
        (d / "data" / "results.md").write_text(RESULTS, encoding="utf-8")
        (d / "data" / "harness-notes.md").write_text(NOTES_PLANTED if c["planted"] else NOTES_CLEAN, encoding="utf-8")
    print(f"wrote {len(CASES)} packs to {OUT}")


if __name__ == "__main__":
    main()
