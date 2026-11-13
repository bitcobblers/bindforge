# Code review — testing seat, round 1 pass c5 (ratification)

**Scope.** `git diff eb8b40ad413a397f71b977f6d0dcd6c903f0d17c..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/010-lightweight-plan-flow.md`.
Purpose of this pass: confirm the fixes that landed after the round-1 approval did not break what
this seat approved. Checks are the testing checklist only.

**What the diff's fixes touch (test-relevant).** `611d97c` gates/seats/budgets alignment,
`2c669fa` the light record, `96f50c2`+`dfaf35a` the round-1 pins, `f85324d` the bound `fix` loop,
`9451435` `--extend-budget` out of the light fix loop, `030aac8`/`8bad668`/`afbf898`/`746071c`
the sticky plan stop and its single-voiced record, `ae8afe6` its pins.

## Checklist

1. **Every numbered Test-strategy item has a citing test** — met. Grepping
   `// proves: Test strategy #` over `tools/orchestration/tests` returns a citation for each of
   #1–#23: registry shape and `full` regression guard (`flows.test.ts:47,60`), the light numbers
   and the operator lane (`flows.test.ts:84,98`), the additive-shape guarantee
   (`flows.test.ts:115,152`), `flowOf` plus the schema arms (`flows.test.ts:173`;
   `checkpoint-compat.test.ts:121,143` for `version: 1` round-trip and `flow: "sideways"`),
   the plan gate (`gates.test.ts:495,505,513` + `light-plan-size.test.ts:184,225,333`),
   frontmatter (`gates.test.ts:522-553`), the ownership matrix (`gates.test.ts:625-727` +
   `light-ownership.test.ts:70-145`), the light transitions (`states.test.ts:336-403`), the
   2-seat funnel (`panel.test.ts:268-351`), reporting (`runner.test.ts:358-425`,
   `escalation.test.ts:94`, `panel.test.ts:351`), prompts (`prompts.test.ts:451-526`),
   the end-to-end run (`light-flow.test.ts:291-360`), the funnel integration
   (`light-flow.test.ts:371-401`), budget shapes (a)/(b)/(c) (`light-budget.test.ts:216,255,277`),
   effective-config resume (`light-budget.test.ts:425,458`), checkpoint compatibility
   (`checkpoint-compat.test.ts:121`), pre-lock gates (`light-gates.test.ts:158-262`), the real
   sweep (`light-ownership.test.ts:70-145`), the oversized plan (`light-plan-size.test.ts:184-378`),
   the lane knob (`light-parallel.test.ts:172-205`), platform (`flows.test.ts:341`), promotion
   (`light-promotion.test.ts:251-412`), validation (`flows.test.ts:243-337`).
2. **Both layers where promised** — met. Registry/validation/gate/transition/reporting units sit in
   `flows`/`gates`/`states`/`panel`/`report`-facing suites; the integration layer is
   `light-flow`, `light-budget`, `light-gates`, `light-plan-size`, `light-ownership`,
   `light-parallel`, `light-promotion`, `checkpoint-compat`.
3. **Revision rounds** — met for this seat's scope. The fix-loop bound (my round-1 finding 3) is
   now pinned at its boundary and at integration depth: `light-budget.test.ts:235` pins exactly two
   `fix` charges and `codeBudgetUsed === 2`; `:306-326` pins the red-tests stop with
   `sessionsCreated === ['plan','implement','fix','fix']`; `:356-384,392-419` pin the fix-round
   record at the literal `(2/2)` **while a `--extend-budget` grant is outstanding**
   (`extensionSessions` left at 1, `FIX_ROUND_STOP` printed), which is exactly the arm
   `states.ts:79` (`flowOf(state) !== 'light' && …`) added. The sticky plan stop is pinned
   end-to-end (`light-plan-size.test.ts:290,319`) and at unit level
   (`escalation.test.ts:173,199,233`, `runner.test.ts:438-486`).
4. **Deterministic** — met. The light suites drive a stub `OpenCodeClient` with a monotonically
   incremented synthetic clock and temp git repos; grepping `light-*.test.ts` for
   `Date.now|setTimeout|Math.random|new Date` returns nothing.
5. **Names describe behavior** — met: `stops a panel that dissents every pass at 6 sessions on the
   pass backstop`, `keeps the plan-size stop recipe singular`, `refuses plan on a full checkpoint`.
6. **`npm run check` in the affected package** — **not evaluable by this seat** (see finding 1).
7. **Untestable additions** — met for the new code in this diff: `hasPlanStop` (three arms,
   `escalation.test.ts:233-264`), `repairSteps` (both branches, `escalation.test.ts:173,199`),
   `planStopStatusLines` (both, `runner.test.ts:454-486`), `budgetLeft`'s light arm (integration),
   `validatePanelPresence` (`flows.test.ts:308`), `flowDef`'s unknown-flow refusal
   (`gates.test.ts:727`, `moves.test.ts:501`), and the new `planStop` state field, which
   round-trips through the real checkpoint in `light-plan-size.test.ts:290-311`
   (written, re-read, and honored by a second `resumePipeline`).

## What the later fixes did not break

- The full flow's byte-equality guards survive: `flows.test.ts:115,135,152`,
  `runner.test.ts:370-398`, `gates.test.ts:437`, `states.test.ts:347` (`plan` has no
  full-flow transition), `escalation.test.ts:213` (a full record gains nothing).
- The reserved-name precedence rule is still the first rule asserted in the ownership matrix
  (`gates.test.ts:658-689`), and the accepted widening is still pinned as **allowed**
  (`gates.test.ts:646-656`, `light-ownership.test.ts:145`).
- The session-count claim for a converging light run is pinned exactly
  (`light-flow.test.ts:300-305` names the four sessions), so the ceiling at `:366` cannot hide a
  regression — my round-1 nit 6 is closed by that exact assertion.
- `decideNext`'s strings stay un-suffixed and byte-equal across flows (`states.test.ts:382-398`);
  the light suffix is asserted only on the record, as the design requires.

## Findings

1. **minor** — `tools/orchestration/tests/pipeline/` (checklist item 6). This seat has no shell
   access (every `shell` call, including `npm run check` in `tools/orchestration`, returns
   `Permission denied: shell`), so the suite-green claim (lint + `format:check` + test) is
   **unevaluated in this pass**, not verified. Per the checklist, an item that cannot be evaluated
   is a finding. Fix: record the tail of `npm run check` in `tools/orchestration` in the run's
   evidence (`03-build-notes.md` or the check runner output) so suite-green is a transcript.
2. **minor** — `tools/orchestration/tests/pipeline/gates.test.ts:661-672` (open from my round 1,
   inside this diff). The reserved-name loop lists 10 stages and omits the document stages
   `proposal-revise`, `proposal-final` and `editorial-review`, while Test strategy #7 and Risks
   row 3 require the precedence for *every stage class* in both flows. A regression letting a
   `proposal-revise` session write `docs/designs/…/.pipeline.json` passes the suite. Fix: add the
   three names to that `stages` array.
3. **minor** — `tools/orchestration/tests/pipeline/prompts.test.ts:467-474` (open from my round 1).
   The plan stage is pinned to `plan.md` and to the `design/planner` agent, and the plan-note
   template is pinned, but no test reads `.opencode/agents/design/planner.md`, whose content the
   design makes a contract (the `designer.md:4-22` permission list verbatim, and *not* `doc-sync`'s
   `git diff*` allow). A widened `edit "*"`, readable `transcripts/**`, a permitted shell, or the
   `git diff*` line arriving is caught only by a live light run. Fix: read `planner.md` in
   `prompts.test.ts`, assert its permissions block equals `designer.md:4-22`, and assert it
   contains no `git diff` and no `shell` allow.
4. **nit** — `tools/orchestration/tests/pipeline/flows.test.ts:323-327`. `validatePlanMaxLines`
   (`flows.ts:243`) rejects non-integers as well as values `<= 0`, and the proposal's contract is
   "absent or a positive integer", but the tests cover absence and `0` only. Fix: add
   `expect(() => configFor('light', { ...cliConfig(), planMaxLines: 12.5 })).toThrow(/planMaxLines/)`
   and a `NaN` case.
5. **nit** — `tools/orchestration/src/pipeline/report.ts:298,306` with
   `tools/orchestration/src/pipeline/runner.ts:3277`. The record now prints the operator-facing
   commands `pipeline run <source> --force` and `pipeline resume --stage plan <slug>`, and the tests
   assert those strings (`escalation.test.ts:180-182`, `runner.test.ts:484`) and, through the API,
   that a resume with `forceStage: 'plan'` re-plans (`light-gates.test.ts:352`,
   `light-plan-size.test.ts:319`). No test parses the printed argv through the exported CLI
   (`src/pipeline/cli.ts:105`, `resume <slug>` + `--stage`), so the new spelling's parseability is
   asserted nowhere: the suite pins the text, not that the text is a command. Fix: add a `cli` test
   that calls `main(['node','pipeline','resume','--stage','plan','002-x'])` and asserts the
   forced stage reaches `resumePipeline`.
6. **nit** — `tools/orchestration/src/pipeline/runner.ts:2377-2388`. `clearPlanStop` writes a
   gate-log record naming the stop it cleared; no test asserts that line
   (`grep 'cleared the standing plan stop'` over the suite is empty). The behavior is pinned
   (`light-plan-size.test.ts:319`), the operator-visible record is not. Fix: in that test, assert
   `gateLog` gains a proceeding `plan-size` entry naming the cleared stop.

## Verdict

**approve.** No blocker or major finding; the fixes that landed after the round-1 approval add
behavior that is tested at both layers, and none of the round-1 approvals — the full-flow
byte-equality guards, the reserved-name precedence, the funnel and session-count pins, the
un-suffixed `decideNext` strings — was loosened. The four items above are pre-existing gaps (2, 3,
4) and evidence/parseability nits (1, 5, 6), none of which is a break introduced by this diff.
