# complexity review r2

verdict: approve

## Scope

Ratification/confirmation pass over one scope target:
`reviews/design/editorial-r2.diff` (120 lines) applied to `02-proposal.md`.
Inputs read: the diff, `02-proposal.md`,
`docs/adrs/020-restart-boundary-for-harness-self-changes.md`, the Revision log.
No review record other than my own report path was opened.

The diff is language-only: `ADR 020` → `ADR-020` (nine sites), `in favor of` →
`in favour of`, `behavior` → `behaviour` (one site), `has imported and renders`
→ `already imports and renders` (§2, Resume), `the line ... prints` →
`the line that ... prints` (§3), and the §3 status bullet gains a `Status
output:` label with re-wrapped lines. No hunk adds, removes or re-scopes a
mechanism, a field, a test or a deferral. Against my previous findings (none),
nothing was broken; no section I cited was made false, because the diff leaves
every mechanism sentence byte-identical apart from wording.

## Checklist (complexity)

1. **Reuse.** Verified in code, not from the doc's own claims: the design reuses
   the existing path predicate's home (`stagePathAllowed`, gates.ts:559), the
   existing NUL-path parser (`nulPaths`, runner.ts:262, already used by the
   ownership sweep at runner.ts:307), the existing writer-side sanitizers
   (`printable`, independence.ts:443; `boundedGateEvidence`, runner.ts:3057),
   the terminal-commit posture of `finalizeDone` (runner.ts:3377), the
   companion-field shape of `planStop` (`StateSchema.planStop`,
   state-store.ts:96), and the status-line builder (`planStopStatusLines`,
   runner.ts:3580, composed by `statusLines`, runner.ts:3598 — the name Test #5
   uses is real). Nothing in `tools/` already answers
   `touchesExecutableSurface`, so the new predicate is not a duplicate.
2. **Trace to a Goal.** Every moving part maps to an ADR-020 Decision:
   predicate + per-window net diff → D1; `restartRequired` record +
   `finalizeRestartBoundary` + loop-top check + resume clear → D2; exit 3 +
   status line → D3; Test #8 provenance pin → D4; Test #7 → the ADR's
   "test that fails when a new executable path is added" (Consequences).
   No part traces only to itself.
3. **Smaller design.** Nothing in the diff makes the design larger. The one
   seam that could be cut is pre-existing and untouched by this diff — see FYI.
4. **New dependencies.** None claimed and none needed: `zod`, `yargs`,
   `@opencode/client` are already the package's only deps
   (package.json:45–48), the engine floor cited (package.json:30–31) is the
   current one, and Test #2's era fixtures exist
   (`tests/pipeline/fixtures/checkpoints/`: run-1-shape, run-2-era,
   light-era, run-3-live).
5. **Thin first slice.** Rollout #1 is one merged change and the design already
   defers the three parts ADR-020 leaves open (self-restart, boundary cap,
   `.opencode/agents/**`); those deferrals match the ADR's "Not decided here"
   (lines 119–123) and none contradicts its Decision or requirements, so they
   are not re-opened here.
6. **Knobs and one-implementation abstractions.** No configuration knob is
   added; `restartRequired` has three named consumers (exit mapping, status
   line, resume clear) and the predicate has two call sites plus the honesty
   test that keeps them honest. No new interface with a single implementation.
7. **Value check.** Proportionate. The whole change is one optional checkpoint
   field, one directory-scope predicate, two hooks in existing finalize sites,
   one loop-top guard, one resume clear and one exit-code branch; against what
   it buys — the run-016 escalation class, which the ADR records as costing a
   human diagnosis and a hand-started process — that is close to the minimum
   honest shape. The design's own rejected options (new `RunStatus` member,
   marker file, per-commit hooks at three sites) are exactly the heavier
   variants, and each rejection is grounded in an existing constraint (A6,
   ADR-006 §3, A2/A4), not in taste.

Check legs note: the session's shell was permission-denied, so the cargo,
rustfmt and npm legs could not be run. That left no checklist item unevaluated:
each code claim the design makes was checked by reading the cited source
(`finalizeStage` runner.ts:1848, `finalizeReviewPass` 1940, `finalizeDone`
3377, `planStopStatusLines` 3580, `runPipeline`/`resumePipeline` 2202/2302,
`StateSchema` version literal state-store.ts:81, `PROMPTS_DIR` pipeline/cli.ts:20,
exit mapping pipeline/cli.ts:214, 227 with thrown-error → 1 at 241).

## Findings

None. Nothing in the diff is new complexity, and no revision made a previously
approved section false.

## FYI (no severity)

- §3 / Rollout 1 — "widen their return from `RunStatus` to
  `{ status, restartRequired? }`" (unchanged by this diff, and recorded as
  decided in the Revision log's r1 Chosen design 3 entry, so it is not a
  finding for this pass): the widening is what makes Rollout 1's "Callers of
  `runPipeline`/`resumePipeline` adapt to the widened result at the seam" cost
  about 80 test call sites in `tests/pipeline/` that read the returned status.
  A smaller seam with the same contract keeps the return type and derives exit 3
  in `cli.ts` from the checkpoint record the pause has already committed — the
  `status` handler already calls `loadState(join(repo, 'docs', 'designs',
  slug))` at pipeline/cli.ts:190, so the read is an existing call and ADR-020
  D3 already says the status "is recorded in the run's checkpoint". Noting for
  the reviser's cost picture only; both options are sound.
- §3 wording change ("Status output:") is consistent with Test #5, which names
  `statusLines` — that function exists (runner.ts:3598), so the bullet's claim
  holds.
- Language, for the editorial seat, not a complexity finding: the diff's
  British spellings (`favour`, `behaviour`) now sit beside American ones in the
  same document — "behavior-safe defaults" (A6), "staleness behavior" (A13) —
  and the repo's ADRs are mostly American (`behavior` in ADR-001/003/004/005/
  007/010, `favor` in ADR-008), though ADR-020:44 itself uses "behaviour".
  Pick one per document.
