# performance review r1

verdict: approve

Inputs used: `02-proposal.md` (whole document, incl. its Revision log — no
prior entries), `docs/adrs/012-input-vocabulary-and-reach-classes.md`. No
prior performance findings existed (first pass).

## Checklist evaluation

1. **Input-path latency budget / per-event cost.** Evaluated: this stage
   defines no keystroke/event path — the compatibility matrix states "Product
   artifacts (CLI, desktop, hub) — not applicable | This stage ships no
   binary" — and the predicate is specified as "a pure function over loaded
   data" taking `tables: &ReachTables, geometry: &KeyboardGeometry` by
   reference. No allocation-heavy loop, table rebuild, or synchronous IO is
   placed anywhere a future caller would be forced to pay per event: all IO
   lives in the explicit `load_reach_tables` / `load_geometry` constructors.
   No budget is needed at this stage and none of the design's costs land on
   an event path. Pass.
2. **O(n²) with unbounded n (bindings × devices × platforms).** Evaluated:
   every loop the design implies is bounded by the closed vocabulary — 48
   graded ids per hand, 48 geometry coordinates — and a `moves` argument whose
   endpoints must "appear in `geometry`, and [be] graded by both hand tables",
   so |moves| ≤ 48 by construction. No bindings × devices × platforms product
   appears anywhere. Pass.
3. **Memory growth of tables/caches.** Evaluated: the only new resident data
   is 2×48 `InputGrade` entries and 48 coordinate pairs, bounded by the closed
   member sets; growing a member set "rides a new vocabulary major", so a
   reader's footprint is pinned to one vocabulary major at a time. Pass.
4. **Startup cost (file reads, schema parsing at boot).** Evaluated: nothing
   consumes `v2` yet ("nothing consumes `v2` yet — ADR-015's transforms will
   load it"), loading is explicit and on-demand, and `load_geometry` runs
   "under the same budget" as the existing parse budget. The new files are
   ~48 short pair strings each. Pass.
5. **Cache invalidation.** Evaluated: the design introduces no cache; loaded
   tables are passed to the predicate by reference, so there is nothing to
   invalidate. The per-document parse-level `classes` pin ("the loader needs
   no cross-file comparison") also removes a load-time comparison pass. Pass.
6. **Research baseline vs. design's expected numbers.** Evaluated via the
   assumption register: A8 quotes the research — "No external benchmark
   validates the five-class table" — and the design ships no performance
   numbers that could diverge from a baseline. The missing external baseline
   is openly acknowledged (A8 `unverified`, Risks row 1); it concerns grade
   correctness, not a perf-number gap. No unexplained gap. Pass.

## Findings

- [nit][new] Chosen design — "The model: `crates/core/src/input.rs` and the
  reach edits" — "`InputGrade` is the parsed pair: `input: InputId`,
  `class: String`" — the Options section settles that "the bound is then
  integer rank arithmetic over table data", but the parsed representation
  stores the class as a `String`, so each predicate call turns each endpoint's
  rank back into a scan-and-compare over the 5-entry `classes` array. At 48
  keys this is negligible (hence nit, not minor), but the stated integer-rank
  mechanism is realized only if the rank is computed at parse time, not at
  every call. Fix: have the loader resolve each `InputGrade`'s class string to
  its index in the pinned canonical `classes` list once at parse time (the
  list is pinned at parse level to one fixed five-id order, so the mapping is
  static), and have `grade()` hand the predicate that ordinal instead of a
  string — same data, one-time work moved to load.

## FYI (no severity)

- The 48 authored grades carry no external benchmark (A8; research Risks
  "No external benchmark") — their authority is fixtures and review. That is
  the correctness seat's ground, not performance; noted for the panel only.
