#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["pypdf>=4"]
# ///
"""Limit gate for paper-from-pack. Limits come from venue.json (written by pack.py,
confirmed by hand).

  check_limits.py --venue venue.json [--pdf X.pdf] [--text DRAFT.tex|DRAFT.md] [--bib refs.bib]

Checks (each only when the limit is set):
  pages        PDF page count. With extra_ref_pages (for example 4 + 1), the pages before
               the page where "References" starts must be <= max_pages and the total
               <= max_pages + extra_ref_pages. Otherwise the total must be <= max_pages.
  words        Words of the body including the abstract, excluding references, code
               blocks and LaTeX markup. Each figure/table adds float_cost_words.
  abstract     Words in the abstract.
  refs         Reference count (bib entries cited, \\bibitem, or a Markdown list under
               a References heading).
Exit 1 on any failure.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _common import cite_keys, is_tex, read, strip_comments_tex  # noqa: E402


def pdf_pages(pdf):
    """-> (total pages, content pages before the references or None).

    content pages = the page holding the References heading when body text precedes
    the heading on that page, otherwise the page before it."""
    from pypdf import PdfReader
    r = PdfReader(str(pdf))
    pages = len(r.pages)
    for i, p in enumerate(r.pages, 1):
        t = p.extract_text() or ""
        m = re.search(r"(?mi)^\s*(?:[IVX]+\.\s*)?(references|bibliography)\s*$", t)
        if m:
            before = len(re.findall(r"[A-Za-z]{3,}", t[: m.start()]))
            return pages, (i if before > 40 else i - 1)
    return pages, None


def _words(s: str) -> int:
    return len(re.findall(r"[A-Za-z0-9][A-Za-z0-9'’\-]*", s))


def tex_stats(text: str):
    text = strip_comments_tex(text)
    m = re.search(r"\\begin\{abstract\}(.*?)\\end\{abstract\}", text, flags=re.S)
    abstract = m.group(1) if m else ""
    body = text.split("\\begin{document}", 1)[-1].split("\\end{document}", 1)[0]
    refs_part = ""
    m = re.search(r"\\begin\{thebibliography\}.*?\\end\{thebibliography\}", body, flags=re.S)
    if m:
        refs_part = m.group(0)
        body = body.replace(refs_part, " ")
    body = re.sub(r"\\bibliography\{[^}]*\}|\\bibliographystyle\{[^}]*\}", " ", body)
    floats = len(re.findall(r"\\begin\{(?:figure|table)\*?\}", body))
    body = re.sub(r"\\begin\{(figure|table)(\*?)\}.*?\\end\{\1\2\}", " ", body, flags=re.S)
    body = re.sub(r"\\(title|author|IEEEauthorblock[NA]|IEEEauthorrefmark)\{[^{}]*\}", " ", body)
    body = re.sub(r"\\(cite|ref|label|url|href|input)[a-z]*\*?(\[[^\]]*\])*\{[^{}]*\}", " ", body)
    body = re.sub(r"\\begin\{[a-zA-Z*]+\}(\[[^\]]*\])?(\{[^{}]*\})?|\\end\{[a-zA-Z*]+\}", " ", body)
    body = re.sub(r"\\[a-zA-Z]+\*?", " ", body)
    nrefs = len(re.findall(r"\\bibitem", refs_part))
    return dict(words=_words(body), abstract=_words(re.sub(r"\\[a-zA-Z]+", " ", abstract)),
                floats=floats, refs=nrefs, keys=set(cite_keys(text)))


def md_stats(text: str):
    text = re.sub(r"<!--.*?-->", " ", text, flags=re.S)
    code_words = sum(_words(c) for c in re.findall(r"```.*?```", text, flags=re.S))
    text = re.sub(r"```.*?```", " ", text, flags=re.S)
    refs = 0
    m = re.search(r"(?mi)^#{1,3}\s*(references|bibliography|sources)\s*$", text)
    refs_part = ""
    if m:
        refs_part = text[m.end():]
        text = text[: m.start()]
        refs = len(re.findall(r"(?m)^\s*(?:\d+[.)]|[-*]|\[\d+\])\s+\S", refs_part))
    abstract = ""
    m = re.search(r"(?mis)^#{1,3}\s*abstract\s*\n(.*?)(?=^#{1,3}\s|\Z)", text)
    if m:
        abstract = m.group(1)
    tables = len(re.findall(r"(?m)(?:^\|.*\n)+", text))
    images = len(re.findall(r"!\[[^\]]*\]\([^)]*\)", text))
    text2 = re.sub(r"(?m)^\|.*$", " ", text)
    text2 = re.sub(r"!\[[^\]]*\]\([^)]*\)", " ", text2)
    text2 = re.sub(r"\]\([^)]*\)", "]", text2)
    text2 = re.sub(r"(?m)^#{1,6}\s*(title|abstract)\b.*$", " ", text2, flags=re.I) if False else text2
    return dict(words=_words(text2), abstract=_words(abstract), floats=tables + images,
                refs=refs, code_words=code_words, keys=set())


def check(venue: dict, pdf=None, text_path=None, bib=None):
    L = venue.get("limits", {})
    out, fail = [], False

    def rep(name, val, lim, ok, extra=""):
        nonlocal fail
        out.append(f"{'PASS' if ok else 'FAIL'}  {name}: {val} (limit {lim}) {extra}".rstrip())
        fail |= not ok

    if pdf and "max_pages" in L:
        pages, content = pdf_pages(pdf)
        extra = L.get("extra_ref_pages", 0)
        if extra:
            note = "" if content is not None else "(no References heading found; counted all pages)"
            content = pages if content is None else content
            rep("content pages", content, L["max_pages"], content <= L["max_pages"], note)
            rep("total pages", pages, L["max_pages"] + extra, pages <= L["max_pages"] + extra)
        else:
            rep("pages", pages, L["max_pages"], pages <= L["max_pages"])
        if "min_pages" in L and pages < L["min_pages"]:
            out.append(f"WARN  pages {pages} below minimum {L['min_pages']}")
    if text_path:
        text = read(text_path)
        st = tex_stats(text) if is_tex(text, str(text_path)) else md_stats(text)
        cost = L.get("float_cost_words", 0)
        total = st["words"] + st["floats"] * cost
        if "max_words" in L:
            rep("words", total, L["max_words"], total <= L["max_words"],
                f"(body+abstract {st['words']}, {st['floats']} float(s) x {cost})")
        if "min_words" in L and st["words"] < L["min_words"]:
            out.append(f"WARN  words {st['words']} below minimum {L['min_words']}")
        if "abstract_max_words" in L:
            rep("abstract words", st["abstract"], L["abstract_max_words"], st["abstract"] <= L["abstract_max_words"])
        if "max_refs" in L:
            n = st["refs"]
            if not n and bib and Path(bib).is_file():
                n = len(st["keys"]) or len(re.findall(r"(?m)^@\w+\{", read(bib)))
            rep("references", n, L["max_refs"], n <= L["max_refs"])
        if not any(k in L for k in ("max_words", "abstract_max_words", "max_refs")) and not pdf:
            out.append(f"INFO  words {st['words']}, abstract {st['abstract']}, refs {st['refs']}, floats {st['floats']} (no text limits set)")
    if not L:
        out.append("WARN  venue.json has no limits; fill them in from the live call")
    return fail, out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--venue", required=True)
    ap.add_argument("--pdf")
    ap.add_argument("--text")
    ap.add_argument("--bib")
    a = ap.parse_args(argv)
    venue = json.loads(read(a.venue))
    if not venue.get("confirmed"):
        print("WARN  venue.json is not confirmed against the live call")
    fail, out = check(venue, a.pdf, a.text, a.bib)
    print("\n".join(out))
    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
