# compatibility review r3c2

verdict: approve

Ratification pass c2. Scope: `git diff 92f71e0 4635f0d` over `02-proposal.md` (the
revisions that landed after the r3 approval), then the full current document.
Inputs: the proposal, `docs/adrs/010-lightweight-plan-flow.md`, the Revision log,
my r3 findings. Checklist: `references/compatibility.md`.

## Checklist walk

1. **Matrix rows have a status.** All thirteen rows carry `yes` / `no` /
   `yes (no-op)`; no `unknown`, no empty cell. The one matrix row the diff touches —
   "`flow: Light` / `flow: sideways` (bad value); a caller's `reviewSeats.code: []`
   panel" — keeps `no` and a note that matches Budget resolution's new pre-lock
   posture, so the revision did not open a coverage hole.
2. **OS input APIs.** N/A by construction: A20 "No new platform surface: the flow
   adds a stage, a seat list and a config record", and the Windows / macOS row
   states the linux-only justification plus the "no shell-out" reason for the meter.
3. **Node floor.** `>=22.23.4` (A20, `package.json` `engines`). Nothing in the
   revised text adds an API above it — the new pre-lock `validated` call and
   `assertForceableStage` posture are plain TS/CP logic.
4. **Native modules.** None introduced. The plan cap is explicitly "the JS count at
   `runner.ts:179-180`", named as "not a shell-out" in the Windows / macOS row, so
   no silent Unix or native dependency arrives with the `plan` stage.
5. **Keyboard reality.** N/A — no keyboard/text-input surface in this design.
6. **File paths.** The diff adds one path by name (`.opencode/agents/design/planner.md`)
   and cites agent frontmatter ranges; no string-concatenated path handling is
   introduced. The Ownership section's intake rule (A28: git plumbing output,
   repo-root-relative, POSIX separators, no absolute and no `.`/`..`) is untouched.
7. **A platform test per supported platform.** The only `yes` platform row is
   `Node >= 22.23.4, linux`, and Test strategy #21 runs the suite on linux at the
   `engines` floor; Windows / macOS read `no` and claim no test.

The diff's other changes (A12's rewrite, the Budget resolution pre-lock sentence,
the `planner.md` bullet, Test strategy #17, Rollout 6 trimming, two Revision log
entries) touch no platform claim I approved; the two rows and the test I cited in
r3 are byte-unchanged, so both r3 nits stand as re-raised, and one new imprecision
appears in the rewritten #17.

## Findings

- [nit][re-raised] Compatibility matrix — "Existing ADRs and briefs (no `flow`
  frontmatter)" row, "`configFor('full', base)` is deep-equal to `base`" (with
  Rollout 1, "this is a no-op for every run (#3)") — unchanged by this revision.
  The registry paragraph scopes the claim to "deep-equal to a **complete** `base`,
  #3", and #3 pins it as "deep-equal to a **complete** `PipelineConfig` literal
  `base`", i.e. by compiles-plus-five-seat resolution on a complete literal, not by
  deep equality on an arbitrary `base`. The matrix row keeps the unqualified `base`,
  and Rollout 1 cites #3 for a no-op claim the registry sentence scopes. Nothing is
  false for the complete CLI-shaped config both sentences describe; only the
  citation is imprecise. Fix: matrix row reads "deep-equal to a complete `base`";
  Rollout 1 reads "no-op for every run (a complete `base` is deep-equal, #3)".
- [nit][re-raised] Test strategy #21 — "no macOS or Windows job exists, so both
  compatibility rows read `no` and no test claims them" — unchanged by this
  revision. The matrix carries one merged row, `| Windows / macOS | no |`, so #21
  cites two compatibility rows where a reader cross-checking the anchor finds one;
  the Deferred bullet ("move those compatibility rows from `no` to `yes`") inherits
  the same plurality. The status claim itself matches that row and the Deferred
  entry. Fix: "no macOS or Windows job exists, so the `Windows / macOS` row reads
  `no` and no test claims it", and the Deferred bullet reads "that compatibility
  row".
- [nit][new] Test strategy #17 (rewritten by this diff) — "Integration — pre-lock
  gates: `flow: sideways`, `--stage research` on a light checkpoint, and a caller's
  `opts.cfg` with `reviewSeats.code: []` fail as a usage error — no lock file, no
  design directory, no entry checkpoint, no `ESCALATION.md`, **zero** client calls"
  — the newly added "no entry checkpoint" joins an absence list asserted jointly over
  three cases with different pre-conditions, and two of them are resumes: a light
  checkpoint (the case's own premise) and, per the new Budget resolution sentence,
  "`validated` runs on it in `runPipeline`/`resumePipeline`". For a resume the design
  directory and a checkpoint are already on disk, so read literally the list is
  unsatisfiable for those cases (the sibling "no design directory" has the same
  shape), and a test author writing #17 has no defined assertion for them. Fix:
  scope the absence list to the fresh-entry case (`flow: sideways`, a fresh
  `runPipeline` with the bad `opts.cfg`) and state the resume cases as "no new lock,
  no new entry save, no `ESCALATION.md` write, zero client calls".

## FYI (no severity)

- A12's rewrite states a universal and its exception in one sentence ("all four
  document-stage primary agents … deny … `shell: "*"`; `doc-sync.md:23-25` is the
  one exception, an allow for `git diff*`") — the `plan`-stage bullet resolves the
  ambiguity for the implementer ("`ops/doc-sync`'s `git diff*` allow is not part of
  it"), so the wording is a prose question, not a compatibility one.
- A12 now cites `stages.ts:101-150` as basis while The Flow registry cites
  `stages.ts:101-138` for "the stage/agent/template rows"; a reader may want the
  wider range labelled with what it covers beyond the twelve rows.
