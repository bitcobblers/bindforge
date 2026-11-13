# Security review r4

verdict: approve

Scope: the editorial r4 diff (`reviews/design/editorial-r4.diff`) — the single
editorial confirmation pass of round 4. Inputs used: `02-proposal.md`,
`docs/adrs/010-lightweight-plan-flow.md`, the Revision log (including the two
appended Editor notes), and the named diff. No prior security findings existed
for this seat (the editorial phase is one language pass per round), so this is a
ratification read: every changed hunk was checked for a security claim it made
false, not for new themes.

## Findings

None. Zero blocker, major, minor or nit.

Confirmation per changed hunk (security-relevant text only):

- Ownership — "POSIX separators, no absolute path and no `.`/`..` segment,
  shapes `moves.ts` Rule 1 rejects outright": the inserted noun *path* keeps the
  A28 intake claim intact (the only intake is git plumbing output —
  repo-root-relative, POSIX separators, absolute and `.`/`..` spellings cannot
  reach the predicate), the sentence's conclusion "so the `flow` parameter parses
  no path; case is **not** normalized — as today" still follows from it, and the
  `moves.ts` Rule 1 rejection is still named. Nothing is weakened: no absolute
  path, dot-segment or shell interpolation is introduced, and #18's
  `docs/../src/x.ts` case still asserts git's real-path normalization at the
  sweep. Checklist item 3: pass.
- Ownership — "full keeps `!inTests`, and test-engineer stages and document
  stages are unchanged": the added conjunction leaves the precedence intact —
  `namesReservedSegment(path)` "stays first and wins in both flows" — and the
  widening stays bounded by "The widening covers exactly `implement`/`fix`".
  The accepted `plan` widening to `docs/**`, pinned as **allowed** in #7/#18 and
  recorded in the Ownership Risks row, is untouched, so the design still claims
  no confinement the code does not perform. Checklist items 1, 6, 7: pass.
- Budget resolution — "`maxPassesPerRound: 3` is the knob that binds, and it
  prints the same record on **every** shape of the round it starts": the added
  clause matches the section heading ("Which knob binds") and the r1 revision-log
  closure (4–6 sessions at two seats, printing the pass-backstop record). The
  arithmetic is unchanged — 6 sessions both dissenters, 4 one dissenter
  (`panel.ts:113-119`), cap checked at pass entry (A16) — so the
  budget-exhaustion stop stays deterministic and bounded, and the `--stage
  code-review` force-run escalation at `runner.ts:2362-2368` is unaffected.
  Checklist item 7: pass.
- Goals — "Light `implement`/`fix` owning test paths" and "`pipeline status` and
  `ESCALATION.md` naming the flow and the effective numbers": gerund form only,
  in parallel with the neighbouring bullets. Test-path ownership is still pinned
  by #7/#18, the `test-update` routing guard is still the State-graph delta, and
  the effective numbers still resolve through the same `configFor`. No grant is
  added and none is removed.
- Context — "a brief takes the same twelve-stage, five-seat path"; Rollout 6 —
  "and against run 003's 43 sessions / 2.2 h": motivation and measurement
  comparisons; no security surface.
- Appended Editor notes (Revision log): author-directed clarification requests.
  They add no mechanism, no path, no privilege, and no data flow. No security
  surface, and the ambiguities they raise are pointer/clarity issues (see FYI).

Checklist coverage, scoped to this diff: (1) trust boundaries — untouched,
`shared binding file` / untrusted-config classification unchanged; (2) parsing
untrusted data — unchanged: the pre-lock `flow` value gate, `validated`'s seat
panel, `planMaxLines` absent-or-positive rule, and "Unreadable files keep their
existing `sourceAdmissible` denial" are all outside the diff and unmodified;
(3) path handling — re-read above, claim intact; (4) secrets/PII — untouched,
nothing newly logged, `ESCALATION.md` content unchanged; (5) dependency
advisories — not applicable to this diff: it adds no dependency, and the
proposal names no new one (A20, "No new platform surface"), so there is no
advisory to webfetch for this scope; (6) privileges — `planner.md` copying
`designer.md:4-22` verbatim and excluding `ops/doc-sync`'s `git diff*` allow is
unmodified, least privilege unchanged; (7) fail-closed — the pre-lock usage-error
posture (no lock, no design directory, no session, zero client calls, #17) is
unmodified.

## FYI (no severity)

- Editorial/correctness: the two appended Editor notes are still unanswered. The
  Budget resolution line "`states.ts:152` keeps `DESIGN_SPECIALTIES` (editorial
  only, Risks row 4)" cross-references the resume budget-drift row, while the
  editorial-round exposure it refers to is the "Two seats are the whole net" row
  (the first row).
- Editorial/correctness: "`maxRounds: 2` bounds the mutating `fix` loop, on the
  runner's revise-round record" does not say which record a light `fix`
  exhaustion prints, given the neighbouring paragraph says a non-converging code
  round prints the pass-backstop record.
- Editorial: the two "Editor notes" paragraphs sit inside `## Revision log` as
  prose, not in the `r<N>: <source> — <resolution>` shape of the entries above
  them.
- Not re-opened, outside this diff and consistent with ADR-010's Consequences:
  the `plan` stage's `docs/**` write surface (a light `plan` rewriting
  `docs/adrs/**` unreverted) stays an accepted widening pinned as **allowed** in
  #7/#18 — the design claims it as accepted, not as confined.
