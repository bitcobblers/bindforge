# Code review — performance — round 1, pass c0

Scope: `git diff 60faf8c24438ed55121757057b578182d2815083...HEAD -- ':(exclude)docs/designs'`
Checklist: `references/performance.md`. Inputs: the diff, `02-proposal.md`,
`docs/adrs/010-lightweight-plan-flow.md`. No other run records read.

**Verdict: request-changes** — 3 major, 1 minor, 1 nit.

The light flow's own cost shape is right and is pinned by tests: the registry
partial (flows.ts:71-86) puts the light run at 2 seats, 3 passes, 6 code
sessions, 12-minute stages, and the drive loop's new stamp rule
(runner.ts:2468-2470) *removes* a `git rev-parse` per ratify pass. The findings
below are the cost the change adds to the **full** flow and the one budget knob
that stopped binding.

## Findings

### 1. major — `tools/orchestration/src/pipeline/stages.ts:108` — full-flow stage timeout tripled

`DEFAULT_CONFIG.stageTimeoutMs` moved `30 * 60_000` → `90 * 60_000`. That knob is
the deadline of the per-session poll loop (`runner.ts:1348`, `1384-1388`), which
re-requests `listSessionMessages` every `pollIntervalMs` (default 2 s) until it
expires. A wedged or unreachable server now costs 90 minutes of wall clock and
≈ 2,700 poll round-trips per attempt (≈ 900 today), and `runWithRetry` runs a
repair attempt, so one stalled stage can burn ≈ 3 h before the run escalates —
on every stage of every full-flow run, whose twelve stages are the flow the
compatibility matrix promises is unchanged ("the twelve `STAGE_BASES` stages, 5
seats, **budgets unchanged**") and whose Non-goals say "No change to the full
flow's stages, seats, budgets".

**Fix:** restore the 30-minute default and put any extra headroom where the
evidence is — a per-stage value in the registry (the pattern this change
introduces), not the global default. If run 010's real need was "this stage is
legitimately slow", prefer a liveness signal (no new assistant turn plus the
nudge budget exhausted) over tripling the absolute cap: the tripling charges the
wedged-session case, which is the case with no work in flight, to pay for the
slow-but-alive case.

### 2. major — `tools/orchestration/src/pipeline/stages.ts:105` — full-flow pass cap raised

`DEFAULT_CONFIG.maxPassesPerRound` moved 6 → 8. At five seats a full-flow design
round can now enter 8 passes, so the binding stop shifts from the backstop
(6 × 5 = 30 sessions) to the phase cap `maxReviews.design: 40` — a **+33 %
worst-case review cost per design round**, over up to `maxRounds: 5`. In the code
phase the session cap binds for full panels, but a single-dissenter panel now
runs 8 passes where 6 capped it (5 + 7 sessions vs 5 + 5). The design fixes both
numbers (`full` = 6 passes / 30 min in the budget table) and the non-goals rule
out full-flow budget changes.

**Fix:** restore 6 and fix the failure the comment describes (run 010 stopped at
6 with four stale approvals and one fresh one) in the funnel, not in the global
price: this branch already adds the rule that a ratify pass must not re-stamp
(`runner.ts:2468-2470`) — that is the convergence the round was one pass from —
and `--extend-budget` is the designed per-run channel for buying one more pass.
Raising the default charges every future full run for one run's near-miss.

### 3. major — `tools/orchestration/src/pipeline/runner.ts:2506-2507` — the light session cap stops binding once any grant exists

```ts
if (!isLight) {
  state.extensionSessions = left - 1;
}
```

For a light run the over-cap pass entry never consumes the grant, and the entry
check escalates only when `left <= 0` (`runner.ts:2492-2515`). So a single
`--extend-budget 1` makes the phase-cumulative cap
(`maxReviews.code: 6`) non-binding for the **rest of the run**: every later
over-cap pass is admitted and logs, and `planPass`'s lift (panel.ts:133-136) is
gated on the same unspent grant, so the +1 pass cap stays lifted too. Bound,
quantified: 2 code rounds × (3 + 1 lifted passes) × 2 seats = **up to 16 review
sessions against a 6-session budget (≈ 2.7×)**, and `extensionSessions` is
state-side, so it survives every resume — the grant is bought once and pays for
the run's remaining passes. The design says the extension is "consumed at the
cap checks" (A16, A27, Budget resolution) and Rollout 6 expects ≈ 9 sessions
plus 1–2 per grant. The test at `light-budget.test.ts:270-284` pins the
single-round case (8 sessions), so the multi-round and resume paths where the cap
is unbound are unpinned.

**Fix:** decrement the grant at the pass-entry check in both flows, and have
`planPass`'s lift read that same (now-consumed) value, so one grant buys exactly
one pass; then pin the total in a test that drives the run into a second code
round with a grant in hand.

### 4. minor — `tools/orchestration/src/pipeline/runner.ts:2620` — the plan document is metered twice, and the second metering spawns git twice

The plan gate calls `proposalSize(opts.repoDir, designDir)`, which does
`existsSync` + `readFileSync` + `git cat-file -e` + `git diff --numstat`
(`runner.ts:171-189`, two `spawnSync` subprocesses). The same stage was metered
one frame earlier for exactly this document — `docStages` includes `plan`
(`runner.ts:1660-1661`) and the number is recorded in the `StageResult`. The gate
reads `size?.docLines` and discards `added`/`removed`, so the two subprocess
spawns and the second read are pure overhead on the one path where the count is
already in hand.

**Fix:** take `docLines` from the `StageResult` the stage just recorded (thread it
onto `StageOutcome`), or call the line-count-only helper — `documentLines`
(gates.ts:127-131) over one read — so the gate costs one read and no subprocess.

### 5. nit — `tools/orchestration/src/pipeline/runner.ts:2083` + `gates.ts:99` — the source document is read and parsed twice per entry

The pre-lock flow read is `parseSource(readFileSync(sourceAbs, 'utf8'), id.kind)`,
then `sourceAdmissible(sourceAbs, id.kind, flow)` re-reads the file
(`await readFile(sourcePath)`) and re-runs `parseSource`. The design asks for
"one frontmatter read per entry, so the value that cleared the pre-lock check is
the value stamped and printed" and for `sourceAdmissible` to "take the same
result as an argument instead of re-reading"; the implementation passes only the
flow string, so the read and the frontmatter scan happen twice.

**Fix:** pass the `ParsedSource` (or the content string) from the pre-lock parse
into `sourceAdmissible` and make that parse the single authority.

## Checklist coverage

1. **Event/poll path** — the per-session poll loop (`runner.ts:1371-1400`) gains
   no fs, network, or table rebuild; all new file and git work sits at stage and
   entry boundaries. `stagePathAllowed`'s new `flowDef(flow)` lookup and
   `traversesDotSegments` split (gates.ts:363-371, 388-390) are O(1) per touched
   path inside the existing sweep. Findings 1 and 4 are the added wall-clock
   items.
2. **Complexity** — no O(n²) over an unbounded n. `identity.ts:79-105` is one
   `readdirSync` of `docs/designs` plus a JSON parse only for directories whose
   slug matches (usually 0–1), and runs once per `pipeline run`. `seatsFor`
   copies a ≤ 5-element array; `planPass` triggers four copies per pass
   (panel.ts:119-124) — noise, not a finding.
3. **Unbounded growth** — `FLOWS`, `SEAT_NAMES`, `LIGHT_ONLY_STAGES` are static;
   the new state field adds bytes per checkpoint only. Finding 3 is the one cost
   path that lost its bound.
4. **Startup** — `pipeline run` gains one sync read + regex scan of the source
   and a `configFor` validation (twice per call) at entry, per stage
   (`stageVars`, `runner.ts:2739`), and per status render. All entry-rate, not
   per-event. Finding 5 is the duplicate read.
5. **Cache invalidation covered** — the new code-stamp refresh rule
   (`runner.ts:2464-2471`: refresh on a full or dissent pass, keep the stamp on
   a ratify pass) is covered: `light-flow.test.ts:374-396` pins one ratify pass,
   two sessions per seat, and equal seat stamps, which fails if the ratify pass
   re-stamped. `panel.test.ts:297-320` pins the ratify-on-current-stamp rule at
   unit level. No finding.
6. **Benchmark-like checks** — `light-flow.test.ts:360` (≤ 2 + seats × 3 = 8
   sessions, the design's Rollout 6 bound) and `light-budget.test.ts:230,252`
   (the 4- and 6-session records) are the cost-bound tests. I could not execute
   the suite from this seat (shell limited to git reads), so the figures quoted
   above are derived from the config arithmetic (`pollIntervalMs` 2 s, the round
   × pass × seat products), not measured.

## Out of specialty, FYI (no severity)

`DEFAULT_CONFIG`'s two budget edits, the `identity.ts:79-105`
existing-run lookup, and the `renderTemplate` placeholder-scan rewrite are not
in the proposal's Rollout list; they read as run-010 escalation fixes riding in
with the light-flow change — the seats that own scope and correctness should
look at them.
