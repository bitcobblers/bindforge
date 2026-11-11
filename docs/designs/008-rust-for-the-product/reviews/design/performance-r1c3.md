# performance review r1c3

verdict: approve

Ratification pass c3: checked only whether the r1 revisions broke what this
seat approved in r1, against the r1 performance findings and the named diff
(04a4ae1 → 796b2a6), then read the full current document.

## Findings

None. The revisions did not break anything this seat approved.

Verification against the r1 finding and the checklist:

- r1 minor (reach-table load point unstated): the cited sentence — "Reach-class
  tables are loaded from `schemas/` data by `bindforge-core`, per ADR-009" — was
  **deleted** from the Harness paragraph and replaced with a test-scope
  sentence: "Reach-class table fixtures are asserted by load + distinctness
  (Test strategy #4), not re-serialization." Test #4 still pins load-into-the-
  in-memory-model. The r1 hot-path worry ("resolution is the keystroke/event
  path") is mooted by the new Context paragraph: "BindForge reads and writes
  binding *files* only ... it never reads or injects input events." The runtime
  load point remains unstated — unchanged open-endedness versus the r1 state
  this seat approved with a minor, not a regression, and the Revision log
  records the fixture-layout change without rejecting the finding; carried as
  FYI below, not re-gated on a ratification pass.
- Checklist 1 (input-path latency): no per-event cost introduced; the Context
  statement removes the event path from the product's scope entirely. The new
  parse budget ("rejects documents larger than 1 MiB or nested deeper than 16
  table/array levels with typed errors") bounds per-file parse cost and is a
  performance improvement.
- Checklist 2 (O(n²)): nothing changed introduces quadratic structure; the new
  majors↔`schemas/*/v*/` equality assertion is a finite test-time directory
  check, not a runtime path.
- Checklist 3 (memory growth): bounded by the 1 MiB / 16-level input budget and
  the retained `MAX_CHAIN_DEPTH` guard; Test #3's pin to the constant instead
  of the literal 32 cannot drift the bound.
- Checklist 4 (startup cost): the `[profile.dev.package.sqlx-macros]` block and
  A6 deletions are consistent — sqlx does not enter `[workspace.dependencies]`
  in this design, so the block was dead config for a dependency that doesn't
  exist yet; no boot-path reads added.
- Checklist 5 (caching/invalidation): the `.sqlx` offline-cache decision stays
  in Options considered, and the Deferred entry hands the *mechanism together
  with* its invalidation job ("the sqlx offline-cache mechanism and the
  hub-path `cargo sqlx prepare --check` job — owned by ADR-014") — mechanism
  and cache-freshness check travel together, no orphaned staleness risk. The
  deferral does not contradict ADR-008: clauses 2–3 fix axum+Postgres via sqlx
  and `schemas/` as contract but say nothing about compile-time SQL
  verification or its CI posture.
- Checklist 6 (research baseline): the changed text makes no expected-number
  claims that could diverge from a baseline; the 1 MiB / 16 / 32 values are
  input guards, not throughput claims.

## FYI (no severity)

- Carryover from r1, still open but unchanged in substance: the runtime load
  point of the reach-class tables is not pinned — the member table still lists
  `crates/core` owning "reach-table loader" as "Created in: this design" while
  the only remaining load statement ("tables load from
  `schemas/reach-classes/v1/` fixture data into the in-memory model", Test #4)
  is test-scope. A one-line pin in ADR-009 (or here) at implementation time —
  embedded-at-build vs packaged-file read, loaded once — closes it; it does
  not gate this pass.
- Excluding `cargo deny` per-PR scanning keeps a live-network advisory DB off
  the gate's critical path (performance-sensible); whether the security
  posture accepts the exclusion is the security seat's call.
