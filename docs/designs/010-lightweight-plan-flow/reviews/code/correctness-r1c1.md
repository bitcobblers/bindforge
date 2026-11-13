# Code review — correctness, round 1 dissent pass c1

Scope: `git diff eb8b40ad413a397f71b977f6d0dcd6c903f0d17c..HEAD -- ':(exclude)docs/designs'`,
read against the current `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md` and
`03-build-notes.md`. Checklist: `references/correctness.md` (faithfulness to the design).

## Round-1 findings — re-check

| # | Round-1 finding | Status in this diff |
| --- | --- | --- |
| 1 | Full-flow budgets raised (`maxPassesPerRound: 8`, `stageTimeoutMs: 90 min`) | **Closed.** `stages.ts:107,117` are back to `6` / `30 * 60_000`; the light values live only in `FLOWS.light.config` (`flows.ts:85-86`); `docs/workflows.md:234,237,245,255` read 6 / 30 min; `flows.test.ts:141-142` pins the constants against `DEFAULT_CONFIG` (the pin my fix asked for). Deviation 11 records it. |
| 2 | Funnel pass-cap lift keyed on an unspent grant; grant never spent | **Closed.** `panel.ts:133` is `cycle >= cfg.maxPassesPerRound` in both flows and the reason names the cap it applied (`panel.ts:136-138`); the pass-entry cap check spends the grant in both flows (`runner.ts:2607-2625`, `state.extensionSessions = left - 1`) and the round-entry check does the same for rounds (`runner.ts:2532-2552`); the intake-time `operatorExtension` entry is gone (`runner.ts:2223-2226, 2283-2292`). Pinned by `light-budget.test.ts:276-358` (one grant buys one 2-seat pass, the printed count decreases, an unspent grant leaves no consent line). |
| 3 | Light test-path widening narrower than the design (`/(^|\/)tests\//`) | **Closed.** `gates.ts:387-394` — inside the `implement`/`fix` branch, reserved names first (`gates.ts:382`), then `if (flow === 'light') return true;`, so the widening is `TEST_PATH_RE`'s whole set, co-located `src/x.test.ts` included. Pinned at predicate level (`gates.test.ts:635-636`) and in a real sweep (`light-ownership.test.ts:89-100`, light keeps, full reverts). `TEST_OWNERSHIP_LIGHT` (`prompts.ts:22-23`) now matches the matrix. |
| 4 | `traversesDotSegments` added a new spine rule in both flows | **Closed.** The function and both assertions are gone; `gates.ts:366-369` states the design's argument (A28 + `moves.ts` Rule 1) and `gates.test.ts:700-701` pins the verdict of the spelling (`plan` on `docs/../src/x.ts` → `true` in both flows), matching #7/#18. Deviation "the predicate parses no path" records it. |
| 5 | Plan-size cap implemented twice (`sourceAdmissible` `'plan'` kind + `documentLines`) | **Closed.** `sourceAdmissible` is `kind: 'adr' | 'brief'` only (`gates.ts:93-124`), `documentLines` is deleted, and `proposalSize` (`runner.ts:180-198`) is the only line counter; the gate is `planOutgrewFlow` (`gates.ts:135-144`) fed the meter's own count, pinned by `light-plan-size.test.ts:203-215`. Deviation 15 records it. |
| 6 | Plan stop ran post-commit and failed open on a missing document | **Closed for position, closed for the empty-document case.** The gate is a `preCommit` (`runner.ts:2726-2754`) invoked between the inference phase and the commit window (`executeStage`, `runner.ts:1878-1889`), over `outcome.docLines` recorded by the meter at `runner.ts:1712-1745`; `docLines === undefined` denies with `plan document not written: …` (`runner.ts:2729-2737`), the run spends no `implement` session (`light-plan-size.test.ts:219-225`), and the refusal escalates without retry (`runner.ts:2384-2389`). Deviation 12 records the missing-document rule. The reporting consequence of the new stop is finding 2 below. |
| 7 | Registry names/signatures drifted from the proposal's contract | **Closed.** `flows.ts:16` exports `Flow = 'full' | 'light'`, `PipelineState.flow?: Flow` (`types.ts:238`), `flowOf`/`configFor`/`assertStageInFlow` carry `Flow`, `configFor(flow, base: Partial<PipelineConfig> = DEFAULT_CONFIG)` (`flows.ts:120-123`), `seatsFor` returns `readonly string[]` (`flows.ts:142`). The open arm survives only at the untrusted seams (`ParsedSource.flow`, `flowDef`, `stagePathAllowed`, `moves.ts`) as the notes state. `SeatPhase = Exclude<ReviewPhase, 'editorial'>` (deviation 20) and `FlowDef.path` (deviation 9) are declared. Residue: `isStageInFlow` is an extra export (listed in "What landed") and `--stage plan` is not forceable — finding 5. |
| 8 | Two frontmatter reads per entry | **Closed.** `runPipeline` parses once, pre-lock (`runner.ts:2176-2187`), and passes that `ParsedSource` into `sourceAdmissible(sourceAbs, id.kind, flow, parsed)` (`runner.ts:2240`); the gate reads only when handed no parse (`gates.ts:101-108`), which is where the unreadable-file denial stays. Deviation 14. |
| 9 | `maxRounds` binding, repurposed counter, printed unit | **Partly closed — and the declared remedy is incomplete: finding 1 (blocker).** The unit is now named (`runner.ts:3220-3227`, `report.ts:243-246`) and deviation 10 declares the repurposing. |

## Findings

### 1. Blocker — the light flow's `fix` loop is unbounded on the red-tests path, and bounded at one round on the dissent path

`tools/orchestration/src/pipeline/states.ts:191,197` · `tools/orchestration/src/pipeline/runner.ts:2560,2675-2679,2711-2715` ·
`03-build-notes.md:115-124` (deviation 10)

Deviation 10 states: "The binding stop is `state.codeRound > cfg.maxRounds` in `states.ts` —
behaviorally the proposal's 2-round fix loop." That is true only for the loop that runs
**through** `code-review` (`states.ts:206`). The fix loop entered from a red `testsGreen`
gate is bound by a different test — `budgetLeft(state, state.codeBudgetUsed, cfg)` at
`states.ts:191` (`case 'implement'`) and `states.ts:197` (`case 'fix'`), i.e.
`codeBudgetUsed < cfg.maxRounds` — and this diff repurposed exactly that counter:
the per-`fix` charge is skipped for light (`runner.ts:2711-2715`), the round-entry charge is
skipped (`runner.ts:2560`), and `codeBudgetUsed` is now incremented by the seats a code pass
launched (`runner.ts:2677-2679`). Two consequences, neither intended by the design:

- **Unbounded loop.** Light `implement` with red tests and no code review yet:
  `codeBudgetUsed` is `0` and stays `0` (no code pass has run), so `budgetLeft` is
  `0 < 2` → `stage('fix')` forever. `fix` replies ok, `runChecks` is red
  (`runner.ts:2775-2782`), `decideNext` answers `fix` again. Nothing else caps it: the
  pass-entry session cap (`runner.ts:2604`) gates review passes only, the pass backstop
  needs a round, `state.codeRound` never moves, `stageFailures` counts stage *failures*
  only, and a fix that replies `ok` is not one. The run spends `fix` sessions
  unboundedly (12 min each) with no escalation — the flow's other deterministic stop is
  disabled on this path, and the design's control for a too-big light run
  (Goals, `02-proposal.md:28`; Risks row 2) is gone.
- **The design's bound is over-tightened where it does bind.** After the first 2-seat
  code pass, `codeBudgetUsed` is `2`, so `budgetLeft` is `2 < 2` → the *first* fix on
  red tests escalates with `tests not green at the code-review budget limit`. The Goals
  ask for "`fix` bounded at 2 rounds" (`02-proposal.md:19-20`); the code gives 1 round
  after review and ∞ before it.
- The repurposed counter also adds no information: `statusLines` prints
  `code 6 sessions` from `codeBudgetUsed` **and** `code 6 sessions` from
  `reviewSessionsUsed(state, 'code')` on the same line (`runner.ts:3227-3231`).

Nothing pins either shape: `states.test.ts:403-419` asserts the *routing* (red → `fix`) and
`light-budget.test.ts` drives only the green path (`package.json` check exits 0).

Fix: keep the budget test in the unit the design states it in. Charge the light flow's
mutating loop the way the full flow's is charged — a round/fix counter that
`states.ts:191,197` and `states.ts:226` read — and print the session count from
`reviewSessionsUsed(state, 'code')`, which is already what the status/report lines do.
Concretely: restore `state.codeBudgetUsed += 1` at `runner.ts:2714` and `runner.ts:2560` for
light, drop the seats-charge at `runner.ts:2677-2679`, and name the unit from
`reviewSessionsUsed`. Then pin both shapes: a light run whose tests stay red from
`implement` escalates (not loops) with the round-limit record, and a light dissent gets its
second `fix` round before the escalation. Update deviation 10 to state which check binds on
which path.

### 2. Major — a real plan stop reports the **design** phase with an empty stage name

`tools/orchestration/src/pipeline/runner.ts:2384-2389,2755-2763` ·
`tools/orchestration/src/pipeline/report.ts:54-62`

The design adds `plan` to `CODE_STAGES` "for reporting only" so a light run stopped at
`plan` is not reported as a design-phase failure it cannot have (A15, Reporting,
`02-proposal.md:306-310`), and Test strategy #10 pins `escalationPhase` is `code` for a
light run at `lastStage: 'plan'`. The unit test constructs that state by hand
(`escalation.test.ts:99-105,161`), but the run never reaches it: `state.lastStage = name`
is assigned *after* `runWithRetry` returns (`runner.ts:2763`), and the pre-commit refusal
throws `StopDriving` from inside it (`runner.ts:2384-2389`). So at `finalizeEscalate`
`state.lastStage` is still `''` → `escalationPhase` returns `design` → the plan stop's
ESCALATION.md opens `Run stopped after stage `` in the **design** phase`, and
`reportFindings` reads the design phase's round — the exact mislabel A15 exists to remove.
`light-plan-size.test.ts:176-197` asserts the size string and the promotion line, never the
phase line, so the gap is unpinned.

Fix: set the stage before the gate can stop the run — assign `state.lastStage = name` (or
`state.lastStage = handle.stage` in the `executeStage` refusal path, `runner.ts:1879-1888`)
before the pre-commit gate is consulted — and extend the `light-plan-size.test.ts` record
assertion to `ESCALATION.md` containing `in the **code** phase`.

### 3. Major — `identity.ts` changes brief-run identity allocation for both flows; the proposal says that machinery needs no change

`tools/orchestration/src/pipeline/identity.ts:67,93-115` · `03-build-notes.md:70-83` (deviation 4)

The proposal's Promotion section is explicit: "Light runs consume `NNN` from the same
counter as decisions (A17), and `nextNnn` and `identityClash` need no change"
(`02-proposal.md:322-324`). The diff adds `nnnOfExistingRun`: for every non-ADR source,
`deriveIdentity` now reads every `docs/designs/*/.pipeline.json`, matches
`source.path`, and pins that `NNN`. It is not flow-gated, so it changes the **full** flow's
entry behaviour for every brief that has a run — a plain re-run is now a clash requiring
`--force`/`--slug`/`--nnn`, which deviation 4 itself calls the "accepted side effect".
That contradicts the design's compatibility claim (`02-proposal.md:13`: "every existing
document and checkpoint behaves exactly as it does today") and is behavior the proposal
never asked for (checklist item 4). It is declared (deviation 4), so it is not an
undeclared deviation; it is scope creep in a shared entry path.

Fix: gate `nnnOfExistingRun` on the flow that needs it — resolve the source's `flow:` key
before `deriveIdentity` (the parse already exists at `runner.ts:2176-2181`) and apply the
checkpoint-identity rule only to a source whose flow is `light` (the promotion path
Test strategy #22 pins) — and keep the full flow's allocation exactly as A17 states. If
the general rule is wanted, land it as its own change with its own design entry.

### 4. Major — the package's own gate is red on landing, against Test strategy #21

`03-build-notes.md:12-17,292-303` · `02-proposal.md:441-443` (#21)

The notes state plainly that `npm run check` stops at `format:check` (two test files
committed unformatted) so the `&&` chain "never reaches vitest", while claiming
"The suite is green — 41 files, 473 passed / 2 skipped". #21 pins "the suite runs on linux
at the `engines` Node floor and `npm run check` stays green". A change that leaves
`npm run check` red lands in a repo whose `fix` stage is summoned by exactly that gate, and
the "suite is green" headline is not what the gate reports. The notes' own fix is one
command: `npx prettier --write tests/pipeline/gates.test.ts tests/pipeline/runner.test.ts`.

Fix: land that formatting (the test owner's stage owns the test paths), and restate the
headline as "`npm run check` green" once it is, so the note's opening claim and the gate
agree.

### 5. Minor — the flow registry's own `plan` stage is not forceable

`tools/orchestration/src/pipeline/stages.ts:187-192,203-210` · `tools/orchestration/src/pipeline/flows.ts:78`

`FlowDef.stages` is documented as the `--stage` gate (`02-proposal.md:103`), and
`assertStageInFlow` implements it, but `assertForceableStage` runs first against
`STAGE_NAMES`, which deliberately excludes `plan` (`LIGHT_ONLY_STAGES`). So
`pipeline run <light-doc> --stage plan` and `pipeline resume <light-slug> --stage plan`
are usage errors ("unknown --stage plan") even though `plan` is the light flow's first
stage — the operator cannot re-plan a run the plan stop ended, only resume into it.
`add`/`assertStageInFlow` alone gives the right answer: `plan` is in `FLOWS.light.stages`
and in no full-flow path, so the flow check is what keeps it out of full runs.

Fix: accept a stage that the flow's registry declares (e.g. admit names in
`FLOWS[flow].stages` in the `--stage` validation, keeping `STAGE_NAMES` as the full-flow
regression guard), and pin `assertStageInFlow('light', 'plan')` plus the CLI-level
acceptance.

### 6. Minor — the plan stop deletes the plan the operator is told to promote from

`tools/orchestration/src/pipeline/runner.ts:214-224,2751` · `03-build-notes.md:187-197` (deviation 17)

The design pins the stop at the stage-ok/pre-commit point so the oversized plan "is never
committed"; deviation 17 goes further and removes the document from the worktree
(`git restore` from `HEAD`, else `rmSync`). The stop's own record is
`… : set flow: full and re-run with --force`, and A26 sells promotion on the plan surviving
in branch history — with this rule the refused plan exists nowhere: not committed, not in
the worktree. The proposal never asks the runner to delete a stage's in-ownership product
(`docs/**` is `plan`'s allowed surface, and the ownership sweep is the design's only
revert mechanism).

Fix: keep the pre-commit posture (no commit) and leave the bytes in the worktree, unstaged,
so the operator can read and promote from them — the terminal `git add -A` risk is the
sweep's job (the refusal path already runs it at `runner.ts:1886`); if the deletion stays,
say so in the escalation string and in the Risks/Deferred row, and pin it.

### 7. Nit — deviation 3's stamp rule (a full-flow behavior change) has no pin

`tools/orchestration/src/pipeline/runner.ts:2567-2580` · `03-build-notes.md:57-69`

The rule "a `ratify` pass keeps the stamp its round entry set" is shared by both flows and
is the mechanism deviation 11 credits with fixing run 010's near-miss; the note asserts
"Full-flow code-phase tests are unaffected", and no full-flow test exercises the ratify
stamp (`codeStamp` appears in tests only in fixtures and hand-built panel/states states).
The rule is right (#13 needs it, and the design phase's blob-id stamp behaves the same way),
so this is a pin request, not a mechanism request: one full-flow pass sequence
(full → dissent → fix → ratify) asserting the ratify pass does not move `codeStamp`.

## Checklist notes

- Item 1 (components/wiring): `flows.ts`, `FLOWS`, `flowOf`, `configFor` + validation,
  `seatsFor` with the five readers threaded (`panel.ts`, `states.ts`, `report.ts`,
  `runner.ts`), `STAGE_BASES.plan`, `prompts/plan.md` (three-line pointer,
  `{{planMaxLines}}` interpolated), `.opencode/agents/design/planner.md` (permissions
  byte-equal to `designer.md:4-22`, no `git diff*` allow), `.opencode/skills/plan-note/
  references/template.md` (five sections, no number), `{{testOwnership}}` in
  `implement.md:4` (and `fix.md`/`fix-tests.md`), `stageVars` gains `planMaxLines` and
  `testOwnership` (`runner.ts:2867-2880`), meter keyed on `docStages`
  (`runner.ts:1712`), `escalationPhase` reporting, one flow line in
  `statusLines`/`escalationMarkdown`, one suffix site (`runner.ts:3104-3111`),
  `moves.ts` threading (`moves.ts:157-166,299,313`) — present and wired.
- Item 2 (assumptions): A16/A27 restored (finding re-check 2); A28 restored (re-check 4);
  A15 honored by the code, broken by the run path (finding 2); A16's `maxRounds`
  meaning is what finding 1 is about.
- Item 3 (deviations declared): the eight round-1 deviations are all in
  `03-build-notes.md`; the undeclared residue I found is the `moves.ts`
  stage-in-flow rule (`moves.ts:162-164`, harmless: every full-flow stage is in
  `FLOWS.full.stages`) — noted, not raised.
- Item 6 (error paths fail closed): the plan stop fails closed, including the missing
  document; the light red-tests path does not (finding 1).
- Item 7 (round-1 blockers/majors): 1-8 addressed, 9 addressed only in part (finding 1).

FYI (out of specialty): `runReviewPass`'s ledger re-sort (`runner.ts:2127-2136`) lands
after `writer.drain()`, so a checkpoint written during the pass can carry the pre-sort
order until the next `checkpoint` call.

Verdict: request-changes (findings 1-4 open).
