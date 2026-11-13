# Code review — testing — round 1, pass c0

Scope: `git diff 60faf8c24438ed55121757057b578182d2815083...HEAD -- ':(exclude)docs/designs'`.
Inputs: that diff, `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`.
Checklist: `.opencode/skills/code-review/references/testing.md` (items 1–7).
Prior findings for this seat: none (first pass).

Verdict: **approve** — 0 blocker, 0 major, 4 minor, 3 nit.

## Checklist walk

**1. Every numbered Test-strategy item has a citing test — PASS.**
All 23 items carry at least one `// proves: Test strategy #N` citation inside this diff:

| # | Cited at |
| --- | --- |
| 1 | `tests/pipeline/flows.test.ts:47,53,60,67,74` |
| 2 | `flows.test.ts:84,98,103` |
| 3 | `flows.test.ts:115,135` |
| 4 | `flows.test.ts:154`, `checkpoint-compat.test.ts:100,142` |
| 5 | `gates.test.ts:495,505,513` |
| 6 | `gates.test.ts:522,528,535,543,553,581`, `flows.test.ts:173` |
| 7 | `gates.test.ts:599,612,628,661,672,684,709,725,733,757`, `moves.test.ts:449,459,473,484,494` |
| 8 | `states.test.ts:293,302,311,324,336,357` |
| 9 | `panel.test.ts:268,283,297,313,323` |
| 10 | `panel.test.ts:336`, `escalation.test.ts:94`, `runner.test.ts:342,352,365`, `proposal-size.test.ts:366,383` |
| 11 | `prompts.test.ts:448,464,476,491,503`, `light-flow.test.ts:429,446` |
| 12 | `light-flow.test.ts:291,319,339,358` |
| 13 | `light-flow.test.ts:371,398` |
| 14 | `light-budget.test.ts:196,217,228,245,267`, `escalation.test.ts:143,156,166` |
| 15 | `light-budget.test.ts:288,320` |
| 16 | `checkpoint-compat.test.ts:100,121` + fixture `fixtures/checkpoints/light-era.json` |
| 17 | `light-gates.test.ts:158,178,193,218,240` |
| 18 | `light-ownership.test.ts:70,84,96,111,127` |
| 19 | `light-plan-size.test.ts:146,171,181,195` |
| 20 | `light-parallel.test.ts:172,191` |
| 21 | `flows.test.ts:247` |
| 22 | `light-promotion.test.ts:164,184,209` |
| 23 | `flows.test.ts:186,198,213,227,235` |

The assertions match the numbers the design fixes: light `stages`/`docStages`/seats
(`flows.test.ts:50,56,70`), the light budget set incl. `stageTimeoutMs = 12 * 60_000`
and `planMaxLines = 80` (`flows.test.ts:88-95`), `--max-parallel` surviving by spread
order (`flows.test.ts:88-100`), the shared knobs equal to `DEFAULT_CONFIG`
(`flows.test.ts:105-111`), the panel-scoping rule (`flows.test.ts:202-211`), the 6- and
4-session backstops (`light-budget.test.ts:204,236`), the 6/6 session-budget resume
(`light-budget.test.ts:263`), `extensionSessions` landing on state not config
(`light-budget.test.ts:278`).

**2. Both layers present where promised — PASS.** Unit: `flows.test.ts`,
`gates.test.ts`, `states.test.ts`, `panel.test.ts`, `prompts.test.ts`,
`checkpoint-compat.test.ts`, `moves.test.ts`, `escalation.test.ts`. Integration:
`light-flow.test.ts`, `light-budget.test.ts`, `light-gates.test.ts`,
`light-ownership.test.ts`, `light-plan-size.test.ts`, `light-parallel.test.ts`,
`light-promotion.test.ts` — each drives `runPipeline`/`resumePipeline`/`enforceOwnership`
against real tmp git repos and a stub `OpenCodeClient`.

**3. Revision-round tests — N/A.** No prior blocker/major was rejected against this
change; this seat has no earlier verdict.

**4. Determinism — PASS.** No `Date.now`, `new Date`, `setTimeout`, `fetch`,
`createServer`, `listen` or `Math.random` in any `light-*.test.ts` (grep over the new
files: zero hits). Stubs drive replies off a monotone fake counter (`clock`) and the
one-poll-after-prompt shape (`light-flow.test.ts:249-267`, `light-budget.test.ts:144-169`,
`light-plan-size.test.ts:103-129`, `light-parallel.test.ts:116-139`); no real server, no
port, no network. Every repo is a `mkdtempSync(tmpdir(), 'bindforge-light-…')` checkout,
so parallel files cannot collide; git is pinned with `-c user.name/-c user.email`.
`pollIntervalMs: 1` keeps the poll loop fake-clock-fast; the light `stageTimeoutMs`
(12 min) and `minNudgeIntervalMs` (20 s) can never fire in-suite, which is the safe
direction. No `.skip`/`.only`/`it.todo` anywhere in `tests/`.

**5. Names describe behavior — PASS.** Names state the operator-visible outcome
("stops a panel that dissents every pass at 6 sessions on the pass backstop",
"refuses a stage the checkpoint flow does not declare", "keeps a light plan write to
the decision record", "reverts a light plan write that spells docs/../src as src"),
not the steps taken to get there.

**6. `npm run check` — NOT EVALUATED (see finding 1).** This seat has no shell
permission: every `shell` call is answered `Permission denied: shell`, so I could not
run `npm run check` in `tools/orchestration` and cannot paste its tail. The checklist
names the runner as the independent verifier, and I did not find anything in the diff
that is likely to break it (no vitest config exists, so vitest 5 defaults apply, and
the new integration tests are the same shape and weight as the passing full-flow tests
in `funnel.test.ts:207`), but the item is unverified here, so it is recorded as a
finding, not a pass.

**7. Untestable additions — PASS with nits.** No knob without a consumer:
`planMaxLines` → `gates.ts:145` + `runner.ts:2621` + `stageVars` (`runner.ts:2748`),
`reviewSeats` → `seatsFor` (`flows.ts:126`), consumed by `panel.ts:36`,
`states.ts:31`, `runner.ts:3083`, `report.ts:238`; `FlowDef.path` →
`runner.ts:3083`/`report.ts:238`, asserted verbatim at `runner.test.ts:346` and
`proposal-size.test.ts:378`; `FlowDef.docStages` → `runner.ts:1660`, asserted at
`light-flow.test.ts:336`. The deferred `planMaxFiles` is genuinely absent (zero hits
across `tools/orchestration`). Every new flow branch is exercised: the light
`plan` case throw (`states.ts:113` → `states.test.ts:304`), the light converged
`done` (`states.ts:211`), the `!isLight` round charge skips (`runner.ts:2447,2450,2603`
→ `light-budget.test.ts:207`), the light extension arm (`panel.ts:135` →
`light-budget.test.ts:280`), the `!isLight` round-cap skip (`runner.ts:2420` — see
finding 3 for its boundary). One dead export is recorded as a nit (finding 7).

## Findings

### 1. minor — `tests/pipeline/` (checklist item 6): the gate's own green run is unverified in this pass
`npm run check` (`tools/orchestration/package.json:23` = lint + format:check + test)
could not be run from this seat: the `shell` tool is denied
(`Permission denied: shell`), so the checklist item "run `npm run check` in the
affected package yourself and paste the tail" has no evidence in this report.
**Fix:** have the run's own check runner (or the coder, in `03-build-notes.md`) record
the tail of `npm run check` from `tools/orchestration`, so the suite-green claim is a
transcript, not an assumption.

### 2. minor — `tools/orchestration/tests/pipeline/gates.test.ts:631-642`: the reserved-name matrix is not "every stage class in both flows"
The design's mitigation for its own top ownership risk is that reserved-name precedence
is "asserted for every stage class in **both** flows (Test strategy #7)"
(proposal Risks row 3). The both-flow loop lists 10 stages — `plan`, `implement`, `fix`,
`test-authoring`, `test-update`, `research`, `proposal`, `design-review`, `doc-sync`,
`code-review`. `proposal-revise`, `proposal-final` and `editorial-review` are document
stages in `FLOWS.full.stages` (`flows.ts:51-53,54`) and appear in no both-flow reserved
case; the older single-flow list (`gates.test.ts:339-350`) carries `editorial-review`
only, and the two proposal stages appear in no reserved-name case at all. A precedence
regression that lets a `proposal-revise` session write
`docs/designs/…/.pipeline.json` passes the suite.
**Fix:** add `proposal-revise`, `proposal-final` (and `editorial-review` to the
both-flow array) to the `stages` list at `gates.test.ts:631`, so the loop is the whole
`STAGE_NAMES` set for both flows.

### 3. minor — `tests/pipeline/states.test.ts:339` + `tests/pipeline/light-budget.test.ts`: the light `maxRounds: 2` fix bound has no boundary test
`maxRounds: 2` binds for light only through `states.ts:202` (`codeRound > cfg.maxRounds`),
because the runner's round-cap escalation is skipped for light (`runner.ts:2419-2420`)
and the round charge is skipped at `runner.ts:2447,2450,2603`. The only light assertion
of that stop is the unit case at `codeRound: 6` (`states.test.ts:343`), which proves the
branch exists, not that the registry's 2 is the number that stops it. In the integration
layer no light run reaches a second code round at all: `light-budget.test.ts` stops every
shape inside round 1 on the pass backstop (6/4 sessions) and
`light-flow.test.ts:374` spends one `fix`. So "fix bounded at 2 rounds" (Goals, ADR-010's
`fix, max 2 rounds`) is unproven at its boundary, and the `!isLight` skips are proven only
as "did not escalate", never as "the graph stopped it at round 3".
**Fix:** add a light case with a panel that dissents once per round (so each round ends
on a dissent and a `fix` runs) and assert the run escalates entering code round 3 on the
byte-equal fix-round string, plus a unit case at `codeRound: 3` with
`configFor('light', DEFAULT_CONFIG)` (and the mirror: round 2 proceeds).

### 4. minor — `tools/orchestration/tests/pipeline/prompts.test.ts:464`: the light `plan` agent's config file has no test
`stageDef('plan')` is asserted to name `design/planner` and `prompts/plan.md` is
asserted to render, but nothing in the suite reads `.opencode/agents/design/planner.md`.
The design makes that file's content a contract: "`design/designer`'s permission list
verbatim (`designer.md:4-22`, shape in A12) — copied, not re-derived, and
`ops/doc-sync`'s `git diff*` allow is not part of it" (proposal, The `plan` stage).
It is a 19-line permission block, and the suite already reads `.opencode/**` files by
path (`prompts.test.ts:242,293,509`), so the claim is checkable here. As it stands, a
missing planner file, a widened `edit "*"` allow, a readable `transcripts/**`, a
permitted `shell`, or the doc-sync `git diff*` line sneaking in is caught only by a live
light run failing at its first session.
**Fix:** in `prompts.test.ts`, read `.opencode/agents/design/planner.md` and
`.opencode/agents/design/designer.md`, assert the planner's `permissions:` block is
equal to designer's lines 4-22, and assert it names no `git diff`/`shell` allow.

### 5. nit — `tools/orchestration/tests/pipeline/flows.test.ts:229`: the plan cap's non-integer arm is untested
`validatePlanMaxLines` (`flows.ts:174-182`) rejects `!Number.isInteger(max)` as well as
`max <= 0`, and the proposal's contract is "absent **or** a positive integer". The tests
cover absence and `0` only, so `planMaxLines: 12.5` (or `NaN`) is an unexercised throw.
**Fix:** add `expect(() => configFor('light', { ...cliConfig(), planMaxLines: 12.5 })).toThrow(/planMaxLines/)`.

### 6. nit — `tools/orchestration/tests/pipeline/light-flow.test.ts:360`: the #12 session bound cannot fail
The bound is `sessionsCreated.length <= 2 + 2*3 = 8` on a run that spends exactly 4
sessions (`light-flow.test.ts:300-305` asserts the list). A regression to 5, 6, 7 or 8
sessions passes it; Rollout 6 reads this number against run 003's 43, so the loose side
is the side that matters.
**Fix:** assert the exact count for the converging run (`toHaveLength(4)`) alongside the
`≤ 8` ceiling, and apply the ceiling where it binds — the every-pass-dissent fixture in
`light-budget.test.ts:198`.

### 7. nit — `tools/orchestration/src/pipeline/flows.ts:27`: `SeatName` is an exported type with no consumer
It is referenced nowhere in `src/` or `tests/` (grep over `tools/orchestration`: only its
own declaration), so nothing can test it and it is free to drift from the
`SEAT_NAMES` list it mirrors.
**Fix:** delete it, or use it — `export const SEAT_NAMES: readonly SeatName[] = [...]`
makes the validator's seat set type-checked against the panels.

## FYI (no severity, out of specialty)

- No `vitest` config exists in the repo, so the 5 s default test timeout applies.
  `light-promotion.test.ts:166,187` each run a light pipeline **and** a full 12-stage
  pipeline to `done` inside one `it`; that is roughly the weight of
  `funnel.test.ts:207`'s single full run, which passes today, so the risk is marginal —
  an explicit `vi.setConfig({ testTimeout })` in the promotion file removes it cheaply.
- `FlowId = 'full' | 'light' | (string & {})` (`flows.ts:19`) is wider than the
  proposal's `flow: 'full' | 'light'`, and `FlowDef` gained a `path` field the
  proposal's interface does not list. Both are consumed and tested
  (`gates.test.ts:686,759`, `checkpoint-compat.test.ts:146`, `runner.test.ts:346`);
  flagging for the conformance seats, not as a testing finding.
