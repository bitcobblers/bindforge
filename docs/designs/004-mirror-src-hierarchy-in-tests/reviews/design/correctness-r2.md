# correctness review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` — the single editorial confirmation pass of round 2. Inputs: 02-proposal.md, docs/adrs/004-mirror-src-hierarchy-in-tests.md, the Revision log, and my own previous findings (none; editorial phase is one language pass per round).

## Findings
- [nit][new] Chosen design ¶2 (as quoted by the diff's appended "Editor notes:" line) — "a relative path reference — code or comment — naming a location this move passes takes its new depth" — the sentence is ambiguous: "passes" could mean "passes through the moved file's old directory" or "passes over / leaves behind", and which references qualify is undefined. It matters beyond style because Rollout 2 writes this one clause into ADR-004 Decision bullet 2 as the permission governing the move commit, and Test strategy #1 checks the moved-file diff against "¶2's allowed set". No correctness gap today: the operative set is pinned by A8's closed enumeration (62 `from '../src/...'` imports in moved files, 16 `new URL` sites, one comment line), which the diff leaves intact — so the Editor note's deferral to the author is acceptable. Fix: when clarifying, make the clause's scope match the enumeration, e.g. "every relative path reference — code or comment — that resolves through the moved file's former directory takes the depth of its new location", replacing the current clause rather than adding to it.

## Checklist evaluation (no severity)
- Register (item 1): the diff touches no A1–A19 row. Facts on touched lines re-checked: A8 decomposition 65+2+16+4 = 87, and 62 moved-file imports + 3 root-tier = 65; "Risk 4" is exactly the baseline-comparison risk row (acceptance gate, 26→26); the 23-name list matches ADR Decision ¶1 one-for-one.
- URL spot-check (item 2, anti-hallucination gate): https://vitest.dev/config/include live — default include `['**/*.{test,spec}.?(c|m)[jt]s?(x)']` is recursive, supporting A5; https://vitest.dev/guide/cli live — positional args are path-substring filters, supporting A6; https://git-scm.com/docs/git-mv and https://git-scm.com/docs/git-config live — consistent with A15. No dead or contradicting source.
- Items 3–5: the four language edits (quote/backtick fix in ¶2, comma in ¶4, "Risks 4"→"Risk 4", "from merge"→"from the merge") are meaning-preserving; the diff's applied side matches the current 02-proposal.md exactly; no component appears under two names or shapes.
- Item 6: the diff does not alter what the design decides; it still answers ADR-004.
- Item 7: the prior round's closed correctness blocker/major (¶2 amendment over A8's 87-site enumeration) is intact — the diff's only ¶2 edit is typographic, outside the amended clause.
- Deferred section re-checked only against the ADR: fixture-string deferral is consistent with ADR bullet 2 ("No ... fixtures"); no contradiction re-opened.

## FYI (no severity)
- The "Editor notes:" line is an open editorial item, not a Revision-log decision; whoever clarifies the ¶2 clause (and its Rollout 2 ADR wording) must keep it in lockstep with A8's closed enumeration and Test strategy #1's allowed set, or the check becomes unenforceable.
