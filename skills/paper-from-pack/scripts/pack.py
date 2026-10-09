#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Intake for paper-from-pack.

  pack.py intake SOURCE [--out-root DIR] [--override]
      SOURCE is a Nudge issue key (e.g. ABC-123) or a path to a pack file.
      Extracts the pack (text between the writer-pack markers), checks the
      Readiness line, classifies the format, creates the working folder
      (drafts/<piece>/ unless the pack names another), writes PACK.md and a
      draft venue.json (limits parsed from the Format line; confirm by hand).

Only reads the issue (nudge issue get). Sends nothing anywhere.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

START, END = "<!-- writer-pack:start -->", "<!-- writer-pack:end -->"
BLOCKING = ("WAITS", "HOLD", "IDEA", "VERIFY")
KEY_RE = re.compile(r"^[A-Za-z][A-Za-z0-9]*-\d+$")


def _walk_strings(o):
    if isinstance(o, str):
        yield o
    elif isinstance(o, dict):
        for v in o.values():
            yield from _walk_strings(v)
    elif isinstance(o, list):
        for v in o:
            yield from _walk_strings(v)


def extract_pack(text: str) -> str | None:
    if START in text and END in text:
        return text.split(START, 1)[1].split(END, 1)[0].strip("\n")
    return None


def load_source(source: str) -> tuple[str, str]:
    p = Path(source)
    if p.is_file():
        raw = p.read_text(encoding="utf-8")
        return (extract_pack(raw) or raw), f"file:{p}"
    if KEY_RE.match(source):
        out = subprocess.run(["nudge", "issue", "get", source, "-o", "json"],
                             capture_output=True, text=True)
        if out.returncode != 0:
            raise SystemExit(f"nudge issue get failed ({out.returncode}): {out.stderr.strip()[:300]}\n"
                             "Fall back to a pack file path.")
        data = json.loads(out.stdout)
        for s in _walk_strings(data):
            pk = extract_pack(s)
            if pk:
                return pk, f"nudge:{source}"
        raise SystemExit(f"No writer-pack markers found in issue {source}. Pass a pack file instead.")
    raise SystemExit(f"{source!r} is neither a file nor an issue key.")


def field(pack: str, name: str) -> str | None:
    m = re.search(r"^\*\*%s:?\*\*:?\s*(.+)$" % re.escape(name), pack, flags=re.M)
    return m.group(1).strip() if m else None


def _n(s: str) -> int:
    return int(s.replace(",", ""))


def parse_limits(fmt: str, rules: str = "") -> dict:
    t = f"{fmt} {rules}"
    L: dict = {}
    m = re.search(r"(\d+)\s*[–-]\s*(\d+)\s*pages?", t)
    if m:
        L["min_pages"], L["max_pages"] = int(m.group(1)), int(m.group(2))
    m = re.search(r"(\d+)\s*pages?\s*\+\s*(\d+)(?:\s*page)?\s*(?:for\s+)?references", t) or \
        re.search(r"(\d+)\s*\+\s*(\d+)\s*pages", t)
    if m:
        L["max_pages"], L["extra_ref_pages"] = int(m.group(1)), int(m.group(2))
        L["pages_include_refs"] = False
    m = re.search(r"up to (\d+) pages? excluding references", t)
    if m:
        L["max_pages"], L["pages_include_refs"] = int(m.group(1)), False
    m = re.search(r"(?:up to |^|\b)(\d+)[- ]pages?\b(?! \+)", fmt)
    if m and "max_pages" not in L:
        L["max_pages"] = int(m.group(1))
    m = re.search(r"([\d,]+)\s*[–-]\s*([\d,]+)\s*words", t)
    if m:
        L["min_words"], L["max_words"] = _n(m.group(1)), _n(m.group(2))
    m = re.search(r"([\d,]+)\s*words including (\d+) per figure or table", t)
    if m:
        L["max_words"], L["float_cost_words"] = _n(m.group(1)), int(m.group(2))
    m = re.search(r"at most (\d+) references", t)
    if m:
        L["max_refs"] = int(m.group(1))
    m = re.search(r"abstract[^.\d]{0,20}?(?:at most|up to|of|≤|<=|max(?:imum)?)\s*(\d+)\s*words", t, flags=re.I)
    if m:
        L["abstract_max_words"] = int(m.group(1))
    if "including references" in t or "incl. refs" in t:
        L["pages_include_refs"] = True
    return L


def classify(fmt: str, rules: str, readiness: str, pack: str) -> str:
    t = f"{fmt} {rules}".lower()
    r = (readiness or "").lower()
    if "outline only" in r or "outline, fact sheet" in r or "do not use ai to write" in t or \
            "ai-detection" in t or "bans ai" in t or "ban ai" in t:
        return "outline-only"
    if "talk proposal" in t or "industrial abstract" in t or "talk via the conference form" in t or \
            re.search(r"\b[12][- ]page (industrial|abstract|talk)", t) or "1 page + 1 for references" in t:
        return "proposal"
    if "acmart" in t or "acm format" in t:
        return "acm"
    if "springer" in t or "ccis" in t or "pattern paper" in t:
        return "springer-ccis"
    if "ieee software" in t or "ieee computer society" in t or "per figure or table" in t:
        return "ieee-cs-magazine"
    if "ieeetran" in t or "ieee format" in t or "ieee conference" in t:
        return "ieee-conf"
    if "per live call" in t:
        return "unknown"
    return "prose-md"


def piece_code(pack: str) -> str:
    m = re.search(r"piece\s+([A-Za-z]+\d+)", pack)
    return m.group(1) if m else "piece"


def workdir_from_pack(pack: str, code: str) -> str:
    m = re.search(r"drafts/[A-Za-z0-9_-]+/", pack)
    return m.group(0).rstrip("/") if m else f"drafts/{code}"


def build_venue(pack: str) -> dict:
    fmt = field(pack, "Format") or ""
    rules = field(pack, "Venue rules") or ""
    rd = field(pack, "Readiness") or ""
    code = piece_code(pack)
    low = f"{fmt} {rules}".lower()
    if re.search(r"double[- ](blind|anonymous)", low):
        db = True
    elif re.search(r"single[- ](blind|anonymous)|non-anonymous", low):
        db = False
    else:
        db = None
    return {
        "piece": code,
        "title": field(pack, "Working title"),
        "deadline": field(pack, "Deadline"),
        "format_line": fmt,
        "readiness": rd,
        "class": classify(fmt, rules, rd, pack),
        "double_blind": db,
        "limits": parse_limits(fmt, rules),
        "ai_prose_banned": ("outline" in rd.lower() and "only" in rd.lower()) or "do not use ai to write" in low,
        "confirmed": False,
        "note": "Parsed by pack.py from the Format line. Confirm against the live call and set confirmed=true.",
    }


def readiness_gate(readiness: str) -> str | None:
    head = re.split(r"[\s:(,]", readiness.strip(), maxsplit=1)[0].upper() if readiness else ""
    if not readiness:
        return "no Readiness line in the pack"
    if head in BLOCKING:
        return head
    if not head.startswith("READY"):
        return f"unrecognised readiness {readiness!r}"
    return None


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    i = sub.add_parser("intake")
    i.add_argument("source")
    i.add_argument("--out-root", default=".")
    i.add_argument("--override", action="store_true", help="start although Readiness blocks")
    i.add_argument("--dry-run", action="store_true")
    a = ap.parse_args(argv)

    pack, origin = load_source(a.source)
    venue = build_venue(pack)
    block = readiness_gate(venue["readiness"])
    if block and not a.override:
        print(f"REFUSED: Readiness is {venue['readiness']!r} ({block}).")
        print("Missing dependency, from the pack's own words: " + venue["readiness"])
        print("Re-run with --override only if the author said so explicitly.")
        return 3
    wd = Path(a.out_root) / workdir_from_pack(pack, venue["piece"])
    venue["override_used"] = bool(block)
    summary = {"origin": origin, "workdir": str(wd), "venue": venue}
    if not a.dry_run:
        wd.mkdir(parents=True, exist_ok=True)
        (wd / "PACK.md").write_text(pack + "\n", encoding="utf-8")
        (wd / "venue.json").write_text(json.dumps(venue, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
