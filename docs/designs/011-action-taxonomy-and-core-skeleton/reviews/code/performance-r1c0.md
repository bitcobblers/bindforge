# Performance review — ADR-011 action taxonomy & core spine (round 1, pass c0)

Scope: `git diff 4aeb57c2188fbdfc1ca68d99080e4d2f3950d945...HEAD -- ':(exclude)docs/designs'`.
Design: `02-proposal.md`; contract: `docs/adrs/011-action-taxonomy-and-core-skeleton.md`.
Specialty: performance only.

## Checklist walk

1. **Event/keystroke path — no sync fs/network, no per-event table rebuilds, avoidable allocations.**
   N/A for this change: the diff ships no input/keystroke path. The only fs read is
   `load_action_registry` → `parse_file(dir/registry.toml)`, a one-shot load on an opt-in
   public API, not a per-event call. No consumer exists yet (Deferred: "how an installed
   artifact locates `schemas/` data"). No boot-time load: nothing parses the registry at
   boot. Item satisfied.

2. **Complexity — O(n²) where n unbounded (bindings × devices × platforms); maps/sets where
   identity lookup intended.**
   Satisfied. `ActionRegistry` stores ids in a `BTreeMap<ActionId, AxisKind>`; `contains`,
   `axis_kind`, and duplicate detection (`registry.entries.insert(...).is_some()` in
   `registry_from_entries`) are map operations — O(log n), no linear scan, no O(n²). The
   identity-lookup-intended check passes: the registry is the closed set and answers from a
   map, not a `Vec::contains`. Id length bound (`is_well_formed_segment`) is O(segment length),
   segment capped at 63 octets.

3. **Unbounded growth — caches, arrays, registries bounded or documented?**
   Satisfied on all three legs. The registry is bounded by the shared parse budget
   (`MAX_DOCUMENT_BYTES` = 1 MiB), documented in `schemas/action-ids/v1/spec.md:47-48, 107-118`
   ("A length bound on the arrays is the shared parse budget", "Documents larger than 1 MiB
   ... rejected before parsing, the cap binding before the file is read"), and asserted by
   `crates/core/tests/action_ids.rs::the_loader_refuses_a_registry_larger_than_the_parse_budget`.
   No cache introduced, so no unbounded cache.

4. **Startup — parsing/reading on boot only what boot needs.**
   Satisfied. Nothing eagerly loads or parses the registry; `load_action_registry` is an
   explicit call. The conformance harness reads the corpus at test time, not product boot.

5. **Cache invalidation logic present and covered by a test.**
   Satisfied (nothing to cover): the change adds no cache and no memoization, so there is no
   invalidation logic. `load_action_registry` re-parses each call — acceptable, as no hot
   consumer exists and the design deliberately declines a cache.

6. **Benchmark-like test or timing note — run it and quote the number.**
   N/A: the diff contains no benchmark, timing note, or complexity assertion to run. The
   size-budget test is a guard assertion, not a timing leg.

## Findings

- **nit — `crates/core/src/document.rs`, `axis_array` (called 3× from `serialize`'s `ActionIds` arm).**
  `serialize` builds each of the three axis arrays by calling `registry.ids()` once per kind;
  `ids()` (`crates/core/src/action.rs`) does `entries.keys().cloned().collect()`, so each call
  allocates a fresh `Vec` and clones every id `String`. Over one serialize that is three
  full-id-set allocations + clones plus three BTreeMap `axis_kind` filter passes (3·n clones +
  3·n log n lookups) where a single pass over the entries, partitioned by `AxisKind`, would do.
  This is avoidable allocation/lookup, but it is the conformance round-trip path (bounded at the
  shipped ~40 ids and by the parse budget), not a hot or event path, so it is a nit, not a
  correctness/perf gate.
  **Fix:** build the three arrays in one iteration over `registry` entries — group ids by
  `AxisKind` (or pass an already-partitioned view) and emit each `kind.name()` array once —
  avoiding the per-kind `ids()` clone. No behavior change; the round-trip the harness asserts
  still holds.

## Verdict

approve — zero open blocker or major findings. The registry uses a map for identity lookup, is
bounded by and documented against the shared parse budget with a test that pins it, and adds no
boot-time parse, event-path work, or unbounded cache.

## FYI (out of specialty, no severity)

- `crates/core/tests/action_ids.rs::the_loader_refuses_a_registry_larger_than_the_parse_budget`
  computes `"#".repeat(budget + 1 - oversized.len())`; if the shipped `registry.toml` ever grew
  past `MAX_DOCUMENT_BYTES` the unsigned subtraction would underflow and the test would panic
  before asserting. Robustness, not performance.
