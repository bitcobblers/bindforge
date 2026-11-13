# performance review r4

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r4.diff` (round 4, single language pass),
against `02-proposal.md` and `docs/adrs/010-lightweight-plan-flow.md`. My seat reported no
findings in the editorial phase of earlier rounds (one language pass per round), so there is no
section I cited that a revision can break; I checked every hunk for a change in technical meaning
under the performance checklist, and read the Revision log first — the two `Editor notes:` lines at
the tail of the proposal are the author's own open clarifications (the Risks-row cross-reference and
the `maxRounds: 2` sentence), not unrecorded findings, and neither is a cost claim.

## Checklist walk (all six items evaluated)

1. **Input-path latency / per-event cost.** The design's event path is the runner loop, and every
   changed hunk is prose. The design keeps cost off that path where it matters: budgets "resolved
   once from `state.flow`" through `configFor`, "one frontmatter read per entry" at entry (the
   post-lock gate "takes the same result as an argument instead of re-reading"), and the five seat
   readers "take the `cfg` their callers already hold" rather than re-resolving per call. No
   allocation, table rebuild, or synchronous IO is added per stage, per pass, or per path.
2. **Structure costs, O(n²) over unbounded n.** `n` is bounded by registry data everywhere: 4
   light stages, 2 seats, `maxPassesPerRound: 3`, `maxReviews.code: 6`, `maxRounds: 2`. The
   ownership predicate stays linear per path in the sweep, and the hunk at Ownership preserves the
   property that keeps it so — "the `flow` parameter parses no path" — the rewrite to "no absolute
   path and no `.`/`..` segment" names the same shapes `moves.ts` Rule 1 rejects, so no per-path
   normalization enters the predicate.
3. **Memory growth.** No new cache or translation table: the registry is a static literal, and the
   plan cap gates a document at 80 lines, which bounds the artifact the meter reads. Nothing to
   invalidate, so nothing unbounded to grow.
4. **Startup cost.** Entry does one source parse, pre-lock, carried on the `deriveIdentity`
   result — strictly cheaper than a second read, and it is the diff's first hunk's neighbourhood;
   the wording change ("pays" → "takes") leaves the cost claim untouched.
5. **Caching and invalidation.** The seat stamp is the `02-proposal.md` blob id (A5) and stale
   approvals take a ratification pass; the invalidation defect this design can hide — a resumed
   light run inheriting `maxRounds: 5` / `code: 20` and looking healthy — is closed by the single
   resolver plus Test strategy #15 asserting the **effective** config, not the flow name.
6. **Baseline vs. expected numbers.** The design quotes the baseline (the six completed runs at
   43–96 sessions; run 003's 43 sessions / 2.2 h) and its expectation (≤ 8 sessions, ≤ 96 min at
   the 12-min stage bound) with the gap explained by shed stages and seats, and Rollout 6 measures
   it. The hunk at Rollout 6 only adds the conjunction "and against run 003's 43 sessions / 2.2 h",
   which makes the comparison grammatical without changing the arithmetic: #12's `2 + seats × 3` =
   8 sessions × 12 min = 96 min checks out, and the reworked Budget-resolution sentence keeps its
   session arithmetic sound — `maxPassesPerRound: 3` × 2 seats = 6 sessions (both dissenting), 2 +
   1 + 1 = 4 (one dissenter), matching Test strategy #14(a)/(b). The new clause "it prints the same
   record on **every** shape" is the same pass-backstop record at 6 and at 4 sessions, which is
   what #14 asserts, and the following sentence ("the session cap is checked at **pass entry**,
   ahead of that backstop") keeps the cumulative cap as the knob that binds at a *later* round
   entry, so calling `maxPassesPerRound` "the knob that binds" inside the first round is not
   contradicted by the table's `code: 6`. No unexplained gap between the design's numbers and the
   baseline.

## Findings

None. Every changed line is a grammar/clarity edit; none introduces a cost on a hot path, changes a
bounded quantity, breaks the resolution seam, or moves a number off the research baseline.

## FYI (no severity)

- Context — "a brief takes the same twelve-stage, five-seat path as an ADR": the ADR's Context says
  "the identical 13-stage path". The proposal's Revision log closed the count as "the twelve
  `STAGE_BASES` rows", so the two documents state the stage count with different units (registry
  rows vs. the ADR's diagram path). Pre-existing on the `a` side of this diff, not touched by the
  edit; worth one reconciling clause by the compatibility seat so the cost claim and the source
  document agree in words.
- Budget resolution — "the last pass a round can enter starts with 4 sessions spent" is exact for
  the both-seats-dissenting shape (2 + 2) and reads as the floor for the one-dissenter shape (3
  spent at that entry). Untouched by this diff; complexity/complexity-adjacent reading.
- The proposal's tail now carries two `Editor notes:` lines addressed to the author. They are
  editorial process artifacts living inside `02-proposal.md`; where they belong in the run's record
  is the editorial seat's call, not a cost concern.
