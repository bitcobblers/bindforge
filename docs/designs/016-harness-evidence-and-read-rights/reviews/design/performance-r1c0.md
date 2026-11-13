# performance review r1

verdict: approve

## Findings

- [minor][new] §4 Budgeted, measured context — "and states a ceiling: at most 25
  reads of files outside that set." — the ceiling bounds the *count* of reads
  but not their size, and unlike every other constant in this design (600
  derived from the observed 440/505-line artifacts and the 2,696-line runaway;
  2,000/6,000 pinned against the 8 MiB checkpoint and the 189,788-token prompt
  baseline) 25 is stated without a derivation. Prior-run artifacts are not
  size-capped by this design (run 2's 2,696-line proposal is still on disk), so
  the worst case of 25 out-of-set reads of large documents plausibly re-creates
  the multi-tens-of-thousands-of-tokens research prompt that Decision 4 exists
  to measure and that A20's 32,768-token resident KV cannot hold — a gap
  between the design's numbers and the research baseline without explanation
  (checklist item 6). Fix: one sentence deriving 25 against the ADR's numbers —
  the estimated worst-case token cost of 25 out-of-set reads against the
  189,788-token largest-prompt / 32,768-resident-prefix figures — or scope the
  ceiling's worst case by stating out-of-set reads are counted per document
  under `docMaxLines`. No mechanism change needed.
- [nit][new] §2 The gate record — "`GateEvidence = { at: number; green:
  boolean; legs: GateLeg[] }`" — the record carries `at` but nothing renders or
  checks it, and the design never states the record's freshness rule: a seat
  reading the block cannot tell which diff the tail belongs to. The
  invalidation is overwrite-at-each-gate-call, which is correct by
  construction (only the gated `implement`/`fix`/`test-update` stages change
  code, and the resumed-pre-016 case renders the absent-record line) — but
  that correspondence is exactly the "invalidation correct, not just fast"
  property, and it is left implicit (checklist item 5). Fix: one sentence at
  the runner bullet ("the gate call replaces `state.gateEvidence` wholesale, so
  a rendered block always describes the gate over the current state; `at` is
  for forensics only") — or render `at` into the block header.

## FYI (no severity)
- The design panel now running the cargo/npm legs (Decision 3) duplicates
  per-seat gate compute, but `maxParallelReviews` stays 1 so the five design
  seats run sequentially against a shared `target/` cache; the wall-clock
  posture is acceptable as written — noted in case the panel-scheduling
  specialty wants it pinned.
- The coarse `grep` denies match the requested regex, not the search path
  (A17), so a legitimate repo-wide grep containing the word `reviews` is
  denied; the ADR accepts coarse denies against deliberate access — this
  misfire surface is a security/correctness-seat concern, not a performance
  one.
