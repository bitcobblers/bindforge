# performance review r3

verdict: approve

## Findings

none. This is the ratification pass over `reviews/design/editorial-r3.diff`
only; I have no open performance findings from earlier rounds, and no hunk in
the diff breaks approved text or makes a section a performance seat would cite
false.

Hunk-by-hunk check against the performance checklist:

- **Context** (comma before "because"): the research-baseline figure — run 004's
  23 test-file moves (01-research.md, A12) — is unchanged, so the gap check
  (checklist #6) still stands with nothing re-stated numerically.
- **Goals / Non-goals** ("at any scope"): no cost semantics touched.
- **Reply field** ("are added to"; "a list over either cap fails `safeParse`
  like any invalid reply"): this is the design's bounded-work sentence — "The
  bounds cap the validation and apply work one reply can make the runner
  spend" — which is what keeps the validator's pairwise rule-6 comparisons and
  the per-pair apply/rollback loops bounded (checklist #2, #3). The new wording
  names both caps exactly as the schema above it defines them — `.max(200)` per
  endpoint, `.max(64)` per array — and now matches Test strategy #1's phrasing
  ("an endpoint over the length cap, and a list over the pair cap fail
  `safeParse`"). More precise than the replaced "an over-limit list"; nothing
  false.
- **Validator rule 5** ("does not exist anywhere"): the `existsSync` disk check
  stays described as the check itself plus the case-insensitivity guard (A10);
  per-pair synchronous IO is bounded by the 64-pair cap. Meaning unchanged.
- **Validator rule 6** ("What remains is a matching whose apply order is
  irrelevant"): the order-irrelevance claim that justifies bounded, no-sort
  sequential apply is untouched.
- **Wiring / Apply** (em-dashes to commas, re-wrapping, "as that of a rename
  error", "which is exact because the pairs are disjoint", "leaving the
  preserved-worktree state"): punctuation and reflow only. The single
  inference→finalize guard re-run of rules 4–7 (bounded at ≤64 pairs), the
  reverse rollback, and the no-sweep/no-commit exit path all read exactly as
  before. No new synchronous IO, subprocess, or cache-invalidation claim
  introduced anywhere in the diff (checklist #1, #4, #5).

## FYI (no severity)
- (none)
