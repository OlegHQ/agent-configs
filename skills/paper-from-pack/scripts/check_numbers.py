#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Number gate for paper-from-pack.

  check_numbers.py DRAFT --pack PACK.md [--root DIR ...] [--source FILE ...] [--allow allow.txt]
      Fails (exit 1) when a number in DRAFT is not found verbatim (after
      normalising thousands commas, a leading v, case) in the pack or in a results
      file the pack names (backticked paths ending .md .csv .tsv .txt .json,
      resolved under --root). Numbers inside "TO VERIFY:" spans are skipped.
      Prints where each accepted number was found, for CLAIMS.md.

  check_numbers.py --compare BEFORE AFTER
      Fails when the multiset of numbers, citation keys or TO VERIFY markers differs
      (used after the prose pass).

Allowlist file: one entry per line, "token  # reason" or "re:PATTERN  # reason".
Number words (two..twelve, hundred...) only warn.
"""
from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _common import (cite_keys, is_tex, norm_token, number_tokens, number_words, read,  # noqa: E402
                     reduce_for_numbers)

PATH_RE = re.compile(r"`([^`\s]+\.(?:md|csv|tsv|txt|json))`")


def load_allow(path):
    exact, pats = {}, []
    if not path:
        return exact, pats
    for line in read(path).splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        entry, _, reason = line.partition("#")
        entry = entry.strip()
        if entry.startswith("re:"):
            pats.append((re.compile(entry[3:].strip()), reason.strip()))
        else:
            exact[norm_token(entry)] = reason.strip()
    return exact, pats


def find_sources(pack_text: str, roots: list[Path], extra: list[str], exclude: list[str] = ()):
    found, missing = [], []
    for p in dict.fromkeys(PATH_RE.findall(pack_text)):
        p = p.replace("~/", "")
        if any(x in p for x in exclude):
            continue
        hit = None
        for r in roots:
            cand = r / p
            if cand.is_file():
                hit = cand
                break
        (found if hit else missing).append(hit or p)
    for e in extra:
        (found if Path(e).is_file() else missing).append(Path(e) if Path(e).is_file() else e)
    return found, missing


def corpus_index(pack_path: str, sources: list[Path]):
    """token -> first (label, line)."""
    idx: dict[str, list[tuple[str, int]]] = {}
    ptext = read(pack_path)
    heading = "pack"
    for i, line in enumerate(ptext.splitlines(), 1):
        if line.startswith("#"):
            heading = "pack " + line.strip("# ").strip()[:50]
        for t, _raw, _ in number_tokens(line, tex=False):
            idx.setdefault(t, []).append((heading, i))
    for s in sources:
        for i, line in enumerate(read(s).splitlines(), 1):
            for t, _raw, _ in number_tokens(line, tex=False):
                idx.setdefault(t, []).append((str(s.name), i))
        # hashes / digests also appear as raw words; covered by tokeniser
    return idx


def words_in_corpus(pack_path, sources):
    txt = read(pack_path).lower() + "\n".join(read(s).lower() for s in sources)
    return txt


def check(draft_path, pack_path, roots, extra, allow_path, exclude=()):
    text = read(draft_path)
    tex = is_tex(text, str(draft_path))
    found, missing = find_sources(read(pack_path), roots, extra, exclude)
    idx = corpus_index(pack_path, found)
    exact, pats = load_allow(allow_path)
    bad, ok, allowed = [], {}, {}
    for t, raw, ln in number_tokens(text, tex):
        if t in idx:
            ok.setdefault(t, (raw, ln, idx[t][:4]))
        elif t in exact:
            allowed.setdefault(t, (raw, ln, exact[t]))
        elif any(p.search(t) for p, _ in pats):
            allowed.setdefault(t, (raw, ln, "pattern"))
        else:
            bad.append((t, raw, ln))
    corp = words_in_corpus(pack_path, found)
    nw = []
    for w, ln in number_words(text, tex):
        if w not in corp:
            nw.append((w, ln))
    return dict(bad=bad, ok=ok, allowed=allowed, found=found, missing=missing, number_words=nw)


def signature(path):
    text = read(path)
    tex = is_tex(text, str(path))
    nums = Counter(t for t, _, _ in number_tokens(text, tex))
    cites = Counter(cite_keys(text))
    tv = len(re.findall(r"TO VERIFY", text))
    return nums, cites, tv


def compare(before, after):
    nb, cb, tb = signature(before)
    na, ca, ta = signature(after)
    problems = []
    for t in sorted(set(nb) | set(na)):
        if nb[t] != na[t]:
            problems.append(f"number {t!r}: {nb[t]}x before, {na[t]}x after")
    for k in sorted(set(cb) | set(ca)):
        if cb[k] != ca[k]:
            problems.append(f"citation {k!r}: {cb[k]}x before, {ca[k]}x after")
    if tb != ta:
        problems.append(f"TO VERIFY markers: {tb} before, {ta} after")
    return problems


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("draft", nargs="?")
    ap.add_argument("--pack")
    ap.add_argument("--root", action="append", default=[])
    ap.add_argument("--source", action="append", default=[])
    ap.add_argument("--allow")
    ap.add_argument("--exclude", action="append", default=[],
                    help="skip named files whose path contains this text (context files that are not data)")
    ap.add_argument("--compare", nargs=2, metavar=("BEFORE", "AFTER"))
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args(argv)
    if a.compare:
        probs = compare(*a.compare)
        if probs:
            print("FAIL: the pass changed numbers, citations or TO VERIFY markers:")
            for p in probs:
                print("  -", p)
            return 1
        print("OK: numbers, citation keys and TO VERIFY markers unchanged.")
        return 0
    if not (a.draft and a.pack):
        ap.error("DRAFT and --pack are required")
    roots = [Path(r) for r in a.root] + [Path(a.pack).resolve().parent, Path.cwd()]
    res = check(a.draft, a.pack, roots, a.source, a.allow, a.exclude)
    print(f"sources read: {len(res['found'])}; named but not found: {len(res['missing'])}")
    for m in res["missing"]:
        print("  not found:", m)
    if not a.quiet:
        for t, (raw, ln, srcs) in sorted(res["ok"].items(), key=lambda kv: kv[1][1]):
            print(f"  ok   {raw!r} (draft line {ln}) <- " + "; ".join(f"{l}:{n}" for l, n in srcs))
        for t, (raw, ln, why) in res["allowed"].items():
            print(f"  allow {raw!r} (draft line {ln}) reason: {why or 'none given'}")
    for w, ln in res["number_words"]:
        print(f"  WARN number word {w!r} at line {ln} is not in the pack; check it is not a count")
    if res["bad"]:
        print("FAIL: numbers not found in the pack or its results files:")
        for t, raw, ln in res["bad"]:
            print(f"  line {ln}: {raw}")
        print("Write TO VERIFY in the draft and add the item to OPEN.md, or allowlist with a reason.")
        return 1
    print("OK: every number in the draft is in the pack or a listed results file.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
