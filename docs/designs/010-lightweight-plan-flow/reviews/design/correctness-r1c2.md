# correctness review r1 (dissent pass c2)

verdict: approve

Scope: the diff `24b804a..4cc9acc` of `02-proposal.md`, then the full current
document. Inputs: the proposal, `docs/adrs/010-lightweight-plan-flow.md`, the
Revision log, my r1c1 findings. Checklist: `references/correctness.md`.

## Round-1 findings — status

- **major (which knob binds)** — **closed for real.** The rewritten paragraph
  matches the code: a dissent keeps the same round (`states.ts:83`
  `review('code', open.length > 0 ? state.codeRound : state.codeRound + 1)`),
  `planPass` returns `{kind:'blocked'}` at `cycle >= cfg.maxPassesPerRound`
  (`panel.ts:113-120`), and `decideNext`'s `case 'code-review'` calls `planPass`
  at `states.ts:181` and escalates on blocked at `states.ts:183` — and
  `decideNext` runs at the top of each drive-loop iteration, ahead of the
  pass-entry cap check at `runner.ts:2357`. So the both-seats-dissent shape
  reaches spent 6 / cycle 3 and dies on the pass-backstop string, the
  one-dissenter shape at spent 4, and the session-budget string
  (`runner.ts:2365`) never prints. "4–6 sessions", "the last pass a round can
  enter starts at spent 4" and "`maxReviews.code: 6` … not a string an operator
  reads" all check out; #14(a) now asserts the pass-backstop record. Only the
  *stated reason* for "never fires" is wrong — see finding 1.
- **minor (`validated` clauses)** — **substantially closed** (the mapping is
  named, `planMaxLines` is now "absent or a positive integer", the full flow's
  absence is stated, #23 updated). The non-emptiness clause and #23 still
  collide on `reviewSeats.design` — finding 2.
- **minor (two code-phase escalation strings)** — **closed.** The row reads
  "naming the fix-round backstop" (matches `states.ts:187-194`), and the
  suffix sentence names the pass-backstop site (`states.ts:183`) and the
  runner's round-entry string, whose text is verbatim
  `${phase} revise-round backstop reached (N/M rounds)` (`runner.ts:2390-2393`).
  Enumeration is incomplete on one site — finding 3.
- **minor (`plan`'s confinement)** — **closed** (`return inDocs`,
  `inDocs = /^docs\//` at `gates.ts:304,315`, doc-sync's `docs/workflows.md`
  named, #7 reworded). One residual overstatement — finding 5.
- **minor (case variants)** — **closed** ("case is **not** normalized — as
  today", #7 scoped to `tests/`/`docs/`; `TEST_PATH_RE` at `gates.ts:286` and
  `inDocs` at `gates.ts:304` are case-sensitive). One blanket overreach —
  finding 4.
- **nit (`complete` base)** — **closed** (#3 now says "a **complete**
  `PipelineConfig` literal `base`").
- **nit (Rollout 6 bound)** — **closed** (named the converging-run bound,
  cites #12's `2 + seats × 3`, escalated runs sit above it).

## Findings

- [minor][new] Budget resolution, "Which knob binds in the light code phase" —
  "The session cap is checked at **pass entry**, ahead of that backstop (A16),
  so it never fires" — the conclusion is true, the stated mechanism is not. The
  cap check (`runner.ts:2357-2359`) *is* ahead of the in-loop `planPass`
  (`runner.ts:2422`), and that ordering is why the cap **would** fire at spent 6
  — read literally, the sentence predicts the session-budget string, which is
  exactly what #14(a) asserts never prints. The reason it never fires is the
  other side of A16: `decideNext` evaluates `planPass` at `states.ts:181` and
  escalates the blocked (pass-backstop) reason at `states.ts:183`, and
  `decideNext` runs before the review branch's cap check on every loop
  iteration, so the iteration at spent 6 ends in escalation and never reaches
  pass entry. Fix: replace "ahead of that backstop (A16), so it never fires"
  with "the pass backstop is decided upstream, in `decideNext`'s
  `planPass` call (`states.ts:181-183`), before the loop reaches the pass-entry
  cap check (`runner.ts:2357`), so the iteration at spent 6 escalates on the
  backstop and no pass entry happens at the cap".
- [minor][re-raised] Budget resolution, "`validated` is part of the contract" +
  Test strategy #23 — "must keep the two keys `seatsFor` reads non-empty —
  `reviewSeats.code`, and `reviewSeats.design`" and "`reviewSeats.code: []` and
  an empty `reviewSeats.design` throw" — the light partial in the budget table
  sets `reviewSeats.design` to `[]`, so *every* light resolved config has an
  empty design list; #23 as written has no flow qualifier, and an implementer
  who resolves `configFor('light', …)` and asserts the throw gets a failure the
  same section calls legal ("`light`'s `design: []` is legal only because
  `light.stages` names no design stage"). The legality clause states a reason,
  not the mechanism the validation keys on. Fix: key the clause on the flow's
  stages — "`reviewSeats.design` non-empty for a flow whose `stages` run a
  design or editorial stage (full's `proposal-final` → editorial reads it
  through `states.ts:25-26`), exempting `light`'s registry `[]`" — and scope
  #23's design assertion to a `full`-flow resolution.
- [minor][new] State graph delta — "The light suffix is appended to every
  code-phase escalation the light flow reaches: `decideNext`'s pass-backstop
  string (`states.ts:183`) and fix-round string (`states.ts:187-194`), and the
  runner's round-entry `code review revise-round backstop reached (2/2
  rounds)`" — "every" and the three-site list disagree: a fourth code-phase
  escalation is reachable in light and is not listed,
  `escalate('tests not green at the code-review budget limit')`
  (`states.ts:170-178`, from `case 'implement'`/`case 'fix'` with
  `testsGreen: false` once `codeBudgetUsed >= maxRounds`). Light is the flow
  where that stop is likeliest (no test engineer, A13/A14), and ADR-010's
  Status section requires an escalated light run to read as "too big for the
  light flow". Fix: add that string to the enumeration (naming
  `states.ts:172,178`), or narrow the sentence to "the three review
  escalations" and add a row for the red-tests limit.
- [nit][new] Ownership — "Paths reach the predicate repo-root-relative and
  POSIX-normalized (A8/A9), so the `flow` parameter parses no path; case is
  **not** normalized — as today." — as a blanket statement this is false for
  the rule the same paragraph puts first: `namesReservedSegment` lowercases
  each segment (`types.ts:195-207`), which is how the code comment at
  `gates.ts:294-300` reads ("no stage class … can own them at any depth or
  under another case"). Fix: scope it — "case is not normalized for the
  `tests/`/`docs/` regexes, as today; the reserved-name rule is
  case-insensitive by design".
- [nit][new] Ownership vs. The `plan` stage — "confining `plan` to the run's
  own directory is template and agent-config work, not predicate work" vs. the
  planner agent spec "edits `docs/designs/**`" — the named agent config grants
  every run's directory, so the mitigation Risks row 3 leans on is not
  implemented by the config the design specifies. Fix: say the planner's grant
  is `docs/designs/**` and that run-directory confinement is the template's
  `{{designDir}}` output path only (no predicate, no agent rule), or narrow
  the planner grant to the run's directory.
- [nit][new] Budget resolution vs. State graph delta — "`maxPassesPerRound: 3`
  is the record that prints on **every** shape" — the State graph delta says of
  the same phase that the deadlock stop "is seat-count-agnostic and fires at 2
  seats; with `maxRounds: 2` it is likelier to be the terminal record", and
  `planPass` returns the `deadlock:` blocked reason from the same site
  (`panel.ts:121-129`), so the pass-count record is not the terminal record on
  that shape. Fix: qualify to "every pass-count shape that is not the deadlock
  stop (`panel.ts:121-129`)".

## FYI (no severity)

- Goals' "Two deterministic stops for a too-big light run: oversized plan,
  budget exhaustion" now sits beside a Budget resolution that says the terminal
  record on the common shape is the pass backstop (a third stop, from the
  ADR's own `maxPassesPerRound: 3` row). The ADR's clause ("`maxRounds: 2`
  spent in the code phase escalates as today, and the reason names the flow")
  is met by the two round records plus the suffix, so this is a framing point
  for the compatibility/complexity seats, not a contradiction.
- Deferred's new parenthetical ("the ownership predicate tests one complete
  path; it extracts nothing from prose") is accurate against `gates.ts:293-316`
  and closes the r1c1 wording cleanly.
