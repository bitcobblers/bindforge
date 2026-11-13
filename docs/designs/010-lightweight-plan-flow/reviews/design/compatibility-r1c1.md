# compatibility review r1c1

verdict: approve

Scope: the `9ae29b5..24b804a` diff of `02-proposal.md` (the revision written to answer
round 1), then the full current document. Inputs: `02-proposal.md`,
`docs/adrs/010-lightweight-plan-flow.md`, the Revision log, my r1 findings.

## Closure check (my r1 findings)

- Windows `unknown` row (was major) — **closed**. The row now reads
  `| Windows / macOS | no | …outside the repo's tested surface, which is linux…`, a
  definite status in the shape of the `yes` row above it; the justification states the
  support surface, not the delta; Test strategy #21's closing clause is aligned with it;
  the Deferred bullet (a Windows/macOS CI job to move those rows to `yes`) is consistent
  with ADR-010, which never mentions Windows or macOS. Checklist item 1 satisfied: no
  `unknown`, no empty cell.
- macOS absent from the OS rows (was minor) — **closed**. macOS is named in the merged
  row with status `no`, so the question is answered; `yes` is not claimed, so checklist
  item 7 asks nothing of it.
- Path form reaching `stagePathAllowed`/`ownershipIssue` (was minor) — **closed**. The
  Ownership section now states "Paths reach `stagePathAllowed`/`ownershipIssue`
  repo-root-relative and POSIX-normalized (A8/A9), so the `flow` parameter parses no
  path: separator and case variants are settled before the predicate sees them", and
  Test strategy #7 asserts a foreign-separator / case-variant path "gets the verdict of
  its canonical spelling". The precedence is now pinned against the normalized form.

Checklist, remaining items: 2 (OS input APIs) is not applicable and is justified by A20
("No new platform surface: the flow adds a stage, a seat list and a config record");
3 (Node floor) — floor named at `>=22.23.4` (A20) and no API above it is used; 4 (native
modules) — pure TS, and the merged OS row explicitly closes the shell-out question
("the JS count at `runner.ts:179-180`, not a shell-out, so nothing here adds a Unix
dependency"); 5 (keyboard reality) — the design has no binding-name input surface;
6 (paths) — separators/case settled by the existing predicate's contract, no per-OS
config dir is added (state stays at `.pipeline.json` in the run dir); 7 — the only `yes`
platform row is linux, and #21 names a linux platform test at the `engines` floor.

## Findings

- [minor][new] Budget resolution / Compatibility matrix — "must carry a non-empty seat
  list for every phase the flow's `stages` runs … and must keep `planMaxLines` a positive
  integer" — the newly added `validated` step is phrased as an invariant on the *resolved*
  config, but the `full` flow's resolved config carries both keys absent: the knob table
  says `planMaxLines | absent (no gate)`, #5 asserts "no gate at all when `planMaxLines`
  is absent (the full flow's config)", and `reviewSeats` is absent from every existing
  `PipelineConfig` literal (A3). Read literally, `configFor('full', …)` — now on the entry
  path of every run, resume **and** `pipeline status` — throws for exactly the legacy
  configs the matrix row "Existing ADRs and briefs (no `flow` frontmatter) | yes" and
  Rollout 1's "no-op for every run" promise passes through. #23 only exercises
  present-and-invalid values, so the absent-key case is unpinned. Fix: add one clause to
  the `validated` sentence — validation applies to *present* values; an absent
  `reviewSeats` resolves through `seatsFor` to the five-seat default and an absent
  `planMaxLines` means no gate, both legal for `full` — and extend #23 with
  `configFor('full', { ...DEFAULT_CONFIG, maxParallelReviews: 1 })` resolving without
  throwing. No new mechanism; it narrows the rule the same section already states for
  `light`'s `design: []`.
- [nit][new] Test strategy #21 — "no macOS or Windows job exists, so both compatibility
  rows read `no` and no test claims them" — the matrix carries a single merged row,
  `| Windows / macOS | no | …`, so #21 points at two rows that are not there; a reader
  cross-checking the cited anchor finds one. Fix: name the row — "so the
  `Windows / macOS` row reads `no` and no test claims it". The status claim itself is
  consistent.
- [nit][new] Compatibility matrix — "the twelve `STAGE_BASES` stages, 5 seats, budgets
  unchanged" — every stage-count mention in the proposal is now twelve, while ADR-010's
  Context describes the same current path as "the identical 13-stage path". The matrix
  row is the place a reader reconciles the two documents, and the row's note is the only
  place the count is asserted. Fix: drop the count from the row's note (keep "`STAGE_BASES`
  stages, 5 seats, budgets unchanged") so the row claims no number the ADR contradicts.

## FYI (no severity)

- The twelve-vs-13 stage count against ADR-010's Context is a cross-document consistency
  question for the correctness/research seat; the Revision log records the count being
  settled to "every mention is the twelve `STAGE_BASES` rows", which leaves the ADR's
  number unmatched.
- A23/A25/A24 are self-declared unverified and Rollout 6 is the data step; the ≤ 8
  sessions / ≤ 96 min bound in Rollout 6 is an operating expectation, not a platform one.
