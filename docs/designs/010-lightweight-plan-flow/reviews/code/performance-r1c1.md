# Performance review — dissent pass c1 (round 1)

- Seat: performance
- Diff under review: `eb8b40ad413a397f71b977f6d0dcd6c903f0d17c..HEAD -- ':(exclude)docs/designs'`
  (source + tests only; 19 files, +829/−284)
- Judged against: `docs/designs/010-lightweight-plan-flow/02-proposal.md`,
  `docs/adrs/010-lightweight-plan-flow.md`
- Scope of this pass: my five round-1 findings, which the last revision claims to answer.

## Verdict

`approve` — zero open blocker/major findings. All five round-1 findings are closed;
nothing in the changed code opens a new performance cost.

## Round-1 findings — resolution check

1. **`stageTimeoutMs` 30 → 90 min (major) — RESOLVED.**
   `stages.ts:117` is back to `30 * 60_000`, with the comment naming it the cost
   ceiling of a wedged session and routing the light flow's shorter stage through
   `FLOWS.light.config`. `flows.ts:86` sets `stageTimeoutMs: 12 * 60_000` for
   light, which is the proposal's budget-table row and ADR-010's table. A wedged
   full-flow stage now costs the pre-change 30 min / ~900 polls at the 2 s default
   (`runner.ts:1400,1440,1474`), and a light one 12 min / ~360. Pinned by
   `flows.test.ts:142,149` (`DEFAULT_CONFIG.stageTimeoutMs === 30 * 60_000` and
   `configFor('full', DEFAULT_CONFIG)`), and by `flows.test.ts:92` for light.
   `docs/workflows.md` prints the same two numbers. The "budgets unchanged"
   compatibility row and the full-flow Non-goal now hold.
2. **`maxPassesPerRound` 6 → 8 (major) — RESOLVED, and the stated cause fixed in the funnel.**
   `stages.ts:104` reads 6; `docs/workflows.md` reads 6 in both tables. The
   run-010 convergence fix landed where I asked for it — the stamp rule at
   `runner.ts:2576-2580`: a `ratify` pass no longer re-stamps `codeStamp` to `HEAD`,
   so it cannot age the approvals it is ratifying. That removes the second ratify
   pass the old cap was raised to buy, so the price of every run is not charged for
   it. The `panel.ts:126-133` pass-lift is deleted, so the backstop is
   `cfg.maxPassesPerRound` in both flows — the full-flow backstop is untouched and
   the light round is bounded at 2 seats × 3 passes = 6 sessions, the design's
   binding knob. Pin: `panel.test.ts` "blocks at the pass cap with a grant in hand,
   naming that cap"; `flows.test.ts:136-150` pins 6/30 min/5/40-20 on `DEFAULT_CONFIG`.
3. **Light over-cap pass entry never consumed the grant (major) — RESOLVED.**
   `runner.ts:2618` decrements unconditionally (`state.extensionSessions = left - 1`),
   so a `--extend-budget 1` grant buys exactly one over-cap pass and the
   phase-cumulative cap `maxReviews.code: 6` stays binding for the rest of the run,
   including after a resume (the field is state-side). The `planPass` +1 lift that
   shared the grant is gone (finding 2), so the two seams agree. My round-1 bound of
   up to 16 review sessions against a 6-session budget is now `6 + 2 × grants`
   (2 seats per granted pass), inside the proposal's Rollout 6 expectation of
   "1–2 sessions for each `--extend-budget` grant". Pins: `light-budget.test.ts`
   "leaves a grant unspent when the round stops at the pass backstop" (grant not
   logged when nothing was extended), "spends one grant per over-cap pass and stops
   when it runs out" (6 → 8, `extensionSessions: 0`, escalation at 8/6), and
   "prints a grant count that decreases and stops when the grants run out"
   (`--extend-budget 2` → 6 → 8 → 10, log counts `['1','0']`, escalation at 10/6).
   The multi-round and resume paths I called unpinned are pinned by the latter two.
4. **Plan gate re-metered a document the stage had just metered (minor) — RESOLVED.**
   `StageOutcome.docLines` (`runner.ts:1204-1210`) carries the size meter's count,
   set once at `runner.ts:1745` from the `proposalSize` call the `docStages` rule
   already makes (`runner.ts:1711-1714`); the plan stop is now a
   `preCommit` gate (`runner.ts:2726-2754`) that reads that count. The
   `proposalSize` call in the drive loop is deleted, so the gate costs zero
   `readFileSync` and zero `spawnSync` git on the happy path and on the refusal.
   The line-count-only helper `documentLines` and its call site are deleted from
   `gates.ts`, so there is one line-count authority — pinned by
   `light-plan-size.test.ts` "denies on the meter's own count, as the stage
   recorded it" (the denial's N equals `results[plan].docLines`).
   `discardRefusedPlan` (`runner.ts:214-224`, two git calls) runs only on the
   refusal, once per run.
5. **Source read + frontmatter-parsed twice per entry (nit) — RESOLVED.**
   `runPipeline` parses once pre-lock and passes the `ParsedSource` into
   `sourceAdmissible` (`runner.ts:2177-2200,2240`; `gates.ts:80-95` takes
   `parsed?` and skips its own `readFile` when handed it). One read per entry, and
   the value that cleared the pre-lock check is the value stamped. The unreadable
   file keeps its own single read and the existing denial. Resume reads no source
   document at all (`runner.ts:2263`).

## Checklist

1. **Event/keystroke path** — no event path in this surface. The new synchronous
   work (`discardRefusedPlan`, the refusal-path sweep at `runner.ts:1881-1890`) runs
   only on a pre-commit refusal, which is terminal (`stopOnGate`,
   `runner.ts:2381-2388`), so a refusal costs one plan session, not a repair
   session. The poll loop is untouched.
2. **Complexity** — nothing new is quadratic. The new pass-ledger re-sort
   (`runner.ts:2116,2132-2136`) is O(k log k) over the pass's own ledger tail
   (k ≤ seats × attempts); `recordAttemptFailure` (`runner.ts:1785-1797`) is the
   reverse scan it was extracted from, once per refused window. `configFor` gains
   `validatePanelPresence`, O(seats), on entry/resume/status/escalation only.
3. **Unbounded growth** — no new registry, cache or array. `seatRank` is per-pass
   and bounded by the seat list; `FLOWS`/`PANELS` are static. `state.results` and
   `gateLog` grow as they did; this change *removes* the per-resume
   `operatorExtension` gate line (`runner.ts:2283-2292`), so grants log where spent
   and never when unspent.
4. **Startup** — boot reads: the pre-lock source parse, now one read (finding 5).
   `gates.ts` no longer imports `configFor`/`DEFAULT_CONFIG`, so the source gate
   resolves no config on every entry path.
5. **Cache invalidation** — the seat-approval stamp cache is the thing this revision
   changes, and the rule is covered: a ratify pass keeps the stamp its round entry
   set (`runner.ts:2576-2580`), and the behavioral pin is exactly one ratify pass
   (`light-flow.test.ts:419-424`; `funnel.test.ts:222` counts `c0 + ratify` = 2
   sessions for a seat). The pass cap is pinned in `panel.test.ts`.
6. **Benchmark-like test / timing note** — the numeric pins in this change are the
   config-constant and session-count assertions quoted above
   (`flows.test.ts:92,142,149`; `light-budget.test.ts` 6/8/10 session counts).
   I could not execute them: `tools/orchestration` has no installed
   `node_modules` in this tree, so `vitest` is unavailable (`npx vitest run …` did
   not start). The numbers were verified by reading the assertions and the
   arithmetic they pin, not by a run.

## Cost delta this revision lands (vs. the round-1 shape)

- Full-flow worst-case stage stall: 90 min → 30 min (~900 polls, not ~2,700);
  a stalled stage with its repair attempt: ~3 h → ~1 h.
- Full-flow design round: backstop 8 → 6 passes, so the binding stop returns from
  `maxReviews.design: 40` to the backstop (6 × 5 = 30 sessions), the design's row.
- Light grant arithmetic: unbounded-for-the-run → `6 + 2 × grants` code sessions.
- Plan stop: 2 fewer subprocess spawns and 1 fewer file read per `plan` stage.
- Entry: 1 fewer source read and frontmatter scan per `pipeline run`.

## FYI (out of specialty, no severity)

- `runner.ts:2549-2551`: the round-entry extension line now says "extended the
  review budget by 1 review session(s) … over-cap design round entered … (N granted
  round(s) left)" while spending a *round* grant; the count decreases correctly, the
  unit in the text does not match what the entry consumed.
- `runner.ts:2618-2625`: one grant buys one over-cap *pass*, i.e. `seats.length`
  sessions (2 at the light panel), while the line prints "by 1 review session(s)".
  Within the design's "1–2 sessions per grant" expectation; the wording is the
  author's call.
