# Design: Run review passes across up to N parallel inference slots (ADR-003)

## Context

The pipeline runs every reviewer session behind the previous one; ADR-002 budgets 40
design + 20 code sessions per run, so that serial lane is the run's dominant wall-clock
cost. ADR-003 replaces ADR-001's sequential lane with a bounded-concurrency lane:
the seats a pass selects launch in fixed order, run up to N at a time, and finalize
once per pass, while every other state-machine semantics stay as written
(source: [ADR-003](../../adrs/003-parallel-inference-lane.md),
[research](./01-research.md)). Today's `tools/orchestration/src/pipeline/runner.ts`
has no seam for this: `executeStage` (lines 604–952) mixes inference with the git
finalize, and the seat loop (lines 1304–1315) is strictly awaited.

## Goals / Non-goals

- Goals:
  - Fan out the seats of `kind: 'review'` passes — design, code, and editorial —
    through an in-process semaphore of size N, launched in the fixed order
    (`DESIGN_SPECIALTIES` / `CODE_SPECIALTIES` in `stages.ts`; ADR-003 names the
    design order `correctness → compatibility → security → performance → complexity`).
  - Split `executeStage` into a git-free inference phase and a finalize phase;
    review passes finalize once, at pass level (one `enforceOwnership` + one
    `commitStage` per pass).
  - Route every checkpoint write made from inside a concurrent pass through one
    in-process write path, drained before pass finalize (ADR-003 §Self-update).
  - Add `PipelineConfig.maxParallelReviews` (default 1) plus the CLI knob
    `--max-parallel <N>` / `$OPENCODE_MAX_PARALLEL`, validated with the same strict
    fail-closed pattern as `--timeout` (ADR-003 §Configuration).
  - Keep N = 1 on the same code path (degenerate fan-out, not a branch).
- Non-goals (each prevents scope creep at code review):
  - Changing which seats a pass contains, the transition table, gates, artifact
    contract, or per-stage session model — ADR-002's `planPass` output is scheduled
    as handed (ADR-003 §What is amended).
  - Resume-safe / cross-version checkpoints, checkpoint trust, corruption recovery
    — ADR-004 owns these.
  - Parallel mutating stages or concurrent pipeline runs; model-driven subagent
    fan-out — rejected in ADR-003's Alternatives.
  - Raising the default above 1; adaptive concurrency; discovering the server's
    slot count.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | The operator's inference server exposes multiple parallel slots; the repo records no slot count, so "N ≤ server slots" is operator guidance the runner cannot verify | unverified | docs/adrs/003-parallel-inference-lane.md:21–24; research "Slot count is machine state" |
| A2 | The OpenCode server's behavior when prompts outnumber slots (queue, reject, or error) is unspecified anywhere in this repo | unverified | research Open question 5; https://opencode.ai/docs/server/ lists no concurrency guarantee |
| A3 | Seats of one pass read the same immutable inputs and write disjoint files (reports keyed by specialty+pass, transcripts keyed by stageKey) | verified | panel.ts:162–178 (`reportPathFor`); runner.ts:872–884 (transcript keyed by stageKey + attempt) |
| A4 | Pass gates are order-insensitive: `verdictsFor` takes the last valid reply per `stage:round`, `allReviewsApproved` is set-based | verified | gates.ts:81–168 |
| A5 | Node's single-threaded event loop makes `state.results.push`, `gateLog` appends, and the semaphore counter non-tearing; races exist only at `await` boundaries | verified | https://developer.mozilla.org/en-US/docs/Web/JavaScript/Event_loop; research Platform notes |
| A6 | Every runner git call is `spawnSync`, so git mutations are serialized by construction; the race is a sweep landing mid-sibling-write, not two gits racing | verified | runner.ts:96–135; https://nodejs.org/api/child_process.html |
| A7 | Checkpoint writes are tmp-file + rename inside the design dir: crash-safe, but unordered between concurrent writers — a concurrent pair can land the older snapshot last, and a stray `.pipeline.json.*.tmp` sits where `git add -A` would sweep it | verified | state-store.ts:157–173; https://man7.org/linux/man-pages/man2/rename.2.html |
| A8 | Review replies carry no `commitMessage` field (`VerdictSchema`), so review commits are already the fallback pattern today | verified | types.ts:19–23, 48; runner.ts:272–277 |
| A9 | A long-lived Node process never reloads its own modules, so the self-update run's driving process executes only code frozen at its start | verified | docs/adrs/003-parallel-inference-lane.md:107–112 (startup code-freeze posture); Node's module cache: https://nodejs.org/api/modules.html#modules_caching |
| A10 | The existing `node >= 22.12.0` engine floor already covers everything the fan-out uses (no new primitive needs a higher floor) | verified | tools/orchestration/package.json:30–32 |
| A11 | `editorial-review` is a `kind: 'review'` stage with a five-seat sequential loop and `-rN` (no pass number) report naming, so the pass-level rule reaches it | verified | stages.ts:107–111; runner.ts:1183–1193; panel.ts:162–178 |

## Options considered

**Per-reviewer commit under a git mutex.** Each seat finalizes itself, serialized by
a lock. Rejected (ADR-003): `enforceOwnership`'s dirty-file scan cannot distinguish a
sibling reviewer's in-flight edits from a permission violation, and `commitStage`'s
`git add -A` would sweep a sibling's half-written report into the wrong commit.
Pass-level finalize removes the interleaving instead of locking around it.

**Model-driven subagent fan-out.** Rejected again for ADR-001's still-valid reason:
pipeline semantics must not depend on model compliance. The cap stays in runner
code.

**Parallel mutating stages / concurrent pipeline runs.** Rejected for now: the
mutating chain has real data dependencies over one worktree; multi-run concurrency
needs per-run worktrees and a per-slug lock. Future ADR (ADR-003 Alternatives).

**Raise the default to N = 5 immediately.** Rejected (ADR-003): the concurrency path
is unexercised until the first post-merge run, and N is a server-capacity decision.

**Third-party limiter (`p-limit` / `p-queue`) vs ~20-line in-process semaphore.**
Rejected the dependency: orchestration's lane machinery is dependency-free, and the
required semantics (FIFO acquire, held across a seat's retry) are a counter plus a
waiter queue (research Prior art).

**One shared env-resolver module vs a local resolver in `src/pipeline/cli.ts`.**
Rejected extraction for now: `resolveReplyTimeoutSeconds` (`src/cli.ts:101–137`) has
its own name and range; the pipeline needs the same *pattern*, not the same
function. A local `resolveMaxParallel` is the smaller change; extraction is deferred.

**Auto-raise `stageTimeoutMs` from N vs document the interaction.** Rejected the
coupling: the ADR gives only guidance ("N > slot count needs a raised timeout"), and
any formula would invent server behavior the repo doesn't document (A2). The deadline
stays as written — measured from each seat's own prompt, queue waiting counted —
with the guidance written into `docs/workflows.md`.

## Chosen design

### Two-phase split of `executeStage` (`src/pipeline/runner.ts`)

`executeStage` (lines 604–952) splits in place, not into a new layer (research
Finding 1):

- `executeInference(opts, state, handle, cfg): Promise<StageOutcome>` — everything
  through `state.results.push` and the transcript write (lines 613–915): render
  prompt, create session, poll/nudge loop, validate reply, `writeTranscript`, append
  the `StageResult`. No git mutation. `headBefore` is no longer captured here; the
  caller captures it before the prompt — once per pass for the fan-out, per stage
  for the single-stage path (today's spot, line 635).
- `finalizeReviewPass(opts, state, headBefore, stageName, cfg)` — the only git
  code for review passes: `enforceOwnership(repoDir, headBefore, stageName,
  state)` + `commitStage(...)`, moved out of the `if (outcome.ok)` block (lines
  917–950). All three review stage names share one ownership rule (`docs/**`,
  gates.ts:260–273), so one sweep is sound (A3, A6). The commit's stage key is
  the pass's stage name, so the message has today's fallback shape with one
  value: `docs: <nnn-slug> design-review stage` (A8: no `commitMessage` in replies).

Non-review stages keep the current combined path: `executeInference` followed
immediately by the single-stage finalize that exists today. Fan-out is guarded by
`next.kind === 'review'` in `drive` — the mutating chain never shares a slot
(ADR-003 §Execution 5).

### Pass-level fan-out in `drive` (`src/pipeline/runner.ts`)

The seat loop (lines 1304–1315, and the editorial twin at 1183–1193) becomes
`runReviewPass`:

```
pass entry: capture headBefore once; create Semaphore(cfg.maxParallelReviews)
  │  seats handed by planPass / specialtiesFor('editorial'), fixed order
  ▼
per seat: await sem.acquire(); release + skip if escalationPending
  └─ runSeatWithRetry: attempt 1; on failure, attempt 2 inside the SAME slot
     (one retry then set escalationPending, runner.ts:1126–1170); transcript
     written, result pushed, failure/gateLog in-memory; saveState → writer
  ▼
await allSettled(seat tasks)          ← barrier
  ▼
writer.drain()                        ← no in-flight checkpoint write, no stray tmp file
  ▼
finalizeReviewPass(headBefore)        ← one enforceOwnership + one commit
  ▼
recordSeatVerdicts → cycle bump → pass gate (planPass re-eval, all set-based)
  ▼
saveState (sequential, as today)
```

- **Order.** `runReviewPass` calls `sem.acquire()` for the seats in the handed
  order; the semaphore's waiter queue is FIFO, so with N < |panel| the session
  *creation* order is still `correctness → … → complexity` even though completion
  order is not. Transcripts already record wall-clock `startedAt`/`endedAt`
  (prompts.ts:50–51, 89–90), so debuggability needs no new field; gates are
  set-based (A4), so nondeterministic completion cannot move a verdict.
- **Retries consume slots.** The permit is taken before attempt 1 and released
  after attempt 2 settles, so total in-flight inference never exceeds N
  (ADR-003 §Execution 3).
- **N = 1 is the same path**: a semaphore of size 1 serializes the seats. The
  canonical caveat lives here: only commit shape and finalize timing change —
  one commit and one ownership sweep per pass, not per seat (ADR-003 §Self-update).
- **Escalate after siblings settle.** Inside a pass, `runSeatWithRetry` does not
  call `finalizeEscalate`; it sets a pass-level `escalationPending` flag and
  records the failure in `state.stageFailures` and the `stageFailure` gate entry
  (as today). A seat that acquires after the flag is set releases its permit and
  returns without prompting, so no seat that today's `StopDriving`
  (runner.ts:1158–1166) would skip ever gets run — at N = 1 control flow is
  exactly today's. After the barrier, if the flag is set: drain the writer, run
  the pass `enforceOwnership` sweep, then
  `finalizeEscalate` once with today's `failed twice … one retry, then escalate`
  message. The sweep reverts out-of-policy sibling files and escalation still
  lands; its commit carries only complete in-policy sibling reports.
- **The editorial pass fans out too** (A11): its `kind: 'review'` registration
  puts it in scope, so it gets the same barrier, one commit, and one ownership
  sweep; its `-rN` report naming (cycle undefined in `reportPathFor`) is unchanged.

### Bounded-concurrency primitives (`src/pipeline/concurrency.ts`, new)

~20 lines, no new dependency (ADR-003 §Execution 3):

- `createSemaphore(limit: number): { acquire(): Promise<() => void> }` — running
  counter + FIFO waiter queue; `acquire()` resolves with a release function. Races
  are only at `await` boundaries, which the single-threaded loop makes safe (A5).

### Drained checkpoint writer (`src/pipeline/state-store.ts`)

`createStateWriter(designDir): { enqueue(state): Promise<void>; drain(): Promise<void> }`
— a one-at-a-time chain over `saveState` keyed to enqueue order, so the last landed
rename is the newest snapshot instead of luck (A7). `runReviewPass` uses it for every
in-pass write (seat failures, gate entries), and the barrier awaits `drain()` before
`finalizeReviewPass` — which also guarantees no half-written
`.pipeline.json.*.tmp` sits in the design dir when the pass's `git add -A` runs.
Writes outside a pass stay the plain sequential `saveState`. `saveState` itself is
unchanged; no `.pipeline.json` schema field is added by this design (nothing needs
persisting: N is per-run config), so `version` stays 1 by default.

### Configuration and CLI

- `stages.ts`: add `maxParallelReviews?: number` to `PipelineConfig` — optional,
  deliberately relaxing the ADR's required field so existing configs compile —
  and `DEFAULT_CONFIG.maxParallelReviews = 1`, the sole default authority; opt-in.
- `src/pipeline/cli.ts`: `--max-parallel <N>` on `run` and `resume`, env fallback
  `$OPENCODE_MAX_PARALLEL`, resolved by a local `resolveMaxParallel(flag, env)` that
  mirrors `resolveReplyTimeoutSeconds` (`src/cli.ts:101–137`): a `^\d{1,5}$` format
  gate on the env string that runs even when the flag overrides it (bad env is a
  config error), a range check (integer ≥ 1) as a separate error naming the source,
  all validation before side effects. The CLI then passes
  `cfg: { ...DEFAULT_CONFIG, maxParallelReviews: n }`.
- No coupling between N and `stageTimeoutMs` in code; `docs/workflows.md` (the CLI
  block at lines 88–93) and the README gain the knob, the `N ≤ server slots`
  guidance, and the note that each seat's deadline starts at its own prompt and
  counts queue waiting (ADR-003 §Configuration; A1, A2).

### Validation posture (dogfooding)

The self-update run never executes its own concurrency code — the driving process
froze its modules at startup and `.pipeline.lock` still admits one process per repo
(A9). The parallel lane is therefore validated by the mock-client tests in Test
strategy #1–#8 plus the first post-merge parallel run (ADR-003 §Self-update).

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| N > real slot count: seats queue or fail server-side; each seat's `stageTimeoutMs` deadline keeps running (A1, A2) | Medium (runner cannot verify N) | High (mass session timeouts → escalation) | Default N = 1; documented `N ≤ slots` guidance; per-run flag, so damage is scoped to one opted-in run |
| Server reacts to over-subscription by erroring prompts instead of queueing | Unknown (A2) | Medium (seat fails twice → escalate) | Failure path unchanged (one retry, then escalate); first post-merge run at N = 2 is the live check |
| Pass commit sweeps a sibling's half-written file | Low after this design | High (wrong file in wrong commit) | Pass-level finalize + drained writer (Chosen design); Test strategy #6, #7 |
| Escalation mid-pass loses sibling work or races its writes | Low | Medium | Escalation deferred to after the barrier; drained writer and one `enforceOwnership` sweep before `finalizeEscalate` |
| Future stage kinds implement the two-phase contract wrong (forget the finalize, or git in inference) | Medium over time | Medium | The split is in one file with the contract stated where both halves live; N = 1 exercises the same path on every run, so a missing finalize shows in the very next dogfooded run |
| Two `saveState` snapshots land out of order (lost reviewer record) | Low after this design | High (resume replays settled work) | Single writer, enqueue-ordered, drained at the barrier; Test strategy #7 |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux, Node ≥ 22.12.0 | yes | The environment this repo develops and CI-checks in; no primitive above the existing floor (A10) |
| macOS | no | Untested; no OS-specific code added (Linux-only CI, Test strategy #10) |
| Windows | no | Same; git stays `spawnSync` (A6) |
| Run with N = 1 (default) | yes | Degenerate fan-out; commit shape and finalize timing differ from pre-change (caveat in Chosen design) |
| Run with N > 1 | yes, conditional on the server exposing ≥ N slots | Opt-in via `--max-parallel` / `$OPENCODE_MAX_PARALLEL`; slot count unverifiable by the runner (A1) |
| Checkpoints written before this change | yes | No checkpoint schema change; `maxParallelReviews` is run config, not persisted state |

## Test strategy

1. Unit — semaphore: never more than N concurrent permits; FIFO waiter order; a
   release wakes the oldest waiter (`src/pipeline/concurrency.ts` in isolation).
2. Unit — full-panel pass with a mock client at N ≥ 5 launches all five design
   seats concurrently (max in-flight observed == |panel|; no in-flight ever
   exceeds N at any N).
3. Unit — dissent pass launches only the dissenting seats (`planPass` output
   scheduled verbatim); ratification pass launches only its stale approvers.
4. Unit — launch order at N < |panel|: mock client records `session.create` order;
   it equals the handed seat order.
5. Unit — a seat that fails twice: attempt 2 runs while holding the same permit
   (a full semaphore blocks attempt 2); no unstarted seat runs; escalation is
   one `enforceOwnership` sweep then one `finalizeEscalate` after siblings settle.
6. Unit — exactly one commit and one `enforceOwnership` sweep per review pass at
   N = 1 and N = 3 (temp git repo; assert commit count and the exact message),
   including the editorial pass with its `-rN` report naming.
7. Unit — checkpoint writer: concurrent `enqueue` calls land in enqueue order (a
   slower first write cannot land last); `drain()` resolves before `finalizeReviewPass`
   mutates git; after a pass commit, no `.pipeline.json.*.tmp` file exists in the
   design dir.
8. Unit — `resolveMaxParallel`: malformed `$OPENCODE_MAX_PARALLEL` is a config
   error even when a valid `--max-parallel` overrides it; flag values < 1 rejected;
   valid values pass through (mirrors the `--timeout` tests in `tests/cli.test.ts`).
9. Integration — end-to-end `runPipeline` review pass on a temp git repo with a
   mock client at N = 2: every report and transcript exists, results and seat
   verdicts are recorded once per seat, the pass produces one commit, the
   checkpoint loads with `loadState`, and the gate log has one pass-gate entry —
   identical control flow to the same fixture at N = 1 (result order included).
10. Platform — Linux CI (`npm run check` in `tools/orchestration`): the full
    suite above plus existing runner/ownership/funnel tests stay green at the
    default N = 1; N > 1 against a real server is exercised only by the first
    post-merge run (Rollout), not CI, per the dogfooding posture.

## Rollout

1. Merge the branch; nothing to flip — the default stays N = 1, so every run keeps
   ADR-002 gates, artifacts, and control flow, with only the pass-commit shape
   changed (Chosen design caveat).
2. No migrations: no new persisted field, `version` stays 1; checkpoints from
   before the merge load unchanged.
3. Docs land with the change: `docs/workflows.md` and the README gain
   `--max-parallel <N>` / `$OPENCODE_MAX_PARALLEL` with the `N ≤ server slots`
   guidance and the timeout/queueing note, and ADR-003 moves out of the "Proposed,
   not implemented" list in `docs/workflows.md` (lines 80–84).
4. The first post-merge parallel run (operator sets `--max-parallel 2..5` within
   their server's slots) is the live validation of the lane; a run that straddles
   the merge with `pipeline resume` is explicitly not covered (ADR-004).
5. Raising the default above 1 is a future decision, not this rollout (ADR-003
   rejected it); operators opt in per run until then.

## Deferred / out of scope

- Cross-version checkpoint resume, checkpoint trust, and recovery from a corrupted
  or replaced checkpoint — ADR-004 (docs/adrs/004-self-update-checkpoint-compatibility.md).
- Concurrent pipeline runs / per-run worktrees and parallel mutating stages —
  future ADR named in ADR-003 Alternatives; no number yet.
- Adaptive concurrency reacting to server rate-limit signals — no follow-up yet
  (research Prior art flags promptfoo's pattern as the obvious future knob).
- Per-stage parallel or model/endpoint routing — ADR-001's `stages.ts` mapping
  anticipates it; this design ships one integer per run (research Open question 6).
- Verifying or discovering the server's slot count (no endpoint documented) —
  no follow-up yet; stays operator guidance (A1, A2).
- Extracting one shared strict env-resolver across `src/cli.ts` and
  `src/pipeline/cli.ts` — no follow-up yet; pattern duplicated deliberately.

## Revision log

- r1: Pass-level fan-out — a pass-level `escalationPending` flag stops unstarted
  seats after a retry-exhausted failure (correctness, performance).
- r1: Pass-level fan-out — escalation runs the ownership sweep before `finalizeEscalate`; the widened commit is enforced (security).
- r1: Compatibility matrix — macOS/Windows: `unknown` → `no`, untested.

## As-built (2026-11-10)

Record of what the branch shipped, written after the implementation and the
code-review panel closed the run. Nothing below changes the design above — it
records drift. Full provenance and finding-by-finding detail:
[`03-build-notes.md`](./03-build-notes.md), deviations 1–10.

**Built as designed:** `src/pipeline/concurrency.ts` (counter + FIFO waiter
queue, no dependency), `createStateWriter` / `CheckpointWriter.drain()` in
`state-store.ts`, the `executeInference` / finalize split with `executeStage`
keeping the combined single-stage path, `runReviewPass` plus its editorial
twin (handed order, permit held across the seat's retry, `Promise.allSettled`
→ `drain()` → one `enforceOwnership` + one `commitStage`), the pass-level
escalation flag that stops unstarted seats and defers `finalizeEscalate`,
`maxParallelReviews` on `PipelineConfig` with `DEFAULT_CONFIG` as sole default
authority, `resolveMaxParallel` plus `--max-parallel` on `run` and `resume`,
and N = 1 on the same code path. Test strategy #1–#10 all landed, as the
named files under `tools/orchestration/tests/`.

**Drift — proposal section → as built, and why:**

| Affected proposal section | What was built | Reason |
|---|---|---|
| Chosen design → Two-phase split | `finalizeReviewPass(repoDir, state, headBefore, stageName, commit)`, not the sketched `(opts, state, headBefore, stageName, cfg)` | The pass needs no `opts`/`cfg`: review replies carry no `commitMessage` (A8), so `commitStage` gets the fallback shape. `commit: false` keeps the sweep without the commit on the escalation path. |
| Two-phase split; Pass-level fan-out → Order | `executeInference` gained an optional fifth parameter, `templateText`; the pass loads its template once before any seat starts | With the per-seat `loadTemplate` await, `session.create` order could interleave with template reads and the Test strategy #4 order assertion is not guaranteed; preloading makes the semaphore the only await ahead of creation. |
| Drained checkpoint writer ("`saveState` itself is unchanged") | `saveState` no longer stamps `updatedAt`; the stamp lives at the write sites — `runner.ts: checkpoint()` for the sequential path, `enqueueSnapshot()` before each `writer.enqueue` inside a pass | A stamp at rename time would let a queued snapshot masquerade as a newer one. The field's meaning ("when this checkpoint was taken") was preserved instead of the function body. |
| Drained checkpoint writer / Compatibility matrix / Rollout 2 ("no `.pipeline.json` schema field is added") | Still true for the lane itself — N is run config, `version` stays 1 — but the run added `SeatRecord.findings?` and relaxed `StageResult.reply` to optional | Both come from subsystems landed outside this design (the review-independence audit; a run-003 checkpoint that could not re-read its own failed attempts), not from the fan-out. Recorded, not argued away. |
| Non-goals ("gates … unchanged") | The pass gate's recorded label changed: `all*ReviewsApproved` → `design-review` / `editorial-review` / `code-review` (`states.ts: gateNameFor`) | The integration test asserts one `gate: 'design-review'` entry per pass; a label naming the stage it gates reads next to that pass's commit and transcripts. Which gate runs and what it decides are untouched, and nothing in code keys on the old names. |
| Pass-level fan-out → "Escalate after siblings settle"; Risks row 4 | The escalation-path ownership sweep is best-effort: wrapped in a try/catch that logs and continues to `finalizeEscalate`. The success-path sweep still throws | `enforceOwnership`/`commitStage` shell out to git and throw on any non-zero exit; an escaping throw would end `drive` before the terminal record — status left `running`, no `escalate` gate entry, no `ESCALATION.md`. |
| Bounded-concurrency primitives | `createSemaphore` also rejects a non-integer or `< 1` limit | A guard beyond the ~20-line sketch, so a config object that omits `maxParallelReviews` can never reach `createSemaphore(undefined)`. |
| Rollout 3 (docs) | Landed. `docs/workflows.md` gained the lane paragraph, the CLI line, the `maxParallelReviews` Tuning entry and the gate-label vocabulary; its whole "Proposed, not implemented" subsection was deleted (ADR-003 was its only entry). `tools/orchestration/README.md` states why its flag table omits the knob | Rollout 3 asked for the docs to land with the change. |

**Landed beyond this design's surface** (mid-run harness work, undeclared
when it landed and declared since — build notes deviations 7–10): the
review-independence audit `src/pipeline/independence.ts` with its wiring into
every attempt's polling loop, the persisted `SeatRecord.findings` ledger and
its own-findings re-injection, the `transcripts/**` / `reviews/**` permission
denies and prompt lines, the `stageTemplate` trigger that hands a tests-red
`fix` the new `prompts/fix-tests.md`, a runner auto-format step that was added
mid-run and then dropped, `printable()` sanitization of model- and
server-authored text, and the findings size caps. None of them is in the
design surface above; the fan-out's contract is unchanged by them.

**Left open at close** (panel-approved minor/nit, non-blocking; the ratifying
seats at cycle 2 approved with these carried): no positive test passes
`ownReportPath`, so the write fence's flagging branch is unpinned; no test
drives a throwing escalation-path sweep; `part.name` still reaches the
terminal un-`printable()`; the `$OPENCODE_MAX_PARALLEL` control-byte
reflection case that `tests/cli.test.ts` has for `--timeout` was not ported.
Gate at close: `npm run check` in `tools/orchestration` exits 0 — eslint and
prettier clean, vitest 224 tests across 26 files.
