"""Shared helpers for paper-from-pack scripts. Standard library only."""
from __future__ import annotations

import re
import sys
import time
from pathlib import Path

# ---------- text reading ----------

def read(path) -> str:
    return Path(path).read_text(encoding="utf-8", errors="replace")


def is_tex(text: str, name: str = "") -> bool:
    return name.endswith(".tex") or "\\documentclass" in text or "\\begin{document}" in text


# ---------- LaTeX / Markdown reduction ----------

_TV = re.compile(r"TO VERIFY:?[^\n}]*")
_STRIP_CMDS = (
    "cite", "citep", "citet", "ref", "eqref", "label", "includegraphics", "url", "href",
    "input", "include", "bibliography", "bibliographystyle", "vspace", "hspace", "setlength",
    "usepackage", "documentclass", "pagestyle", "thispagestyle", "cline", "hyphenation",
    "footnotemark", "newcommand", "renewcommand", "definecolor", "color", "textcolor",
)


def strip_comments_tex(text: str) -> str:
    out = []
    for line in text.splitlines():
        m = re.search(r"(?<!\\)%", line)
        out.append(line[: m.start()] if m else line)
    return "\n".join(out)


def tex_body(text: str) -> str:
    """Return text between \\begin{document} and \\end{document} (or all)."""
    m = re.search(r"\\begin\{document\}", text)
    if m:
        text = text[m.end():]
    m = re.search(r"\\end\{document\}", text)
    if m:
        text = text[: m.start()]
    return text


def reduce_for_numbers(text: str, tex: bool) -> str:
    """Blank out things that carry digits but are not claims (keeps line numbers)."""
    if tex:
        text = strip_comments_tex(text)
        text = tex_body(text) if "\\begin{document}" in text else text
        for c in _STRIP_CMDS:
            text = re.sub(r"\\%s\*?(\[[^\]]*\])?(\{[^{}]*\})*" % c, lambda m: " " * 0, text)
        text = re.sub(r"\\\\\[[^\]]*\]", " ", text)
        text = re.sub(r"\\begin\{[a-zA-Z*]+\}(\[[^\]]*\])?(\{[^{}]*\})?", " ", text)
        text = re.sub(r"\\end\{[a-zA-Z*]+\}", " ", text)
        text = re.sub(r"\\(sub)*section\*?", " ", text)
        text = re.sub(r"\\[a-zA-Z]+", " ", text)
        text = text.replace("\\%", "%")
    else:
        text = re.sub(r"```.*?```", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)
        text = re.sub(r"<!--.*?-->", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)
        text = re.sub(r"\]\([^)]*\)", "]", text)           # link targets
        text = re.sub(r"https?://\S+", " ", text)
        text = re.sub(r"(?m)^\s*#+\s*\d+(\.\d+)*[.)]?\s+", "# ", text)   # numbered headings
        text = re.sub(r"(?m)^\s*\d+[.)]\s+", "- ", text)                  # ordered-list numbers
    text = _TV.sub(" ", text)
    return text


_NUM = re.compile(r"(?<![A-Za-z0-9_.])([A-Za-z]{0,2}\d+(?:[.,]\d+)*[A-Za-z0-9_]*)")
_NUMWORDS = {
    "two": 2, "three": 3, "four": 4, "five": 5, "six": 6, "seven": 7, "eight": 8,
    "nine": 9, "ten": 10, "eleven": 11, "twelve": 12, "twenty": 20, "thirty": 30,
    "hundred": 100, "thousand": 1000, "million": 10**6,
}


def norm_token(tok: str) -> str:
    t = tok.lower()
    if re.fullmatch(r"\d{1,3}(,\d{3})+(\.\d+)?", t):
        t = t.replace(",", "")
    t = t.rstrip(".,")
    if re.fullmatch(r"v\d.*", t):
        t = t[1:]
    return t


def number_tokens(text: str, tex: bool):
    """Yield (normalised_token, raw, lineno)."""
    red = reduce_for_numbers(text, tex)
    for i, line in enumerate(red.splitlines(), 1):
        for m in _NUM.finditer(line):
            raw = m.group(1)
            if not re.search(r"\d", raw):
                continue
            yield norm_token(raw), raw, i


def number_words(text: str, tex: bool):
    red = reduce_for_numbers(text, tex)
    for i, line in enumerate(red.splitlines(), 1):
        for m in re.finditer(r"\b(" + "|".join(_NUMWORDS) + r")\b", line, flags=re.I):
            yield m.group(1).lower(), i


def cite_keys(text: str) -> list[str]:
    keys = []
    for m in re.finditer(r"\\cite[a-z]*\*?(?:\[[^\]]*\])*\{([^}]*)\}", text):
        keys += [k.strip() for k in m.group(1).split(",") if k.strip()]
    for m in re.finditer(r"\[@([A-Za-z0-9_:.\-]+(?:;\s*@[A-Za-z0-9_:.\-]+)*)\]", text):
        keys += [k.strip().lstrip("@") for k in m.group(1).split(";")]
    return keys


# ---------- network log ----------

ALLOWED_HOSTS = {
    "export.arxiv.org", "arxiv.org", "api.crossref.org", "dblp.org",
    "github.com", "objects.githubusercontent.com", "release-assets.githubusercontent.com",
    "relay.fullyjustified.net", "data1.fullyjustified.net",
}


def log_request(logfile, url: str, purpose: str):
    if logfile:
        with open(logfile, "a", encoding="utf-8") as f:
            f.write(f"{time.strftime('%Y-%m-%dT%H:%M:%S%z')}\tGET\t{url}\t{purpose}\n")


def die(msg: str, code: int = 2):
    print(msg, file=sys.stderr)
    sys.exit(code)
