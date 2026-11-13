# performance review r1

verdict: approve

Scope: whole document (full-panel pass c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/011-action-taxonomy-and-core-skeleton.md`, the Revision log (r0: no
prior reviews, no rejected findings). First pass for this seat.

## Checklist walk

1. **Input-path latency / per-event cost.** The stage introduces no event
   path: "This stage ships no binary" (Compatibility matrix) and "Nothing in
   this stage acts on the attribute (A17)" (Axis kind). No keystroke path is
   created or touched, so no per-event allocation, table rebuild, or
   synchronous IO is added; the only IO is `load_action_registry`'s single
   bounded file read. Evaluated: pass.
2. **Structure costs O(n²) with unbounded n.** The costs that scale —
   cross-array uniqueness checking at parse time, and `contains` /
   `axis_kind` lookups — are bounded by the closed registry ("40 ids",
   additions only by PR review per Stability). No bindings × devices ×
   platforms product exists in this stage's design surface. Evaluated: pass.
3. **Memory growth.** The registry is a closed set (40 ids, version-gated,
   withdrawal only via a new major directory); no translation tables or
   caches are introduced. Bounded. Evaluated: pass.
4. **Startup cost.** `load_action_registry` reuses the same
   `parse_file(dir.join("registry.toml"))` path as every contract document,
   "so the size budget binds before the registry is read" (The model) — one
   small file under the existing parse cap, mirroring `load_reach_tables`.
   Evaluated: pass.
5. **Caching invalidation.** No cache is introduced, so invalidation
   correctness is vacuously satisfied; there is no partially-stale state this
   stage can produce. Evaluated: pass (see FYI on the deferred locate/cache
   question for later stages).
6. **Research baseline vs design numbers.** Under this pass's named inputs I
   could not quote `01-research.md`, but the proposal states no expected
   performance numbers anywhere, so there is no design-vs-baseline gap to
   reconcile within the permitted inputs. Evaluated on that basis: no gap.

## Findings

- [nit][new] Chosen design — "The model: `crates/core/src/action.rs`" —
  "`ActionRegistry` — `version` plus the ids in document order, with `ids()`,
  `contains(&ActionId)`, and `axis_kind(&ActionId) -> Option<AxisKind>`" —
  the cost model of `contains`/`axis_kind` is unstated, and "the ids in
  document order" reads as a flat `Vec`, which would make every lookup a
  linear scan once ADR-013's translation resolves each binding's id. At 40
  ids the scan is negligible and the growth discipline keeps n small, so this
  gates nothing; it is only that the contract sentence should not make
  document order an accidental property of the lookup storage. Fix: one
  clause in the model section — `ids()` yields document order, the lookup
  representation behind `contains`/`axis_kind` is the implementation's
  choice — no new mechanism added.

## FYI (no severity)

- The Deferred item "How an installed artifact locates `schemas/` data" is
  consistent with ADR-011 (the ADR is silent on data location), so it is not
  re-opened here; the input-path latency budget properly belongs to the
  013/015/017 designs that create the hot path, and nothing in this document
  forecloses one.
- Revision log is r0 (no reviews yet): nothing was previously rejected, and
  this seat has no prior findings to re-check.
