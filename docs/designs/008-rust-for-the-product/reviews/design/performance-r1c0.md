# performance review r1

verdict: approve

Scope: whole document (panel pass c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/008-rust-for-the-product.md`, the Revision log (empty — "No findings
yet — r0 initial draft", so every finding below is `new`). No prior performance
findings to re-check.

## Checklist walk

1. **Input-path latency budget.** This design commits no event path: model
   semantics go to ADR-009, codec/CLI/scanner to ADR-012, desktop runtime to the
   desktop design — none of those deferrals contradict ADR-008's Decision. The
   only in-scope runtime behavior, `resolve_chain`, is bounded
   (`Cycle` / `ChainTooDeep` at `MAX_CHAIN_DEPTH = 32`), so no unbounded
   recursion or work sits on any call. One residual gap around the reach-table
   loader is reported below.
2. **Structure costs.** Fixture walk is linear in fixture count; reach tables
   are flat per-hand tables ("per-hand (left/right) tables kept distinct",
   Test strategy #4); no O(n²) over bindings × devices × platforms is introduced.
3. **Memory growth.** Reach-class tables are bounded repo data; the only cache
   in the design (`.sqlx/`) is build-time and bounded by the hub's committed
   query set. Bounded.
4. **Startup cost.** Gap — see the finding.
5. **Cache invalidation.** The canonical sqlx rule ties cache regeneration to
   the PR that changed hub queries or migrations *and* adds a hub-path CI job
   running `cargo sqlx prepare --check --workspace` against Postgres to catch a
   stale cache — invalidation is checked, not just fast. The risk table pairs it
   with Test strategy #7. Pass.
6. **Research baseline.** The proposal states no expected performance numbers,
   so there is no design-vs-baseline gap to explain; its only quantitative
   constant (`MAX_CHAIN_DEPTH = 32`) is documented as a guardrail with its home
   in `schemas/README.md`. Nothing to flag. (The design already anticipates the
   one compile-time cost it touches: heavy sqlx macro work is mitigated in the
   root `Cargo.toml` via `[profile.dev.package.sqlx-macros] opt-level = 3`, A6.)

## Findings

- [minor][new] Chosen design → "Formats contract: `schemas/` and the
  conformance harness" — "Reach-class tables are loaded from `schemas/` data by
  `bindforge-core`, per ADR-009." — the reach-table *loader* is created in this
  design (member table: `crates/core` owns "reach-table loader", "Created in:
  this design"; Test strategy #4 tests it), and its data source is fixed, but
  its cost model is not stated: compile-time embedding vs runtime file read +
  TOML parse, and once-per-process vs per-call loading. Test #4's "into the
  in-memory model" implies load-once but the design never says so; a per-call
  read of `schemas/reach-classes/v1/` would put synchronous file IO and schema
  parsing on every resolution — i.e. the keystroke/event path, which this
  checklist treats as major — and a runtime read of repo paths also fails in a
  packaged deb/AppImage artifact. Fix: amend that sentence to state the tables
  are embedded into `bindforge-core` at compile time (e.g. `include_str!` of
  `schemas/reach-classes/v1/`) and parsed into the in-memory model once at
  process start, with chain/reach resolution touching only the in-memory
  tables. This replaces the open-ended "loaded from `schemas/` data" phrasing;
  the contents themselves stay deferred to ADR-009 (only the load point is
  pinned).

## FYI (no severity)

- Workspace shape — "Dependency edges point one way: nothing →
  `bindforge-core`; `transforms`, `adapters`, `scanner` → `core`" — the first
  clause reads as the inverse of the second and of the intended rule (core
  depends on nothing). Likely a correctness-seat wording catch.
- Gates — the `rust` CI job names no dependency/`target` caching (e.g.
  `actions/cache` keyed on `Cargo.lock` + toolchain channel); with two skeleton
  crates this is free today but will slow the gate as members land. CI
  throughput is not one of my checklist items, so it is noted, not filed.
