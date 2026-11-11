# performance review r4

verdict: approve

## Findings

None. No open blocker, major, minor, or nit finding from the performance checklist against the current `02-proposal.md` (r4 state), ADR-005, and the Revision log.

Checklist basis (no findings raised):

1. Latency path — there is no keystroke/event path in this design; the cost points are reply validation and finalize. Both are bounded: `moves: z.array(MovePairSchema).default([]).max(64)` with 200-char endpoints, and the proposal states "The bounds cap the validation and apply work one reply can make the runner spend." The per-pair `git cat-file -t` subprocess (rule 4) and the apply guard's re-run of rules 4–7 add at most a bounded, constant-factor subprocess cost on a stage path dominated by inference latency — and the duplication is explained (Risks row: "`applyMoves` re-runs rules 4–7 as one guard immediately before the first rename"). No per-event table rebuild or synchronous hot-path IO beyond this.
2. Structure costs — rule 6's disjointness comparisons are quadratic in the pair count, but the pair count is schema-capped at 64, so n is never unbounded. No bindings × devices × platforms blow-up exists in this design.
3. Memory growth — the only new retained state is the checkpoint's `moves` field: optional, default `[]`, bounded by the caps, additive per A13 (`version` stays 1, `state-store.ts` untouched). No new caches or tables.
4. Startup cost — no new boot-time file reads or schema parsing; "the only git call this change adds is the validator's `git cat-file -t`" and there is no new git version floor.
5. Caching/invalidation — no new cache is introduced. The design's staleness handling is the rules 4–7 re-guard across the inference→finalize window plus the `.pipeline.lock` single-writer invariant, which is a correct invalidation story, not merely a fast one.
6. Baseline consistency — the figures the proposal and ADR cite are coherent with the design's expectations: run 004's motivating payload is 23 file pairs (A12), well inside the cap of 64; the ADR's cited A15 similarity of 91–99% for content-identical moves sits comfortably above the default 50% rename-detection threshold the design relies on for `R` entries. No unexplained gap.

## FYI (no severity)

- Scope note for the runner/editor: the named scope file `docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r4.diff` does not exist. I read the exact named path twice and confirmed absence with an exact-filename search that surfaces no other file in any review directory (no browsing of reports or transcripts occurred). I therefore applied the checklist to the current full proposal text, which is a named input; since my previous-round findings are "(none)" and the current text carries no performance defect, the ratification holds on the text as it stands. If a round-4 editorial diff exists outside the workspace, this seat should be re-run against it.
- For the complexity seat: the apply guard deliberately re-runs validator rules 4–7, duplicating validation work by design; I judged the bounded duplication justified by the TOCTOU risk — a different seat may want to confirm the duplication is documented where implementers will see it.
