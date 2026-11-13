# Code review — correctness seat — round 1, dissent pass c2

Scope reviewed: `git diff 96f50c2110560362e5fb61f05bd313f04bf503cb..HEAD -- ':(exclude)docs/designs'`,
read against `docs/designs/010-lightweight-plan-flow/02-proposal.md` (full current document) and
`docs/adrs/010-lightweight-plan-flow.md`. Inputs: that diff, the proposal, the ADR, and this seat's
round-1 findings. No other report or transcript was opened. `03-build-notes.md` sits under the
excluded `docs/designs/` tree, so deviation *declarations* were judged from the code's own
deviation references, not from the notes' prose.

Verdict: **request-changes** — 1 major open.

Note on method: this review environment grants no shell, so `npm run check` / `prettier --check`
could not be executed. Findings that depend on the gate's outcome are marked as unverified rather
than passed.

## Checklist

1. **Components named by the proposal** — present and wired: `flows.ts`
   (`FLOWS`, `flowOf`, `configFor` + validation, `seatsFor`, `assertStageInFlow`),
   `STAGE_BASES.plan` (`stages.ts:143`), `prompts/plan.md` (three-line pointer),
   `.opencode/agents/design/planner.md` (permission list byte-identical to
   `designer.md:4-22`, `git diff*` allow not copied), `.opencode/skills/plan-note/`
   with `references/template.md`, `{{planMaxLines}}` + `{{testOwnership}}` in
   `stageVars` (`runner.ts:2885-2886`) and in all three carrying templates
   (`implement.md:4`, `fix.md:4`, `fix-tests.md:7`), `planOutgrewFlow` (`gates.ts:135`),
   the `docStages` meter call site (`runner.ts:1695`), `plan` in `CODE_STAGES`
   for reporting (`report.ts:36-47`), the flow line in `statusLines`/
   `escalationMarkdown` (`runner.ts:3219-3221`, `report.ts:237-239`),
   `stagePathAllowed(stage, path, flow)` and `flowOf` threading into the sweep
   and moves (`runner.ts:296,1274`). Seat readers resolve through `seatsFor`
   with no stale `*_SPECIALTIES` import outside `flows.ts` and the design-sanctioned
   editorial one (`states.ts:177`).
2. **Assumption register** — honored on the paths this pass touched. A16/A27: the
   light fix loop is bounded by the per-`fix` charge read by `budgetLeft`
   (`states.ts:78-80`, `runner.ts:2712-2720`), and the light flow has no extension
   channel on that gate, declared in the code at `states.ts:70-76` (deviation 10).
   A17: `nextNnn`/`identityClash` untouched, the checkpoint-identity rule is
   flow-scoped (`identity.ts:72-73`). A11: `flow` is optional at `version: 1`.
3. **Deviations** — the code names deviation 10 (budget unit and the skipped
   light round-entry charge / extension), deviation 4 (light checkpoint identity),
   deviation 21 (registry as the whole `--stage` gate) at the sites that implement
   them, which is the naming this seat asked for. The proposal's "parses the source
   text once, right after `deriveIdentity`" (Selecting the flow) is now *before*
   `deriveIdentity` (`runner.ts:2165-2188`), forced by the flow-scoped identity
   rule; the code declares the reason in place. Behaviorally equivalent (one read,
   `sourceKind` shares the kind rule), so recorded here as an observation, not a finding.
4. **Scope creep** — the r1 finding 3 creep is gone: `nnnOfExistingRun` is reached
   only for a `light` source, and a full entry keeps A17's next-free number
   (`identity.ts:69-73`, pinned `light-promotion.test.ts:396-399`).
5. **Notes' claims** — not evaluable from the permitted inputs (excluded path).
6. **Error paths fail closed** — the plan stop fails closed on the missing document
   (`runner.ts:2734-2743`); the pre-lock flow validation, panel validation and
   `--stage` check all run before the lock, the directory, the checkpoint and the
   first session (`runner.ts:2167-2213`). One gap: the plan stop is not sticky on
   the resume path (finding A).
7. **Rounds** — r1 findings 1, 2, 3, 5, 6, 7 verified fixed with pins; finding 4
   not fully verifiable here (finding B).

## Findings

### A. major — the plan stop is bypassed by the repair command its own record prints

- **Location**: `tools/orchestration/src/pipeline/runner.ts:1868-1877` (new
  `state.lastStage = stageName` in the pre-commit refusal),
  `tools/orchestration/src/pipeline/states.ts:118-124` (`case 'plan'` → `stage('implement')`),
  `tools/orchestration/src/pipeline/report.ts:301-307` (footer prints
  `pipeline resume <nnn>-<slug>`), read with `runner.ts:2731-2760`.
- **Problem**: the round-1 fix makes the plan stop's record accurate —
  `lastStage` is now `plan` when the size gate refuses — and that moves the
  checkpoint's resume landing point. A light run stopped by the size gate carries
  `lastStage: 'plan'`, `flow: 'light'`, `codeBudgetUsed: 0`; the next
  `pipeline resume 002-x` takes `case 'plan'` → `stage('implement')`. So the
  command `ESCALATION.md` ends with ("Fix the underlying cause … then run:
  `pipeline resume 002-x`") walks straight past the flow's one deterministic plan
  stop: the 120-line plan the flow refused — now committed by the terminal
  escalation commit, which is the posture finding 6 asked for — becomes the
  design document `implement` builds against and the two seats check faithfulness
  against, with the implement session (12 min) spent and no cap downstream
  (`codeBudgetUsed` is 0, `codeRound` 0). Before the fix this shape re-planned on
  resume, because `lastStage` was `''` and `case ''` sent a light run to `plan`;
  the changed code makes the bypass real. The design's own remedy for this stop is
  promotion (`flow: full` + `--force`), so the record prints two instructions that
  lead to opposite outcomes, and the Risks row "the 80-line plan gate stops a plan
  that outgrew the flow" (row 2) is the mitigation this path drops.
- **Fix**: make the stop sticky and the record single-voiced. Either record the
  stop on the state (an additive optional field, ADR-006 §2 style) when the
  pre-commit gate refuses, and have `case 'plan'` escalate on the promotion record
  while it stands — cleared by the next `plan` pass that clears `planOutgrewFlow` —
  so a plain `resume` re-plans; and print the promotion recipe as that record's
  repair line (`escalationMarkdown` knows the gate reason) instead of the generic
  `pipeline resume`. Then pin the shape: a light run resumed after a plan stop
  never spends an `implement` session on the refused plan.

### B. nit — the format-gate fix is not verifiable from the permitted inputs

- **Location**: `tools/orchestration/tests/pipeline/gates.test.ts:706-708`,
  `tools/orchestration/tests/pipeline/runner.test.ts:385-398`; the claim under test
  (`npm run check` green, Test strategy #21) lives in the excluded
  `docs/designs/…/03-build-notes.md`.
- **Problem**: r1 finding 4 asked to land the formatting so the `&&` chain in
  `package.json:23` reaches `vitest`. The two cited spots now read
  prettier-canonical at the repo's `printWidth: 100` (the wrapped
  `expect(… ).toBe(\n  false,\n)` at `gates.test.ts:706-708` is what prettier
  emits for a 105-column call; `runner.test.ts:395-397` likewise), so there is no
  evidence of an open defect. This seat has no shell, so `prettier --check .` and
  the suite were not executed and the gate's greenness is unverified, not passed.
- **Fix**: one run of `npm run check` in the package (the test owner's paths) to
  confirm the chain reaches `vitest`, and restate the notes' headline as
  "`npm run check` green" from that run's output.

## Resolved from round 1 (verified against this diff, with pins)

1. **blocker** — the repurposed `codeBudgetUsed`. The per-`fix` charge is
   unconditional in both flows (`runner.ts:2712-2720`), the light round-entry
   charge stays skipped (`runner.ts:2566`) with the reason and the binding check
   named in place (`runner.ts:2527-2532`, `states.ts:63-80`), and the seats charge
   is gone (that site now carries the cycle counters, `runner.ts:2677-2681`).
   Consequence 1 is closed and pinned: a light run whose tests stay red from
   `implement` spends exactly `plan, implement, fix, fix` and escalates
   `tests not green at the code-review budget limit` with the light suffix
   (`light-budget.test.ts:306-326`). Consequence 2 is closed: the dissent loop gets
   its second `fix` round and the counter reads 2, not the 6 sessions
   (`light-budget.test.ts:232-236, 273-274`), and the printed unit follows the
   record — `rounds` from `codeBudgetUsed`, sessions from `reviewSessionsUsed`
   (`runner.ts:3231-3235`, `report.ts:243-245`, pinned `runner.test.ts:400-410`).
2. **major** — the plan stop's phase label. `lastStage` is assigned in the
   refusal path before the stop is reported (`runner.ts:1868-1877`), `plan` is in
   `CODE_STAGES` (`report.ts:36-47`), and the record's phase line is pinned in the
   integration test: `Run stopped after stage \`plan\`` + `in the **code** phase` +
   the negative design-phase assert (`light-plan-size.test.ts:194-196`).
3. **major** — identity scope creep. Gated on the flow that needs it
   (`identity.ts:47-76, 101-141`), full allocation untouched
   (`light-promotion.test.ts:396-399`), light re-entry pinned to the identity that
   holds the plan (`:382-391`), first light entry still takes the shared counter
   (`:415-425`), and `identityClash` reports the checkpoint and branch `--force`
   overrides (`:403-410`).
4. **major** — formatting. See finding B: canonical at the cited spots, gate not
   executable in this environment.
5. **minor** — `plan` forceability. The flow registry is the whole `--stage` gate
   at both pre-lock points (`runner.ts:2205-2213, 2270-2276`), and no `choices`
   list in `cli.ts:114,143` narrows it, so the runner check is the CLI-level
   acceptance: light `--stage plan` runs the plan on entry and on resume, and a
   full checkpoint refuses it pre-lock with nothing written and zero client calls
   (`light-gates.test.ts:332-379`, `flows.test.ts:229-239`). `full`'s accepted set
   is identical to what `assertForceableStage` accepted, so no full-flow
   regression.
6. **minor** — the refused plan survives. The gate keeps the pre-commit posture
   and leaves the bytes where the stage wrote them, with the "deleting the stage's
   own product is not a mechanism this design has" reason in place
   (`runner.ts:2752-2759`); the plan exists in the worktree, is tracked by the
   terminal escalation commit, and no `docs: plan` commit is minted
   (`light-plan-size.test.ts:202-208`).
7. **nit** — the shared ratify-stamp rule now has a full-flow regression guard:
   full → dissent → `fix` → ratify converges with every seat pinned to the one
   `codeStamp` (`funnel.test.ts:446-479`), the rule the light three-pass
   convergence also reads.

## FYI (no severity)

- `assertForceableStage` (`stages.ts:203-210`) is now referenced only from
  `budget.test.ts:153-162`; the production gate is `assertStageInFlow`. It is a
  live regression guard for `STAGE_NAMES`, so keeping it is defensible, but its
  docstring ("Validates a `--stage` value") now describes a test-only role.
- `testOwnershipLine(flow: string)` (`prompts.ts:26`) takes an open `string`; every
  caller passes a `Flow`, and any other value silently renders the full-flow line.
- The editorial pass's seat list is `specialtiesFor('editorial')` with the default
  config (`runner.ts:2492, 2502, 2515`), so a caller-supplied `reviewSeats.design`
  does not reach the editorial round. Self-consistent (the gate checks the seats
  that ran) and outside the light flow; not a finding.
