# Performance review — ADR-008 product scaffold (round 1, pass c0)

Scope: `git diff 881d3b48641d7dbb6114de03c5770bc12073bc45...HEAD -- ':(exclude)docs/designs'`
(32 files, all new: workspace/crate scaffold, `crates/core` src + tests, `schemas/` tree,
CI workflows, docs).
Inputs: the diff, `02-proposal.md`, `docs/adrs/008-rust-for-the-product.md`, prior
findings (none — first pass). Specialty: performance checklist only.

## Checklist walk

1. **Event/keystroke path** — no event/keystroke path exists or is introduced: ADR-008
   fixes the product as file-only ("files remain the product"), and the proposal's
   Context states no input-event API is targeted. The closest hot path is the
   untrusted-document parse path the design names as the hub's upload-validation path
   ("the hub validating uploads with the same compiled types"). `parse` performs no
   fs/network — it takes `&str`; `load_reach_tables`'s synchronous `fs::read_to_string`
   is a one-shot table load, not a per-event path. Findings 1–3 are avoidable
   allocations on the parse path.
2. **Complexity** — no unbounded O(n²). `read_document` is linear in document size,
   which is bounded by `MAX_DOCUMENT_BYTES` before any allocation (`toml.rs:39-43`);
   duplicate-key detection is the `BTreeMap` insert itself (O(log k) per entry,
   `toml.rs:75`); array nesting is depth-limited at 16 (`toml.rs:229-233`).
   `resolve_chain` (`chain.rs:26-42`) is iterative, pre-checks `derives.len()` against
   `MAX_CHAIN_DEPTH` *before* building anything, and uses a `HashSet<&str>` for the
   identity check — correct collection, bounded ≤ 33 entries.
   `ReachTables::hand` (`reach.rs:66-68`) linearly scans the `tables` Vec, but the
   loader only ever pushes one entry per `Hand` variant (`reach.rs:78`), so the scan
   is ≤ 2 elements — O(1) by construction; a map would buy nothing.
3. **Unbounded growth** — no caches, arrays, or registries are introduced. Every
   growing structure (document entries, `chain`, `classes`) is bounded by a documented
   budget: 1 MiB / 16 nesting levels / `MAX_CHAIN_DEPTH = 32`, and the budgets are
   documented in `schemas/README.md:40-50`, as the proposal requires ("the constants
   and the budget are documented in `schemas/README.md`").
4. **Startup** — no binary or app exists yet. The only eager reader,
   `load_reach_tables`, reads exactly the two hand files for one version directory
   (`reach.rs:76-113`); nothing walks `schemas/` at load time (only test suites walk
   it). Pass.
5. **Cache invalidation** — evaluated as N/A rather than skipped: there is no cache in
   `crates/core/src` or the tests (the reach tables are re-read from disk per
   `load_reach_tables` call; no memoization exists anywhere in the diff), so there is
   nothing to invalidate and no test obligation arises from this change.
6. **Benchmark-like check** — the diff contains no benchmark-style test or timing
   note, so there is nothing to run and quote. (No test executed this pass: the review
   sandbox denied shell access; no checklist item required a run.)

## Findings

1. **minor — `crates/core/src/toml.rs:60` — per-line `Vec<char>` allocation on the
   untrusted-document parse path.**
   `let chars: Vec<char> = trimmed.chars().collect();` allocates a fresh
   4-bytes-per-character buffer for **every line** of every document parsed. This is
   the path the design explicitly puts under adversarial load ("untrusted input at
   every entry point", the hub upload-validation consequence), and a hostile-but-valid
   1 MiB single-line document forces one ~4 MiB `Vec<char>` allocation on top of the
   document itself. Bounded, so not a blocker — but avoidable.
   **Fix:** iterate `trimmed.as_bytes()` and track positions as byte indices; the
   grammar's structural characters (`= " [ ] , #` and ASCII key/digit chars) are all
   ASCII, and no UTF-8 continuation byte is ASCII, so byte-level scanning is exactly
   correct while string contents are still collected as `char`s via the slice.
2. **nit — `crates/core/src/toml.rs:75` — `entries.insert(key.clone(), value)` clones
   every key.**
   The clone exists only so `key` is still owned for the duplicate-key error message
   at line 78; on the success path the clone is pure waste.
   **Fix:** `match entries.entry(key) { Occupied(e) => Err(... using e.key() ...),
   Vacant(v) => v.insert(value) }` — zero clone on the accept path.
3. **nit — `crates/core/src/document.rs:91` (also 86, 94) — `supported_majors()`
   allocates on every parse, including the accept path.**
   `supported_majors()` builds a `Vec<String>` (one heap `String` each call); the
   version check at line 91 allocates it just to compare one value, and a rejecting
   document allocates it twice more (lines 94 and the error branch at 86).
   **Fix:** compare against the `MAJOR_V1` constant directly
   (`version != MAJOR_V1`), building the `supported` vector only inside the error
   branches.
4. **nit — `crates/core/src/toml.rs:201-224` — integer values round-trip through a
   `String`.**
   `parse_integer` accumulates digits into a `String`, then `format!("-{digits}")`
   allocates again for negatives before `parse::<i64>()`. Every integer in the
   document pays an allocation it doesn't need.
   **Fix:** accumulate the value arithmetically (`v = v.checked_mul(10)...` style) and
   apply the sign at the end; the overflow refusal replaces the `parse` `map_err`.

## Out-of-specialty FYI (no severity)

- `crates/core/tests/conformance.rs:47-49`: each valid fixture is parsed twice (once
  directly, once inside `rejection()`). Irrelevant at scaffold corpus size; noted only
  so it isn't mistaken for a production double-parse.

## Verdict

No blocker or major findings: hot paths are bounded by documented budgets, complexity
is linear, the chain guard's pre-check keeps its `HashSet` small, and no cache or
unbounded structure is introduced. The four allocation findings are cleanup on the
parse path. **approve**.
