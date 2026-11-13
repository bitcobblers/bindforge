# Build notes: Restart boundary for harness self-changes (ADR-020)

Implementation of [02-proposal.md](./02-proposal.md) in `tools/orchestration`.
Branch `pipeline/020-restart-boundary-for-harness-self-changes`; the tests
landed first (326e8a3, "test: add restart boundary suite for ADR-020
self-change pause") and this change is the source side.

## What was built (Rollout step 1, item by item)

- `types.ts` — `RestartBoundary` (`{ stage, paths, at }`) and
  `PipelineState.restartRequired?` beside `planStop`.
- `state-store.ts` — `RestartBoundarySchema`, an optional `z.object` in
  `StateSchema` (`stage` min length 1, `paths: string[]`, `at: number`);
  `status` and `version` untouched (ADR-006 §2 grammar, A6).
- `gates.ts` — `touchesExecutableSurface(path)` beside `stagePathAllowed`:
  directory scope under `tools/orchestration/src/` or
  `tools/orchestration/prompts/` (§1's two prefixes, no path parsing).
- `runner.ts` —
  - `surfacePathsInWindow`: the window's **net** diff, `git diff -z
    --no-renames --name-only <headBefore>..HEAD` through the existing
    `nulPaths` parser, filtered through the predicate (§1);
  - the detection hook `recordRestartBoundary` after `recordStageCommit` in
    `finalizeStage` and in `finalizeReviewPass` (commit-path only), so the
    stage/pass commit and any `enforceOwnership` revert commits net out (A2,
    A4);
  - the loop-top pause: at the top of `drive`'s `for (;;)` body, **before
    `decideNext`** (§1), a standing record calls `finalizeRestartBoundary`
    and returns;
  - `finalizeRestartBoundary` (beside `finalizeDone`, done-path terminal
    pattern): `restart-boundary` gate entry naming stage + bounded hit paths
    (`printable` + `boundedGateEvidence` posture, r2 security revision),
    terminal save under the ADR-006 §3 guard, the save-before-commit
    `chore: <nnn>-<slug> restart boundary` commit naming the paths in its
    body, and the printed summary naming the stage, paths, and
    `pipeline resume <slug>`;
  - resume-is-the-restart: `resumePipeline` gate-logs the boundary it crosses
    (`restart-boundary-crossed`) and clears the record before driving, so a
    forced `--stage` resume lands on the forced stage (§2);
  - `restartBoundaryStatusLines` in `statusLines` while the record stands
    (§3, `planStopStatusLines` shape);
  - the widened seam: `runPipeline`/`resumePipeline` return
    `PipelineRunResult { status, restartRequired? }` (§3).
- `cli.ts` — `exitCodeFor`: `done → 0`, standing `restartRequired → 3`, any
  other non-done `→ 2`, thrown error `→ 1` (the catch around `main`'s
  dispatch — see Deviation 10); the header comment carries the table;
  `PROMPTS_DIR` exported (see Deviations 4).
- Rollout step 3: `docs/workflows.md` gains "When a run pauses at a restart
  boundary (ADR-020)" after the lock paragraph — the 0/3/2/1 table, the
  `pipeline status` line, and the resume-from-the-worktree driver posture.

## Deviations from the proposal

1. **The widened seam leaves pre-existing test call sites red (expected,
   test attention needed).** Rollout 1 says "Callers of `runPipeline`/
   `resumePipeline` adapt to the widened result at the seam"; all src-side
   callers (`cli.ts`) adapted. 72 assertions across 17 test suites still
   destructure the pre-020 bare `RunStatus` (`const status = await
   resumePipeline(...); expect(status).toBe('done')`) and now compare a
   string against `{ status: 'done' }`. The new ADR-020 integration suite is
   already written against the widened result, so the suite must move the old
   pins in the same change; this stage cannot edit test files. See the gate
   evidence for the exact inventory.
2. **`restart-boundary-integration.test.ts > the driver loop > still
   escalates on a red check gate` contradicts §1's mandated check point.**
   §1: the pause sits "at the top of `drive`'s `for (;;)` body, before
   `decideNext` runs". In the actual runner a coder stage **finalizes even
   when the check gate is red** — the commit lands in `finalizeStage`
   (runner.ts:1940–1955) and `checkGate` runs afterwards in the same
   iteration (runner.ts:2988–3012), which is exactly what makes the existing
   fix-loop routing work. So in this fixture the implement window *does*
   finalize and open a record, and the loop-top pause pre-empts the
   fix-round escalation the fixture expects: result `running` + record, not
   `escalated`. The fixture's comment ("a stage that never finalized never
   opened a window to classify") describes a stage-failure path; a red check
   gate is not one. Test-side fix options: assert the pause posture for the
   red-gate sibling (pause and escalation stay distinguishable at the exit
   code — 3 vs 2), or drive the sibling to a stage that genuinely fails to
   finalize. The implementation follows §1.
3. **`restart-boundary-integration.test.ts > consecutive boundaries` cannot
   converge under the light flow's pinned pass cap.** The fixture passes
   `cfg.maxPassesPerRound: 6`, but the effective config for a light run is
   the flow registry's (flows.ts:91, `maxPassesPerRound: 3`) — pre-existing
   behavior pinned by `light-budget.test.ts` ("the effective config on a
   light resume"). The stub keys rejections on the per-process session
   ordinal, and every resume builds a fresh stub, so the testing seat
   re-dissents in each process: round 1 needs r1 + r1c1 + r1c2 + the
   ratification pass = 4 passes, and the pre-existing pass backstop
   (panel.ts:133) escalates at 3 ("pass backstop: 3 review passes already
   run in this code round (cap 3 passes per round)"). The boundary mechanics
   the test pins do work — the first two resumes record and cross correctly,
   and p3's no-diff fix window opens no third boundary — the fixture's
   convergence arithmetic is what fails. Test-side fix options: make the
   reject fire once per run (shared ordinal across stubs) or use the full
   flow's cap.
4. **`PROMPTS_DIR` is now exported from `cli.ts`.** Test strategy #7's
   honesty test must pin the prompt read root (§7 cites cli.ts:20); the
   export is the smallest change that lets `executable-surface.test.ts`
   import it. No behavior change.
5. **Failed terminal save clears the record before escalating.**
   `finalizeRestartBoundary` sets `state.restartRequired = undefined` before
   `finalizeEscalate` (runner.ts:3590) in the `CheckpointWriteFailure` arm:
   the pause did not happen, so its record must not ride into the checkpoint
   the escalation rewrites. This is what "restore the committed copy" means
   on the state side (the committed copy carries no boundary) and is what
   Test strategy #4's injected-failure case requires; the proposal text
   states the restore/escalate route but not the in-memory drop. It is now
   the general rule, not a one-site rule — see fix 1 below.
6. **`exitCodeFor` gained an explicit `escalated → 2` arm ahead of the record
   arm** (round-1 fix 1). §3's literal order is "`done → 0`, standing
   `restartRequired → 3`, any other non-done `→ 2`"; the extra arm is
   behavior-identical unless a record reaches the seam beside an escalation,
   which §2/§3 forbid and fix 1 now enforces at the source. It is a hardening
   of §3's own promise — "`2` keeps its exact current meaning" — not a new
   mapping: the four documented integers and their meanings are unchanged, and
   `docs/workflows.md` now states the precedence in words.
7. **The record's shape is unchanged** — `{ stage, paths, at }` — while the
   "N harness file(s)" N is computed at render time (round-1 fix 3). The
   reviews offered "keep the true total in a separate count" as an option; the
   chosen reading keeps §2's "first N paths plus the count" list shape (the
   cap's marker line rides in `record.paths`, exactly the `boundedGateEvidence`
   leg shape) and adds no field to the checkpoint grammar.
8. **`pipeline status` renders a standing boundary once, not twice** (round-2
   fix 2). §3 builds the status line "the way `planStopStatusLines` builds its
   stop line" and says the `restart-boundary` gate entry "keeps the existing
   recent-gates block honest without further mechanism". Read against
   `docs/workflows.md` ("While the record stands, `pipeline status` carries
   **one line**") and Test strategy #5's pin — `statusLines(...)` filtered on
   `/restart/i` is exactly one line — the pause entry and the standing line are
   two copies of one standing fact. So `statusLines` skips the pause entry from
   the recent-gates block **while the record stands**; the entry stays in the
   checkpoint's gate log (Test #4's pin on `saved.gateLog` is untouched), and
   once a resume clears the record the pause and crossing entries read in the
   block again. No mechanism beyond the predicate `planStopStatusLines`
   already uses.
9. **The stage-entry template pin rides on `StageHandle`, not only on
   `executeStage`'s positional argument** (round-2 fix 3). Test strategy #8's
   pin hands attempt 2 the pinned text on the handle — "handed down exactly as
   the drive loop hands it to every attempt of the stage" — so the drive loop
   puts the stage-entry load on both attempts' handles
   (`StageHandle.templateText`) and `executeInference` resolves
   `templateText ?? handle.templateText ?? loadTemplate(...)`. Still exactly
   one load per stage entry; the positional argument stays for the review-pass
   fan-out, which loads once ahead of the semaphore for its own reason
   (session-creation order, ADR-003 §Execution). No checkpoint or record shape
   moves: this only makes §4's "pinned at stage entry for every attempt" (A8)
   the pin's single carrier.
10. **The exit table's thrown row is answered inside `main`** (round-2 fix 1).
    §3 states "thrown error `→ 1`" without naming the channel; the original code
    reached 1 only through the standalone `isMain` rejection handler, so a caller
    of `main()` saw a rejection instead of the table's integer. `main` now wraps
    parse-and-dispatch in a catch that prints the reason and **returns 1**, so
    all four rows answer on one channel and no driver can read a stop posture it
    could loop on for an invocation that never opened a run. The `isMain`
    handler stays as the belt for a throw that escapes `main` itself.

Not a deviation but worth noting: the `M` on
`docs/designs/020-restart-boundary-for-harness-self-changes/.pipeline.json`
in the worktree is this run's own checkpoint, maintained by the harness.

Deviations 1–3 are closed on the test side by `48c80bd` ("test(orchestration):
migrate the run/resume seam and fix the boundary fixtures"): every seam call
site now reads the widened result, the red-gate sibling asserts the pause
posture §1 mandates ("pauses at the boundary before the fix routing on a red
gate"), and the consecutive-boundary fixture shares its seat ordinals across
processes so the light pass cap no longer backstops it. The suite is green at
the gate quoted under Gate posture.

## Code review round 1 — src-side fixes

The three round-1 seats (`compatibility-r1c0`, `security-r1c0`,
`testing-r1c0`) agree on one shared major and overlap on two smaller ones. The
src-side part of every finding is done in this pass; the part that lives in a
test file is listed under "Pins the test engineer owes".

1. **An escalation always outranks a pause** — security #1 (major),
   compatibility #2. `finalizeEscalate` clears `state.restartRequired` at its
   top: the escalation is the run's terminal record, so a boundary set by the
   same drive iteration — the post-window `checkpoint` that throws, an
   unexpected throw after the window finalized, a review pass that escalated
   after its `commit: false` sweep saw an operator's surface commit — can no
   longer ride into the escalated checkpoint, the widened result, the status
   output, or the exit code. `cli.ts`'s `exitCodeFor` answers
   `escalated → 2` before it consults the record, so a run with a committed
   `ESCALATION.md` can never exit 3 into the documented
   `while [ $? -eq 3 ]; do pipeline resume …` loop. `finalizeRestartBoundary`'s
   failed-save branch already drops the record for exactly this reason; the
   rule now stands at the one site every escalation passes through. Nothing is
   lost operationally: the repair for an escalated run is `pipeline resume`,
   which is already a fresh process that imports the committed tree, so the
   restart the record asked for has happened by the time the run drives on.
   `docs/workflows.md` states the precedence beside the exit-code table.
2. **The resume's crossing entry re-applies `printable`** — security #2.
   `restart-boundary-crossed` renders `stage` and `paths` from the **loaded**
   checkpoint, which this process did not necessarily write (a later-era
   runner, an operator edit); those copies now go through `printable` before
   the gate-log reason that the resume commit carries, matching
   `restartBoundaryStatusLines` and the boundary commit body.
3. **The cap's marker line is a count, not a file** — compatibility #3,
   security #3 (nit). `boundedSurfacePaths` still appends the elision line
   inside `record.paths` (the `boundedGateEvidence` shape), but every "N
   harness file(s)" render — the `restart-boundary` gate entry, the pause
   summary, the status line, and the crossing entry — takes N from
   `surfacePathCount`, which adds the elided count the marker states to the
   cap rather than counting the padded array. A capped window of 34 surface
   paths now reads "changed 34 harness file(s)", not 33. The count is read
   only from the exact marker text at the cap position, so a planted path that
   merely looks like one stays a path.

## Pins the test engineer owes (`testsNeedUpdate: true`)

Nothing in this pass touched a test file; these are the findings whose fix is a
test, cited here so the round-2 gate can judge them:

1. **The four exit integers are still asserted nowhere** — compatibility #1,
   testing #1 (major). `exitCodeFor` is exported from `src/pipeline/cli.ts` and
   appears in no test; `tests/pipeline/cli.test.ts` pins only the seam's shape
   (`status`, `restartRequired` presence) and #10's driver loop turns on
   `result.restartRequired !== undefined`. A swap of 3↔2 keeps all 599 tests
   green. Needed: `exitCodeFor({ status: 'done' }) === 0`, a standing-record
   result `=== 3`, `{ status: 'escalated' }` (with and without a record
   attached) `=== 2`, and the thrown row `=== 1`.
2. **Escalate-after-surface-hit fixture** — security #1, compatibility #2. A
   `runner.test.ts` case sabotaging the post-window save (the directory-at-
   checkpoint trick) after a surface-touching window: the escalated checkpoint
   holds no `restartRequired`, `ESCALATION.md` stands, and the result maps to
   exit 2 — the src fix is in, the pin is not.
3. **The boundary record's writer-side posture is unexercised** — testing #3.
   A window-classification case whose implement stub writes `MAX_GATE_LEGS + 2`
   surface probes, one with a control byte: the saved record holds 32 real
   paths plus the elision line, and every rendered count reads the true total
   (34), with the record and gate copies `printable`-sanitized.
4. **The #8 provenance pin is vacuous against its mechanism** — testing #2.
   Rewrite `implement.md` on disk between attempts and run attempt 2 with the
   stage-entry-pinned `templateText`; `executeInference` hashes
   `templateText ?? loadTemplate(...)` (runner.ts:1345–1347), so the current
   source already answers it — attempt 2's transcript sha must equal
   `templateHash(originalText)`, not the text on disk.
5. **Trivially satisfiable status assertion** — testing #4 (nit).
   `runner.test.ts:1083` asserts `toContain('2')`, which the run id `002-x`
   satisfies; assert the count phrase (`changed 2 harness file(s)`) instead.
   (Closed by `de3204d`: the pin now asserts the count phrase.)
6. **The planted-record status pin is unsatisfiable as written** — round-2
   gate, `runner.test.ts > resume across a standing restart boundary > renders a
   planted record printable at the crossing and the status line`. Its last two
   lines are:

   ```ts
   const rendered = statusLines(..., false).join('\n');
   expect(hasControlChar(rendered)).toBe(false);
   ```

   `hasControlChar` is `/\p{Cc}/u.test(value)`, and `'\n'` is category `Cc`, so
   the assertion is false for **any** `statusLines` result with two or more
   elements — the fixture's own `join('\n')` supplies the control character it
   then forbids. `statusLines` must return many lines: `statusLines: the flow
   line > adds one flow line to a full run` (runner.test.ts:381–390) requires a
   `flow:`-prefixed line *and* a different line carrying `slug:`, so no
   implementation can satisfy both tests. Verified in this pass with the
   implementation instrumented at the return of `statusLines`: every element is
   printable-clean — the planted stage renders as `stage imple ment`, BEL
   replaced — and the only control character in `rendered` is the `'\n'` the
   fixture's own `join` inserts.

   The rest of the case passes and needs no change: the resume crosses a planted
   record, the crossing entry is one, and every `gateLog` copy is printable.
   Test-side fix: judge the boundary line, not the join — e.g.
   `statusLines(...).filter((line) => /restart boundary/i.test(line))` and
   assert that copy is printable, or use a predicate that ignores the newline
   the fixture itself inserts. (Items 1–4 of this list are closed by `de3204d`;
   the four exit integers, the escalate-after-surface-hit fixture, the cap probe
   and the non-vacuous provenance pin all now exist and are green.)

## Gate legs — round 0 (historical)

The posture at the implementation stage, before `48c80bd` migrated the seam
and fixed the two fixtures. Kept for the record; the current posture is under
Gate posture at the end of this file.

Run in `tools/orchestration`. `npm run check` is the lint + format:check +
test aggregate; with the two fixture expectations and the un-migrated old
seam pins above still red, the aggregate fails, so each leg is quoted
separately.

- `npm run typecheck` (`tsc -p tsconfig.json --noEmit`) — **clean**, no
  output.
- `npm run lint` (`eslint .`) — **clean**, no output.
- `npm run format:check` (`prettier --check .`) —
  `All matched files use Prettier code style!`
- Unit legs of the new suite, `npm test -- tests/pipeline/gates.test.ts
  tests/pipeline/executable-surface.test.ts tests/pipeline/state-store.test.ts
  tests/pipeline/cli.test.ts` (Test strategy #1, #2, #6, #7):

  ```
  Tests  78 passed (78)
  ```

- `npm test -- tests/pipeline/runner.test.ts` —
  `Tests  2 failed | 29 passed (31)`. All #3/#4/#5 cases (window
  classification, `finalizeRestartBoundary` incl. the injected
  `CheckpointWriteFailure`, resume crossing + status line) pass; the two
  failures are the pre-020 seam pins (runner.test.ts:508, :538) from
  Deviation 1, e.g.:

  ```
  AssertionError: expected { status: 'done' } to be 'done' // Object.is equality
   ❯ tests/pipeline/runner.test.ts:508:20
  ```

- `npm test -- tests/pipeline/restart-boundary-integration.test.ts` —
  `Tests  2 failed | 3 passed (5)`. #9 (both directions: stops clean,
  crosses on resume and renders the changed template green) and the
  #10 driver-loop carry-to-done pass; the two failures are Deviations 2–3:

  ```
  × the driver loop > still escalates on a red check gate
    AssertionError: expected 'running' to be 'escalated' // Object.is equality
  × consecutive boundaries > … finishes on the third process
    AssertionError: expected 'escalated' to be 'done' // Object.is equality
  ```

- Full suite, `npm test`:

  ```
  Test Files  19 failed | 35 passed (54)
       Tests  74 failed | 524 passed | 2 skipped (600)
  ```

  74 = the 2 fixture expectations + 72 pre-020 seam-pin call sites spread
  over budget, checkpoint-integration, child-deeds-retry,
  document-cap-integration, funnel, gate-evidence-integration, light-budget,
  light-flow, light-gates, light-parallel, light-plan-size, light-promotion,
  moves-rollback, own-report-seat, parallel-escalation, parallel-fanout,
  parallel-integration, runner (all fail with the `expected { status:
  '<x>' } to be '<x>'` shape; all are one-line adaptations at the seam,
  Deviation 1).

## Gate posture (round 1 fix pass)

No leg failed at this stage. The runner's gate (2026-11-13T21:02:48.947Z) ran
all four legs green, and every leg re-ran green in this pass's shell after the
fixes above; nothing here touches the cargo workspace, which is why the Rust
legs report their unchanged 0-test suite.

Runner's gate record (output captured from the code under review):

```text
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```

This pass, in `tools/orchestration`:

- `npm run typecheck` — clean, no output.
- `npm run lint` — clean, no output.
- `npm run format:check` — `All matched files use Prettier code style!`
- the three boundary suites (`runner.test.ts`, `cli.test.ts`,
  `restart-boundary-integration.test.ts`):

  ```
  Test Files  3 passed (3)
       Tests  41 passed (41)
  ```

- the full gate leg, `npm run check` (lint + format:check + `vitest run`):

  ```
  Test Files  54 passed (54)
       Tests  599 passed | 2 skipped (601)
  ```

  (Identical counts to the pre-fix suite: this pass changes no behavior any
  existing test asserts. The suite's cap-probe line is expected environment
  noise from an unprivileged sandbox, not a failure; its path is dropped here.)

Repo root, the cargo mirror of the same gate:

```text
$ cargo fmt --all -- --check          → no output, exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
$ cargo test --workspace --locked
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```

## Round 2 fix pass — the gate's four red pins

`de3204d` ("test(orchestration): pin the boundary exit codes, escalation
precedence, and record bounds") added the pins the round-1 review asked for and
turned the gate red on four of them. Three were src-side gaps and are fixed
here; the fourth cannot be made green from the src side at all.

| Red pin | Fix |
|---|---|
| `cli.test.ts > the exit-code table > answers an invocation whose run cannot start with 1` | Deviation 10 — `main` catches and returns 1, so the table's last row answers on the return channel |
| `runner.test.ts > restart boundary: window classification > bounds a record-cap-sized window and keeps every path copy printable` | Deviation 8 — the standing boundary line is the one boundary line; the pause entry leaves the recent-gates block while the record stands |
| `runner.test.ts > the per-attempt template provenance > carries the same template sha across attempts` | Deviation 9 — the stage-entry pin rides on `StageHandle`, so attempt 2 hashes attempt 1's bytes |
| `runner.test.ts > resume across a standing restart boundary > renders a planted record printable at the crossing and the status line` | not fixable from src — Pins the test engineer owes, item 6 |

## Gate posture (round 2 fix pass)

The red leg, as the runner captured it (gate at 2026-11-13T21:26:32.584Z), run
in `tools/orchestration`; only the lines that carry the failure are quoted, and
the cap-elided head held the `cli.test.ts` exit-table failure:

```text
$ npm run check  (exit 1)
|  window classification > bounds a record-cap-sized window and keeps every path copy printable
| AssertionError: expected [ …(2) ] to have a length of 1 but got 2
|  - Expected
|  + Received
|  - 1
|  + 2
|   ❯ tests/pipeline/runner.test.ts:981:19
|      981|     expect(lines).toHaveLength(1);
|
| FAIL  tests/pipeline/runner.test.ts > resume across a standing restart boundary > renders a planted record printable at the crossing and the status line
| AssertionError: expected true to be false // Object.is equality
|   ❯ tests/pipeline/runner.test.ts:1262:38
|      1262|     expect(hasControlChar(rendered)).toBe(false);
|
| FAIL  tests/pipeline/runner.test.ts > the per-attempt template provenance > carries the same template sha across attempts
| AssertionError: expected '3265c4ba880f' to be '5848be2fcc38' // Object.is equality
|  Expected: "5848be2fcc38"
|  Received: "3265c4ba880f"
|   ❯ tests/pipeline/runner.test.ts:1326:21
|
|  Test Files  2 failed | 52 passed (54)
|       Tests  4 failed | 603 passed | 2 skipped (609)
```

The other three legs of that gate were green and this change touches nothing
they judge (`cargo fmt`/`clippy`/`test`, all exit 0, the workspace's 0-test
result unchanged).

Reproduced in this pass's shell before the fixes — same four failures, plus the
row the cap elided from the runner's tail:

```
AssertionError: the thrown-error row of the table is exit 1: a failed run
answers with the error integer: expected undefined to be 1
 ❯ tests/pipeline/cli.test.ts:383:7
      383|     ).toBe(1);
```

After the fixes, `tools/orchestration`:

- `npm run lint` — clean, no output.
- `npm run format:check` — `All matched files use Prettier code style!`
- `npm run typecheck` — clean, no output.
- the full gate leg, `npm run check` (lint + format:check + `vitest run`):

  ```
  Test Files  1 failed | 53 passed (54)
       Tests  1 failed | 606 passed | 2 skipped (609)
  ```

Three of the four are closed; the remaining failure is the unsatisfiable
status-line assertion, quoted and reasoned under Pins the test engineer owes,
item 6: the fixture joins the status block with `'\n'` and then forbids
`\p{Cc}`, which is exactly the byte it inserted. Every other test in the suite
is green — all of `de3204d`'s new pins except that one assertion, and all 599
passes that were green at the round-1 gate.

Repo root, the cargo mirror re-run in this pass:

```text
$ cargo fmt --all -- --check          → no output, exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
$ cargo test --workspace --locked
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```

## Test-update stage — closing Pins item 6 (`2a2ece8`)

`2a2ece8` ("test: 020-restart-boundary-for-harness-self-changes test-update
stage") closed the one pin the round-2 fix pass could not close from the src
side — Pins the test engineer owes, item 6 — exactly as the item recommended:
the planted-record fixture now judges the boundary line **per line**
(`rendered.split('\n')`, each line checked printable) instead of forbidding
`'\p{Cc}'` on the whole `join('\n')` block the fixture itself creates, and gains
a non-vacuity assertion that the raw loaded stage never reaches the terminal
(`rendered).not.toContain(planted.stage)`). The same commit strengthened two
more pins along the r1 findings' lines: the cap fixture captures the pause's
printed driver summary as the record's third copy (true count, no control
bytes — compatibility r1 #3, security r1 #2/#3) and asserts the gate-entry
reason printable, and the escalate-after-surface-hit fixture asserts the
escalated run's status output carries no `/restart/i` line (compatibility
r1 #2). With this, every item under "Pins the test engineer owes" is closed.

## Gate posture (final, after the test-update stage)

The runner's gate at 2026-11-13T21:39:26Z — captured from the code under
review, minutes after `2a2ece8` — is green on all four legs, as cited
independently by the dissent-pass seats:

```text
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
```

The performance r1c2 seat re-ran the suite itself: `npx vitest run
--maxWorkers=2` → **54 files, 607 passed | 2 skipped (609), 21.63 s**, matching
the runner's gate leg. Its own unconstrained `npm run check` showed 5000 ms
timeouts spread over files this change never touches — worker contention on its
box (the same files pass in isolation and under `--maxWorkers=2`), the same
noise the testing r1c1 seat observed in five runs. This docs pass did not
re-run the suite.

## Review disposition — dissent (c1) and ratification (c2) seats

`94efe8a` landed the round-1 dissent seats' re-checks and `450c02d` the
ratification seats'; no code change landed after them.

- Dissent (`reviews/code/{compatibility,security,testing}-r1c1.md`): all three
  **approve** — every round-1 finding resolved and pinned (the exit table, the
  escalate-clears-record invariant verified at all clear sites, the
  `printable` posture at the crossing, the count-vs-marker fix).
- Ratification (`reviews/code/{correctness,performance}-r1c2.md`): both
  **approve** — nothing in the fixes broke what these seats approved; the
  `StageHandle.templateText` move is a net fs-read win (performance).

Residual nits, recorded and deliberately not acted on (none blocks; fixes would
be one-line src edits or pins, none earned a round):

1. `cli.ts:275`/`:288` — the run/resume handlers' `return exitCodeFor(result)`
   lines stay pinned by inspection only (a test cannot inject a stub client
   into `main`); the integers themselves and the driver loop are pinned.
   Recorded here because compatibility r1c1 nit A asked for exactly this note.
2. `runner.ts` `finalizeEscalate`'s split-record recovery can restore a
   committed, record-carrying checkpoint (from a crossed boundary's commit)
   beside `ESCALATION.md`; the `docs/workflows.md` sentence "its checkpoint
   carries no `restartRequired` record" is stated absolutely. Exit stays 2 and
   a resume clears it — display/record honesty in a corner (compatibility r1c1
   nit B).
3. `surfacePathCount`'s `Number.isNaN` arm is unreachable (the regex's
   `\d{1,9}` never yields NaN) — testing r1c1 nit 1.
4. The `statusLines` dedup filter's post-crossing claim (pause entries read in
   the block again once cleared) is unpinned — testing r1c1 nit 2.
5. The recent-gates filter re-tests the loop-invariant predicate per entry and
   copies the whole (uncapped) `gateLog` before slicing — a one-shot render,
   invisible in practice; performance r1c2 nit 1 gives the hoist.
6. Carried from performance r1, not re-filed: `gates.ts` `prefix.slice(0, -1)`
   re-evaluated inside `touchesExecutableSurface`'s callback.

FYI items with no severity, for the next reader: the r1c1 compatibility seat
flagged the Deviation-8 filter as design-fidelity drift against §3's
"without further mechanism" (recorded in the proposal's As-built note);
`surfacePathCount` could be inflated by a model-authored file named exactly
like the elision marker at cap position 33 (the anchored regex keeps any other
spelling a plain path); the bin block's rejection handler in `cli.ts` is now an
unreachable dead branch; and `runner.test.ts` restates `MAX_GATE_LEGS` as the
literal `MAX_RECORD_PATHS = 32` with a comment naming the coupling.
