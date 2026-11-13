# security review r1c1 (dissent pass on the r1 revision)

verdict: approve

Scope reviewed: the `9ae29b5..24b804a` diff of `02-proposal.md`, then the full
current document, `docs/adrs/010-lightweight-plan-flow.md`, the Revision log,
and my own r1 findings. Checklist applied: `references/security.md`.

## Closure of my r1 findings

- **[major, seat panel is untrusted config data] — closed.** The Options entry
  now states the premise ("A panel that lives in config is untrusted data, so
  the resolver that reads it validates it"), and Budget resolution makes
  `validated` part of the `configFor` contract: known specialties only, a
  non-empty seat list for every phase the flow's `stages` runs, `planMaxLines`
  a positive integer, and a throw at the bad-`flow` posture — i.e. fail closed,
  pre-lock, with no lock, no directory, no session. The bare "fall back to the
  five specialties when absent" clause is gone from the contract; the fallback
  now lives once in `seatsFor`, on the path that is not `configFor` output, and
  its fail direction (a light call site fanning out to five seats) is *more*
  audit, not less, and is recorded as a residual in Budget resolution and in
  the Risks row. `reviewSeats.code: []` and `['corectness']` are named in the
  text as the shapes the check kills, and Test strategy #23 pins empty-panel,
  unknown-specialty, `planMaxLines: 0` and flow-over-base precedence. Spread
  order (`...base` then `...FLOWS[flow].config`) also means a caller cannot
  shrink the light panel below the registry's two seats.
- **[minor, `plan` absent from the stated code rules] — closed.** Ownership now
  says the widening covers exactly `implement`/`fix` and that `plan` is
  governed by the document-stage rule in both flows with
  `namesReservedSegment` first, and Test strategy #7 names `plan` in the
  reserved-name enumeration for both flows plus the `docs/designs/**`-allow /
  `src/**`-deny pair; #18 pins it in a real sweep. One caveat on the new
  path-normalization premise is below.
- **[minor, file gate fails open] — closed by deletion.** `planMaxFiles` and
  `planOutgrewFlow(lines, files, cfg)` are gone; the gate is
  `planOutgrewFlow(lines, cfg)`, the ADR's one plan stop, and the "missing
  section counts as 0 items" text no longer exists. The deferral is consistent
  with ADR-010 ("Two deterministic stops": oversized plan by line count, budget
  exhaustion; Consequences accepts that nothing in the tool stops an operator
  picking `light` for a wide-consequence change). Crucially the Risks row that
  was false as written — "`planMaxFiles` catches files-spanned" — now reads
  "The 80-line plan gate stops a plan that outgrew the flow … the ADR's
  accepted cost", so the mitigation no longer claims a stop the design deleted.
- **[nit, two frontmatter reads] — closed.** Selecting the flow parses once,
  carries the parsed frontmatter on the `deriveIdentity` result, sets
  `state.flow` from **that one parsed value**, and passes the same result to
  the post-lock `sourceAdmissible` gate: the value that cleared the pre-lock
  check is the value stamped and printed.

## Findings

- [minor][new] Ownership — the security spine, widened in one flow — "Paths
  reach `stagePathAllowed`/`ownershipIssue` repo-root-relative and
  POSIX-normalized (A8/A9), so the `flow` parameter parses no path: separator
  and case variants are settled before the predicate sees them." — this is the
  load-bearing premise for the section's path-safety argument and for the new
  Test strategy #7 clause, and it is cited to A8/A9, neither of which records
  it: A8 records the predicate's shape (`namesReservedSegment` first,
  `TEST_PATH_RE` denials) and A9 records that move validation reuses the same
  predicate. Nothing in the register names the code that canonicalizes,
  rejects absolute paths, and rejects `..`, so a reader of this design cannot
  confirm the claim the security argument rests on — and the design states no
  behavior for a path that arrives *non*-canonical (checklist 3, 7:
  canonicalization must be a stated property, and malformed input must fail
  closed, not inherit the verdict of a spelling the predicate did not intend).
  Test strategy #7 compounds it: the clause "a path spelled with a foreign
  separator or a case variant arrives normalized and repo-root-relative, so it
  gets the verdict of its canonical spelling" is an assertion about the
  ingestion plumbing, and #7 is a unit test of the predicate, which cannot
  observe what it is handed. Fix: add one assumption row (verified, with the
  normalizing/rejecting site: the path intake feeding `runner.ts:279` and
  `moves.ts`) stating that paths reach the predicate repo-root-relative,
  POSIX-normalized, with absolute and `..` forms denied there, and change the
  Ownership sentence's "(A8/A9)" to cite that row; then move the
  separator/case-variant/`..` assertion into the integration test that drives
  the real sweep (#18), leaving #7 to assert the predicate's verdict on the
  canonical spelling.

## FYI (no severity)

- Rollout 6's target "≤ 8 sessions, ≤ 96 min" sits below the design's own
  derived worst case: a code phase that closes at 7 sessions (Budget
  resolution, Test strategy #14a) plus `plan`, `implement` and up to 2 `fix`
  stages is more than 8 sessions, so the serial-default 12-min bound is a
  target, not the bound the arithmetic supports (performance seat).
- With the file gate deleted, the plan's `Files to touch` section has no code
  consumer at all — the runner never reads the plan's shape, only its line
  count — so the five-section template is enforced by seat faithfulness alone
  (correctness/complexity seat).
