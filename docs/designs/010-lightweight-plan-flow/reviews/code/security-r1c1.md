# Code review — security, round 1 dissent pass (c1)

Scope: `git diff eb8b40ad413a397f71b977f6d0dcd6c903f0d17c..HEAD -- ':(exclude)docs/designs'`,
read against `docs/designs/010-lightweight-plan-flow/02-proposal.md` and
`docs/adrs/010-lightweight-plan-flow.md`. Inputs: the diff, the proposal, the ADR, and my own
round-1 findings. Checklist: `references/security.md` (external input at the boundary, path
handling, injection, secrets, new dependencies, filesystem writes, clean rejection of malformed
input).

Verdict: **approve**. All seven round-1 findings are resolved in the current tree; one new
asymmetry is introduced by the round-1 fix itself (minor).

## Round-1 findings — re-check

1. **major — full-flow cost ceilings raised outside the design's authority** — **resolved.**
   `DEFAULT_CONFIG.maxPassesPerRound` is 6 (`stages.ts:107`) and `stageTimeoutMs` is
   `30 * 60_000` (`stages.ts:117`), with comments that record the run-010 evidence and state the
   pass-cap remedy is a funnel change, not a constant. `docs/workflows.md:234,237,245,255` now
   matches the code (full 6 / light 3; full 30 min / light 12). The values are pinned on the
   constant itself, not on a copy of it: `flows.test.ts:140-150` asserts
   `DEFAULT_CONFIG.maxPassesPerRound === 6`, `stageTimeoutMs === 30 * 60_000`,
   `maxRounds === 5`, `maxReviews === {40, 20}`, and `configFor('full', DEFAULT_CONFIG)`
   reproduces them. The 12-min light pin (`flows.test.ts:92`) is unchanged.
2. **major — the light `--extend-budget` grant was never consumed** — **resolved.** The `!isLight`
   guard is gone: the over-cap pass entry consumes the grant unconditionally
   (`runner.ts:2606-2618`), and the pass-cap lift in `planPass` is gone — the backstop is
   explicitly unbuyable (`panel.ts:129-140`), pinned by `panel.test.ts:341` ("blocks at the pass
   cap with a grant in hand"). One grant buys exactly one over-cap pass: `light-budget.test.ts:298`
   (6 → 8, `extensionSessions` 1 → 0, one consent line, next over-cap entry escalates at 8/6) and
   `:332` (grant 2 → sessions 6 → 8 → 10, printed grant counts `['1','0']` — decreasing, and the
   multi-round case my r1 noted as untested is now covered). The full flow's consumption is
   unchanged (`budget.test.ts:210,252`).
3. **minor — seats validated against the union of both panels** — **resolved.** `validateSeatNames`
   (`flows.ts:209-236`) checks each phase against its own `PANELS` entry, so
   `reviewSeats.code: ['complexity']` and `reviewSeats.design: ['testing']` are pre-lock usage
   errors (`flows.test.ts:290-303`), and a non-array phase key is the same clean refusal.
   `validatePanelPresence` (`flows.ts:186-200`) turns the JSON-shaped one-key `reviewSeats` into
   the usage error instead of `TypeError: undefined is not iterable` (`flows.test.ts:308-319`);
   the light partial, as last writer, still resolves a one-key base.
4. **minor — plan-size stop failed open on a missing document** — **resolved.** The pre-commit
   gate refuses when the meter produced no measurement
   (`runner.ts:2729-2737`, `plan document not written: <dir>/02-proposal.md`), terminal via
   `stopOnGate` (`runner.ts:2384-2389`), so no `implement` session is spent. Covered by
   `light-plan-size.test.ts:260` (missing file) and `:280` (empty file measures 0, documented as
   "a cap, not a floor" — refusal is the unmeasurable document). The refused oversized plan is
   discarded so it cannot ride into the terminal `git add -A` (`runner.ts:214-224`,
   `light-plan-size.test.ts:191-192`).
5. **minor — `dissentingSeats(state, 'design')` dropped the caller's cfg** — **resolved.** The
   cfg is threaded at every reader (`states.ts:87,149`; `panel.ts:123-124`), and
   `states.test.ts:176-224` asserts a caller-supplied design panel drives the dissenter list.
6. **nit — source document read and parsed twice per entry** — **resolved.** `runPipeline` parses
   once pre-lock (`runner.ts:2176-2181`) and threads that `ParsedSource` into
   `sourceAdmissible` (`runner.ts:2240`; `gates.ts:93-108`), so the value that cleared the
   pre-lock check is the value stamped (`state.flow = flow`, `runner.ts:2219`) and judged.
7. **nit — unreachable `'plan'` branch in `sourceAdmissible`, duplicate size authority** —
   **resolved.** The branch is gone; the plan cap is `planOutgrewFlow` fed by the meter's own
   count carried on the stage outcome (`runner.ts:1742-1745`, `2739`), asserted as one authority
   in `light-plan-size.test.ts:203-215`, and the full flow keeps measure-and-warn
   (`gates.ts:135-144`, `gates.test.ts:515-518`).

## Checklist

1. *External input validated at the boundary* — pass. The `flow:` frontmatter is case-sensitive,
   registry-gated pre-lock (`gates.ts:54-75`, `runner.ts:2182-2189`); the caller panel is
   validated by name, per phase, shape, presence and emptiness scoped to the phases the flow runs
   (`flows.ts:172-283`); `planMaxLines` is absent-or-positive-integer. No evaluation of document
   content.
2. *Path handling* — pass. No interpolated shell: every git call is `spawnSync/spawn` with an
   argv array. The ownership predicate parses no path (git plumbing output only, A28), reserved
   names are matched segment-by-segment case-insensitively (`types.ts:197-210`) and win first in
   both flows (`gates.ts:371-384`), asserted in `gates.test.ts:660` and
   `light-ownership.test.ts:132-143` (the `docs/../src` spelling arrives as git's real path).
3. *Injection* — pass. The frontmatter regexes are linear (lazy scan + end-anchored section
   match, `gates.ts:55-66`); no user string enters a shell, and escalation reasons pass through
   `printable` (`runner.ts:3095-3099`).
4. *Secrets/PII* — pass. Nothing in the diff logs credentials; gate logs print numbers, seat
   names and paths.
5. *New dependencies* — pass. The diff touches no manifest; the registry test pins the existing
   engines floor and asserts no platform branch (`flows.test.ts:340-355`).
6. *Filesystem writes* — pass. The new writes are the refused-plan restore/unlink
   (`runner.ts:214-224`, guarded by `existsSync`/`statSync` on a run-directory-derived path) and
   the escalation record, which keeps the tmp+rename atomicity (`runner.ts:3118-3126`).
7. *Clean rejection* — pass. Bad flow value, unknown/misphased seat, one-key panel, zero plan
   cap, stage outside the flow and an unknown flow id are all usage errors before lock, design
   directory, checkpoint or session (`runner.ts:2182-2212`, `flows.ts:104-110`,
   `flows.test.ts:229-327`, `gates.test.ts:726`).

## New findings

- **minor** `tools/orchestration/src/pipeline/runner.ts:2529` (with `states.ts:70-72`) —
  `--extend-budget` grants are half-consumed in the light flow: the round grant is never spent
  there. `state.extensionRounds` is incremented on every `--extend-budget`
  (`runner.ts:2224-2225, 2290-2291`) and consumed at exactly one site, the round-entry check,
  which the round-1 fix gates on `!isLight`. Its only other reader is `budgetLeft`
  (`states.ts:71`), so a light run that ever holds a round grant satisfies the red-test
  `budgetLeft(state, state.codeBudgetUsed, cfg)` tests at `states.ts:191` and `:197`
  permanently: the `fix → red tests → fix` loop stays legal for the checkpoint's whole life,
  spending a coder session per iteration, with no gate log (the `operatorExtension` line lives
  inside the `!isLight` branch) and no decreasing count in any record. The design's second
  deterministic stop — "budget exhaustion … the reason names the flow" (ADR-010, *When a light
  run is too big*) — is therefore bypassable on that path by a grant that is never spent and
  never audited, the same shape my r1 finding 2 reported for sessions. The session cap
  (`runner.ts:2604-2626`) and the hard `codeRound > cfg.maxRounds` cap (`states.ts:206-210`) are
  unaffected, and reaching the path needs an operator grant plus a red `testsGreen`, so this is
  minor, not major.
  *Fix:* consume the round grant where the light flow's own rounds are entered (e.g. charge the
  red-test `fix` entry in the light branch, gated on the remaining grant) and log the
  `operatorExtension` record there, so the printed count is the checkpoint's count; assert in
  `light-budget.test.ts` that a light run with `--extend-budget` and red tests escalates at the
  round cap and that `extensionRounds` decreases. Untested today: no light test touches
  `extensionRounds` (only `budget.test.ts:210,252`, full flow).

## FYI (no severity)

`runner.ts:2697` logs `'all five seats approve the current revision stamp'` on a converged pass;
a light run reaches this line and its gate record then says five seats two lines above the
`seatSummary` that prints two. Cosmetic record text, no security import.
