# Correctness review r1 (dissent pass c1)

verdict: request-changes

Scope: `git diff 9ae29b5a 24b804a9` on `02-proposal.md`, then the full current
document, plus `docs/adrs/010-lightweight-plan-flow.md` and the Revision log.
Checklist: `references/correctness.md`.

Closures checked first (my r1 findings):

- **r1 major — `--extend-budget` lost to the flow partial: CLOSED, the rebuttal is sound.**
  A27 holds against the code: `state.extensionSessions`/`extensionRounds` are written on the
  run and resume paths (`runner.ts:2096-2097`, `2150-2151`) and consumed at the pass-entry cap
  check (`runner.ts:2359-2377`, `state-store.ts:78-79`); no `PipelineConfig` key is touched, so
  the flow partial cannot overwrite it. The new Budget-resolution paragraph states this correctly.
- r1 minor (shape-dependent binding stop): **partly closed** — the shape split is now stated, and
  the closure text introduces a new arithmetic error (finding 1).
- r1 minor (`## Files to touch` gate keys on prose): **closed** by deleting `planMaxFiles` and
  deferring it; the Deferred entry is consistent with the ADR ("Two deterministic stops",
  ADR-010 §When a light run is too big), so it is not re-opened.
- r1 minor (13 vs twelve stages): **closed** — all three mentions now read twelve, and
  `STAGE_BASES` (`stages.ts:101-138`) has exactly twelve rows.
- r1 minor (seat fallback implemented twice): **closed** — `seatsFor` in `flows.ts`, both
  call sites delegate, residual named in Risks row 4, Test #15 asserts the panel.
- r1 minor (#6 vs #17 outcome): **closed** — #6 now names the pre-lock usage error and reserves
  the denial for unreadable files.
- r1 nit (ADR-010 "skill carries the length cap"): **closed** — The `plan` stage now records the
  supersession.

Spot-checks for the anti-hallucination gate: `google.github.io/eng-practices/review/developer/small-cls.html`
(live; "The number of files that a change is spread across also affects its 'size'… 200-line change
in one file might be okay, spread across 50 files… too large" — supports A22) and
`abseil.io/resources/swe-book/html/ch09.html` (live; "only one LGTM is necessary", "most code reviews
at Google are reviewed by precisely one reviewer", "about 200 lines of code" — supports A21).

## Findings

- [major][new] Budget resolution, "Which knob binds in the light code phase" + Test strategy #14(a)
  — "both seats dissenting admits a 2-seat pass at 5 spent and closes the phase at 7 — the shape
  where **`maxReviews.code: 6`** binds, on the session-budget record" — the light flow cannot reach
  that record, and 7 is not reachable in that shape. `reviewSessionsUsed` counts per-seat sessions
  (`panel.ts:50-52`, results entries `code-review:<specialty>`), and a dissent keeps the **same**
  round (`states.ts:83`: `review('code', open.length > 0 ? state.codeRound : state.codeRound + 1)`),
  so the both-seats shape is one round of three passes: 2 + 2 + 2 = **6 sessions**, with pass entries
  at spent 0, 2 and 4 — `spent >= cap` (`runner.ts:2357-2359`) never fires, and spent is always even,
  so "5 spent" and "closes at 7" are unreachable. The next `planPass` call returns
  `{kind: 'blocked'}` at `cycle >= cfg.maxPassesPerRound` (`panel.ts:113-119`), and
  `decideNext`'s `case 'code-review'` escalates it (`states.ts:183`): the terminal record is
  `code review: pass backstop: 3 review passes already run in this code round (cap 3 passes per
  round) — the funnel is not converging` at 6 sessions. With 2 seats, a round's 3 passes cost exactly
  the cap, so `maxPassesPerRound: 3` is the record that prints on **every** shape (6 sessions both
  seats dissenting, 4 sessions one dissenter), and `maxReviews.code: 6` never prints. Test #14(a)
  asserts "escalation on the session-budget record at 6 spent … the phase closing at 7", an
  assertion the cited mechanism cannot produce; the implementation will chase it.
  Fix: pick one and name the mechanism. (a) Keep `maxPassesPerRound: 3` and rewrite the paragraph
  to "the pass-backstop record prints at 6 sessions in the both-seats shape and 4 in the one-dissenter
  shape; `maxReviews.code: 6` is the ceiling it coincides with, not the string an operator reads",
  and make #14(a) assert the pass-backstop string at 6 sessions with the light suffix. (b) If the
  design wants the session-budget string to be the light cap's record (the ADR's "Budget exhaustion…
  the reason names the flow"), set light `maxPassesPerRound: 4`, so a pass entry at spent 6 hits
  `spent >= cap` and prints `code review session budget exhausted (6/6 sessions)` — then state the
  per-round band as 4–8 sessions, and drop "closes at 7" (it needs a 1-seat pass leaving 5 spent).
- [minor][new] Budget resolution, "`validated`" — "must keep `planMaxLines` a positive integer" and
  "must carry a non-empty seat list for every phase the flow's `stages` runs" — both clauses have no
  true reading for the **full** flow's own resolved config, which the same section promises is
  untouched. `planMaxLines` is a new optional field absent from `DEFAULT_CONFIG` (`stages.ts:75-84`)
  and from the budget table ("absent (no gate)"), so a resolved full config cannot make it "a positive
  integer", and Test #5 asserts the absent case works. The seat clause is keyed on the flow's
  `stages`, and `full.stages` runs `editorial-review`, while `reviewSeats` is keyed `{ design, code }`
  and `ReviewPhase` includes `'editorial'` (`states.ts:13`, called with `'editorial'` at
  `runner.ts:2322-2345`) — the editorial phase has no seat-list key to validate, and `seatsFor`'s
  signature (`phase: ReviewPhase`) is handed that value by the `specialtiesFor` delegate.
  Fix: write "absent, or a positive integer", and name the phase mapping in `seatsFor` — `editorial`
  reads `reviewSeats.design`, today's `phase === 'code' ? CODE : DESIGN` shape (`panel.ts:31-32`) —
  and scope the non-emptiness clause to the `design`/`code` keys.
- [minor][regression] State graph delta, "`case 'code-review'` backstop exhausted | escalate, naming
  the revise-round backstop", and Budget resolution, "`maxRounds: 2` bounds the mutating `fix` loop,
  whose record is the revise-round backstop" — the code phase has **two** escalation strings and the
  revision renamed the one this row reaches. From `decideNext`'s `case 'code-review'` the string is
  `… and the fix-round backstop is exhausted (2/2)` (`states.ts:185-194`) — r1's wording, which was
  right; the revise-round string `code revise-round backstop reached (2/2 rounds)` comes from the
  runner's round-entry check (`runner.ts:2382-2395`). The row promises "same message **plus**
  `the light flow (flow: light); promote …`", so which sites carry the suffix is now unstated, and
  Test #8 asserts only "escalation naming the light flow".
  Fix: restore "naming the fix-round backstop" in the row, and state that the light suffix is
  appended at both code-phase escalation sites (`states.ts:187-194` and the runner's round-entry
  escalation), naming the two strings.
- [minor][new] Ownership — "the new name `plan` is governed by the **document-stage** rule in both
  flows — writes confined to the run's `docs/designs/NNN-<slug>/` paths" — the cited rule is
  `return inDocs` with `inDocs = /^docs\//` (`gates.ts:304,314-315`): it confines a document stage to
  **any** path under `docs/`, which is how `doc-sync` writes `docs/workflows.md`. A light `plan` can
  therefore write `docs/adrs/…`, and Test #7 ("`plan` allows the run's `docs/designs/**` and denies
  `src/**`") does not assert the run-directory confinement the paragraph claims, so the mitigation in
  Risks row 3 is stated more narrowly than it is.
  Fix: say the document-stage rule is `docs/**` and that the run's own directory is enforced by the
  template/agent config, not the predicate — or name the predicate change that narrows document
  stages to the run's directory and add it to Test #7.
- [minor][new] Ownership + Test strategy #7 — "separator and case variants are settled before the
  predicate sees them" and "a path spelled with a foreign separator or a case variant arrives
  normalized and repo-root-relative, so it gets the verdict of its canonical spelling" — only the
  reserved-name rule is case-insensitive (`namesReservedSegment`, `types.ts:195-207`); `TEST_PATH_RE`
  (`gates.ts:286`) and `inDocs` (`gates.ts:304`) are case-sensitive, so `Tests/x.test.ts` is **allowed**
  under `full` where the canonical spelling is denied, and `Docs/designs/plan.md` is denied where the
  canonical spelling is allowed. The separator half is sound: paths reach the predicate from
  `git diff-tree --name-only -z` and `dirtyFiles` (`runner.ts:261-278`), POSIX and repo-root-relative.
  Fix: keep the separator claim, drop "a case variant … canonical spelling" from the guarantee and
  from #7, and record that case variants of the `tests/`/`docs/` prefixes are settled by the
  case-sensitive regexes, exactly as today.
- [nit][new] Chosen design, "`configFor('full', base)` is deep-equal to a **complete** `base`" — the
  qualifier appears in this one place; the Compatibility row, Rollout 1 and Test strategy #3 all say
  plain "deep-equal to `base`", and #3 is the assertion, so an implementer who passes a partial
  literal gets a failure the design says is impossible. Fix: qualify #3 ("a complete `PipelineConfig`
  literal"), the other two can stay.
- [nit][new] Rollout 6 — "check them against the derived bound — ≤ 8 sessions, ≤ 96 min at the 12-min
  stage bound" — that is Test #12's converging-path bound (`total sessions ≤ 2 + seats × 3`), not a
  number derived from the declared budgets: a light run that goes to the ceiling (plan + implement +
  2 `fix` + 6 review sessions) is 10–12 sessions and 120–144 min, so the first light run that
  legitimately escalates reads as missing the bound. Fix: name it the converging-run bound and cite
  #12's formula, keeping 96 min as the wall-clock ceiling for that path.

## FYI (no severity)

- Selecting the flow relabels the ADR's "a value that is neither `full` nor `light` is a
  source-gate denial" (ADR-010 §Selecting the flow) as a pre-lock usage error; the position the ADR
  requires is preserved and A18 explains why the named gate cannot run pre-lock, so this is the
  ADR/compatibility seat's call, not a correctness finding.
- The Deferred entry "if it lands it counts path-shaped tokens through the parser the ownership
  predicate already uses" names a parser that does not exist: the ownership predicate tests one
  complete path with regexes (`gates.ts:286,293-316`), it extracts no path tokens from prose. If the
  deferral is meant to keep that option open, say the extractor is new code.
