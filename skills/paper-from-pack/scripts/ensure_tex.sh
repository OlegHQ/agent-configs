#!/usr/bin/env bash
# Build helper for paper-from-pack.
#
#   ensure_tex.sh detect  [CLASS ...]      which engine would be used
#   ensure_tex.sh compile FILE.tex [OUTDIR]  build a PDF (system TeX first, Tectonic fallback)
#   ensure_tex.sh install-tectonic         download + verify the pinned Tectonic into the skill cache
#
# Order: (1) system TeX (latexmk or pdflatex+bibtex) when the document class is found by
# kpsewhich; (2) pinned Tectonic, downloaded into $PFP_CACHE (default: <skill>/.cache),
# SHA-256 verified against the table below; (3) stop with a message and the Docker option.
# Force an engine with PFP_ENGINE=system|tectonic. Nothing is installed globally; no
# settings are edited. Network use: the GitHub release asset and the Tectonic bundle only.
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="${PFP_CACHE:-$SKILL_DIR/.cache}"
TECTONIC_VERSION="0.17.0"
# Pinned bundle. Tectonic 0.17.0 default bundle; override only deliberately.
TECTONIC_BUNDLE="${PFP_TECTONIC_BUNDLE:-https://relay.fullyjustified.net/default_bundle_v33.tar}"

# SHA-256 of tectonic-<version>-<triple>.tar.gz. aarch64-unknown-linux-musl was verified by
# downloading and hashing locally; the others are the digests GitHub lists for the release
# assets (gh api repos/tectonic-typesetting/tectonic/releases/tags/tectonic@0.17.0).
declare -A SHA256=(
  [aarch64-unknown-linux-musl]="b10954a95404f3ab2328d2fa59a5ebab8e657f893fab096f98be8db7c0c979b8"
  [x86_64-unknown-linux-musl]="8533d07f9ccbd7a65824b9e0459041bca34af1eb33daba48f59215593753a3b7"
  [aarch64-apple-darwin]="a3f1cac7c5678f01661a92212f58480ae3b0634115d880dbc59e2953ded45667"
  [x86_64-apple-darwin]="7c90ef5b6ddb1eb1937e4337add5237b79338e4b9676459fa91187d24d6cdf80"
)

triple() {
  local os arch
  os="$(uname -s)"; arch="$(uname -m)"
  case "$os-$arch" in
    Linux-aarch64|Linux-arm64) echo aarch64-unknown-linux-musl ;;
    Linux-x86_64) echo x86_64-unknown-linux-musl ;;
    Darwin-arm64) echo aarch64-apple-darwin ;;
    Darwin-x86_64) echo x86_64-apple-darwin ;;
    *) echo "unsupported" ;;
  esac
}

docker_hint() {
  cat >&2 <<'MSG'
Alternative without a local install: run TeX Live in Docker (nothing is installed on the host):
  docker run --rm -v "$PWD":/work -w /work texlive/texlive:latest latexmk -pdf -interaction=nonstopmode DRAFT.tex
Or on Debian/Ubuntu install TeX once with apt (see README.md).
MSG
}

class_of() { sed -n 's/^[[:space:]]*\\documentclass\(\[[^]]*\]\)\{0,1\}{\([A-Za-z0-9_-]*\)}.*/\2/p' "$1" | head -1; }

system_has() { # all given classes found by kpsewhich
  command -v kpsewhich >/dev/null 2>&1 || return 1
  command -v latexmk >/dev/null 2>&1 || command -v pdflatex >/dev/null 2>&1 || return 1
  local c
  for c in "$@"; do [ -n "$(kpsewhich "$c.cls" 2>/dev/null)" ] || return 1; done
}

tectonic_bin() { echo "$CACHE/tectonic-$TECTONIC_VERSION/tectonic"; }

install_tectonic() {
  local t; t="$(triple)"
  if [ "$t" = unsupported ] || [ -z "${SHA256[$t]:-}" ]; then
    echo "No pinned Tectonic build for $(uname -sm)." >&2; docker_hint; return 1
  fi
  [ -x "$(tectonic_bin)" ] && return 0
  command -v curl >/dev/null 2>&1 || { echo "curl is required to fetch Tectonic" >&2; docker_hint; return 1; }
  local url="https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%40${TECTONIC_VERSION}/tectonic-${TECTONIC_VERSION}-${t}.tar.gz"
  local tmp; tmp="$(mktemp -d)"
  echo "Downloading $url" >&2
  if ! curl -fsSL --retry 2 -o "$tmp/t.tar.gz" "$url"; then
    echo "Download failed." >&2; docker_hint; rm -rf "$tmp"; return 1
  fi
  local got
  if command -v sha256sum >/dev/null 2>&1; then got="$(sha256sum "$tmp/t.tar.gz" | cut -d' ' -f1)"; else got="$(shasum -a 256 "$tmp/t.tar.gz" | cut -d' ' -f1)"; fi
  if [ "$got" != "${SHA256[$t]}" ]; then
    echo "SHA-256 mismatch for Tectonic ${TECTONIC_VERSION} ${t}: expected ${SHA256[$t]}, got $got. Not using it." >&2
    docker_hint; rm -rf "$tmp"; return 1
  fi
  mkdir -p "$CACHE/tectonic-$TECTONIC_VERSION"
  tar -xzf "$tmp/t.tar.gz" -C "$CACHE/tectonic-$TECTONIC_VERSION"
  chmod +x "$(tectonic_bin)"
  rm -rf "$tmp"
  echo "Tectonic ${TECTONIC_VERSION} installed in $CACHE (SHA-256 verified)." >&2
}

cmd="${1:-}"; shift || true
case "$cmd" in
  detect)
    classes=("$@"); [ ${#classes[@]} -gt 0 ] || classes=(IEEEtran)
    if [ "${PFP_ENGINE:-}" != tectonic ] && system_has "${classes[@]}"; then
      echo "engine=system (latexmk/pdflatex; classes: ${classes[*]})"
      for c in "${classes[@]}"; do echo "  $c.cls: $(kpsewhich "$c.cls")"; done
    elif [ -x "$(tectonic_bin)" ]; then echo "engine=tectonic (cached $(tectonic_bin))"
    else echo "engine=none (system TeX lacks ${classes[*]}; Tectonic not installed: run install-tectonic)"; fi ;;
  install-tectonic) install_tectonic ;;
  compile)
    src="${1:?usage: compile FILE.tex [OUTDIR]}"; out="${2:-$(dirname "$src")}"
    mkdir -p "$out"
    cls="$(class_of "$src")"; [ -n "$cls" ] || cls=article
    run_tectonic() {
      install_tectonic || exit 1
      export TECTONIC_CACHE_DIR="$CACHE/tectonic-cache"; mkdir -p "$TECTONIC_CACHE_DIR"
      "$(tectonic_bin)" -X compile --bundle "$TECTONIC_BUNDLE" --outdir "$out" "$src"
    }
    if [ "${PFP_ENGINE:-}" != tectonic ] && system_has "$cls"; then
      echo "engine=system class=$cls" >&2
      sysout="$(mktemp)"
      if command -v latexmk >/dev/null 2>&1; then
        ( cd "$(dirname "$src")" && latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir="$(cd "$out" && pwd)" "$(basename "$src")" ) >"$sysout" 2>&1 && rc=0 || rc=$?
      else
        ( cd "$(dirname "$src")" && for i in 1 2; do pdflatex -interaction=nonstopmode -halt-on-error -output-directory="$(cd "$out" && pwd)" "$(basename "$src")"; done ) >"$sysout" 2>&1 && rc=0 || rc=$?
      fi
      tail -n 40 "$sysout"
      if [ "$rc" -ne 0 ]; then
        if grep -q "File .* not found" "$sysout" && [ "${PFP_NO_FALLBACK:-0}" != 1 ]; then
          echo "System TeX lacks a package ($(grep -m1 'File .* not found' "$sysout")). Falling back to pinned Tectonic." >&2
          rm -f "$sysout"; run_tectonic
        else
          rm -f "$sysout"; exit "$rc"
        fi
      fi
      rm -f "$sysout"
    else
      echo "engine=tectonic (system TeX lacks class '$cls' or PFP_ENGINE=tectonic)" >&2
      run_tectonic
    fi ;;
  *) sed -n '2,13p' "$0"; exit 2 ;;
esac
