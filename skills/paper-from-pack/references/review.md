# Hostile review (in session)

No external service, no other model. Three reviewers, run as subagents if the agent supports them (give each only: the compiled draft or text, `PACK.md`, the brief below; not CLAIMS.md or your notes), otherwise one after another in this session, starting each with a fresh read of the draft and a one-line note to yourself that you are now that reviewer.

Each reviewer returns: summary of the claim in two sentences; the three strongest objections with the exact sentence challenged; what evidence would answer each; accept/weak accept/weak reject/reject for this venue's type of paper (position, tool, practitioner article...) with one sentence why.

Calibrate to the paper type: a position or vision paper is not faulted for missing a full empirical study if it states its limits; a practitioner article is judged on usefulness and honesty.

1. **Methods reviewer.** Is the measurement valid for the claim? Confounds, sample, case selection, definitions fixed in advance, repetition and variance, threats to validity stated, any number that does not follow from the data, any claim stronger than the evidence, conflict of interest handled, reproducibility (can a stranger re-run it).
2. **Practitioner-relevance reviewer.** A senior engineer who maintains such systems. Would they change anything they do? Is the recommendation concrete and costed? Is anything obvious or already standard practice? Is the writing usable by someone who skips to the table?
3. **Related-work reviewer.** Knows the closest works named in the pack. For each: is it cited fairly, is the difference stated correctly, is anything described more strongly than its source supports, is there a named closest work the draft ignores or a "no one has done this" claim that the pack does not support.

Write `REVIEW.md`: a section per reviewer, then "Findings by type": factual error / limit violation (you fix these and log them), claim too strong (recommend cut), missing evidence (collection task), style (author decides). The author decides everything except factual and limit fixes. List each automatic fix with before and after.
