#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["pypdf>=4"]
# ///
"""Double-blind identity gate.

  check_identity.py --strings strings.txt FILE_OR_DIR... [--pdf X.pdf]

strings.txt: one identifying string per line (author name, handles, employer, tool
owner, repository owner, ORCID, e-mail). Multi-word names also match each word of
4+ letters and "Last, First". Matching is case-insensitive and ignores spaces,
dashes and underscores. Strings match on word boundaries. Also flags e-mail addresses, ORCID ids, self-references, and PDF metadata (Author, Creator, Title) that
contain a listed string. Exit 1 on any hit.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _common import read  # noqa: E402

EXTS = {".tex", ".md", ".bib", ".txt", ".csv", ".json", ".bbl", ".cls", ".sty"}


def squash(s: str) -> str:
    return re.sub(r"[\s_\-]+", "", s.lower())


def expand(strings):
    out = {}
    for s in strings:
        s = s.strip()
        if not s or s.startswith("#"):
            continue
        out[s] = s
        parts = [p for p in re.split(r"\s+", s) if p]
        if len(parts) > 1:
            out[f"{parts[-1]}, {' '.join(parts[:-1])}"] = s
            for p in parts:
                if len(re.sub(r"\W", "", p)) >= 4:
                    out[p] = s
    return out


def term_regex(term: str):
    words = [w for w in re.split(r"[\s_\-]+", term.lower()) if w]
    if not words:
        return None
    return re.compile(r"(?<![a-z0-9])" + r"[\s_\-]*".join(re.escape(w) for w in words) + r"(?![a-z0-9])")


def scan_text(label, text, terms, strict_urls=False):
    hits = []
    pats = [(t, o, term_regex(t)) for t, o in terms.items()]
    for i, raw in enumerate(text.splitlines(), 1):
        low = raw.lower()
        for term, origin, rx in pats:
            if rx and rx.search(low):
                hits.append((label, i, f"{term!r} (from {origin!r})", raw.strip()[:100]))
        for m in re.finditer(r"[\w.+-]+@[\w-]+\.[\w.-]+", raw):
            hits.append((label, i, "e-mail address", m.group(0)))
        for m in re.finditer(r"\b\d{4}-\d{4}-\d{4}-\d{3}[\dX]\b", raw):
            hits.append((label, i, "ORCID", m.group(0)))
        if strict_urls:
            for m in re.finditer(r"(?:github|gitlab|bitbucket)\.(?:com|io)/[\w.-]+", raw, flags=re.I):
                if "anonymous" not in m.group(0).lower():
                    hits.append((label, i, "code-hosting URL (--strict-urls)", m.group(0)))
        if re.search(r"(?i)\b(our|my) (previous|earlier|prior) (work|paper|post|article|talk)\b", raw):
            hits.append((label, i, "self-reference to earlier work", raw.strip()[:100]))
    return hits


def pdf_meta_text(pdf):
    from pypdf import PdfReader
    r = PdfReader(str(pdf))
    meta = r.metadata or {}
    txt = "\n".join(f"{k}: {v}" for k, v in meta.items())
    body = "\n".join((p.extract_text() or "") for p in r.pages)
    return txt, body


def run(strings, paths, pdf=None, strict_urls=False):
    terms = expand(strings)
    hits = []
    files = []
    for p in paths:
        p = Path(p)
        if p.is_dir():
            files += [f for f in p.rglob("*") if f.is_file() and f.suffix in EXTS]
        elif p.is_file():
            files.append(p)
    for f in files:
        hits += scan_text(str(f), read(f), terms, strict_urls)
    if pdf:
        meta, body = pdf_meta_text(pdf)
        hits += scan_text(f"{pdf} [metadata]", meta, terms, strict_urls)
        hits += scan_text(f"{pdf} [text]", body, terms, strict_urls)
    return hits


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("paths", nargs="*")
    ap.add_argument("--strings", required=True)
    ap.add_argument("--pdf")
    ap.add_argument("--strict-urls", action="store_true", help="flag every code-hosting URL, not only ones containing a listed string")
    a = ap.parse_args(argv)
    hits = run(read(a.strings).splitlines(), a.paths, a.pdf, a.strict_urls)
    if hits:
        print("FAIL: identity leaks")
        for f, i, what, ctx in hits:
            print(f"  {f}:{i}: {what}: {ctx}")
        return 1
    print("OK: no identity strings found.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
