# Design: Restart boundary for harness self-changes (ADR-020)

## Context

[ADR-020](../../adrs/020-restart-boundary-for-harness-self-changes.md)
(status `proposed`) answers run 016's failure, described there from its gate
log: the long-lived runner imported its own modules at process start while a
stage commit rewrote the stage prompt templates under it — templates are read
from disk at stage time — and strict render turned that staleness into an
escalation the run's correct code could not repair (A1, A11). This design adds
to `tools/orchestration/src/pipeline/`: after each runner-minted commit window
the runner diffs the window against the harness's executable surface, records
a `restart-required` pause in the checkpoint, stops with a driver-visible
status, and lets `pipeline resume` — a fresh process that has already
imported the changed tree — carry the run on. The checkpoint grammar
([ADR-006](../006-self-update-checkpoint-compatibility/02-proposal.md)) is
kept: one additive-optional field, `version` still 1. Research input:
[01-research.md](./01-research.md).

## Goals / Non-goals

- Goals:
  - After every runner commit window, detect whether the window's net tree
    change touched what the runner executes or renders from — its imported
    sources and its stage-time templates (ADR-020 Decision 1).
  - Stop such a run as a clean pause: checkpoint record, boundary commit,
    gate-log entry and printed summary, with no escalation semantics and no
    attempt, round or session charged (Decision 2).
  - Give the driver a distinct exit code and `pipeline status` a line for a
    paused run, so restart-and-continue is routine automation (Decision 3).
  - Keep the prompt contract visible: the existing per-attempt
    `template @ sha` provenance is the record each stage's template revision
    is judged by, and it stays pinned by a test (Decision 4).
  - Ship the honesty test the ADR promises: a new executable location added
    without the surface rule turns the suite red.
- Non-goals (each prevents scope creep at code review):
  - The runner restarting itself, a boundary count per run, and
    `.opencode/agents/**` surface membership — each left undecided by ADR-020
    ("Not decided here"); see Deferred.
  - Hot-reloading the runner's own modules — rejected by ADR-020; the
    checkpoint plus `resume` is the restart primitive.
  - Loosening strict render's defaults, or forbidding self-change runs from
    adding prompt variables — both rejected by ADR-020, not revisited.
  - A checkpoint schema version bump or migration ladder — the ADR-006 §2
    grammar stands untouched (A6); this change adds one optional field.
  - Multi-run concurrency or per-run worktrees — ADR-003's territory.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | Stage templates are read from disk at stage entry while the runner's own ES modules are fixed at process start: Node caches a loaded module for the process's lifetime, no documented eviction lever applies to ES modules, and the prompts dir resolves relative to the CLI module, so a fresh process from the run's worktree already imports and renders the committed tree | verified | prompts.ts:81–83; cli.ts:20; https://nodejs.org/api/modules.html#caching ; research "Platform notes" |
| A2 | Stage commits are runner-minted at three sites — `finalizeStage`, `finalizeReviewPass`, and the revert commits `enforceOwnership` can mint — and models never commit | verified | runner.ts:1848–1892, 1940–1961, 363–377; docs/workflows.md:109–123 |
| A3 | `stagePathAllowed` confines every stage except `implement`/`fix` to `docs/**` (test stages additionally to tests), so a surface-touching commit can only land from a coder window | verified | gates.ts:559–589 |
| A4 | The ownership sweep reverts out-of-ownership paths, and its revert commits sit inside the same stage window, so a failed attempt to change the harness leaves no net surface diff | verified | runner.ts:292–379 |
| A5 | `RunStatus` is exactly `running`/`done`/`escalated` in the type and the checkpoint schema, and the CLI maps `done→0`, every other status `→2`, thrown errors `→1` | verified | types.ts:202; state-store.ts:97; cli.ts:17, 214, 227 |
| A6 | The checkpoint grammar admits evolution only as new optional fields with behavior-safe defaults; no field is ever removed, retyped, renamed or re-interpreted, and `version` stays 1 | verified | state-store.ts:58–79 |
| A7 | Zod object schemas strip unrecognized keys by default, but `loadState` casts the raw parse, so a pre-020 reader of a checkpoint carrying `restartRequired` loads it with the key inert to every state consumer; the key round-trips through its later saves | verified | https://zod.dev/api (Objects) ; state-store.ts:80, 239–256 |
| A8 | Every attempt's transcript already names `- template: <file> @ <sha12>`, hashed from the exact text rendered, and the template is pinned at stage entry for all attempts of that stage | verified | prompts.ts:85–87; runner.ts:2453–2467, 1343–1345 |
| A9 | The run lock releases on every exit path (`finally`) and is re-asserted each drive-loop iteration, so a fresh `resume` immediately after a pause is safe | verified | runner.ts:2263, 2298, 2325, 2392, 2548–2554 |
| A10 | `planStop` is the in-tree shape for a sticky, record-driven stop: an optional companion field, `status` untouched, cleared by the repair that answers it | verified | types.ts:290–301; states.ts:136–138; runner.ts:2406–2429 |
| A11 | Run 016's incident timeline (runner imported 12:38, escalated 13:57 on `{{gateEvidence}}`) | unverified | docs/adrs/020-restart-boundary-for-harness-self-changes.md:16–32 — the ADR's own text; run 016's records were not opened |
| A12 | A boundary costs one extra process start — lock create, checkpoint read, branch ensure — where a stage window costs minutes (30 min full / 12 min light timeouts), i.e. seconds against the escalation it replaces | unverified | runner.ts:2263–2277 (mechanism); stages.ts:98–127; ADR:116–117 — mechanism verifiable in code, the magnitude never measured |
| A13 | Whether the OpenCode server re-reads the agent permission files under `.opencode/agents/` per session was not established in the read set, so their staleness behavior is unknown | unverified | research Open question 6; docs/adrs/020-restart-boundary-for-harness-self-changes.md:119–123 |

## Options considered

**A new `RunStatus` member `restart-required`.** Rejected: the grammar's
evolution rule forbids retyping an existing field (A6), and widening the
`status` enum is exactly that — every consumer of `status`, including the
split-record predicate (A5), changes with it, and a pre-020 reader would
refuse the checkpoint instead of tolerating one unknown key (A7). `planStop`
is the established companion-field shape for a sticky record-driven stop
(A10), and it keeps the pause out of the status enum.

**A marker file beside the checkpoint (a `RESTART.md` sibling of
`ESCALATION.md`).** Rejected: a second terminal record re-creates exactly the
split-record class ADR-006 §3 exists for — marker truth against checkpoint
truth — and the pause would need fence rules, resume deletion, and status
plumbing for a fact the checkpoint already carries. The checkpoint record is
the only marker; the ADR's "marker" is its record, printed summary, and
commit, not a new file.

**Per-commit hooks at all three commit sites.** Rejected in favour of one
net-diff check per window: the three sites (A2) would drift as the harness
grows, while §1's net-diff listing, taken once in each finalize window,
already sees the stage/pass commit and the ownership-revert commits
together, and nets a reverted attempt's changes out (A4).

**The runner relaunches itself instead of exiting.** Considered, not rejected
on merit, deferred: ADR-020 leaves it undecided, and v1's pause-and-exit
matches the watch-and-restart posture the research found dominant (driver
relaunches; systemd/Kubernetes replace whole processes rather than mutating
one) and needs nothing the lock and checkpoint do not already give (A9).

## Chosen design

### 1. The executable surface, and where the boundary is checked — Decision 1

Canonical statement of the rule: `touchesExecutableSurface(path)` in
`gates.ts`, beside `stagePathAllowed`, returns true for a path under
`tools/orchestration/src/` or `tools/orchestration/prompts/`. Directory scope,
not a static file list: a module or template created inside either directory
is executable surface the moment it exists, and every imported pipeline
module — `flows.ts`, `stages.ts` and the rest — falls under the first prefix
for free. Agent permission files are not surface in v1 (Deferred; A13).

Detection is the window's **net** diff. `finalizeStage` and
`finalizeReviewPass` (runner.ts:1848–1892, 1940–1961) already receive the
window's `headBefore` and mint its commit; each, after recording the commit,
runs `git diff -z --no-renames --name-only <headBefore>..HEAD` through the
existing `nulPaths` parser, filters it through the predicate, and on a
non-empty hit sets
`state.restartRequired = { stage, paths, at }` (§2). That single check covers
all three runner commit sites (A2): the stage/pass commit is the window's net
effect, and revert commits `enforceOwnership` mints cancel their cause inside
the same range, so a stage that tried to change the harness outside its
ownership and was swept leaves no hit (A3, A4).

The drive loop then refuses to open another window: at the top of `drive`'s
`for (;;)` body, before `decideNext` runs (runner.ts:2440–2441), a standing
record calls `finalizeRestartBoundary` (§2) and returns. `states.ts` gains no
transition — the pause is recorded state, not a state-graph edge; unlike the
plan stop, it never reaches `decideNext`, because every resume clears it
before driving (§2).

Which stages can open a boundary at all follows from the ownership matrix
(A3): only `implement`/`fix` can commit under the two prefixes, so the
boundary opens only after a coder window, and ordinary product runs — which
touch no `tools/orchestration` path — never see one, as ADR-020 requires.

The ADR's "test that fails when a new executable path is added" is Test
strategy #7.

### 2. The pause: one record, one boundary commit, one clean stop — Decision 2

Representation: `PipelineState` gains the optional companion field
`restartRequired?: { stage: string; paths: string[]; at: number }`
(`types.ts`, beside `planStop`), validated as an optional `z.object` in
`StateSchema`; `status` stays `running` and `version` stays 1. This is the
grammar's sanctioned move (A6) and the `planStop` shape (A10); a pre-020
reader loads the checkpoint with the unknown key inert (A7), and every
existing status consumer is unchanged (A5).

`finalizeRestartBoundary` (in `runner.ts`, beside `finalizeDone`) follows the
done path's terminal pattern (runner.ts:3377–3425): it gate-logs a
`restart-boundary` entry naming the stage and hit paths; saves the checkpoint
under the ADR-006 §3 terminal-save guard — a record that cannot be written
restores the committed copy and escalates, because a pause that cannot be
recorded must not masquerade as a clean stop; then mints a terminal
`chore: <nnn>-<slug> restart boundary` commit whose body names the changed
paths, so the committed checkpoint carries the record and the worktree is not
left dirty (the save-before-commit posture, runner.ts:3395–3396). It prints
the one-line summary §3 names. The paths are model-authorable, so every
copy of a path takes `printable` into the record, gate-log entry, commit
body and summary, and the record keeps the `boundedGateEvidence` bounding
(first N paths plus the count). Nothing else distinguishes it from a normal
stage end: no `ESCALATION.md`, no `stageFailures` increment, no round,
session or attempt charge — the pause reads the commit *after* the stage
completed green, and the next stage's charge lands once, in the resumed
process. The lock releases through the existing `finally` (A9), so the driver
can resume immediately.

Resume is the restart. The boundary commit is on the run's branch and
templates resolve from the runner's own module path, so any fresh process
started from the run's worktree already imports and renders the changed tree
(A1). `resumePipeline` — before it drives — gate-logs the boundary it is
crossing and clears the record, mirroring the split-record posture resume
already uses (runner.ts:2345–2378); because it clears at entry, a forced
resume (`--stage`) lands on the forced stage, not back at the boundary check.
Flow, budgets and the next stage read from the checkpoint, so the run picks
up at the same stage with new code in force: a stage that ran before the
boundary rendered the old template, the next stage renders the new one with
the new renderer — run 016's `{{gateEvidence}}` class goes green.

### 3. What the driver sees — Decision 3

- Exit code, canonical statement: `runPipeline`/`resumePipeline` widen their
  return from `RunStatus` to `{ status, restartRequired? }`, and `cli.ts`'s
  run/resume handlers (cli.ts:201–228) map `done → 0`, standing
  `restartRequired → 3`, any other non-done `→ 2`, thrown error `→ 1`; the
  header comment (cli.ts:17) and `docs/workflows.md` carry the table. `2`
  keeps its exact current meaning — "read `ESCALATION.md`" — so no existing
  driver automates past an escalation by code collision; the research's
  FreeBSD `sysexits` note is why the code stays this simple and the record,
  not the integer, is the durable contract.
- The pause's own summary: the line that `finalizeRestartBoundary` prints names
  the stage, the changed paths, and the repair command
  (`pipeline resume <slug>`) — the "summary a driver logs".
- Status output: while the record stands, `pipeline status` gains one line,
  built the way `planStopStatusLines` builds its stop line (runner.ts:3580–3589):
  the stage that opened the boundary, how many harness files it changed, and the
  resume posture. The `restart-boundary` gate-log entry keeps the existing
  recent-gates block honest without further mechanism.

### 4. The prompt contract is pinned by what already ships — Decision 4

No new record. The two mechanisms Decision 4 asks for exist today: a stage's
template is loaded once at stage entry and pinned for every attempt
(runner.ts:2453–2467 — run 010 r8's lesson), and every transcript's
provenance names the template file and the sha of the exact text rendered
(A8). Across a boundary the pre-boundary transcripts carry the pre-boundary
sha, so a stage is judged against the template it was given because that is
what its own record names. The v1 obligation is only that this line stays
true, and Test strategy #8 pins it; if a later round wants the run record to
cross-reference boundaries, the §2 record already names the stage and paths.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A driver misreads exit 3 (treats a pause as an escalation, or ignores the difference) | Low — one new code, pinned by Test #6 and the driver-loop pin #10 | Low — resuming is the right next step for a pause, and the record says so in the run's own checkpoint | Distinct codes pinned both ways (pause→3, escalation→2); the checkpoint record, not the integer, is the contract (research prior art: sysexits' regret) |
| A `resume` launched from a different, older checkout imports pre-change code and drives on, the record inert to it (A7) | Low — needs a second checkout at an old revision | Medium — that run re-stales; that is today's status quo, not a regression this change worsens | The printed summary names the run's worktree as the place to relaunch; the boundary commit makes the pause visible in history after the fact |
| The surface rule drifts as the harness grows (a new importable location appears outside the two prefixes) | Medium over the harness's lifetime | Medium — a self-change there would restart nothing, reproducing the run-016 class | The ADR demands exactly this guard: Test #7 fails the suite on any import location or read root the rule does not name |
| Consecutive self-change windows cost a boundary round trip each | Common for self-change runs, zero for product runs (A3) | Low — one process start of seconds against minutes-long stage windows (A12) | Every boundary is gate-logged, so thrashing is visible in the record; a per-run cap is Deferred (ADR-020's "too large for a single pass") |
| A pause mid-review-round is mistaken for a pass failure by an operator reading the transcript | Low — the pause follows a green pass commit and the status line names it | Low | The boundary entry, commit and status line all state "not a failure"; Test #4 pins that no failure accounting moves |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux | yes | The environment the pipeline runs in today; the `tools/orchestration` suite exercises everything here, run locally — the repo has no CI workflow (same basis as ADR-006's matrix) |
| macOS | partial | Uses only primitives ADR-006 already treated as partially supported there (POSIX-flavored lock, plain git plumbing); this change adds nothing platform-specific but no macOS run exercises it |
| Windows | no (unverified) | Unchanged from ADR-006's row: untested, no CI workflow exists; the net-diff call inherits the same caveat as the lock |
| Node `>= 22.23.5` (engine floor) | yes | No new primitive: git plumbing, the existing checkpoint write, one zod field (package.json:30–31) |
| A pre-020 runner resumed onto a paused checkpoint | partial | It loads — the unknown key is inert to it and `version` is still 1 (A7) — but it drives on without seeing the pause; this change promises older processes can read newer checkpoints, never that they honour the new record |

## Test strategy

Unit (extending `tools/orchestration/tests/pipeline/`):

1. `gates.test.ts` — `touchesExecutableSurface`: true for
   `tools/orchestration/src/pipeline/runner.ts`,
   `tools/orchestration/src/pipeline/flows.ts`, and
   `tools/orchestration/prompts/implement.md`; false for `docs/**`,
   `crates/**`, `tools/orchestration/tests/**`,
   `tools/orchestration/package.json`, and `.opencode/agents/coder.md`.
2. `state-store.test.ts` — a checkpoint carrying
   `restartRequired: { stage, paths, at }` writes and reads back; a payload
   with an unrecognized sibling key parses and loads with it inert (A7);
   every era fixture in `fixtures/checkpoints/` still loads with no
   `restartRequired` key materialised.
3. `runner.test.ts` — window classification: a `finalizeStage` and a
   `finalizeReviewPass` window whose net diff touches a template or a src
   file set the record, naming the stage and the hit paths; a `docs/**`-only
   window does not; a window whose surface-path write was fully reverted by
   the sweep does not (the net rule, A4).
4. `runner.test.ts` — `finalizeRestartBoundary`: gate-logs
   `restart-boundary`, saves the checkpoint, and the boundary commit lands
   the record in the committed checkpoint; `ESCALATION.md` is absent,
   `stageFailures` and every budget counter are unmoved; an injected
   `CheckpointWriteFailure` restores the committed checkpoint and escalates
   instead of pausing.
5. `runner.test.ts` — `resumePipeline` with the record standing gate-logs the
   boundary, clears the field, and drives on; `statusLines` prints the
   boundary line while the record stands and no line when it is absent.
6. `cli.test.ts` — exit mapping end of the seam: `done→0`, a run ending with
   the record standing `→3`, an escalated run `→2`, a thrown error `→1`.
7. New `executable-surface.test.ts` — the ADR's honesty test: walking the
   import graph from the pipeline entry points (`src/pipeline/cli.ts`,
   `src/cli.ts`, `src/opencode-runner.ts`), every in-repo module reached is
   under `tools/orchestration/src/`, and the prompt read root (`PROMPTS_DIR`,
   cli.ts:20) resolves under `tools/orchestration/prompts/`; the test names
   both §1 prefixes, so a new importable location or stage-time read root
   fails the suite until the rule names it.
8. Provenance pin (Decision 4): a stage's transcript names
   `tools/orchestration/prompts/<file> @ <sha>` computed from the template
   text loaded at stage entry, and attempt 2 of the same stage carries the
   identical sha — the entry pin (runner.ts:2453–2467) is the record
   §4 relies on.

Integration (mock client, temp git repos, per `stage-lifecycle.test.ts` /
`parallel-integration.test.ts`):

9. Run 016's scenario end to end: a fixture run whose `implement` commit
   adds a template variable to a stage template and to the renderer in one
   window; the first process stops with exit 3 and the record naming the
   template path, no `ESCALATION.md`, and the boundary commit at HEAD;
   `pipeline resume` gate-logs the boundary, clears the record, and the next
   stage renders the new placeholder without escalating.
10. Driver loop: `run; while exit 3: resume` carries a self-change run to
    exit 0; the same fixture with a red check gate still exits 2 — the two
    stop kinds stay distinguishable in both directions.
11. Two consecutive self-change windows open two boundaries, each gate-logged
    and cleared by its own resume, and the run completes on the third
    process — v1's uncapped posture (Deferred names the cap's owner).

Platform: the `tools/orchestration` suite on Linux (no CI workflow exists in
this repo, so the suite is run locally — ADR-006's Platform basis) covers all
of the above; the macOS and Windows rows are not exercised in this run and
are recorded as such in the matrix.

## Rollout

1. One merged change in `tools/orchestration`: the `types.ts` field, the
   `StateSchema` field, the `gates.ts` predicate, the `runner.ts` hooks,
   `finalizeRestartBoundary`, the loop-top check and the resume clear, and
   the `cli.ts` mapping. `version` stays 1 and there is no migration: every
   existing checkpoint reads as "no boundary standing" (the same absence rule
   as `flow`, `planStop` and `gateEvidence`). Callers of `runPipeline`/
   `resumePipeline` adapt to the widened result at the seam.
2. Runs in flight at merge time are untouched until one of their stage
   commits touches the surface; the first such commit opens their boundary.
3. `docs/workflows.md` gains the operator section: the exit-code table
   (0/3/2/1), how a paused run reads in `pipeline status`, and the
   resume-is-the-restart posture — drivers relaunch from the run's worktree.
4. Acceptance: ADR-020 flips `proposed → accepted` when its own run 020
   completes, exactly as ADR-006's did; the design is then exercised by the
   first later self-change run that adds a prompt variable (the ADR-016
   class) and goes green across its boundary.

## Deferred / out of scope

- The runner restarting itself instead of pausing for the driver — ADR-020's
  "Not decided here"; v1's rule is pause-and-exit only.
- Whether the agent permission files under `.opencode/agents/` belong to the
  surface — ADR-020's "Not decided here" (their per-session re-read behaviour
  is unestablished, A13; a flow-config module under `src/**` is surface only
  incidentally, by §1's rule).
- How many boundaries one run may cross before the run is too large for a
  single pass — ADR-020's "Not decided here"; v1 counts them only in the
  gate log.
- Defaulting unknown template variables at the render boundary, and forbidding
  self-change runs from adding prompt variables — both rejected by ADR-020.
- Hot-reloading the runner's own modules — rejected by ADR-020 on mechanism.
- The lock-ID nonce / stale-lock race — already deferred by ADR-006;
  unchanged here.
- Windows and macOS verification of the lock and git posture — no follow-up
  yet; the matrix rows carry the status.

## Revision log

- r1 Chosen design 2 (research OQ1): decided — the pause is an optional
  companion field beside `status` (the `planStop` shape); a new `RunStatus`
  member rejected because it retypes `status`, the move ADR-006 §2 forbids.
- r2 Chosen design 2 (security r1): decided — boundary paths keep the
  writer-side posture: `printable` copies, `boundedGateEvidence` bounding.
- r2 Chosen design 1 (research commit-site-coverage risk; security r1):
  decided — one net-diff check per finalize window via `git diff -z
  --no-renames --name-only` through `nulPaths`, not per-commit hooks at
  three sites; revert pairs net out for free.
- r1 Chosen design 3 (research OQ3): decided — exit 3 for the pause with `2`
  untouched; the widened run/resume result carries the record to the CLI.
- r1 Chosen design 4 (research OQ7): decided — Decision 4 needs no new
  record; the existing per-attempt `template @ sha` line stands, pinned by
  Test #8.
- r1 Deferred (research OQ4/OQ5/OQ6): boundary cap, runner self-restart, and
  `.opencode/agents/**` surface membership deferred to ADR-020's
  "Not decided here".

## As-built (2026-11-13)

What run 020 actually shipped, recorded against this design — the design text
above stands as approved. Full detail, the deviation inventory, and the gate
records live in [03-build-notes.md](./03-build-notes.md) ("Deviations from the
proposal", review sections).

Shipped as designed: §1's `touchesExecutableSurface` in `gates.ts` with the two
prefixes and directory scope, one net-diff check per finalize window (`git diff
-z --no-renames --name-only` through `nulPaths`), the pause check at the top of
`drive`'s loop before `decideNext`, §2's `restartRequired?: { stage, paths, at
}` optional companion field (`status` untouched, `version` still 1), the
terminal-save guard, the `chore: <nnn>-<slug> restart boundary` commit, the
resume gate-log + clear, §3's four exit integers, and §4's unchanged provenance
line. Test strategy #1–#11 all shipped and are green at the final gate.

Drift, with reason (details and sites in the build notes):

- **§2/§3 — an escalation always outranks a pause.** `finalizeEscalate` clears
  `state.restartRequired` before its terminal save, and `exitCodeFor` gained an
  explicit `escalated → 2` arm ahead of the record arm; the failed-save arm of
  `finalizeRestartBoundary` likewise drops the in-memory record before
  escalating. Reason: code review r1 (security #1, compatibility #2) — a record
  set in the same drive iteration as an escalation would otherwise exit 3 and
  send the documented `while [ $? -eq 3 ]; do resume; done` driver loop past a
  committed `ESCALATION.md`. The four documented integers and their meanings
  are unchanged. Affected: Chosen design 2 (record lifecycle), Chosen design 3
  (mapping order); `docs/workflows.md` states the precedence.
- **§2 — the "first N paths plus the count" list keeps the count as a marker
  line inside `paths`**, not a separate number: `boundedSurfacePaths` caps at
  `MAX_GATE_LEGS` and appends an elision line (the `boundedGateEvidence` leg
  shape), and every "N harness file(s)" render reads N back through
  `surfacePathCount`. Reason: add no field to the checkpoint grammar beyond the
  one §2 sanctions; the reviews offered a count field and it was declined.
  Affected: Chosen design 2 (record shape — same `{ stage, paths, at }`).
- **§1/§3 — a coder window that finalized with a surface hit pauses even when
  its check gate went red**: the loop-top check pre-empts the fix-routing
  escalation, so that run exits 3 (pause), not 2. Reason: §1's stated check
  point ahead of `decideNext`, in a runner that finalizes the stage before
  routing a red gate. Affected: Chosen design 1/3 interaction; the
  round-0 integration fixture expecting escalation was retargeted on the test
  side (`48c80bd`) to pin the pause posture §1 mandates.
- **§3 — the standing boundary line is the only boundary line in `pipeline
  status`**: while the record stands, `statusLines` filters the
  `restart-boundary` pause entry out of the recent-gates block. This is
  "further mechanism" beyond §3's sentence that the gate-log entry keeps the
  block honest "without further mechanism" (also raised as a design-fidelity
  FYI by the compatibility r1c1 seat). Reason: Test strategy #5 pins exactly
  one boundary line; two copies of one standing fact was the round-2 red pin.
  Affected: Chosen design 3 (status output).
- **§4 — the stage-entry template pin rides on `StageHandle.templateText`**,
  and `executeInference` resolves `templateText ?? handle.templateText ??
  loadTemplate(...)`, so the non-vacuous #8 pin (template rewritten on disk
  between attempts) proves the entry pin. Mechanism only: still one load per
  stage entry, no record or checkpoint shape moves. Affected: Chosen design 4.
- **§3 — the thrown-error row answers on the return channel**: `main` wraps
  parse-and-dispatch in a catch that prints the `printable` reason and returns
  1, rather than leaving exit 1 to the bin block's rejection handler. Reason:
  §3 did not name the channel, and a caller of `main()` otherwise saw a
  rejection, not the table's integer. Affected: Chosen design 3.
- **Test-facing exports (no behavior change)**: `PROMPTS_DIR` from `cli.ts`
  (so §7's honesty test can pin the read root), `exitCodeFor` (so the table's
  integers are pinned), and the surface-prefix constants beside
  `touchesExecutableSurface` (so `executable-surface.test.ts` names both §1
  prefixes). Affected: Chosen design 1/3 surfaces only.
- **§2 naming detail**: the resume's gate-log entry is named
  `restart-boundary-crossed` (paired with `finalizeRestartBoundary`'s
  `restart-boundary` entry through one constant); §2 named only the pause
  entry.
