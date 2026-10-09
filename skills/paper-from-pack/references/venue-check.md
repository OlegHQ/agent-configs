# Venue check

1. Take the call URL from the pack (the "Venue rules" line or the venue check file it names). If none, stop and ask.
2. Fetch the live page (read only; log the URL in `network.log`). If it is JavaScript-only or returns 403, say so; use the author-provided text or an archive snapshot and mark the check "not live".
3. Compare, one line each, pack versus live page:
   deadline (date, time zone), length (pages or words, whether references are included), template or class, blind/anonymity rules, concurrent-submission and prior-publication wording, AI-use policy and required statement, archival status, attendance duty, supplementary-material rules.
4. Write the table to `OPEN.md` under "Venue check (accessed <date>)", quote the wording that matters.
5. Hard limits: length, template, anonymity, concurrent-submission ban that touches this piece, closed or moved call, a new AI-prose ban. If any changed, **stop** and ask. Soft differences (a deadline that moved later, an extra optional field): continue and record.
6. If the pack's own venue file already has quotes, you still re-read the live page; a stale pack is the point of this step.
7. Set `"confirmed": true` in `venue.json` only after the limits there match the live page.
