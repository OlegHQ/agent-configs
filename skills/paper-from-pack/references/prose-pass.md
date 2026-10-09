# Prose pass

1. `cp DRAFT.tex DRAFT.before-unslop.tex` (or .md).
2. Apply the `unslop` skill (github.com/cursor/plugins, path pstack/skills/unslop) to the prose only. If your agent cannot invoke it (it may be marked manual-only), read the installed copy and apply its patterns yourself; if it is not installed, apply the list below and say so in OPEN.md. Do not copy the unslop text into the output.
3. Then the paper-specific list. Remove, by editing:
   - Filler openers and closers; "In this paper, we..." repeated; "It is worth noting"; "Overall,"; "In conclusion,".
   - Banned words: delve, leverage, robust, seamless, landscape, crucial, pivotal, tapestry, testament, underscore, foster, showcase, comprehensive, holistic, novel (as praise), cutting-edge, state-of-the-art (as praise), paradigm, game-changing.
   - Rule-of-three lists used for rhythm; "not only X but also Y"; em-dash chains (use none); colons as connectors; sentence that restates the previous one; paragraph that only announces the next; stacked hedges ("may potentially suggest").
   - Passive voice with a hidden actor; adverbs holding up weak verbs.
4. Keep: every number, citation key, `TO VERIFY` marker, limitation, LaTeX command, label, and the AI-use statement. Do not change a claim's strength. Do not add soul by inventing anecdotes; use only the interview text in the pack.
5. Prove nothing moved: `check_numbers.py --compare before after` and `check_claims.py`. Re-run the full step 7.
6. Grep for leftovers: `grep -n -i -E "delve|leverag|robust|seamless|landscape|crucial|pivotal|worth noting|—" DRAFT.*` (use the list above) and fix.
7. This pass improves quality. It is not a way to hide AI use; the statement stays.
