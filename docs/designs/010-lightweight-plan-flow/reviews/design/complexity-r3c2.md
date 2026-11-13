# Complexity review r3 — ratification pass c2

verdict: approve

Scope: the c2 diff (`92f71e0` → `4635f0d`, the two blobs of `02-proposal.md`) and
the full current document. Ratification lens: does the revised text break
anything this seat approved, or make a section it cited false? My prior verdict
carried no open findings, so the test applied is: does any **new** text add a
moving part that does not trace to a Goal, duplicate an existing seam, add a
dependency, or create a knob/abstraction with no consumer?

Inputs used: `02-proposal.md` (current), `docs/adrs/010-lightweight-plan-flow.md`,
the proposal's Revision log. No review reports or transcripts opened.

## Checklist walk

1. **Reuse of new modules/interfaces** — the diff adds no package, module,
   interface, file, stage, knob, or state field. The only new artifact it touches
   is the `plan` stage's agent, and the revised bullet makes it the cheap
   option: "`design/designer`'s permission list verbatim (`designer.md:4-22`,
   shape in A12) — copied, not re-derived". Reusing `design/designer` itself is
   not available: ADR-010's Decision names agent `design/planner`, and the copy
   adds no new `StageKind` and no new reply contract.
2. **Every moving part traces to a Goal** — the diff's only mechanism addition
   is `validated` running on a caller's `opts.cfg` **pre-lock** in
   `runPipeline`/`resumePipeline`. It traces to Goal 2 ("validated before the
   lock and before any inference"), to the failure it prevents
   (`reviewSeats.code: []` makes `seatsConverged` vacuously true, A2), and it is
   pinned by Test strategy #17. Nothing else in the diff adds machinery.
3. **A 30–50 % smaller design?** — the diff is net-additive in prose only: A12's
   document-stage universal is narrowed to four named agents with one named
   exception (`doc-sync.md:23-25`), and the `plan` bullet excludes
   `ops/doc-sync`'s `git diff*` allow from the copied list. Claim-narrowing
   removes no mechanism and adds none; nothing in the changed text is cheaper to
   delete than to keep.
4. **New dependencies** — none; the diff names no package, library, tool, or
   platform surface.
5. **Second half deferred?** — unchanged by the diff: `planMaxFiles`,
   docs-only/audit flows, per-flow model routing, per-stage flow recording and
   the warn-vs-gate consistency stay Deferred. The Revision log's shortening of
   the Rollout 6 summary is a log edit; the Rollout body still carries the
   converging bound (`2 + seats × 3`), the ≈ 9-session escalated ceiling and the
   `--extend-budget` grant arithmetic.
6. **Knobs with no consumer; abstraction with one implementation** — the diff
   creates no knob. `validated` now has two call points (the pre-lock fail-fast
   and inside `configFor`) behind one implementation, so Budget resolution's
   "the **only** place flow budgets are computed" still holds: the spread merge
   stays single-sited and the early call only fails fast on the caller's
   partial. #23's scoped check reads registry data (`the registry's design: []
   as written — registry data, not a tunable`), so it adds no tunable.
7. **Value check** — proportionate, and the diff does not move it. The light
   flow's footprint is one registry module, one agent file, one three-line
   template pointer, one skill with a template, two optional `PipelineConfig`
   fields and one optional state field, measured against the ADR's 43-session
   floor and Rollout 6's ≤ 8-session target. The c2 additions are one
   fail-fast call and narrower claims about existing files; they buy the
   pre-lock guarantee the ADR demands at close to zero structural cost.

## Consistency of the revised text with the sections that cite it

- Budget resolution's pre-lock claim, the Compatibility row
  ("`flow: Light` / `flow: sideways` (bad value); a caller's `reviewSeats.code: []`
  panel | no | Pre-lock usage error") and Test strategy #17
  ("pre-lock gates … no lock file, no design directory, no entry checkpoint,
  … **zero** client calls") state one posture, matching `assertStageInFlow`'s
  "a usage error, before the lock" and Selecting the flow's pre-lock parse.
  The stale phrasing "at the config-read seam" is gone, so the resolution seam
  (line 113's `flowOf`) is not now described as a validation point.
- "A flow with no design stage resolves the registry's `design: []` as written"
  agrees with the knob table (`reviewSeats.design` → `[]` (no design phase)) and
  #23 (empty `design` throws under `full`, resolves under `light`), and keeps
  the scoped rule in the canonical block, as r1c3/r1c4 required.
- A12 is the single shape authority the `plan` bullet points at, and the r2 + r3
  log entry records the narrowing as a deliberate decision with its reason, so
  this is not a re-opened finding.

## Findings

None. The revisions did not break what this seat approved, and did not make any
section the complexity seat cited false.

## FYI (no severity)

- A12 is now an enumeration over four named agents with one named exception; a
  fifth document-stage primary agent added later makes the row stale, and the
  `plan` bullet inherits that copy — claim freshness for the correctness seat,
  not a cost here.
- Validating a caller's `opts.cfg` pre-lock in `resumePipeline` implies reading
  the checkpoint before `acquireLock`; the sentence states the ordering for the
  run path ("before `acquireLock` and `freshState`"). Lock ordering is
  security/correctness ground.
- The Compatibility row now mixes a source-document gate and a caller-config
  gate in one row; legibility only.
