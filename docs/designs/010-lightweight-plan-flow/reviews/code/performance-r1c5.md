# Code review — performance (round 1, pass c5: ratification)

Scope: `git diff 96f50c2110560362e5fb61f05bd313f04bf503cb..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/010-lightweight-plan-flow.md`.
This pass asks one question: did the fixes that landed after round 1 break the cost bounds that
pass approved? My round-1 findings are re-checked against the current source.

## Verdict

**approve.** No blocker and no major finding. The cost story I approved in round 1 is intact, and
the budget-unit fix that landed since then **tightens** it: it closes an unbounded light `fix`
loop (see "What the later fixes changed for cost" below). My three round-1 nits are all still
open — none is made worse by the revisions — and checklist item 6 remains unevaluated by
execution in this tree.

## What the later fixes changed for cost

The round-1 design had the light flow repurposing `state.codeBudgetUsed` into a session count
(`codeBudgetUsed += plan.seats.length` per pass, no per-`fix` charge). Three consumers read that
field as rounds:

- `states.ts:214` / `:220` — the red-`testsGreen` entries from `implement` and `fix`
- `states.ts:248` — the code-review dissent entry
- `states.ts:229` — the `codeRound > cfg.maxRounds` hard cap

With no per-`fix` charge, a light run entering the `fix` loop from a **red** gate charged nothing,
so `budgetLeft` read `0 < 2` on every iteration: that path spent `fix` sessions with no escalation
site — the checklist item 3 shape (unbounded growth) in the flow's mutating loop. The current code
charges every `fix` in both flows (`runner.ts:2755-2761`), counts the phase's sessions through
`reviewSessionsUsed` (`panel.ts:54-56`, read at the pass-entry cap `runner.ts:2652-2653`), and
removes the extension channel from the light round gate (`states.ts:78-80`). Result, verified by
reading the drive loop and the state graph:

- light `fix` entries per run: ≤ `maxRounds` = 2 (charged at `runner.ts:2761`, gated at
  `states.ts:214/220/248`; a `fix` charge lands before `runWithRetry`, so a failed stage charges too)
- light code rounds: ≤ `maxRounds` = 2 (`states.ts:229`, both flows), passes per round ≤
  `maxPassesPerRound` = 3 (`panel.ts:133-138`)
- light code review sessions: ≤ `maxReviews.code` = 6 cumulative, plus 2 seats per
  `--extend-budget` grant, each grant consumed at the entry it pays for (`runner.ts:2666`)
- light stage sessions at the escalation shape: 1 plan + 1 implement + 2 fix + 6 review = 10, at
  the 12-min `stageTimeoutMs` (`flows.ts:83-87`) → ≤ 120 min serial, consistent with the
  proposal's Rollout 6 estimate (plan 1, implement 1, the fix rounds, the code phase's 6
  cumulative, plus 1–2 sessions per grant)

So the deterministic stops are all still reachable and every loop is bounded. Nothing I approved
in round 1 became unbounded.

## Checklist

1. **Event/keystroke path — pass.** No new synchronous fs or network on a per-event path. The
   additions are memory writes on the state (`recordPlanStop` `runner.ts:2355-2363`,
   `clearPlanStop` `runner.ts:2364-2386`) and string building in reporting
   (`repairSteps` `report.ts:245-256`, `planStopStatusLines` `runner.ts:3259-3278`).
   `sourceKind` (`identity.ts:78-80`) is a `relative()` + regex, no fs.
2. **Complexity — pass.** `reviewSessionsUsed` is one `filter` over `state.results`, and
   `results` is bounded by the caps above; it is called at pass entry, in `statusLines`
   (twice) and in `escalationMarkdown` — O(bounded) reads, no nested scan. No O(n²) over an
   unbounded dimension. No map/set opportunity is being missed.
3. **Unbounded growth — pass, and improved.** The one new state field is a single bounded
   string (`types.ts:239-250`, schema `state-store.ts:71-75`). The `gateLog` append in
   `clearPlanStop` fires at most once per clearing `plan` pass. `discardRefusedPlan`
   (2–4 git subprocesses per refused plan) is deleted. The round-1 unbounded light `fix`
   loop is closed, as above.
4. **Startup — pass, slightly cheaper for `full`.** `deriveIdentity` now takes `flow` and calls
   `nnnOfExistingRun` only for a light entry (`identity.ts:66-73`), so a `full` entry skips that
   `docs/designs` scan entirely; the light scan reads at most one `.pipeline.json` per
   slug-matching directory (`identity.ts:119-141`). Pre-lock cost is unchanged: one
   `readFileSync` of the source, ordered before `deriveIdentity` (`runner.ts:2172-2195`).
5. **Cache invalidation — pass, covered by a test.** The sticky stop is the state's cache of the
   gate's verdict, has exactly one writer (`recordPlanStop`) and one clear site
   (`clearPlanStop`, gated by the same `planOutgrewFlow` verdict), and the clear is pinned by
   `light-plan-size.test.ts` ("clears the stop when a re-plan lands under the cap",
   `forceStage: 'plan'` → `done`, `implement` runs, the 60-line plan is the document).
   `hasPlanStop` (the reader) is pinned three ways in `escalation.test.ts`.
6. **Benchmark-like pin — unevaluated by execution (finding below).** The cost pins
   (`flows.ts:83-87` light 12 min / 3 passes / 2 rounds / 6 sessions / cap 80;
   `stages.ts:90-117` full 30 min / 6 passes / 5 rounds / 40+20; the light session arithmetic
   6 → 8 with one grant; the `(120 lines, cap 80)` denial) were verified by reading the
   assertions and their arithmetic. They could not be run in this tree.

## Findings

### nit — checklist item 6 (cost pins) is still unevaluated by execution
Location: `tools/orchestration/tests/pipeline/light-budget.test.ts:214-420`,
`light-plan-size.test.ts:181-335`, `runner.test.ts:397-487`, `funnel.test.ts:432-480`
Problem: the tree has no installed dependencies (no `node_modules` anywhere in the repo, so no
`vitest` binary), and this review environment permits only `git` shell commands, so
`npx vitest run tests/pipeline/…` cannot start. The performance checklist asks me to run the
benchmark-like pins and quote the number; the cost numbers above are therefore read off the
assertions and the registry, not measured, and the item is recorded as unevaluated rather than
passed.
Fix: no code change. In a tree with dependencies installed, run
`npm ci && npm run test` (or `npx vitest run tests/pipeline/{light-budget,light-plan-size,runner,funnel,escalation}.test.ts`)
and confirm the cost pins are green; keep them as the regression guard for the 30-minute
full-flow stage ceiling, the 6-pass backstop, and the light 12 min / 3 passes / 2 rounds /
6 sessions bounds.

### nit — the round-entry extension record prints the wrong cost unit
Location: `tools/orchestration/src/pipeline/runner.ts:2594-2600`
Problem: unchanged from round 1. That entry spends a **round** grant (`state.extensionRounds`,
line 2593) against `cfg.maxRounds`, but prints "extended the review budget by 1 review
session(s) over the caps of 40/20: over-cap design round entered under operator extension
(N granted round(s) left)" — sessions and the session caps, the units of the *other* grant.
The gate log line is the record an operator resumes from, so the grant's price is
misstated there. The consumption itself is correct.
Fix: name the unit of the entry that spent the grant: "extended the revise-round budget by 1
round over the backstop of `${String(cfg.maxRounds)}`: over-cap ${phase} round entered under
operator extension (N granted round(s) left)"; keep the session wording for the pass entry.

### nit — the pass-entry extension record prints 1 session for a grant that buys the panel width
Location: `tools/orchestration/src/pipeline/runner.ts:2652-2673`
Problem: unchanged from round 1, and now demonstrated by the tests. One grant buys one over-cap
**pass**, i.e. `plan.seats.length` sessions — 2 at the light panel, 5 at the full code panel —
while the consent line prints "extended the review budget by 1 review session(s)".
`light-budget.test.ts` pins the price at 2 sessions per grant (6 → 8 with one
`--extend-budget 1`), so the printed unit understates the cost of the grant by the panel width,
which is exactly what an operator prices when deciding to buy one. It stays inside the
proposal's Rollout 6 expectation ("1–2 sessions for each `--extend-budget` grant"), so no
budget is unbound.
Fix: print the panel width the entry launches — `plan.seats.length` is in scope (the `planPass`
result is bound at line 2623, before the cap check at 2652), e.g. "over-cap ${phase} review pass
entered under operator extension (2 seats, N session grant(s) left)".

## FYI (outside this seat)

`repairSteps` prints `pipeline run <source> --force` for a plan stop and its note promises "on the
identity the run already has" (`report.ts:245-256`). When the source is a brief, the new
identity rule allocates the next free `NNN` for a `flow: full` entry (`identity.ts:66-73`,
pinned by `light-promotion.test.ts` "keeps the full flow's next-free allocation for a brief"), so
that command opens a second identity beside the plan; `docs/workflows.md` records exactly that
and tells the operator to pin `--nnn`/`--slug`. Cost reading only: the brief's promotion is a
second full-flow run's worth of work, not a re-run of the first identity.
