# complexity review r1c2

verdict: approve

Ratification pass c2, complexity seat. Scope: the blob diff between the r1-approved
proposal and the current text, plus the full current document. My r1 verdict carried
no findings, so the c2 question is only whether the three revisions (escalationPending
flag, ownership sweep before `finalizeEscalate`, macOS/Windows `no`, and the config
and commit-message rewordings) broke something approved or made cited text false.

## Checklist walk (changed sections only)

1. **Reuse.** No new package/module/interface appears in the diff. The escalation
   change reuses `enforceOwnership` as an existing function call, and the
   `finalizeReviewPass` signature *drops* a parameter (`stageKey`) — a net reduction
   of one moving part. No new dependency in `tools/` to re-check.
2. **Traces to a stated Goal.** The `escalationPending` flag traces to Goal
   "Keep N = 1 on the same code path" and ADR-003 §Execution 3 ("a seat that fails
   twice escalates after its siblings settle"): without it, seats that today's
   `StopDriving` skips would run under N = 1, breaking the same-path goal. Sound.
3. **Smaller design?** The flag is one boolean plus one check after `acquire()`;
   dropping it would change N = 1 control flow, which the ADR forbids. The
   escalation-path sweep is one call to machinery that already exists. Nothing in
   the changed text admits a 30–50% smaller variant.
4. **New dependencies:** none added.
5. **Deferrability:** the escalation sweep was added to close a security finding
   (Revision log); it is a reuse, not a new subsystem, so deferring it is not the
   smaller-cost option.
6. **Knobs/abstractions:** `maxParallelReviews?: number` with `DEFAULT_CONFIG` named
   "the sole default authority" removes an ambiguity about where defaults live rather
   than adding a knob; the flag has exactly one writer and one reader path.
7. **Value:** complexity remains proportionate — the r1-approved budget plus a flag
   and a reused call. Proportionate to what the feature brings.

## Findings

- [minor][new] Chosen design — Two-phase split of `executeStage` ("`finalizeReviewPass`
  … the only git code for review passes") vs Pass-level fan-out ("drain the writer,
  run the pass `enforceOwnership` sweep, then `finalizeEscalate` once") — the
  escalation-path revision makes the success-path-only claim stale: the document now
  describes a second review-pass git sequence (sweep, then `finalizeEscalate`'s own
  commit) without saying where that sweep call lives, so it is ambiguous whether
  `enforceOwnership` has one call site or two for review passes. This slightly
  widens the two-phase contract the Risks table relies on ("forget the finalize, or
  git in inference"). Fix: one clause — run the pass sweep inside `finalizeReviewPass`
  on both exits and let it skip `commitStage` when `escalationPending` is set (then
  the sentence stays literally true), or reword to "the pass's single
  `enforceOwnership` call site, used by both the success and escalation exits;
  `commitStage` runs only on the success exit."
- No blocker or major findings; nothing in the diff contradicts ADR-003
  (§Execution 3 escalation-after-siblings, §Self-update N = 1 commit-shape caveat,
  §Configuration default 1). Making the config field optional is a flagged,
  semantics-preserving relaxation of the ADR's type spelling (default 1 and opt-in
  are kept, and the CLI always merges `DEFAULT_CONFIG`), not a contradiction of its
  Decision.

## FYI (no severity)

- Chosen design — "`docs: <nnn-slug> design-review stage`": as written the example
  could be read as one literal message for all review passes; "the pass's stage
  name" implies `code-review`/`editorial-review` passes carry their own names.
  Whether those are the exact stage names is for the correctness seat against
  `stages.ts`.
- Compatibility matrix macOS/Windows `unknown` → `no` is an honesty tightening;
  the compatibility seat owns whether "no" overstates a platform-agnostic claim.
- Test strategy #6 "assert … the exact message" now pins the fallback message
  shape; fine, just noting it hardens the same contract the finding above clarifies.
