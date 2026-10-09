# Build

`scripts/ensure_tex.sh compile DRAFT.tex [outdir]`

Order
1. System TeX: `latexmk -pdf` (or `pdflatex` twice) when `kpsewhich <documentclass>.cls` finds the class. Recommended setup on Debian/Ubuntu (done once by the user, not by this skill):
   `sudo apt install texlive-latex-base texlive-latex-recommended texlive-latex-extra texlive-publishers texlive-fonts-recommended texlive-fonts-extra texlive-bibtex-extra texlive-science latexmk`
   Distribution classes can be older than a venue's current template: compare `\ProvidesClass` dates (grep in the `.cls` found by `kpsewhich`) with the call and note a difference in OPEN.md. If a package is missing during a system build, the script falls back to Tectonic unless `PFP_NO_FALLBACK=1`.
2. Pinned Tectonic (fallback for machines without TeX): Tectonic 0.17.0 release asset from github.com, SHA-256 checked against the table in `ensure_tex.sh`, extracted into `<skill>/.cache/` (override with `PFP_CACHE`), run with a pinned bundle URL and a skill-local `TECTONIC_CACHE_DIR`. No global install. `ensure_tex.sh install-tectonic` does only the download. Supported: Linux x86_64/aarch64, macOS x86_64/arm64. Other platforms: step 3.
3. If the download, the hash or the build fails the script stops with a message. Alternatives: Docker (`docker run --rm -v "$PWD":/work -w /work texlive/texlive:latest latexmk -pdf DRAFT.tex`), or install TeX with the system package manager.

Force: `PFP_ENGINE=tectonic` or `system`. Check what would be used: `scripts/ensure_tex.sh detect IEEEtran`.

Bibliography: BibTeX with the class's style (`IEEEtran`, `ACM-Reference-Format`, `splncs04`). Keep `refs.bib` next to the .tex. Tectonic runs bibtex itself; latexmk does too.

Read the log for overfull boxes and undefined references; undefined citations are a failure.
Tectonic does not need `pdfinfo`; page counts come from `check_limits.py` (pypdf).
