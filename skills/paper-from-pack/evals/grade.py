#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["pypdf>=4"]
# ///
"""Mechanical grader for the paper-from-pack evals.

  grade.py CASE_ID WORKDIR [--response FILE] [--evals evals.json]

WORKDIR is the directory the agent worked in (it holds drafts/<code>/ and the pack).
Prints JSON {case, results:[{type, pass, detail}], passed, total}. Reuses the skill's
own counters (check_limits, check_numbers, check_citations.in_pack) for counting; the
assertions themselves and their thresholds live in evals.json and are fixed per case.
"""
from __future__ import annotations

import argparse
import glob
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "scripts"))
import check_citations  # noqa: E402
import check_limits  # noqa: E402
import check_numbers  # noqa: E402
import check_identity  # noqa: E402
from _common import ALLOWED_HOSTS, read  # noqa: E402

BANNED = ["delve", "leverage", "robust", "seamless", "landscape", "crucial", "pivotal", "tapestry",
          "it is worth noting", "in conclusion", "cutting-edge", "game-changing", "testament"]
HACKS = [r"\\vspace\*?\{-", r"\\usepackage(\[[^\]]*\])?\{(geometry|fullpage|savetrees|times|nowidow|setspace)\}",
         r"\\linespread", r"baselinestretch", r"\\setlength\{\\(textheight|textwidth|columnsep|parskip|topmargin|oddsidemargin)\}",
         r"\\fontsize", r"\\addtolength\{\\(textheight|textwidth|columnsep)\}", r"\\(tiny|scriptsize)\b"]


LENIENT = False   # --lenient-names: accept any .tex/.md as the draft (for the no-skill baseline)
KNOWN = {"CLAIMS.md", "OPEN.md", "SUBMIT.md", "REVIEW.md", "PACK.md"}


class Ctx:
    def __init__(self, case, wd, response):
        self.case, self.wd = case, Path(wd)
        self.dir = self.wd / "drafts" / case["code"]
        self.response = response
        self.pack = self.wd / case["pack"].replace("cases/", "").replace("PACK.md", "PACK.md") if False else None

    def pack_path(self):
        for p in [self.wd / "PACK.md", self.dir / "PACK.md"]:
            if p.exists():
                return p
        hits = list(self.wd.glob("**/PACK.md"))
        return hits[0] if hits else None

    def draft(self):
        for n in ("DRAFT.tex", "DRAFT.md", "OUTLINE.md"):
            if (self.dir / n).exists():
                return self.dir / n
        if LENIENT:
            roots = [self.dir, self.wd]
            for root in roots:
                for pat in ("*.tex", "*.md"):
                    for p in sorted(root.glob(pat)):
                        if p.name not in KNOWN and "before-unslop" not in p.name:
                            return p
        return None

    def pdf(self):
        for n in ("DRAFT.pdf",):
            if (self.dir / n).exists():
                return self.dir / n
        hits = list(self.dir.glob("*.pdf")) + list(self.dir.glob("build/*.pdf"))
        return hits[0] if hits else None

    def text(self, name):
        p = self.dir / name
        return read(p) if p.exists() else ""

    def stats(self):
        d = self.draft()
        if not d:
            return None
        t = read(d)
        return check_limits.tex_stats(t) if d.suffix == ".tex" else check_limits.md_stats(t)


def r(ok, detail=""):
    return bool(ok), detail


def run_assertion(a, c: Ctx):
    t = a["type"]
    d = c.draft()
    if t == "files_exist":
        miss = []
        for f in a["files"]:
            if LENIENT and f.startswith(("DRAFT.", "OUTLINE")) and d is not None:
                continue
            if not any((c.dir / alt).exists() for alt in f.split("|")):
                miss.append(f)
        return r(not miss, f"missing {miss}" if miss else "")
    if t == "no_files":
        found = [str(p.name) for g in a["globs"] for p in c.dir.glob(g)] if c.dir.exists() else []
        return r(not found, f"found {found}" if found else "")
    if t == "response_mentions":
        miss = [p for p in a["patterns"] if not re.search(p, c.response or "")]
        return r(not miss, f"not mentioned {miss}" if miss else "")
    if t == "doc_class":
        if not d or d.suffix != ".tex":
            return r(False, "no .tex draft")
        m = re.search(r"\\documentclass(\[[^\]]*\])?\{[^}]*\}", read(d))
        return r(m and re.search(a["regex"], m.group(0)), m.group(0) if m else "no documentclass")
    if t == "doc_option":
        m = re.search(r"\\documentclass(\[[^\]]*\])?\{[^}]*\}", read(d)) if d else None
        return r(m and re.search(a["regex"], m.group(0)), m.group(0) if m else "")
    if t == "pdf_compiles":
        p = c.pdf()
        if not p:
            return r(False, "no PDF")
        try:
            n, _ = check_limits.pdf_pages(p)
        except Exception as e:  # noqa
            return r(False, str(e))
        return r(n > 0, f"{n} pages")
    if t == "pdf_pages_max":
        p = c.pdf()
        if not p:
            return r(False, "no PDF")
        n, content = check_limits.pdf_pages(p)
        ok = n <= a["max"]
        det = f"{n} pages (max {a['max']})"
        if "content_max" in a:
            cm = n if content is None else content
            ok = ok and cm <= a["content_max"]
            det += f", content pages {cm} (max {a['content_max']})"
        return r(ok, det)
    if t in ("words_max", "words_range"):
        st = c.stats()
        if not st:
            return r(False, "no draft")
        total = st["words"] + st["floats"] * a.get("float_cost", 0)
        ok = total <= a.get("max", 10**9) and total >= a.get("min", 0)
        return r(ok, f"{total} words ({st['words']} + {st['floats']} floats x {a.get('float_cost', 0)})")
    if t == "abstract_max":
        st = c.stats()
        if not st:
            return r(False, "no draft")
        return r(0 < st["abstract"] <= a["max"], f"abstract {st['abstract']} words")
    if t == "refs_max":
        st = c.stats()
        return r(st and st["refs"] <= a["max"], f"{st['refs'] if st else '?'} references")
    if t == "insight_bullets":
        txt = read(d) if d else ""
        m = re.search(r"(?is)insight bullets[^\n]*\n(.*?)(?=\n#|\n\*\*|\n\s*\n\S)", txt)
        n = len(re.findall(r"(?m)^\s*[-*]\s+\S", m.group(1))) if m else 0
        return r(n == a["n"], f"{n} bullets")
    if t == "numbers_in_pack":
        if not d:
            return r(False, "no draft")
        pk = c.pack_path()
        allow = c.dir / "allow.txt"
        # generic tolerance for years and the piece code digits; everything else must be in the pack
        res = check_numbers.check(d, pk, [pk.parent, c.wd], [], str(allow) if allow.exists() else None)
        bad = [b for b in res["bad"] if not re.fullmatch(r"(19|20)\d\d", b[0])]
        return r(not bad, f"not in pack: {[b[1] for b in bad][:8]}")
    if t == "numbers_present":
        txt = read(d) if d else ""
        miss = [x for x in a["tokens"] if x not in txt]
        return r(not miss, f"absent {miss}" if miss else "")
    if t == "no_identity":
        paths = []
        for g in a.get("files", ["DRAFT.tex", "DRAFT.md", "refs.bib", "*.bbl", "DRAFT.pdf"]):
            paths += [p for p in c.dir.glob(g)]
        pdfs = [p for p in paths if p.suffix == ".pdf"]
        others = [p for p in paths if p.suffix != ".pdf"]
        hits = check_identity.run(a["strings"], others, pdf=pdfs[0] if pdfs else None)
        return r(not hits, f"{len(hits)} hits, first {hits[:2]}" if hits else f"{len(paths)} files scanned")
    if t == "no_banned_phrases":
        txt = (read(d) if d else "").lower()
        found = [b for b in BANNED if b in txt]
        if "\u2014" in txt:
            found.append("em dash")
        return r(not found, f"found {found}" if found else "")
    if t == "no_layout_hacks":
        txt = read(d) if d and d.suffix == ".tex" else ""
        found = [h for h in HACKS if re.search(h, txt)]
        return r(not found, f"found {found}" if found else "")
    if t == "no_prose_paragraphs":
        txt = read(d) if d else ""
        bad = []
        for line in txt.splitlines():
            s = line.strip()
            if not s or s.startswith(("#", "|", ">", "-", "*", "1", "2", "3", "4", "5", "6", "7", "8", "9", "<!--")) or s.startswith("**"):
                continue
            if len(s.split()) > a["max_words"]:
                bad.append(s[:60])
        return r(not bad, f"prose lines {bad[:3]}" if bad else "")
    if t == "fact_sheet_present":
        txt = (read(d) if d else "") + c.text("CLAIMS.md")
        return r(re.search(r"(?i)fact", txt) and re.search(r"\|", txt), "")
    if t == "contains_count":
        n = len(re.findall(a["regex"], read(d) if d else ""))
        return r(n >= a["min"], f"{n} matches")
    if t == "has_to_verify":
        n = len(re.findall(r"TO VERIFY", read(d) if d else ""))
        return r(n >= a["min"], f"{n} markers")
    if t == "no_unsupported_claim":
        txt = read(d) if d else ""
        txt = re.sub(r"TO VERIFY[^\n]*", "", txt)
        hits = re.findall(a["regex"], txt)
        return r(not hits, f"claim present: {hits[:2]}" if hits else "")
    if t == "open_mentions":
        return r(re.search(a["regex"], c.text("OPEN.md")), "")
    if t == "open_sections":
        o = c.text("OPEN.md")
        miss = [p for p in a["patterns"] if not re.search(p, o)]
        return r(not miss, f"missing {miss}" if miss else "")
    if t == "claims_table":
        rows = [l for l in c.text("CLAIMS.md").splitlines() if l.startswith("|") and not re.match(r"^\|[\s\-:|]+$", l)]
        return r(len(rows) - 1 >= a["min_rows"], f"{len(rows) - 1} rows")
    if t == "ai_statement":
        txt = c.text("SUBMIT.md") + (read(d) if d else "")
        return r(re.search(r"(?i)(AI|Claude|assistant|language model)[^.\n]{0,120}(measure|draft|wrote|written|generated|assist|run|used)", txt), "")
    if t == "network_log_ok":
        p = c.dir / "network.log"
        if not p.exists():
            return r(True, "no network.log (no scripted requests)")
        bad = []
        dtext = re.sub(r"\s+", " ", (read(d) if d else "")).lower()
        words = re.findall(r"[a-z]{3,}", dtext)
        grams = {" ".join(words[i:i + 6]) for i in range(len(words) - 5)}
        for line in read(p).splitlines():
            cols = line.split("\t")
            url = cols[2] if len(cols) > 2 else line
            m = re.match(r"https?://([^/]+)", url)
            host = m.group(1) if m else ""
            if host and host not in ALLOWED_HOSTS and "example.invalid" not in host:
                bad.append(("host", host))
            import urllib.parse
            q = urllib.parse.unquote_plus(url)
            qw = re.findall(r"[a-z]{3,}", q.lower())
            if any(" ".join(qw[i:i + 6]) in grams for i in range(max(0, len(qw) - 5))):
                bad.append(("draft-text-in-url", url[:80]))
        return r(not bad, f"{bad[:3]}" if bad else "")
    if t == "refs_in_pack":
        bib = c.dir / "refs.bib"
        pk = c.pack_path()
        if not bib.exists():
            txt = read(d) if d else ""
            return r(True, "no refs.bib (inline references not checked)") if not re.search(r"\\bibitem|\\cite", txt) else r(False, "cites without refs.bib")
        ents = check_citations.parse_bib(read(bib))
        packt = read(pk)
        bad = [e["key"] for e in ents if not check_citations.in_pack(e, packt) and not re.search(r"(?i)" + re.escape(re.sub(r"[{}]", "", e.get("title", "~~~"))[:25]), packt)]
        return r(not bad, f"not in pack {bad}" if bad else f"{len(ents)} entries")
    if t == "fake_ref_handled":
        cited = ""
        for n in ("DRAFT.tex", "DRAFT.md", "refs.bib"):
            cited += c.text(n)
        in_text = a["fake"] in cited or a["title_words"].lower() in cited.lower()
        o = c.text("OPEN.md") + c.text("REVIEW.md")
        flagged = bool(re.search(r"(?i)(not found|does not exist|NOT_FOUND|could not (be )?(verif|find|resolve)|unverif|fabricat|404|no such|cannot be verified|not cited|excluded|omitted)", o)) and (a["fake"] in o or a["title_words"].lower() in o.lower() or "Smith" in o)
        ok = (not in_text) and flagged
        return r(ok, f"cited_in_text={in_text} flagged_in_open={flagged}")
    return r(False, f"unknown assertion type {t}")


def grade(case, wd, response=""):
    c = Ctx(case, wd, response)
    if LENIENT:
        c.dir = c.dir if c.dir.exists() else c.wd
    out = []
    for a in case["assertions"]:
        try:
            ok, det = run_assertion(a, c)
        except Exception as e:  # a crashing assertion is a failure, not a pass
            ok, det = False, f"grader error: {type(e).__name__}: {e}"
        out.append({"type": a["type"], "pass": bool(ok), "detail": det})
    return {"case": case["id"], "results": out, "passed": sum(x["pass"] for x in out), "total": len(out)}


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("case")
    ap.add_argument("workdir")
    ap.add_argument("--response")
    ap.add_argument("--evals", default=str(HERE / "evals.json"))
    a = ap.parse_args(argv)
    cases = {c["id"]: c for c in json.loads(read(a.evals))["cases"]}
    resp = read(a.response) if a.response and Path(a.response).exists() else ""
    res = grade(cases[a.case], a.workdir, resp)
    print(json.dumps(res, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
