# Performance review r2

Scope: the editorial diff `reviews/design/editorial-r2.diff` (round 2 single
language pass), read against `02-proposal.md` and
`docs/adrs/016-harness-evidence-and-read-rights.md`. No prior performance
findings existed (the editorial phase has no dissent pass for this seat), so
this is a confirmation pass: does any changed passage break something the
performance seat approved or make a cited section false?

verdict: approve

## Checklist evaluation

1. **Input-path latency / per-event cost.** All hunks are language-only:
   "like"→"as" rewrites in §2 ("exactly as the boolean does today"),
   "spawned failed"→"spawning failed", "deeds audit"→"deed audit",
   "per seat"→"per-seat", Oxford commas, and the §4 "going over length"
   rewording. None adds cost on any event path; the gate still runs only at
   the one post-`implement`/`fix`/`test-update` stage-ok point, and the
   "going over length turns the attempt into a normal stage failure" rewording
   preserves the same measure-and-gate placement.
2. **Structure costs (O(n²) over unbounded n).** Every bound is byte-for-byte
   intact in the touched text: `MAX_GATE_TAIL_CHARS = 2_000`,
   `MAX_GATE_LEGS = 32`, `MAX_EVIDENCE_BLOCK_CHARS = 6_000`, `docMaxLines:
   900` (Rollout #2). The reworded elision sentence ("appended as
   `boundedFindings` does") keeps the same A7 bounding pattern. No quadratic
   shape introduced.
3. **Memory growth.** §2's tail capture ("a rolling tail buffer holding the
   last `MAX_GATE_TAIL_CHARS` of decoded output per leg") is untouched by the
   diff; the checkpoint field stays bounded and optional (A6); the Risks row
   "one `gateEvidence` field cloned into every later snapshot" still reads
   "Bounded legs × bounded tail ≪ 8 MiB loader cap". No growth path opened.
4. **Startup cost.** No hunk touches boot-time file reads or schema parsing;
   the checkpoint loader sentence and the optional-`()` zod addition are
   unchanged.
5. **Caching/invalidation.** No cache is introduced or reworded by the diff.
6. **Baseline quotes.** The ADR's numbers (189,788-token largest prompt, 72%
   of window, 32,768 resident KV) still back the Goals non-goal ("Decision 4
   refuses this on the ADR's measured facts") and the Risks mitigation
   (two pinned budgets + the new `- tokens:` line); the diff neither cites a
   new number nor contradicts a baseline.

All 15 diff hunks are grammar/punctuation/reflow or the two appended editor
notes; none alters a mechanism, constant, or bound. No finding.

## Findings

(none)

## FYI (no severity)

- The appended Editor note on Options considered — "the seat-board clone
  problem A7 exists for" — sits inside the checkpoint-size justification
  (8 MiB cap, bounded tail). The performance substance is pinned elsewhere
  (§2 constants, A6/A7, Risks row), so this is prose ambiguity already logged
  for the author, not a performance defect.
- The Editor note on Test strategy #3 — "a sibling names the child session" —
  concerns an audit-identity test expectation (correctness seat's territory);
  the capture/limits half of #1 is unaffected.
