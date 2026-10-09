#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""CLAIMS.md gate.

  check_claims.py CLAIMS.md DRAFT

CLAIMS.md holds a Markdown table with columns: ID | Type | Claim | Source | Anchor
Type is one of fact, number, citation, limitation, interview, inference, strategy.
Fails when a row has no source, when a fact/number/limitation row cites only the
author's memory, or when its Anchor (a short verbatim phrase of the draft) is missing
from the draft. Limitation rows are mandatory anchors: this is how the prose pass is
checked for dropped limitations.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _common import read  # noqa: E402

TYPES = {"fact", "number", "citation", "limitation", "interview", "inference", "strategy"}


def norm(s):
    s = re.sub(r"\\[a-zA-Z]+\*?(\{([^{}]*)\})?", lambda m: m.group(2) or " ", s)
    s = s.replace("\\%", "%").replace("~", " ").replace("``", '"').replace("''", '"')
    s = s.replace("\u2019", "'").replace("--", "-")
    return re.sub(r"[\s{}$\\]+", " ", s).strip().lower()


def rows(text):
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) >= 5 and not set(cells[0]) <= set("-: ") and cells[0].lower() != "id":
            yield cells[:5]


def main(argv=None):
    if len(argv or sys.argv[1:]) != 2:
        print(__doc__)
        return 2
    claims, draft = (argv or sys.argv[1:])
    dtext = norm(read(draft))
    problems, n = [], 0
    for cid, typ, claim, src, anchor in rows(read(claims)):
        n += 1
        t = typ.lower()
        if t not in TYPES:
            problems.append(f"{cid}: unknown type {typ!r}")
        if not src or src.lower() in {"-", "none", "tbd", "?"}:
            problems.append(f"{cid}: no source")
        elif t in {"fact", "number", "limitation"} and re.fullmatch(r"(?i)(memory|author memory|assumed)", src):
            problems.append(f"{cid}: {t} claim sourced only to memory")
        if t in {"limitation", "number", "citation"} and not anchor:
            problems.append(f"{cid}: {t} row needs an anchor phrase")
        if anchor and norm(anchor.strip("`")) not in dtext:
            problems.append(f"{cid}: anchor not found in draft: {anchor!r}")
    if n == 0:
        problems.append("no claim rows found")
    if problems:
        print("FAIL:")
        for p in problems:
            print("  -", p)
        return 1
    print(f"OK: {n} claim rows, all sourced and anchored.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
