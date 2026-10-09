# ieee-cs-magazine (IEEE Software / Computer style)

Output: Markdown `DRAFT.md` (the venues take Word/PDF through a submission system; no public LaTeX template is assumed). Template: `assets/templates/ieee-cs-magazine.md`. If the call requires a specific template (check in step 2), say so in OPEN.md and give the Markdown as the source text.
Limits (from the pack, confirm live): total words including a fixed cost per figure or table (for example 4,200 words, 250 per float, so about 2,950 words of prose with three figures and two tables), abstract at most 150 words, at most 15 references, a set number of insight bullets (usually three; exactly the call's number), short author bio.
Counting: `check_limits.py --text DRAFT.md` counts body and abstract words, excludes references, code blocks (reported separately), and adds `float_cost_words` per Markdown table or image. Plan the word budget before writing: floats x cost, abstract, bullets, bio; the rest is prose.
Magazine style: practical, argued, for practitioners; sections of 300-500 words; sidebars count as words; no heavy theory; numbers with context; claims tied to evidence; each figure or table must earn its cost.
Structure when the pack gives none: Standfirst (abstract) / Insight bullets / The situation / What we measured or did / What it shows / What to do / Limits / References / Bio.
Citations: numbered list at the end, at most the call's maximum; magazines prefer a few strong references over a survey.
Cut order for fit: drop the weakest figure (saves its cost), then second examples, then related work, then background.
