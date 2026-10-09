"""Tests for paper-from-pack scripts. Run: uv run --with pytest --with pypdf pytest skills/paper-from-pack/tests"""
import json
import subprocess
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
SCRIPTS = HERE.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import check_citations  # noqa: E402
import check_claims  # noqa: E402
import check_identity  # noqa: E402
import check_limits  # noqa: E402
import check_numbers  # noqa: E402
import pack  # noqa: E402

FIX = HERE / "fixtures"


def draft(tmp_path, body, name="d.tex"):
    p = tmp_path / name
    p.write_text("\\documentclass{article}\n\\begin{document}\n" + body + "\n\\end{document}\n" if name.endswith(".tex") else body)
    return p


# ---- numbers ----

def test_number_not_in_pack_is_caught(tmp_path):
    d = draft(tmp_path, "We ran 120 runs and 4 tools. Median 7.5 s. We saw 77 failures.")
    res = check_numbers.check(d, FIX / "pack_ok.md", [FIX], [], None)
    assert [b[1] for b in res["bad"]] == ["77"]


def test_numbers_from_pack_and_results_pass(tmp_path):
    d = draft(tmp_path, "18 of 40 builds were stale; 120 runs.")
    res = check_numbers.check(d, FIX / "pack_ok.md", [FIX], [], None)
    assert res["bad"] == []


def test_allowlist_and_to_verify_span(tmp_path):
    d = draft(tmp_path, "In 2098 we saw 120 runs. TO VERIFY: the 9999 figure.")
    allow = tmp_path / "allow.txt"
    allow.write_text("2098  # venue year\n")
    res = check_numbers.check(d, FIX / "pack_ok.md", [FIX], [], allow)
    assert res["bad"] == [] and "2098" in res["allowed"]


def test_number_changed_by_prose_pass_is_caught(tmp_path):
    a = draft(tmp_path, "We ran 120 runs over 4 tools \\cite{k1}.", "a.tex")
    b = draft(tmp_path, "We ran 112 runs over 4 tools \\cite{k1}.", "b.tex")
    assert check_numbers.compare(a, b)
    c = draft(tmp_path, "Over 4 tools we ran 120 runs \\cite{k1}.", "c.tex")
    assert check_numbers.compare(a, c) == []
    d = draft(tmp_path, "Over 4 tools we ran 120 runs.", "d.tex")
    assert any("citation" in p for p in check_numbers.compare(a, d))


# ---- citations ----

def test_fabricated_doi_is_flagged(monkeypatch):
    def nf(url, purpose):
        raise check_citations.NotFound(url)
    monkeypatch.setattr(check_citations, "fetch", nf)
    status, _ = check_citations.check_entry({"key": "x", "doi": "10.9999/does.not.exist", "title": "Fake"})
    assert status == "NOT_FOUND"


def test_metadata_mismatch_is_reported(monkeypatch):
    monkeypatch.setattr(check_citations, "lookup_arxiv", lambda i: {
        "title": "A real paper on caching", "authors": ["smith"], "year": "2021", "venue": "arXiv"})
    e = {"key": "k", "eprint": "2101.00001", "title": "Another title entirely", "author": "Jones, A", "year": "2019"}
    status, det = check_citations.check_entry(e)
    assert status == "MISMATCH" and len(det) == 3


def test_crossref_title_markup_is_stripped():
    assert check_citations.strip_markup("D <scp>e</scp> F <scp>laker</scp>: tests &amp; more") == "D e F laker: tests & more"
    e = {"key": "k", "title": "DeFlaker"}
    meta = {"title": check_citations.strip_markup("D <scp>e</scp> F <scp>laker</scp>"), "authors": [], "year": ""}
    assert check_citations.compare(e, meta) == []
    # Crossref may split a title and a subtitle
    e2 = {"key": "k", "title": "{DeFlaker}: Automatically detecting flaky tests"}
    assert check_citations.compare(e2, {"title": "D e F laker", "authors": [], "year": ""}) == []
    assert check_citations.compare(e2, {"title": "Something unrelated entirely", "authors": [], "year": ""})


def test_entry_not_in_pack():
    e = {"key": "k", "eprint": "2999.99999"}
    assert not check_citations.in_pack(e, (FIX / "pack_ok.md").read_text())
    assert check_citations.in_pack({"key": "k", "eprint": "2101.00001"}, (FIX / "pack_ok.md").read_text())


def test_fetch_refuses_foreign_host_and_long_query(tmp_path):
    check_citations.NETLOG = str(tmp_path / "n.log")
    with pytest.raises(RuntimeError):
        check_citations.fetch("https://example.com/x", "t")
    with pytest.raises(RuntimeError):
        check_citations.fetch("https://api.crossref.org/works?q=" + "a" * 500, "t")


# ---- limits ----

def make_pdf(path, pages):
    from pypdf import PdfWriter
    w = PdfWriter()
    for _ in range(pages):
        w.add_blank_page(width=612, height=792)
    with open(path, "wb") as f:
        w.write(f)


def test_over_length_pdf_fails(tmp_path):
    pdf = tmp_path / "x.pdf"
    make_pdf(pdf, 5)
    fail, out = check_limits.check({"limits": {"max_pages": 4}}, pdf=pdf)
    assert fail and "FAIL  pages" in out[0]
    make_pdf(pdf, 4)
    fail, _ = check_limits.check({"limits": {"max_pages": 4}}, pdf=pdf)
    assert not fail


def test_over_length_abstract_fails(tmp_path):
    t = tmp_path / "d.md"
    t.write_text("# T\n\n## Abstract\n" + "word " * 151 + "\n\n## Body\nshort\n")
    fail, out = check_limits.check({"limits": {"abstract_max_words": 150}}, text_path=t)
    assert fail and any("abstract" in o and "FAIL" in o for o in out)
    t.write_text("# T\n\n## Abstract\n" + "word " * 150 + "\n\n## Body\nshort\n")
    assert not check_limits.check({"limits": {"abstract_max_words": 150}}, text_path=t)[0]


def test_magazine_float_cost_and_refs(tmp_path):
    t = tmp_path / "d.md"
    t.write_text("# T\n\n" + "word " * 100 + "\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n## References\n1. A\n2. B\n3. C\n")
    fail, out = check_limits.check({"limits": {"max_words": 300, "float_cost_words": 250, "max_refs": 2}}, text_path=t)
    assert fail
    assert any("FAIL  words" in o for o in out) and any("FAIL  references" in o for o in out)


def test_tex_word_count_excludes_markup(tmp_path):
    t = draft(tmp_path, "\\section{Intro}\nOne two three \\cite{a}. % comment words here\n"
                        "\\begin{table}\\caption{ignored words}\\end{table}")
    fail, out = check_limits.check({"limits": {"max_words": 254, "float_cost_words": 250}}, text_path=t)
    assert not fail, out   # Intro One two three = 4 words + 1 table x 250
    fail, out = check_limits.check({"limits": {"max_words": 253, "float_cost_words": 250}}, text_path=t)
    assert fail


def test_ref_page_rule(monkeypatch):
    monkeypatch.setattr(check_limits, "pdf_pages", lambda p: (5, 4))
    assert not check_limits.check({"limits": {"max_pages": 4, "extra_ref_pages": 1}}, pdf="x")[0]
    monkeypatch.setattr(check_limits, "pdf_pages", lambda p: (5, 5))
    assert check_limits.check({"limits": {"max_pages": 4, "extra_ref_pages": 1}}, pdf="x")[0]
    monkeypatch.setattr(check_limits, "pdf_pages", lambda p: (6, 4))
    assert check_limits.check({"limits": {"max_pages": 4, "extra_ref_pages": 1}}, pdf="x")[0]


# ---- identity ----

def test_identity_string_in_double_blind_draft_is_caught(tmp_path):
    d = draft(tmp_path, "Tool E is available at github.com/jdoe-labs/tool. Contact jane@example.org. "
                        "Janet Doe wrote it.")
    hits = check_identity.run(["Janet Doe", "jdoe-labs"], [d])
    kinds = " ".join(h[2] for h in hits)
    assert "jdoe-labs" in kinds and "e-mail" in kinds and "Doe" in kinds
    assert check_identity.run(["agentpack"], [draft(tmp_path, "Agent Package Manager (APM)", "n.tex")]) == []
    clean = draft(tmp_path, "Tool E is built by the author. The code is at an anonymous link.", "c.tex")
    assert check_identity.run(["Janet Doe", "jdoe-labs"], [clean]) == []


def test_identity_in_pdf_metadata(tmp_path):
    from pypdf import PdfWriter
    pdf = tmp_path / "m.pdf"
    w = PdfWriter()
    w.add_blank_page(100, 100)
    w.add_metadata({"/Author": "Janet Doe"})
    with open(pdf, "wb") as f:
        w.write(f)
    assert check_identity.run(["Janet Doe"], [], pdf=pdf)


# ---- claims ----

def test_claims_anchor_and_source(tmp_path):
    claims = tmp_path / "CLAIMS.md"
    head = "| ID | Type | Claim | Source | Anchor |\n|---|---|---|---|---|\n"
    row1 = "| C1 | limitation | one skill only | results.md:3 | one small skill |\n"
    claims.write_text(head + row1 + "| C2 | fact | x |  |  |\n")
    d = tmp_path / "d.tex"
    d.write_text("We used one small skill.")
    assert check_claims.main([str(claims), str(d)]) == 1
    claims.write_text(head + row1)
    assert check_claims.main([str(claims), str(d)]) == 0
    d.write_text("We used a skill.")
    assert check_claims.main([str(claims), str(d)]) == 1


# ---- pack / intake ----

def test_intake_refuses_waits(tmp_path, capsys):
    p = tmp_path / "p.md"
    p.write_text((FIX / "pack_ok.md").read_text().replace("READY", "WAITS: a fixed release"))
    assert pack.main(["intake", str(p), "--out-root", str(tmp_path), "--dry-run"]) == 3
    assert "REFUSED" in capsys.readouterr().out
    assert not (tmp_path / "drafts").exists()
    assert pack.main(["intake", str(p), "--out-root", str(tmp_path), "--override"]) == 0
    assert (tmp_path / "drafts/Z1/PACK.md").exists()


def test_markers_extracted(tmp_path):
    p = tmp_path / "issue.md"
    p.write_text("intro\n<!-- writer-pack:start -->\n**Readiness:** READY\n<!-- writer-pack:end -->\ntail")
    text, _ = pack.load_source(str(p))
    assert text.strip() == "**Readiness:** READY"


@pytest.mark.parametrize("fmt,cls", [
    ("Position paper, 1–4 pages, IEEE conference format (IEEEtran), double-blind", "ieee-conf"),
    ("Short paper 5 + 1 pages, ACM format (acmart)", "acm"),
    ("4,200 words including 250 per figure or table, at most 15 references", "ieee-cs-magazine"),
    ("Pattern paper, Springer CCIS, up to 10 pages", "springer-ccis"),
    ("Commentary, about 800–1,000 words, exclusive", "prose-md"),
    ("Talk proposal, up to 2 pages, non-archival", "proposal"),
])
def test_classify(fmt, cls):
    assert pack.classify(fmt, "", "READY", "") == cls


def test_limits_parse():
    L = pack.parse_limits("4,200 words including 250 per figure or table, at most 15 references, abstract of at most 150 words")
    assert L == {"max_words": 4200, "float_cost_words": 250, "max_refs": 15, "abstract_max_words": 150}
    L = pack.parse_limits("4 pages + 1 for references, IEEE format")
    assert L["max_pages"] == 4 and L["extra_ref_pages"] == 1
