# Code review — correctness, r1 dissent pass c4

**Scope:** `git diff 8bad668511aa451580ae5bfee1e9be9e5f5195d6..HEAD -- ':(exclude)docs/designs'`,
read against the full `02-proposal.md` and `docs/adrs/010-lightweight-plan-flow.md`.
**Checklist:** `references/correctness.md` only.
**Inputs:** the diff, `02-proposal.md`, ADR-010, this seat's round-1 findings. No report or
transcript was opened. No shell was granted this pass, so nothing was executed.

**Verdict: approve.** No open blocker or major finding. My round-1 major is closed and pinned;
one new minor record-text defect and the round-1 nit (unrunnable checklist item) stay open.

---

## 1. Round-1 findings, checked first

### Major (round-1 #1, `hasPlanStop` keyed on the stage) — **closed**

The predicate is now keyed on the stop, exactly as asked:

- `report.ts:252-254` — `hasPlanStop(state) = flowOf(state) === 'light' && state.planStop !== undefined`.
  The stage no longer selects the record, and the doc comment (`report.ts:237-251`) states the
  reading that the field's own doc gives (`types.ts:248` — "its absence is 'no stop standing'").
- Both consumers go through it: `repairSteps` (`report.ts:281`) and `planStopStatusLines`
  (`runner.ts:3272`). `planStopStatusLines` now quotes `state.planStop` and returns `[]` when
  there is no stop, so the dead `?? '<stand-in>'` branch is gone, as the fix asked
  (`runner.ts:3271-3280`).
- Shape (a) from my finding — a light `implement` that fails with `lastStage` left at `'plan'`
  and no stop on the state — now prints the plain `pipeline resume <slug>` and no plan-stop
  claim. Pinned at `escalation.test.ts:199-209` (record prints the plain resume, no
  `/plan stop/i`, no `/re-escalat/i`, no `--stage plan`, no promotion line) and
  `runner.test.ts:454-470` (running light at `'plan'`, escalated light at `'plan'` with no stop,
  a full run, and an escalated light `implement` all print no plan-stop line, in both the
  marker-quiet and marker-present renderings).
- The states the report tests hand it now carry `planStop`: `escalation.test.ts:177`
  (`{ lastStage: 'plan', flow: 'light', planStop: planReason }`) and `runner.test.ts:478`.
  `escalation.test.ts:233-265` pins the predicate directly on all four shapes — recorded stop
  (true), stage alone (false), full flow with a stop on the state (false, the forged-checkpoint
  case).
- The gate that writes the record sets it before the escalation's save (`recordPlanStop`
  at `runner.ts:2365-2368`, called at `2786`/`2805` before `stopOnGate` → `finalizeEscalate`,
  which saves the checkpoint at `3182`), so the two printed moves are unchanged for the real
  stop. `clearPlanStop` (`runner.ts:2777-2791`) has exactly one author and clears on a plan
  that proceeds.

Two supporting pieces of the fix are present and correct:

- **The bypass is closed on the minted shape, not only on a hand-built one.** `executeStage`
  advances the stage before the stop is reported (`runner.ts:1868-1885`, `state.lastStage =
  stageName` at 1881), so the plan stop's own escalation lands with `lastStage: 'plan'`. That
  makes `decideNext`'s `case 'plan'` guard (`states.ts:136-138`) fire on the shape the gate
  actually writes — a fresh light run, `lastStage` `''` at entry — and it makes the record's
  "the plan stop is sticky … resuming this run as it stands re-escalates" sentence
  (`report.ts:295`) and status's "a plain resume re-escalates" (`runner.ts:3277`) true rather
  than aspirational. It also fixes the A15 mislabel on the minted record: the escalation reads
  the **code** phase, not **design**. Pinned end to end at `light-plan-size.test.ts:204-206`
  (`Run stopped after stage \`plan\``, `in the **code** phase`, not the design phase) and
  `290-312` (the saved state's `lastStage` is `'plan'`, `decideNext` answers `escalate` with
  the promotion reason, a plain resume spends no `implement` session, the second record says
  the same single thing).
- The in-flow repair is wired and pinned: `--stage plan` is flow-gated pre-lock on both entry
  paths (`runner.ts:2219`, `2282`), the re-plan clears the stop, and the run finishes
  (`light-plan-size.test.ts:319-331`). A full checkpoint's `--stage plan` is a pre-lock usage
  error with zero client calls and nothing written (`light-gates.test.ts:367-379`).

### Nit (round-1 #2, `npm run check` never run) — **still open, unevaluated**

No shell was granted this pass either, so `prettier --check .` and the vitest run behind
`npm run check`'s `&&` chain remain unexecuted by this seat, and Test strategy #21's
"`npm run check` stays green" is a claim I could not evaluate — which the skill counts as a
finding, not a pass. Carried as a nit below (the static reading is unchanged: no open defect
found in the reviewed files).

---

## 2. Checklist, this pass

1. **Components and interfaces named in the proposal exist, are wired, match names/signatures.**
   Walked: `flows.ts` (`Flow`, `FlowDef` incl. `stages`/`docStages`/`config`, `FLOWS`, `flowOf`,
   `configFor`, `seatsFor`, `assertStageInFlow`) — every declared symbol exists with the declared
   signature, the `light` row's stage list/`docStages`/config partial match the proposal's table
   (`flows.ts:73-90`), and `full`'s partial is empty (`:71`). `STAGE_BASES.plan` is
   `{ kind: 'proposal', agent: 'design/planner', template: 'plan.md' }` (`stages.ts:143`) as
   specified. `planOutgrewFlow(lines, cfg)` (`gates.ts:135-144`) produces the proposal's string
   (`(N lines, cap 80)`, with `: set flow: full and re-run with --force` appended by the runner
   at `runner.ts:2805-2808`). `stagePathAllowed(stage, path, flow)` (`gates.ts:371`) keeps
   `namesReservedSegment` first (`:382`) and widens exactly `implement`/`fix` (`:387-395`);
   `moves.ts:157-166` and the sweep (`runner.ts:303`) thread the same value from `state`.
   `plan` joins `CODE_STAGES` for reporting only (`report.ts:36-47`); the meter keys on
   `docStages` (`runner.ts:1701-1704`); `escalationPhase`/`statusLines`/`escalationMarkdown`
   resolve through `configFor(flowOf(state))` (`report.ts:323`, `runner.ts:3294`).
   `planner.md:4-22` is `designer.md:4-22` verbatim (the `doc-sync` `git diff*` allow is not
   part of it); `prompts/plan.md` is the three-line pointer naming `{{planMaxLines}}` and
   `{{designDir}}/02-proposal.md`; the `plan-note` skill carries the sections and no number
   (`SKILL.md:19-20` points at the prompt's number); `{{testOwnership}}` replaces the hard-coded
   rule in `implement.md:4`, `fix.md:4` and `fix-tests.md:7`, and no "Never edit test files"
   literal survives outside `prompts.ts:21-25`. Both light seat agents exist
   (`codereview/code-correctness.md`, `code-testing.md`), and reports land at
   `reviews/code/<seat>-r<round>c<pass>.md` via `reportPathFor` (`runner.ts:3046`), matching
   #12's path (`light-flow.test.ts:330-332`).
2. **Assumption register honored.** Spot-checked the rows the diff touches: A2/A3 (panel from
   config, additive optional fields — `stages.ts:71,77`), A4/A27 (three resolution seams all
   go through `configFor`; `--extend-budget` stays state-side, `runner.ts:2294-2302`, consumed
   at pass entry `2655-2673`), A6 (one meter read, its count rides to the gate on the outcome,
   `runner.ts:1732`), A8/A9/A28 (no path parsing; `flowDef(flow)` validated before any rule,
   `gates.ts:374`), A11 (`flow`/`planStop` are optional, `version` stays `1`, `state-store.ts:70-75`;
   legacy era fixtures plus `light-era.json` round-trip at `checkpoint-compat.test.ts:121-148`),
   A15 (see §1), A16 (session cap at pass entry ahead of the pass backstop, `2652-2654`;
   the light round-entry block is skipped, `2575-2609`, and the `fix` charge is the light loop's
   whole bound, `2754-2762`, read on both entry paths at `states.ts:214/220` and `248`).
   No assumption is silently changed.
3. **Deviations declared.** `03-build-notes.md` is in `docs/designs/` and is outside this
   pass's inputs, so the register itself is not checkable here. The diff's new
   behaviors that the proposal does not spell out — the sticky-stop `planStop` field,
   the `repairSteps` two-move plan-stop footer, the `plan stop:` status line, the light
   round-entry skip — are cited in-code as declared deviations (25, 29, 10, 21). FYI below.
4. **Scope creep.** Nothing in the diff does work the proposal did not ask for. The
   `planStop` machinery is the answer to this panel's r1 findings on the plan stop; it stays
   inside the flow's one deterministic stop and reaches no full-flow path (`preCommit` is set
   only for `plan`, `runner.ts:2775-2810`, and `assertStageInFlow` keeps `plan` out of `full`).
5. **Build-notes claims.** Not checkable within this pass's inputs (see 3).
6. **Error paths exist and fail closed.** The plan gate refuses on the over-cap plan, on the
   unmeasurable document (`runner.ts:2778-2786`), and clears only when the gate proceeds;
   it stops before the implement session (`light-plan-size.test.ts:245-251`) and skips the
   stage commit (`:218`). Pre-lock gates (`flow` value, `--stage` vs flow, caller's empty
   panel) throw before lock/directory/checkpoint/session (`runner.ts:2182-2186,2199,2219`;
   `light-gates.test.ts:367-379` asserts zero client calls). Pre-commit refusal sweeps
   ownership before the terminal commit (`runner.ts:1875-1884`).
7. **Previous round's blocker/major addressed.** Yes — §1.

---

## 3. Findings

### Minor — `pipeline status` tells a two-seat light run that five seats approved

**Location:** `tools/orchestration/src/pipeline/runner.ts:2740`

**Problem:** the converged-review gate-log line is hard-coded
`'all five seats approve the current revision stamp'`. This call site is one the diff
threaded (`planPass(state, phase, cfg)` at `2734`), and in the light flow the panel is
`['correctness', 'testing']`, so the run's own record — the `recent gates:` block
`statusLines` prints (`runner.ts:3321-3324`) — claims five approvals for a run that launched
two. It is reachable on the happy path of every light run
(`light-flow.test.ts:293-317` converges and lands this line in the checkpoint's gate log),
and it is the same class of record inaccuracy the design's Reporting section exists to
remove: the proposal makes the seat panel declarable data and requires the report to print
"exactly the flow's seats" (Reporting; `seatSummary` does). The neighbouring lines are
already flow-neutral — `runner.ts:2636` says "all seats already approve the current
revision" — and the pre-seat fallback at `2911` reads the flow's panel, so 2740 is the one
straggler. No state, budget, ownership, or control-flow effect.

**Fix:** drop the literal count from the line, e.g.
`` `all ${String(plan.seats.length)} seats approve the current revision stamp` `` (the
panel `planPass`/`seatsFor` resolved), or reuse the flow-neutral wording already used at
`runner.ts:2636`. Pin it on a light converged run (assert the `code-review` converged gate-log
reason names two seats, not five) so a future third flow cannot re-hard-code a number.

### Nit — Test strategy #21's "`npm run check` stays green" is unverified by this seat

**Location:** `tools/orchestration/package.json:23`;
`tools/orchestration/tests/pipeline/gates.test.ts:706-708`;
`tools/orchestration/tests/pipeline/runner.test.ts:395-397`

**Problem:** unchanged from round 1: no shell was granted this pass, so
`prettier --check .` and the vitest run behind the `&&` chain were never executed here, and
the checklist item (#21) stays unevaluated — a finding, not a pass. The static reading is
clean: the cited spots read prettier-canonical at the repo's `printWidth` 100, and every line
over 100 columns in the reviewed `src/pipeline`/`tests/pipeline` files is an unbreakable
string or template literal.

**Fix:** one `npm run check` run in `tools/orchestration` (the test owner's path) confirming
the chain reaches vitest and exits 0, and restate the build-notes headline as
"`npm run check` green" from that run's output.

---

## 4. FYI (no severity)

- Checklist item 3 (undeviated diff) is only half-checkable inside this seat's inputs: the
  deviation register lives in `03-build-notes.md`, which the caller excludes. The new
  behaviors the proposal does not spell out — `PipelineState.planStop` and the sticky-stop
  escalation, the two-move plan-stop repair footer, the `plan stop:` status line, the light
  round-entry budget skip, and `state.lastStage` advancing on a pre-commit refusal — are each
  cited in-code as declared deviations (25, 29, 10, 21) at the site that implements them;
  the register should carry them under those numbers.
- `repairSteps` prints `pipeline resume --stage plan <slug>` while `types.ts:245-246` shows the
  same repair as `pipeline resume <slug> --stage plan`. Both parse (yargs takes the option in
  either position); one spelling would read better in the record.
- A light plan file that exists and measures 0 lines passes the gate and goes on to
  `implement` (`light-plan-size.test.ts:372-379`). Consistent with the proposal, which fixes
  the cap as a ceiling and no floor; noting it because the two seats then review an empty
  `02-proposal.md`.
