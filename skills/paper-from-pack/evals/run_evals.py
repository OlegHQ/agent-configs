#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["pypdf>=4"]
# ///
"""Run the paper-from-pack evals: one fresh agent session per case, with the skill and
(for a subset) without it, grade mechanically, write a results table.

  run_evals.py [--modes with,baseline] [--cases id,id] [--parallel 4] [--out DIR]
               [--agent-cmd "claude -p {prompt} --model {model} ..."] [--model sonnet]
               [--max-sessions 30] [--timeout 1800] [--regrade DIR]

--agent-cmd is a shell-style template; {prompt}, {model} and {cwd} are substituted.
The default drives the Claude Code CLI non-interactively. To use another agent put its
non-interactive command here (the prompt is a single argument). The with-skill sessions
get the skill copied to .claude/skills/ and .agents/skills/ inside the case work directory.
Sessions run in a throwaway directory containing only the synthetic pack.
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import json
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILL = HERE.parent
sys.path.insert(0, str(HERE))
import grade as G  # noqa: E402

DEFAULT_CMD = ("claude -p {prompt} --model {model} --permission-mode acceptEdits "
               "--allowedTools Bash Read Write Edit Glob Grep WebFetch --no-session-persistence")

PROMPT_WITH = (
    "Use the paper-from-pack skill. Its instructions are in .claude/skills/paper-from-pack/SKILL.md "
    "(also copied to .agents/skills/paper-from-pack/); scripts and references are beside it. "
    "The writer pack is ./PACK.md and its data files are under ./data/. Work in this directory and use it as --out-root. "
    "The venue in this pack is synthetic, so the live call page cannot be fetched; record that in OPEN.md as the skill says. "
    "Do not ask me questions; where the skill says to ask the author, write the question in OPEN.md and continue unless the skill says to stop. "
    "Do not submit or contact anyone. When you finish, reply with a summary of the files and the check results."
)
PROMPT_BASE = (
    "Here is a writer pack at ./PACK.md (data files under ./data/). Follow its instructions and produce the deliverables it asks for "
    "in this directory, in the venue's format. Do not ask me questions; write open points in OPEN.md. "
    "If the format needs a PDF, compile it (a TeX installation with latexmk and pdflatex is available). "
    "When you finish, reply with a summary of the files you produced."
)


def prepare(case, wd: Path, with_skill: bool):
    if wd.exists():
        shutil.rmtree(wd)
    src = HERE / Path(case["pack"]).parent
    shutil.copytree(src, wd)
    if with_skill:
        for tgt in (wd / ".claude" / "skills" / "paper-from-pack", wd / ".agents" / "skills" / "paper-from-pack"):
            shutil.copytree(SKILL, tgt, ignore=shutil.ignore_patterns("evals", ".cache", "__pycache__", ".pytest_cache", "tests"))


def run_one(case, mode, out: Path, cmd_t, model, timeout):
    wd = out / mode / case["id"]
    prepare(case, wd, mode == "with")
    prompt = PROMPT_WITH if mode == "with" else PROMPT_BASE
    argv = [x.replace("{prompt}", prompt).replace("{model}", model).replace("{cwd}", str(wd)) for x in shlex.split(cmd_t)]
    t0 = time.time()
    try:
        p = subprocess.run(argv, cwd=wd, capture_output=True, text=True, timeout=timeout)
        resp, rc = p.stdout + ("\n[stderr]\n" + p.stderr[-2000:] if p.returncode else ""), p.returncode
    except subprocess.TimeoutExpired as e:
        resp, rc = f"TIMEOUT after {timeout}s\n{(e.stdout or b'')[-2000:] if isinstance(e.stdout, bytes) else (e.stdout or '')}", 124
    secs = time.time() - t0
    (wd / "agent_response.txt").write_text(resp, encoding="utf-8")
    res = G.grade(case, wd, resp)
    res.update(mode=mode, seconds=round(secs), exit_code=rc)
    (wd / "grade.json").write_text(json.dumps(res, indent=1), encoding="utf-8")
    return res


def table(results, cases):
    by = {(r["case"], r["mode"]): r for r in results}
    lines = ["| case | with skill | baseline | with: failed assertions | wall time (s) with / baseline |", "|---|---|---|---|---|"]
    for c in cases:
        w, b = by.get((c["id"], "with")), by.get((c["id"], "baseline"))
        f = lambda r: f"{r['passed']}/{r['total']}" if r else "not run"
        fails = "; ".join(f"{x['type']} ({x['detail'][:60]})" for x in (w["results"] if w else []) if not x["pass"]) or "none"
        t = f"{w['seconds'] if w else '-'} / {b['seconds'] if b else '-'}"
        lines.append(f"| {c['id']} | {f(w)} | {f(b)} | {fails} | {t} |")
    for mode in ("with", "baseline"):
        rs = [r for r in results if r["mode"] == mode]
        if rs:
            p, t = sum(r["passed"] for r in rs), sum(r["total"] for r in rs)
            lines.append(f"\n**{mode}**: {p}/{t} assertions ({100 * p / t:.0f}%) over {len(rs)} cases; "
                         f"cases with all assertions passing: {sum(r['passed'] == r['total'] for r in rs)}/{len(rs)}")
    return "\n".join(lines)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--modes", default="with,baseline")
    ap.add_argument("--cases")
    ap.add_argument("--parallel", type=int, default=4)
    ap.add_argument("--out", default=str(HERE / "runs" / time.strftime("%Y%m%d-%H%M%S")))
    ap.add_argument("--agent-cmd", default=DEFAULT_CMD)
    ap.add_argument("--model", default="sonnet")
    ap.add_argument("--max-sessions", type=int, default=30)
    ap.add_argument("--timeout", type=int, default=1800)
    ap.add_argument("--lenient-names", action="store_true", help="accept any .tex/.md as the draft (regrade the baseline fairly)")
    ap.add_argument("--regrade", help="re-grade an existing runs directory without launching agents")
    a = ap.parse_args(argv)
    G.LENIENT = a.lenient_names
    cases = json.loads((HERE / "evals.json").read_text())["cases"]
    if a.cases:
        want = set(a.cases.split(","))
        cases = [c for c in cases if c["id"] in want]
    out = Path(a.regrade or a.out)
    jobs = []
    for c in cases:
        for m in a.modes.split(","):
            if m == "baseline" and not c["baseline"]:
                continue
            jobs.append((c, m))
    results = []
    if a.regrade:
        for c, m in jobs:
            wd = out / m / c["id"]
            if wd.exists():
                r = G.grade(c, wd, (wd / "agent_response.txt").read_text() if (wd / "agent_response.txt").exists() else "")
                old = json.loads((wd / "grade.json").read_text()) if (wd / "grade.json").exists() else {}
                r.update(mode=m, seconds=old.get("seconds", 0), exit_code=old.get("exit_code", 0))
                (wd / "grade.json").write_text(json.dumps(r, indent=1))
                results.append(r)
    else:
        if len(jobs) > a.max_sessions:
            print(f"refusing: {len(jobs)} sessions exceed --max-sessions {a.max_sessions}")
            return 2
        out.mkdir(parents=True, exist_ok=True)
        with cf.ThreadPoolExecutor(a.parallel) as ex:
            futs = {ex.submit(run_one, c, m, out, a.agent_cmd, a.model, a.timeout): (c, m) for c, m in jobs}
            for f in cf.as_completed(futs):
                r = f.result()
                print(f"{r['case']:18} {r['mode']:8} {r['passed']}/{r['total']} {r['seconds']}s", flush=True)
                results.append(r)
    results.sort(key=lambda r: (r["case"], r["mode"]))
    (out / "results.json").write_text(json.dumps(results, indent=1), encoding="utf-8")
    md = table(results, cases)
    (out / "results.md").write_text(md + "\n", encoding="utf-8")
    print(md)
    return 0


if __name__ == "__main__":
    sys.exit(main())
