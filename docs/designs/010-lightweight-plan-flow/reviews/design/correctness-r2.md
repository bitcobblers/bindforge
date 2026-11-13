# correctness review r2

scope: `reviews/design/editorial-r2.diff` (the round-2 editorial pass over `02-proposal.md`)
inputs read: `02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md`, the Revision log, the named diff
verdict: request-changes

## Findings

- [blocker][new] The `plan` stage, planner-agent bullet — "`mode: primary`, edits `docs/designs/**`,
  denies `.pipeline.json`, reads and writes `transcripts/**`, and allows shell." — the editorial rewrite
  inverted a denial list into a grant. The pre-diff text ("denies `.pipeline.json`, reading/writing
  `transcripts/**`, and shell") put all three under `denies`, which is exactly the cited basis, the
  `design/designer` shape (A12): `.opencode/agents/design/designer.md` sets `edit`/`read`
  `docs/designs/**/transcripts/**` → `deny` and `shell` `*` → `deny`. The current sentence says the
  planner **reads and writes** transcripts and **allows** shell, so an implementer following the
  proposal as written ships a planner with shell and transcript writes. That is a technical change
  inside a language-only pass, and it contradicts three sections the diff did not touch: this bullet's
  own basis ("the `design/designer` shape (A12), one line narrower"), Ownership ("the widening covers
  exactly `implement`/`fix`", "its code surface is the `docs/**` every document stage already has"),
  and Risks row 3, which records the widened surfaces — shell for a document-stage agent is a widening
  ADR-010's Consequences does not accept. It is also untested: Test strategy #7 covers
  `stagePathAllowed`, not agent permissions, so nothing catches it. Fix: restore the denial scope in
  the same words, e.g. "`mode: primary`, edits `docs/designs/**`, and denies `.pipeline.json`,
  `transcripts/**` (read and edit), and shell" — no new mechanism, it replaces the two verbs
  "reads and writes" / "allows" with the denials they replaced.

## Checklist notes (no finding)

- Assumption register: every row the diff touches (A12 as cited basis for the planner shape) is the
  one place the pass broke a `verified` row; no unverified assumption is stated as fact in the
  changed text (A23/A24/A25/A26 stay `unverified`, and the changed Rollout 6 sentence keeps them as
  the measurement target).
- URL spot-check (anti-hallucination gate): `abseil.io/.../ch09.html` (live; supports "one LGTM",
  "one and only one reviewer", "about 200 lines"), `smartbear.com/...best-practices-for-peer-code-review/`
  (live; "lightweight code review takes less than 20% the time of formal reviews and finds just as
  many bugs"), `google.github.io/eng-practices/.../small-cls.html` (live; "The number of files that a
  change is spread across also affects its 'size'" — A22's files-spanned signal). No dead or
  contradicting source among the rows the diff cites.
- The other nine hunks are meaning-preserving: the Oxford comma (Goals), "the same pre-lock posture
  as a bad `flow` value" (Budget resolution, same claim as the pre-diff "bad-`flow` posture"),
  "starts with 4 sessions spent" (keeps the 2+2+2 max case), "a `--stage code-review` force run after
  a round that died to the backstop at 6 spent sessions" (keeps the phase-cumulative reading), the
  added "on" before `state.lastStage` (State graph delta), "the latter" for `testOwnership`,
  "heavier flow"/"heavy flow" → "full flow" (matches the escalation string `set flow: full`), the
  Rollout 1 comma, and "plus 1–2 sessions for each `--extend-budget` grant the operator buys"
  (state-side grants per A27, additive to the ceiling).
- Deferred section: unchanged by this diff; every deferral still matches ADR-010's Deferred list and
  Consequences — nothing re-opened.
- Revision log: current, one entry per r1/r1c1–c4 decision, including the deliberate closures
  (`maxReviews.code: 12` not chosen, the rejected seat-list design kept in Options, the accepted
  `docs/**` widening, Windows/macOS `no`). The blocker above is not among them and is not a rejected
  finding re-raised.

## FYI (no severity)

- Compatibility/consistency seat: the proposal's Context says "the six completed runs spent 43–96
  stage sessions each"; ADR-010's Context says "The seven runs on disk" above a six-row table whose
  range is 43–96. The proposal's number matches the table; the ADR's word "seven" is the stale side.
- Performance seat: Rollout 6's escalated ceiling "≈ 9 sessions — plan 1, implement 1, the fix
  rounds, and at most the code phase's 6 cumulative" sums to 10 with `maxRounds: 2` counting two fix
  rounds; the text is outside this diff (reflowed only), so it is not a regression from this pass.
