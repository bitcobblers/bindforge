# complexity review r1

pass: c0 (full panel), scope: whole document
verdict: approve

Inputs read: `02-proposal.md` (incl. Revision log: "r0: no reviews yet"),
`docs/adrs/011-action-taxonomy-and-core-skeleton.md`. No prior complexity
report for this run to compare against for this pass (r0 log, first seat pass).

## Checklist walk

1. **Reuse vs. new packages/modules.** No new package; one new module
   (`action.rs`) built explicitly on shipped pieces: the same `parse_file`
   loader shape as `load_reach_tables`, an `AxisKind` mirroring `Hand`,
   existing `CoreError` variants, the unchanged harness walk, the existing
   version gate. The Options section actively rejects the duplicating
   alternatives (Rust literal with drift test, `build.rs` codegen). Clean.
2. **Trace to a stated Goal.** Every moving part traces: contract tree and
   fixtures → Goal 1; typed model + loader → Goal 2; 40-id enumeration →
   Goal 3; classification/extension rules → Goal 4; `data/` READMEs and
   lineage retarget → Goal 5 (and the `data/` dirs are mandated by ADR-011
   "One"). The axis-kind attribute records-but-does-not-act, which is the
   ADR's own clause, not speculative machinery. No YAGNI found.
3. **A 30–50% smaller design?** Not while meeting the ADR: the ADR demands
   the spec+fixtures under `schemas/` run by CI, and typed ids with axis
   kind in `crates/core`. The obvious cuts (aliases/tombstones,
   machine-checkable schema language, `toml-test`-style runner contract,
   reserved-namespace mechanism, per-id table rows) are all *already*
   declined in Options/Deferred — the design is close to the floor for what
   the ADR requires.
4. **New dependencies.** None; std-only, `Cargo.lock` byte-unchanged. Clean.
5. **Deferrable second half?** The rollout is one thin slice: contract data,
   one module, two test targets, two READMEs, docs retarget. Nothing in the
   second half (docs/lineage) carries behaviour that could be cut to thin
   the merged slice meaningfully; deferring the `data/` dirs would contradict
   the ADR. No actionable deferral found.
6. **Knobs with no consumer / one-implementation abstractions.** No config
   knobs. Namespaces deliberately kept as convention, not a grammar set —
   an anti-mechanism choice. One small redundancy found (below).
7. **Value check.** The complexity looks proportionate. One new format
   directory, one model module, zero new dependencies, zero harness/CI
   changes, and the design repeatedly answers questions with "two rules, no
   third mechanism." The fixture count matches the spec rule count
   one-for-one, following the 008 run's established pattern. The one-way-door
   risk is met with written discipline (extend-don't-mutate, major-directory
   withdrawal) rather than machinery — the right trade at this stage.

## Findings

- [nit][new] Chosen design → "The model: `crates/core/src/action.rs`" —
  "`contains(&ActionId)`, and `axis_kind(&ActionId) -> Option<AxisKind>`" —
  the design's own registry shape makes membership and axis kind one fact
  ("the arrays *are* the id set, so there is no id without an axis kind"),
  so `contains` is a second public API surface that only re-expresses
  `axis_kind(id).is_some()`; every test in the strategy (2, 6) consumes
  `ids()`/`axis_kind` rather than `contains`. Small, but it is the one knob
  in the design with no distinct consumer. Fix: drop `contains` and let
  callers use `axis_kind(id).is_some()`, or name the consumer that needs
  membership without a kind.

## FYI (no severity)

- The `serialize` arm + `Document::ActionIds` variant exist so the unchanged
  `conformance.rs` walk round-trips the corpus "like `profile` does"; the
  alternative was the `reach-classes` exemption ("asserted by load"), which
  would have touched the harness. Either is small; the author's choice keeps
  `conformance.rs` frozen, which is consistent with the Goals. A correctness
  or compatibility seat may want to confirm the walk's round-trip rule
  applies to every format directory without a per-format list.
