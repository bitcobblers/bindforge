# Code review — correctness seat, round 1 dissent pass (c1)

Scope: `git diff 91dfbea2f0ab34c063a14036b3afdc1f4b4227d9..HEAD -- ':(exclude)docs/designs'`,
read against the full `02-proposal.md`, `docs/adrs/003-parallel-inference-lane.md`, and
this seat's round-1 findings. Note: shell was denied in this seat's sandbox, so the
verification is static (file-level); the diff range was reconstructed from the current
tree and the build-notes provenance entries, and every code-level claim below was checked
against the landed source directly.

## Previous findings — resolution check

1. **major: undeclared independence subsystem + `SeatRecord.findings` field — RESOLVED.**
   `03-build-notes.md` deviation 7 now declares `src/pipeline/independence.ts`, its wiring
   into the attempt polling loop (runner.ts:832–849), the own-findings re-injection
   (`reviewVars` prevFindings, runner.ts:1901–1910; panel.ts:146–156), the persisted
   optional `SeatRecord.findings` (types.ts:107–114; state-store.ts SeatSchema:32–38),
   the prompt lines and agent denies, with provenance as a mid-run harness change, and it
   records the contradiction with the proposal's "no `.pipeline.json` schema field"
   sentence instead of arguing it away (additive, optional, `version` stays 1 — ADR-003
   §Self-update posture holds). The findings ledger is size-capped as the note claims
   (`boundedFindings` / `MAX_RECORDED_FINDINGS = 40` / `MAX_FINDING_FIELD_CHARS = 1000`,
   applied at record time runner.ts:1673 and again on injection runner.ts:1906).

2. **major: commit-time prettier + fix-stage template trigger undeclared — RESOLVED.**
   `formatTouchedFiles` is fully deleted: the only remaining traces are the explanatory
   comment at runner.ts:267–285 and the test-file tombstone
   (pipeline-fix-trigger.test.ts:15–20); no runner path execs `node_modules/.bin`
   anything (grep clean across src and tests). Deviation 9 declares the drop with the
   security rationale and the ownership-route replacement; deviation 8 declares the
   `stageTemplate` trigger (runner.ts:657–666), pinned by pipeline-fix-trigger.test.ts.
   Deviation 6 declares the `docs/workflows.md` bullet rewrite (workflows.md:75–85) and
   the `coder.md` `edit docs/workflows.md allow` (coder.md:11–13).

3. **minor: `updatedAt` frozen at run creation — RESOLVED.**
   `checkpoint()` stamps `state.updatedAt = Date.now()` before every sequential
   `saveState` (runner.ts:428–431) and is the only `saveState` call site on the
   sequential path; `enqueueSnapshot()` stamps before each `writer.enqueue` inside a
   pass (runner.ts:1213–1216), and the writer's synchronous `structuredClone` at
   enqueue (state-store.ts:215) preserves the stamp. `saveState` and `enqueue` still
   never stamp, exactly as deviation 1 records, and the integration test now pins the
   production behavior (`pipeline-parallel-integration.test.ts:232–234`:
   `updatedAt > startedAt` from a fixture that hand-starts both at 0).

## Checklist pass

1. Components named by the proposal exist and are wired: `concurrency.ts`
   (counter + FIFO waiter queue, permit handed directly to the oldest waiter),
   `executeInference` / `finalizeReviewPass` split, `runReviewPass` with the barrier
   (`allSettled` → `drain()` → rethrow first rejection → one sweep + one commit),
   escalation flag checked at acquire and after attempt 1, retry inside the same
   permit, `createStateWriter`, `maxParallelReviews?: number` with
   `DEFAULT_CONFIG.maxParallelReviews = 1`, `resolveMaxParallel` (env format gate runs
   even when the flag overrides; range ≥ 1; resolved before client/lock/checkpoint).
   The signature deviations are declared (deviations 2 and 4).
2. Assumption register honored: A4/A8 hold (`VerdictSchema` has no `commitMessage`;
   `commitStage` gets `undefined` reply, fallback shape); A5–A7 hold in the writer and
   semaphore code; A11: the editorial twin goes through `runReviewPass` with
   `pass: undefined` and keeps `-rN` naming.
3. Deviations: every gap found in the code appears in `03-build-notes.md` (deviations
   1–9, including the gate-label rename recorded per deviation 3 — `gateNameFor` in
   states.ts:34–42, documented in workflows.md:116–119, and nothing in code keys on the
   old labels — grep clean).
4. Scope creep: the two subsystems this seat flagged are now declared; the remaining
   non-proposal behavior in the diff (printable sanitization, finding caps, gate label
   naming) is declared with provenance. No undeclared behavior found in this pass.
5. Build-notes claims spot-checked against the code — deviations 1, 2 (one shared
   `design-review.md` template across the pass, so one preload serves every seat;
   `stageDef` ignores specialty for templates), 3, 4 (`assertSweepRanBeforeEscalate`
   exists), 5 (try/catch only around the `commit: false` sweep; the success path still
   throws), 6 (workflows.md CLI block, Tuning entry, ADR-003 removed from
   "Proposed, not implemented"; README:24–25 rationale), 7 (segment regexes vs the old
   substring, write fence, honest residual comment), 8, 9 — all true. The test-execution
   claims (four `npm run check` runs green, 224 tests / 26 files) could not be re-run
   from this sandbox; the tree does contain exactly 26 test files and all named ADR-003
   test files, and the static content is consistent.
6. Error paths fail closed: independence violation → attempt fails → fresh-session
   retry → escalate; writer refusal leaves the last good checkpoint untouched;
   escalation sweep best-effort but escalation always lands (deviation 5, recorded).
7. All three previous findings genuinely addressed in this diff (above).

## New findings (changed code only)

- **minor** — `tools/orchestration/src/pipeline/independence.ts:143–149`: the new
  cross-seat report **write fence** has no positive test. No test in
  `tests/pipeline-independence.test.ts` (or anywhere) ever passes
  `ownReportPath`; every write case exercises the `own === undefined` skip. The
  flagging branch (`key !== own` → "wrote a review report") is unexercised.
  Fix: add a case passing `ownReportPath: '.../security-r1c1.md'` with a `write`/
  `edit`/`patch` part aimed at a sibling report, expecting one violation — plus a
  negative case writing the handed path with `ownReportPath` set.
- **nit** — `tools/orchestration/src/pipeline/independence.ts:135,142,148` +
  `runner.ts:846`: build-notes' "all model- and server-authored strings reaching the
  terminal … pass through `printable()`" is not quite true for `part.name` (a
  server-supplied tool name interpolated raw into the violation text and printed by
  `console.warn`). Paths/command text go through `brief()`/`printable()`; the tool name
  does not. Fix: wrap `part.name` in `printable()` at the violation-construction site,
  or narrow the note's wording.

## Verdict

**approve.** Zero open blocker/major findings; the round-1 majors and the minor are all
resolved and declared, and the two items above are minor/nit.
