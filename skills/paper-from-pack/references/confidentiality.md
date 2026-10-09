# Confidentiality guard

Pack contents and draft text may be unpublished and may describe work the author must not expose.

Allowed:
- Your own model (the agent session).
- Venue call pages: read only; GET requests; nothing from the pack in the URL.
- `scripts/check_citations.py`: arXiv, Crossref, dblp, with an identifier (arXiv id, DOI) or one reference title (max 250 characters). The script enforces a host allowlist, refuses long queries, rate-limits to one request per second per host, and appends every URL to `network.log`.
- `scripts/ensure_tex.sh`: the pinned Tectonic release asset on github.com and the pinned bundle host, only when system TeX cannot build the document.

Not allowed: sending any pack or draft sentence to a search engine, a plagiarism or "AI detector" service, another LLM, a grammar service, an online LaTeX editor, a paste site; reading `.env` files, key stores or credential files; installing anything globally (apt, npm -g, pip --user, brew); editing agent settings, hooks or MCP config; creating accounts; telemetry.

If a reviewer or tool wants one of those, skip it and say so in OPEN.md.

Before finishing: `cat network.log`. Every line must be one of the allowed kinds. If the log is missing, no request was made by the scripts; say how you fetched the venue page (and note that curl calls are not logged unless you append them).
Venue page fetches: append them yourself: `printf '%s\tGET\t%s\tvenue call\n' "$(date -Iseconds)" "$URL" >> network.log`.
