#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Citation gate for paper-from-pack.

  check_citations.py refs.bib --pack PACK.md [--draft DRAFT.tex] [--log citations.tsv]
                     [--netlog network.log] [--save-meta DIR] [--offline]

For every BibTeX entry:
  1. it must come from the pack: its arXiv id or DOI appears in the pack text, or the
     entry carries  packref = {exact phrase from the pack}  that appears there;
  2. it is looked up by identifier (arXiv API for eprint, Crossref for doi, dblp by
     title otherwise) and title, first-author surname and year are compared;
  3. the manual log (citations.tsv: key, sentence supported, read depth, note) must
     have a row for the key with read depth full|abstract; "not-read" fails.
Requests send only an identifier or a title (max 250 chars), never draft text, and
every URL is appended to the network log. Mismatches are reported, never fixed.
Exit 1 on NOT_FOUND, MISMATCH, NOT_IN_PACK or a missing/unread log row.
UNRESOLVED (no identifier and no dblp hit) fails only with --strict.
"""
from __future__ import annotations

import argparse
import difflib
import json
import re
import sys
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _common import ALLOWED_HOSTS, cite_keys, log_request, read  # noqa: E402

UA = "paper-from-pack/1.0 (citation metadata check)"
_last: dict[str, float] = {}
NETLOG = None


class NotFound(Exception):
    pass


def fetch(url: str, purpose: str) -> str:
    """The only network function. Host allowlist, length guard, logging, rate limit."""
    u = urllib.parse.urlparse(url)
    if u.scheme != "https" or u.hostname not in ALLOWED_HOSTS:
        raise RuntimeError(f"host not allowed: {url}")
    if len(u.query) > 400:
        raise RuntimeError("query too long; refusing to send possible draft text")
    wait = 1.0 - (time.time() - _last.get(u.hostname, 0))
    if wait > 0:
        time.sleep(wait)
    _last[u.hostname] = time.time()
    log_request(NETLOG, url, purpose)
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=25) as r:
            return r.read().decode("utf-8", "replace")
    except urllib.error.HTTPError as e:
        if e.code in (404, 410):
            raise NotFound(url)
        raise


# ---------- bib parsing ----------

def parse_bib(text: str):
    entries = []
    for m in re.finditer(r"@(\w+)\s*\{\s*([^,\s]+)\s*,(.*?)(?=\n@\w+\s*\{|\Z)", text, flags=re.S):
        body = m.group(3)
        f = {}
        for fm in re.finditer(r"(\w+)\s*=\s*(?:\{((?:[^{}]|\{[^{}]*\})*)\}|\"([^\"]*)\"|(\d+))", body):
            f[fm.group(1).lower()] = (fm.group(2) or fm.group(3) or fm.group(4) or "").strip()
        entries.append({"type": m.group(1).lower(), "key": m.group(2), **f})
    return entries


def strip_markup(s: str) -> str:
    """Crossref titles may carry HTML/JATS markup such as <scp>e</scp>."""
    import html
    return " ".join(html.unescape(re.sub(r"<[^>]+>", "", s or "")).split())


def clean(s):
    s = re.sub(r"[{}\\]", "", s or "")
    return re.sub(r"\s+", " ", s).strip()


def norm_title(s):
    """Lower-case letters and digits only: robust to spacing damage from markup."""
    return re.sub(r"[^a-z0-9]", "", clean(s).lower())


def first_surname(authors: str):
    a = clean(authors).split(" and ")[0].strip()
    if not a or a.lower() == "others":
        return ""
    return (a.split(",")[0] if "," in a else a.split()[-1]).lower()


# ---------- lookups (return dict title, authors[list of surnames], year) ----------

def lookup_arxiv(aid: str):
    aid = re.sub(r"^arxiv:", "", aid, flags=re.I)
    xml = fetch("https://export.arxiv.org/api/query?" + urllib.parse.urlencode({"id_list": aid}), "arxiv id")
    root = ET.fromstring(xml)
    ns = {"a": "http://www.w3.org/2005/Atom"}
    ent = root.find("a:entry", ns)
    if ent is None or ent.find("a:title", ns) is None or "Error" in (ent.findtext("a:title", "", ns) or ""):
        raise NotFound(aid)
    return {
        "title": " ".join((ent.findtext("a:title", "", ns)).split()),
        "authors": [n.findtext("a:name", "", ns).split()[-1].lower() for n in ent.findall("a:author", ns)],
        "authors_full": [n.findtext("a:name", "", ns) for n in ent.findall("a:author", ns)],
        "year": (ent.findtext("a:published", "", ns) or "")[:4],
        "abstract": " ".join((ent.findtext("a:summary", "", ns) or "").split()),
        "venue": "arXiv",
    }


def lookup_doi(doi: str):
    doi = re.sub(r"^(https?://(dx\.)?doi\.org/|doi:)", "", doi.strip(), flags=re.I)
    j = json.loads(fetch("https://api.crossref.org/works/" + urllib.parse.quote(doi, safe="/"), "doi"))["message"]
    yr = ""
    for k in ("issued", "published-print", "published-online"):
        if j.get(k, {}).get("date-parts", [[None]])[0][0]:
            yr = str(j[k]["date-parts"][0][0])
            break
    return {
        "title": strip_markup(" ".join(filter(None, [(j.get("title") or [""])[0], (j.get("subtitle") or [""])[0]]))),
        "authors": [a.get("family", "").lower() for a in j.get("author", [])],
        "year": yr,
        "venue": (j.get("container-title") or [""])[0],
    }


def lookup_dblp(title: str):
    t = clean(title)[:250]
    j = json.loads(fetch("https://dblp.org/search/publ/api?" + urllib.parse.urlencode(
        {"q": t, "format": "json", "h": 5}), "dblp title"))
    hits = j.get("result", {}).get("hits", {}).get("hit", [])
    best = None
    for h in hits:
        info = h.get("info", {})
        r = difflib.SequenceMatcher(None, norm_title(info.get("title", "")), norm_title(t)).ratio()
        if not best or r > best[0]:
            auth = info.get("authors", {}).get("author", [])
            auth = [auth] if isinstance(auth, dict) else auth
            best = (r, {"title": info.get("title", ""), "authors": [a["text"].split()[-1].lower() for a in auth],
                        "year": info.get("year", ""), "venue": info.get("venue", "")})
    if not best or best[0] < 0.6:
        raise NotFound(t)
    return best[1]


# ---------- comparison ----------

def compare(entry, meta):
    probs = []
    r = difflib.SequenceMatcher(None, norm_title(entry.get("title", "")), norm_title(meta["title"])).ratio()
    a_, b_ = norm_title(entry.get("title", "")), norm_title(meta["title"])
    contained = len(min(a_, b_, key=len)) >= 8 and (a_ in b_ or b_ in a_)   # subtitle or acronym differences
    if r < 0.85 and not contained:
        probs.append(f"title differs (similarity {r:.2f}): bib {clean(entry.get('title'))!r} vs found {meta['title']!r}")
    fs = first_surname(entry.get("author", ""))
    if fs and meta["authors"] and fs not in [a.lower() for a in meta["authors"]]:
        probs.append(f"first author {fs!r} not among found authors {meta['authors'][:4]}")
    if entry.get("year") and meta.get("year") and entry["year"] != meta["year"]:
        probs.append(f"year {entry['year']} vs found {meta['year']}")
    return probs


def in_pack(entry, pack: str):
    low = pack.lower()
    ids = []
    if entry.get("eprint"):
        ids.append(entry["eprint"].lower())
    if entry.get("doi"):
        ids.append(entry["doi"].lower())
    if any(i and i in low for i in ids):
        return True
    pr = clean(entry.get("packref", ""))
    return bool(pr) and pr.lower() in low


def read_log(path):
    rows = {}
    if path and Path(path).is_file():
        for line in read(path).splitlines():
            if not line.strip() or line.startswith("#") or line.lower().startswith("key\t"):
                continue
            c = line.split("\t")
            if len(c) >= 3:
                rows[c[0].strip()] = {"sentence": c[1].strip(), "depth": c[2].strip().lower()}
    return rows


def check_entry(e, save_dir=None, offline=False):
    """-> (status, details)"""
    if offline:
        return "UNRESOLVED", ["offline"]
    if not e.get("eprint") and not e.get("doi") and re.search(r"https?://", e.get("howpublished", "") + e.get("url", "")):
        return "UNRESOLVED", ["software or web source: no bibliographic index; check the URL by hand (no request sent)"]
    try:
        if e.get("eprint"):
            meta = lookup_arxiv(e["eprint"])
        elif e.get("doi"):
            meta = lookup_doi(e["doi"])
        elif e.get("title"):
            meta = lookup_dblp(e["title"])
        else:
            return "UNRESOLVED", ["no eprint, doi or title"]
    except NotFound:
        return "NOT_FOUND", ["identifier or title does not exist in the index"]
    except Exception as ex:  # network failure etc.
        return "UNRESOLVED", [f"lookup failed: {ex}"]
    if save_dir:
        Path(save_dir).mkdir(parents=True, exist_ok=True)
        (Path(save_dir) / f"{e['key']}.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
    probs = compare(e, meta)
    return ("MISMATCH", probs) if probs else ("VERIFIED", [f"{meta['title']} ({meta['year']}, {meta.get('venue','')})"])


def main(argv=None):
    global NETLOG
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("bib")
    ap.add_argument("--pack", required=True)
    ap.add_argument("--draft")
    ap.add_argument("--log")
    ap.add_argument("--netlog", default="network.log")
    ap.add_argument("--save-meta")
    ap.add_argument("--offline", action="store_true")
    ap.add_argument("--strict", action="store_true")
    a = ap.parse_args(argv)
    NETLOG = a.netlog
    entries = parse_bib(read(a.bib))
    pack = read(a.pack)
    log = read_log(a.log)
    fail = False
    print(f"{len(entries)} bib entries")
    for e in entries:
        k = e["key"]
        status, det = check_entry(e, a.save_meta, a.offline)
        notes = []
        if not in_pack(e, pack):
            status = "NOT_IN_PACK"
            det = ["neither its arXiv id/DOI nor a packref phrase appears in the pack"] + det
        if status in {"NOT_FOUND", "MISMATCH", "NOT_IN_PACK"} or (status == "UNRESOLVED" and a.strict):
            fail = True
        row = log.get(k)
        if a.log is not None:
            if not row or not row["sentence"]:
                notes.append("no row in the manual log (sentence supported)")
            elif row["depth"] == "software" and not (e.get("eprint") or e.get("doi")):
                notes.append("software or web source cited by name and version; no claim drawn from its content")
            elif row["depth"] not in {"full", "abstract"}:
                notes.append(f"read depth {row['depth']!r}: read at least the abstract")
            elif row["depth"] == "abstract":
                notes.append("read at abstract depth only; say so in OPEN.md")
            if notes and (not row or row["depth"] not in {"full", "abstract", "software"} or not row["sentence"]
                          or (row["depth"] == "software" and (e.get("eprint") or e.get("doi")))):
                fail = True
        print(f"{status:12} {k}: " + "; ".join(det + notes))
    if a.draft:
        used = set(cite_keys(read(a.draft)))
        keys = {e["key"] for e in entries}
        for k in sorted(used - keys):
            print(f"MISSING      cited key without a bib entry: {k}")
            fail = True
        for k in sorted(keys - used):
            print(f"UNCITED      bib entry never cited: {k}")
    print("FAIL" if fail else "OK")
    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
