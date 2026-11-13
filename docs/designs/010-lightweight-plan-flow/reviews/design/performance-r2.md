# performance review r2

verdict: approve

Ratification scope: the editorial r2 diff only (`reviews/design/editorial-r2.diff`),
read against `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md` and the
Revision log. No performance finding in r1 (the editorial phase is a single
language pass per round), so the test applied is: does any revised line break a
cost claim, a bound, a number, or a baseline comparison?

Checked, performance lens, every hunk in the diff:

- Goals / Budget resolution (`transcripts, and reply contract unchanged`;
  `A violation throws at the same pre-lock posture as a bad flow value`) — comma
  and phrasing only; no cost, cap, or path-count claim moves.
- Budget resolution seat-reader paragraph (`a light flow's full pass launches the
  flow's two seats`) — parse fix; the launch count (two seats) and the
  no-leftover-`*_SPECIALTIES`-import claim are unchanged.
- "Which knob binds" (`starts with 4 sessions spent`; `a --stage code-review
  force run after a round that died to the backstop at 6 spent sessions
  escalates`) — the cost arithmetic is intact: 4–6 sessions per non-converging
  round, pass-backstop at 6 (both dissenters) / 4 (one dissenter), cap checked at
  pass entry, phase-cumulative `maxReviews.code: 6`, `maxReviews.code: 12`
  declined. No number changed.
- State graph delta (`and on state.lastStage in implement/fix/code-review`) —
  grammar; the escalation site count stays **one** (`finalizeEscalate`), which is
  the session-cost-relevant fact.
- The `plan` stage (agent permission verbs; `the full flow's path`; `the latter
  rendered`) — permissions/terminology; `planMaxLines` stays registry-owned,
  interpolated into the prompt, gated at the `proposalSize` site, and the
  meter stays the JS line count, so no per-stage extra read.
- Risks row (size-policing) and The `plan` stage (`full flow` for `heavier
  flow`) — terminology only; the 80-line cap and the escalation strings' content
  are unchanged.
- Rollout 1 (comma) and Rollout 6 (`plus 1–2 sessions for each --extend-budget
  grant the operator buys`) — the measurement sentence reflows without changing
  the per-grant cost. The numbers the performance checklist cares about all
  survive verbatim: `2 + seats × 3` = ≤ 8 sessions, ≤ 96 min at the 12-min stage
  bound in the serial default, escalated ceiling ≈ 9 sessions, and the baseline
  comparison against run 003's 43 sessions / 2.2 h (ADR Context table: 43
  sessions, 2.2 h — design quotes it, no unexplained gap).

No revised line introduces allocation, synchronous IO, table rebuild, unbounded
growth, a cache/invalidation change, a startup read, or a changed budget number.
The only startup-path edit is the wording of the config-validation throw, which
the text now places at the same pre-lock posture as the flow-value check — the
cheap side of the CLI path, consistent with the design's one-frontmatter-read-per-
entry rule.

## Findings
(none — zero open blocker and major findings)

## FYI (no severity)
- Budget resolution — "A violation throws at the same **pre-lock posture** as a bad `flow` value" —
  the revised wording now asserts a throw *position* for `validated`, not just that it
  throws; whether `configFor` is genuinely reached before the lock in `runPipeline` /
  `resumePipeline` / `status` is a correctness question, not a cost one (no sessions are
  spent at a usage error either way, so no performance exposure here).
