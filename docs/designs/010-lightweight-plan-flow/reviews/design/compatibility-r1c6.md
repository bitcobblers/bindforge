# Compatibility review r1c6

Scope: the diff `4cc9acc → c1b351d` (blob ids of `02-proposal.md`) plus the full
current document. Ratification pass: I report only what the revisions broke of
what I approved in r1, or what makes a section I cited false.

verdict: approve

## Checklist walked (compatibility)

1. **Matrix completeness** — every row carries a status; no `unknown`, no empty
   cell. The `Windows / macOS` row reads `no` (the r1 `unknown` closure held),
   and the `Node older than the engines floor` row reads `no`. Nothing in the
   diff touched a status cell.
2. **OS input APIs** — not engaged: the diff adds no platform surface, and A20
   plus the matrix's first row justify it as registry/config/TS only.
3. **Node floor** — `>= 22.23.4` named (A20, matrix row 1). The new text names
   git plumbing (`diff-tree --name-only -z`, `status --porcelain=v1 -z`, A28)
   and the JS line count at `runner.ts:179-180`, not a Node API above the floor
   and not a shell-out.
4. **Native modules** — none introduced; no packaging story owed.
5. **Keyboard reality** — not engaged.
6. **File paths** — the revised Ownership paragraph plus new A28 state the
   intake (repo-root-relative, POSIX separators, no absolute, no `.`/`..`
   segment; `moves.ts` Rule 1 rejects those shapes as usage errors) and keep
   case **unnormalized as today**, pinned in #7 and #18. That is the
   path-hygiene story the `inDocs = /^docs\//` prefix rule needs, and it is now
   asserted in both directions; on the only supported platform (linux) case
   sensitivity is the filesystem's, and the matrix claims no case-insensitive
   platform. No finding.
7. **Platform test per supported platform** — the matrix's only supported
   platform is linux at the floor; #21 names that job. Satisfied.

## Findings

- [nit][re-raised] Compatibility matrix / Rollout 1 — "`configFor('full', base)`
  is deep-equal to `base`; the twelve `STAGE_BASES` stages, 5 seats, budgets
  unchanged" and "this is a no-op for every run (#3)" — the registry paragraph
  narrowed the claim to "deep-equal to a **complete** `base`, #3", and #3 pins a
  partial literal by compile-plus-five-seat resolution, not by deep equality. The
  matrix row keeps the unqualified `base`, and Rollout 1 now cites #3 for a
  "no-op" claim that the sentence in the registry paragraph scopes. Nothing is
  false for the complete CLI-shaped config both sentences describe, so this is
  citation precision only. Fix: borrow the qualifier in the matrix row — "deep-equal
  to a complete `base`" — and let Rollout 1 read "no-op for every run (a complete
  `base` is deep-equal, #3)".
- [nit][re-raised] Test strategy #21 — "no macOS or Windows job exists, so both
  compatibility rows read `no` and no test claims them" — the matrix carries one
  merged row, `| Windows / macOS | no | …`, so a reader cross-checking the anchor
  finds one row where the test cites two. The status claim matches that row and
  the Deferred CI bullet. Fix: name the row — "so the `Windows / macOS` row reads
  `no` and no test claims it".

## Closed from my r1 list

- `reviewSeats.design: []` vs. `validated` (minor) — **closed by this diff.** The
  canonical block now reads "must keep `reviewSeats` non-empty **for every phase
  the flow's `stages` runs** — `code` for every flow … and `design` under `full`",
  and #23 states "`reviewSeats.code: []` throws for **every** flow, an empty
  `reviewSeats.design` throws under `full` and **resolves** under `light` (the
  registry's own `[]`, as #1 pins — the check is scoped to the phases the flow's
  `stages` runs)". That matches the knob table's `design: []` for light and keeps
  #2, #12, #15 and #23 consistent; the flow-scoping lives in the canonical
  signature line (`validated({...}, flow)`) as well as the prose.
- The `Windows / macOS` status (r1 `unknown`) stays closed: the row reads `no`,
  the Deferred bullet defers the CI job, and nothing in the diff adds a
  platform-specific path.

## FYI (no severity)

- Risks, Ownership row — "ADR-010's Consequences records the widened surfaces"
  over-states the source: `docs/adrs/010-lightweight-plan-flow.md` Consequences
  records "`stagePathAllowed` … gains a flow parameter, widening **the coder's**
  write surface in one flow", and says nothing about a `plan` stage reaching
  `docs/adrs/**` or a sibling run's directory. Rollout 4 states the same
  reference accurately ("the coder's widened surface"). The claim belongs to the
  correctness seat; the widened `plan` surface itself is a security-surface
  matter, and the design now pins it as allowed in #7/#18 rather than claiming a
  confinement, which is the honest shape.
- Rollout 6 — "a run that escalates legitimately sits above it, at ≈ 9 sessions —
  plan 1, implement 1, the fix rounds, and at most the code phase's 6 cumulative"
  lists parts that sum to 10 at `maxRounds: 2` (1 + 1 + 2 + 6). The `≈` keeps it
  non-falsable; flagging the arithmetic for the performance seat.
- Options — "each rejected there on the ownership-matrix and session-count
  grounds" compresses ADR-010's five rejections; "no plan artifact" and `lane`
  were rejected on audit-chain and naming-collision grounds. Source-fidelity,
  not compatibility.
