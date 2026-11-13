# Code review — correctness (dissent pass c3)

Range: `git diff 945143568beb315686c133f53f30ff7b5ef02d78..HEAD -- ':(exclude)docs/designs'`,
read against `docs/designs/010-lightweight-plan-flow/02-proposal.md` (full current document)
and `docs/adrs/010-lightweight-plan-flow.md`. Checklist: `references/correctness.md`.
Inputs: the diff, the proposal, the ADR, this seat's round-1 findings. No shell was granted in
this environment, so `git diff` was reconstructed by reading the current files at the cited
lines, and no command (`prettier --check`, `vitest`, `npm run check`) was executed.

## Round-1 findings — checklist item 7

**Major (finding 1, the plan-stop bypass) — fixed, with one new open defect.**

The fix is the shape the finding asked for, and it is in the diff:

- `PipelineState.planStop` (`types.ts:239-250`) and `StateSchema.planStop: z.string().optional()`
  (`state-store.ts:71-75`) — additive-optional, ADR-006 §2, `version` untouched.
- The refusal records the stop: `recordPlanStop` (`runner.ts:2364-2367`) is the return value of
  both pre-commit refusals (`runner.ts:2785`, `2804-2807`), so the stop is on the checkpoint
  before `finalizeEscalate` saves it (`runner.ts:3181`).
- `decideNext`'s `case 'plan'` escalates on the record while it stands (`states.ts:124-139`), so
  a plain `pipeline resume` after a plan stop spends **zero** sessions — it re-escalates and
  never reaches `stage('implement')`. Pinned by `light-plan-size.test.ts:290-312`
  (`decideNext` → `escalate`, reason names promotion, second resume's `sessionsCreated` has no
  `implement`).
- The stop clears at the gate, in one author: `clearPlanStop` (`runner.ts:2376-2387`) is called
  from `preCommit` when `planOutgrewFlow` proceeds (`runner.ts:2788-2790`), so
  `pipeline resume --stage plan <slug>` — CLI: `resume <slug>` + `--stage`
  (`cli.ts:140-146`, `223`), `plan` in `FLOWS.light.stages` — re-plans and the run finishes.
  Pinned by `light-plan-size.test.ts:319-331`, `light-gates.test.ts:352-362`.
- The record is single-voiced at the stop: `repairSteps` (`report.ts:269-299`) returns the
  promotion recipe and the in-flow re-plan for a plan stop and never the bare
  `pipeline resume <id>`; `finalizeEscalate` prints the same list on the terminal
  (`runner.ts:3230-3236`) and `escalationMarkdown` the same notes (`report.ts:379-384`).
  `light-plan-size.test.ts:273-282` and `escalation.test.ts:171-183` assert
  `not.toContain('pipeline resume 002-x')` for the plan stop, and keep the bare resume for a
  code-phase stop. The gate's own string already names the promotion, and the light suffix is
  gated on `implement`/`fix`/`code-review` (`runner.ts:3159-3166`), so the promotion is named
  once (`light-plan-size.test.ts:256-265`, `escalation.test.ts:158-164`).

The predicate that selects that record is the new defect below: it keys on the **stage** instead
of the **stop**, which makes the record's plan-stop claim fire for light runs that never hit the
plan gate, and suppresses the plain resume exactly where the state machine honors it.

**Nit (finding 2, `npm run check`) — still open, unchanged in kind.** See finding 2.

## Findings

### 1. major — the plan-stop record fires for light runs with no plan stop, and hides the repair they honor

Location: `tools/orchestration/src/pipeline/report.ts:247-249` (`hasPlanStop`), call sites
`report.ts:271` (`repairSteps`), `report.ts:382` (`escalationMarkdown`),
`runner.ts:3294-3301` (`statusLines`); producers `runner.ts:2818`, `2839`, `2458-2465`,
`2492-2500`, `runner.ts:2789`/`2376-2387`.

Problem: `hasPlanStop` is `flowOf(state) === 'light' && state.lastStage === 'plan'`. It ignores
`state.planStop`, the field the same revision added and documented as the stop's record:
`types.ts:247-248` says "Additive-optional … **its absence is 'no stop standing'**". The stage is
not the stop. Two shapes are reachable in the light flow and read as a plan stop:

- **A light `implement` stage that fails.** `state.lastStage = name` runs only *after*
  `runWithRetry` returns (`runner.ts:2810-2818`); every stage-failure escalation inside it
  (two failed replies, `runner.ts:2492-2500`; half-applied moves, `runner.ts:2457-2465`) calls
  `finalizeEscalate` and throws `StopDriving`, so `lastStage` stays `'plan'` — the value the
  *successful* plan stage persisted at `runner.ts:2839` — and `planStop` was cleared at
  `runner.ts:2789`. The record then prints "The plan stop is sticky: resuming this run as it
  stands re-escalates on this record, so the refused plan is never built from" and offers only
  `pipeline run <source> --force` and `pipeline resume --stage plan <id>`. Nothing was refused,
  and the repair the state machine honors is the omitted one: `planStop` absent means
  `states.ts:136-139` falls to `stage('implement')`, so plain `pipeline resume <slug>` retries
  the coder. The footer points the operator at a full-flow promotion and a wasted re-plan.
- **A healthy running light run.** After a plan that clears the cap, the checkpoint carries
  `lastStage: 'plan'`, no `planStop`, `status: 'running'` (`runner.ts:2818`, `2839`), and the
  next stage is the 12-minute `implement`. `pipeline status` for that whole window prints
  `plan stop: the plan gate refused this plan — a plain resume re-escalates; the moves that
  respect it: pipeline run … --force   or   pipeline resume --stage plan 002-x`
  (`runner.ts:3294-3301`, with the fallback string at 3296). That is a false statement of the
  run's standing condition on a run that is not stopped, in a flow whose design goal is that the
  operator reads the effective state (`02-proposal.md` Reporting; ADR-010 §Status).

Both are new in this revision — `hasPlanStop` and the multi-step repair rendering are the
round-1 fix's own code. The design's requirement is met for the plan stop itself (Test strategy
#19), so this is not a regression of the fix; it is the fix's predicate being one notch too wide.

Fix: key the predicate on the stop, not the stage:

```ts
export function hasPlanStop(state: PipelineState): boolean {
  return flowOf(state) === 'light' && state.planStop !== undefined;
}
```

For the plan stop this is identical (the gate sets the field before the escalation's checkpoint
save), the two printed moves stay exactly as they are, `state.planStop ?? '…'` in
`statusLines:3296` loses its dead fallback, and the implement-failure record and the running
status fall back to the plain-resume shape the state machine answers. Then update the states the
report tests hand it — `escalation.test.ts:171-176` builds `{ lastStage: 'plan', flow: 'light' }`
with no `planStop`, i.e. a shape the gate never writes — to carry `planStop: planReason`, and add
the two pins this finding needs: (a) a light run escalated with `lastStage: 'plan'` and **no**
`planStop` prints the plain `pipeline resume <slug>` and no plan-stop claim; (b) a running light
state at `lastStage: 'plan'` with no `planStop` prints no `plan stop:` line.

### 2. nit — the package's `npm run check` chain (Test strategy #21) is unverified from this review's inputs

Location: `tools/orchestration/package.json:23`; `tools/orchestration/tests/pipeline/gates.test.ts:706-708`;
`tools/orchestration/tests/pipeline/runner.test.ts:395-397`.

Problem: unchanged in kind from round 1. This pass granted no shell, so `prettier --check .` and
the vitest run behind `npm run check`'s `&&` chain were never executed, and #21's "`npm run check`
stays green" is unverified rather than passed. The static reading gives no evidence of an open
defect: the two spots cited in round 1 read prettier-canonical at the repo's `printWidth 100`
(706-708 and 395-397 are exactly the wraps prettier emits for those calls), and every line above
100 columns in the reviewed `src/pipeline/**` and `tests/pipeline/**` files is an unbreakable
string/template literal (e.g. `report.ts:219`, `runner.ts:3164`, `flows.ts:70`,
`runner.test.ts:363`), which prettier leaves alone.

Fix: one run of `npm run check` in `tools/orchestration` (the test owner's path) confirming the
chain reaches vitest and exits 0, and record that run's output as the note's headline.

## Out of specialty (FYI, no severity)

`statusLines`'s plan-stop line has no test asserting its text anywhere in
`tools/orchestration/tests` (grep for `plan stop:` / `re-escalates` is empty) — the shape is
pinned only through `repairSteps` in the two report tests. Testing seat's call.

## Verdict

`request-changes` — one major open (finding 1). Round-1's major (the plan-stop bypass) and its
single-voice requirement are implemented and pinned; the remaining defect is the reporting
predicate that claims a plan stop for light runs that never reached the gate.
