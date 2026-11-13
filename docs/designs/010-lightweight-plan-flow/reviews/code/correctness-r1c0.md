# Code review — correctness seat, round 1, pass c0

Scope reviewed: `git diff 60faf8c24438ed55121757057b578182d2815083...HEAD -- ':(exclude)docs/designs'`
(plus `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`, `03-build-notes.md` as the
deviation record). Checklist: `.opencode/skills/code-review/references/correctness.md`.

Verdict: **request-changes** — 1 blocker, 3 major.

The registry itself is faithful: `FLOWS` (`flows.ts:45-84`) declares ADR-010's stage lists,
`docStages`, the light partial (`maxRounds 2`, `maxReviews {design:0,code:6}`,
`maxPassesPerRound 3`, `12*60_000`, `planMaxLines 80`, seats `correctness, testing`);
`configFor` (`flows.ts:113-119`) is the single resolver with `DEFAULT_CONFIG` first, base, flow
partial last, and `--max-parallel` survives by spread order; `seatsFor` is the one panel reader and
`funnelSpecialties`/`specialtiesFor` delegate to it (`panel.ts:32-37`, `states.ts:26-32`); the five
seat readers take a `cfg` and every production call site hands one (`runner.ts:2465,2580,2725,3096,3097`,
`report.ts:253`, `states.ts:87,122,207`); the light transitions, the `testsNeedUpdate` guard, the
converged→`done` edge, the meter keyed on `docStages` (`runner.ts:1660`), `plan` in `CODE_STAGES`
for reporting (`report.ts:36-47`), the pre-lock flow gate (`runner.ts:2081-2095,2165-2173`),
the single escalation suffix (`runner.ts:2974-2981`), the flow line in status and the report
(`runner.ts:3082-3089`, `report.ts:237-247`), `plan`'s stage row, template, agent (verbatim copy of
`designer.md:4-22`) and skill all check out. What does not check out is listed below.

## Findings

### 1. [blocker] `tools/orchestration/src/pipeline/stages.ts:105,108` — the full flow's budgets were changed

`DEFAULT_CONFIG.maxPassesPerRound` is `8` (was `6`) and `stageTimeoutMs` is `90 * 60_000`
(was `30 * 60_000`). The base values are recorded by this run's own research
(`01-research.md:48-50`: "`maxPassesPerRound: 6`, `stageTimeoutMs: 30 * 60_000`") and by the
proposal's budget table (`02-proposal.md:171-172`: full = 6 passes, 30 min). The proposal's
Non-goals (`02-proposal.md:36`) say "No change to the full flow's stages, seats, budgets,
escalation text, or `version: 1`". `03-build-notes.md` lists eight deviations and never mentions
these two knobs (checklist item 3: undeclared deviation), and the change ships a self-contradiction:
`docs/workflows.md:235` and `:251`, both landed in this diff, still document `stageTimeoutMs` as
"30 min" while the code says 90. Nothing pins the values either — `flows.test.ts:122,145` assert
against literals carrying `6`, so `configFor('full', base)` passing proves nothing about
`DEFAULT_CONFIG`. Raising the full flow's pass backstop and stage timeout is exactly the drift
this design set out to make impossible, and a light-flow run is the wrong place to buy it.

Fix: restore `maxPassesPerRound: 6` and `stageTimeoutMs: 30 * 60_000`. The light flow needs
neither — `FLOWS.light.config` already sets `3` and `12 * 60_000`. If the 8-pass / 90-min values
are genuinely wanted (the `stages.ts:92-104` comment argues for 8 from run 010), they are a
separate change: declare them in `03-build-notes.md`, reconcile `docs/workflows.md:235,251`,
and add a test that asserts the full flow's resolved numbers.

### 2. [major] `tools/orchestration/src/pipeline/panel.ts:129-135` + `runner.ts:2506-2514` — the funnel was modified for light, and `--extend-budget` stopped being consumed

`planPass` now computes `passCap = cfg.maxPassesPerRound + (flowOf(state) === 'light' &&
(state.extensionSessions ?? 0) > 0 ? 1 : 0)`, and the pass-entry cap check no longer decrements
the grant for light (`if (!isLight) state.extensionSessions = left - 1`). The proposal says the
opposite in two places: "The funnel (`panel.ts`) runs unmodified on the light seat list"
(`02-proposal.md:244-245`) and "`--extend-budget` never reaches this function: it writes
`state.extensionSessions` / `extensionRounds`, consumed at the cap checks (A16, A27)"
(`02-proposal.md:162-163`); A27 pins the grant as "consumed at the pass-entry cap check".
The deviation is undeclared, and no test exercises the lift (`light-budget.test.ts:267-285`
asserts only that the grant lands on the state). Consequences of the non-consuming branch: a
single `--extend-budget 1` lifts the pass cap in **every** later pass and round of the run
(it is never spent), the `operatorExtension` gate log prints
"(`left` session grant(s) left)" while the state's grant is untouched, and `planPass`'s blocked
reason prints `cap 3 passes per round` after 4 passes ran.

Fix: drop the `passCap` expression (keep `cycle >= cfg.maxPassesPerRound`) and restore
`state.extensionSessions = left - 1` for both flows. If an operator-facing pass extension is
wanted, it is a funnel change with its own design, its own grammar in the `--extend-budget`
message, and a reason string that names the effective cap.

### 3. [major] `tools/orchestration/src/pipeline/gates.ts:404-411` — the light test-path widening is narrower than the design's, and the coder's prompt promises the wider one

The light branch is `return !inTests || /(^|\/)tests\//.test(path)`. `TEST_PATH_RE`
(`gates.ts:361`) is `/(^|\/)tests\/|\.test\.ts$/`, so a co-located test path such as
`src/x.test.ts` is `inTests` and has no `tests/` segment → **denied** for light `implement`/`fix`.
The design's widening is by the test-path predicate: A8 ("`implement`/`fix` are denied
`TEST_PATH_RE`") and Ownership ("light `implement`/`fix` may write test paths",
`02-proposal.md:289-293`). The light coder is told the wider rule —
`TEST_OWNERSHIP_LIGHT` says "you own the test files" (`prompts.ts:22-23`) — so the write is
silently reverted by the sweep (`runner.ts:282`), `applyOwnershipReport` (`runner.ts:1719-1724`)
sets `testsNeedUpdate`, and light ignores that flag (`states.ts:183,191`), so nothing tells the
stage where its file went. `gates.test.ts:596-609` and `light-ownership.test.ts:72-94` only use
paths under a `tests/` directory, so the gap is unpinned. `03-build-notes.md` records it only as
a code comment ("only ones under a real `tests/` directory", `gates.ts:381`), not as a deviation.

Fix: make the widening the designed one — inside `stage === 'implement' || stage === 'fix'`,
`if (flow === 'light') return true;` (the reserved-name rule at `gates.ts:396` already won), and
pin `stagePathAllowed('implement', 'src/x.test.ts', 'light') === true` in the ownership matrix.
If the narrower rule is intended, declare it in `03-build-notes.md` and make
`TEST_OWNERSHIP_LIGHT` state that only paths under a `tests/` directory are owned.

### 4. [major] `tools/orchestration/src/pipeline/gates.ts:370-372,399-401` — new path-shape rule in the ownership spine, in both flows

`traversesDotSegments(path)` refuses any path with a `.`/`..` segment, ahead of every stage rule.
The proposal assigns the predicate no path parsing: "so the `flow` parameter parses no path"
(`02-proposal.md:302`), with A28's argument that git's plumbing output cannot carry those
spellings and that `moves.ts` Rule 1 rejects them. This also changes full-flow verdicts:
`stagePathAllowed('research', 'docs/../src/x.ts')` was `true` (`inDocs` matches the literal
prefix) and is now `false`, so the flow-neutral spine the design promised to leave alone moved.
`gates.test.ts:665-666` pins the new rule, while `02-proposal.md:388` (#7) states the opposite
expectation ("a POSIX repo-root-relative path gets the verdict of its own spelling") and #18
expects that spelling to reach the sweep as git's real path.

Fix: delete `traversesDotSegments` and its two assertions, and rely on the A28 intake argument
plus `moves.ts` Rule 1, as the design does. If the belt is wanted (defence in depth against a
future intake that normalizes nothing), declare it in `03-build-notes.md`, state that it applies
to both flows, and correct #7/#18 so the design and the matrix agree.

### 5. [minor] `tools/orchestration/src/pipeline/gates.ts:88-109,132-135` — the plan cap exists twice, and the second copy is dead in production

`sourceAdmissible` grew a `kind: 'plan'` arm that caps the document it is handed via
`planOutgrewFlow(documentLines(content), configFor(flow, DEFAULT_CONFIG))`, but the runner only
ever calls it with `id.kind` (`runner.ts:2142`), which is `'adr' | 'brief'`. The gate that runs is
`runner.ts:2616-2633`, measuring the run's own `02-proposal.md`. So `gates.test.ts:715-730`
exercises a path no run takes, and `documentLines` is a second line-count authority next to
`proposalSize`'s (`runner.ts:181`) — the two agree today, and nothing keeps them agreeing.

Fix: drop the `'plan'` arm of `sourceAdmissible` and `documentLines`, and let
`light-plan-size.test.ts` (which drives the real gate) be the gate's test; reuse
`proposalSize`'s count at the single call site.

### 6. [minor] `tools/orchestration/src/pipeline/runner.ts:2616-2633` — the plan gate is post-commit, and fails open when the plan is missing

The design puts the stop at "the same stage-ok/pre-commit point where `proposalSize` runs"
(`02-proposal.md:276-279`). Here it runs after `executeStage` (`runner.ts:1810`), whose
`finalizeStage` has already committed the plan (`runner.ts:1782-1791`), so an oversized plan
lands on the branch before the run escalates. Worse, `planOutgrewFlow(size?.docLines ?? 0, cfg)`:
when the `plan` stage replies ok and writes no `02-proposal.md`, `proposalSize` returns
`undefined`, the gate sees `0` lines, allows it, and the run spends an `implement` session with
no design document and an empty seat stamp (`blobStamp`, `runner.ts:200-205`).

Fix: capture the `size` already computed pre-commit at `runner.ts:1659-1662` and gate on it, and
deny a `plan` stage whose `size` is `undefined` (the stage produced its one artifact or it failed).

### 7. [minor] `tools/orchestration/src/pipeline/flows.ts:19,22,87,113,126` — the registry's exported names and signatures drift from the proposal, undeclared

The proposal's contract (`02-proposal.md:98-124`) is `type Flow = 'full' | 'light'`,
`FLOWS: Record<Flow, FlowDef>`, `flowOf(state: Pick<PipelineState,'flow'>): Flow`,
`configFor(flow: Flow, base?: Partial<PipelineConfig>)`, `seatsFor(phase: ReviewPhase, cfg):
readonly string[]`. The code exports `FlowId = 'full' | 'light' | (string & {})`, `SeatPhase`,
`configFor(flow, base: PipelineConfig = DEFAULT_CONFIG)` and `seatsFor(...): string[]`. The
widening is the load-bearing part: `PipelineState.flow?: FlowId` (`types.ts:238`) accepts any
string, so the compiler stops guarding the flow value — only the zod enum
(`state-store.ts:70`) and the pre-lock check (`runner.ts:2087`) do. `03-build-notes.md` deviation 1
declares only the `assertStageInFlow` parameter order. `FlowDef.path` (`flows.ts:36-37`) is
likewise a new field the proposal does not list (it is used, correctly, by the flow line).

Fix: export `Flow = 'full' | 'light'` and use it for `PipelineState.flow`, `flowOf`, `configFor`,
`assertStageInFlow`; keep the open `string` arm only for untrusted text (`ParsedSource.flow`,
the registry lookup). Alias `SeatPhase` to `ReviewPhase`, return `readonly string[]`, and take
`base?: Partial<PipelineConfig>`. Whatever remains, list it in `03-build-notes.md`.

### 8. [minor] `03-build-notes.md:96-101` — "one frontmatter read per entry" is not what the code does

The note claims the entry path reads the frontmatter once. `runPipeline` reads and parses the
source at `runner.ts:2083` (`parseSource(readFileSync(sourceAbs, 'utf8'), id.kind).flow`), then
`sourceAdmissible` reads and parses the same file again (`gates.ts:97,101`). The proposal's
mechanism (`02-proposal.md:210-220`) — parse once right after `deriveIdentity`, carry the parsed
frontmatter on the `deriveIdentity` result, and hand that result to `sourceAdmissible` as an
argument — is not implemented, so "the value that cleared the pre-lock check is the value stamped
and printed" rests on the file not changing between the two reads.

Fix: pass the `ParsedSource` (and its `body`) into `sourceAdmissible` and delete its `readFile`,
or state the two reads plainly in `03-build-notes.md`.

### 9. [minor] `tools/orchestration/src/pipeline/runner.ts:2419-2451,2566-2568,3091` — the light `maxRounds` consumer moved, and the repurposed record is printed as rounds

For light, the round-entry revise-round backstop and the round charge are both skipped
(`if (!isLight)`), and `state.codeBudgetUsed` is incremented by the pass's seats. The binding
`maxRounds: 2` check is then `state.codeRound > cfg.maxRounds` in `states.ts:202` — behaviorally
right (the fix loop stops at 2 rounds), and worth keeping. Two follow-ons are not right:
the proposal names the runner's revise-round record as the knob's consumer
(`02-proposal.md:200-201`), and the record is now dead for light, undeclared; and `statusLines`
prints the repurposed counter under its old label —
`budgets: design 0 rounds, code 6 rounds` (`runner.ts:3091`, and `report.ts:244-246` in
ESCALATION.md) — where 6 is review **sessions**. The proposal's Reporting section asks the status
and the report to name "the effective numbers"; a light board labelled "6 rounds" is not one of them.

Fix: print the light record with its unit (`code 6 sessions` for `flowOf(state) === 'light'`,
keeping the full-flow line byte-equal), and add the deviation to `03-build-notes.md`:
`maxRounds` binds for light at `states.ts:202`, not at `runner.ts:2422`.

## Checklist walk

1. Components and interfaces: present and wired; naming/signature drift is finding 7.
2. Assumption register: A16/A27 silently changed (finding 2); A8's widening narrowed (finding 3);
   A28's intake argument used to justify a new rule (finding 4). A1-A7, A9-A15, A17-A22 honored.
3. Deviations: findings 1, 2, 3, 4, 6, 7, 9 are gaps not in `03-build-notes.md`; declared
   deviations 1-8 check out against the code (param order ✓, renderer default ✓, per-pass-kind
   code stamp at `runner.ts:2466-2471` ✓, `nnnOfExistingRun` ✓, `slug:` label ✓, dead-code
   removal ✓, `ParsedSource.body` and the brief gate ✓).
4. Scope creep: findings 2, 4, 5.
5. Build-notes claims true except finding 8 (and the count slip below).
6. Error paths fail closed: bad `flow:` value, `--stage` outside the flow, empty/unknown seat
   panel, `planMaxLines: 0` — all pre-lock usage errors with no lock, no design directory, zero
   sessions (`runner.ts:2087-2113,2165-2173`, `flows.ts:151-201`); the plan stop escalates with
   the promotion line and spends no `implement` session. The one fail-open path is finding 6.
7. Previous-round blocker/major findings: none on this seat's record; not applicable.

## FYI (no severity)

`FLOWS.full.path` (`flows.ts:63`) prints "plan" as a step of the **full** flow in
`pipeline status` and ESCALATION.md, in a release that gives that word a stage of its own, and
the string omits `proposal-final`, `editorial-review` and `doc-sync`.
`03-build-notes.md:8,148` says "eight files" while `:130-132` names nine.
`pipeline resume <light-slug> --stage plan` is refused by `assertForceableStage`
(`stages.ts:178-183` excludes `LIGHT_ONLY_STAGES`), so a light run cannot be forced onto its
first stage; the proposal does not ask for that.
