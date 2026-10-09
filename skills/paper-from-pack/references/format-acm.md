# acm (acmart)

Template: `assets/templates/acm-sigconf.tex`: `\documentclass[sigconf,anonymous,review]{acmart}`. For single-blind or camera-ready remove `anonymous,review`. Style `ACM-Reference-Format`. The CCS concepts block and rights block are required by the class; use `\setcopyright{none}` and empty conference fields for review versions and note in OPEN.md that the author must insert the venue's rights block at camera-ready.
Limits: usually "5 + 1" (content pages + references) or "10 + 2"; `venue.json` carries `max_pages` and `extra_ref_pages`; `check_limits.py` finds the References heading to separate them. With `review` the line numbers add width; page count is the same.
Distribution acmart may differ from the venue's current version: print `\ProvidesClass`/version from the `.cls` and compare with the call; note in OPEN.md.
Skeleton: Abstract, Introduction, Background/Related work, Method, Results, Discussion/threats, Conclusion. Use the pack's structure when given.
Double-blind: `anonymous` option hides authors; also check .bib, acknowledgements, links, PDF metadata with `check_identity.py`.
Open-access fees or ORCID requirements: copy from the call into SUBMIT.md as TO VERIFY.
