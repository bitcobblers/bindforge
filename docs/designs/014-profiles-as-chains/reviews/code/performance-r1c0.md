# Code review — performance seat — round 1, pass c0

- Diff: `git diff 4cfcd8db4477247e8e6ecb26ecade40451e0ad91...HEAD -- ':(exclude)docs/designs'`
- Design: `docs/designs/014-profiles-as-chains/02-proposal.md`; ADR-014
- Inputs: the diff, the proposal, the ADR only.

## Checklist walk

1. **Event/keystroke path** — pass. This stage ships no binary and no
   keystroke path; `resolve` and the chain ops do no I/O. All fs is in
   `load_chain`, once per link, ≤ `MAX_CHAIN_DEPTH` links per call.
2. **Complexity** — no O(n²) over an unbounded n: every scan is bounded by
   the parse budgets (`MAX_CHAIN_DEPTH` = 32 links, `MAX_PATCH_DELTA_ELEMENTS`
   = 256 per list, `MAX_MANIFEST_ROWS` = 256). One join is a linear scan
   where the spec's own cost sentence promises a lookup — finding P-1.
   `resolve`'s layout state is a `BTreeMap`, correct.
3. **Unbounded growth** — pass. No cache, registry, or static is added.
   `Chain::links` is bounded by `MAX_CHAIN_DEPTH`; delta lists by
   `MAX_PATCH_DELTA_ELEMENTS`, refused at parse (`document.rs:914-929`).
4. **Startup** — pass. Nothing new reads or parses at boot; the conformance
   walk only runs in test.
5. **Cache invalidation** — no cache exists in the change; nothing to cover.
6. **Benchmark-like tests run** — `cargo test --locked -p bindforge-core
   --test chain_ops`: **71 passed; 0 failed; finished in 0.00s** (debug).
   That suite includes the two budget pins: exactly `MAX_CHAIN_DEPTH` opened
   links resolves and `MAX_CHAIN_DEPTH + 1` refuses as `ChainTooDeep`
   (`tests/chain_ops.rs:1284-1316`), and a delta list of exactly 256 parses
   while the 257th element refuses (`tests/chain_ops.rs:852-876`). The
   composite-budget tests are real and fast.

## Findings

### P-1 (minor) — manifest action join is a per-link linear scan, not the "one per-resolution lookup" the spec states

- **Location:** `crates/core/src/chain_ops.rs:523-537` (`check_joins`),
  scan at line 525; dedupe at `chain_ops.rs:549-570` (`touched_actions`)
- **Problem:** the action join runs
  `manifest.actions.iter().any(|row| &row.action == action)` once per unique
  touched action of every link. For a maximum-size valid chain that is up to
  32 links × ≤ 256 unique actions × a scan over up to 256 rows ≈ ~1–2 M
  `ActionId` string comparisons per `resolve`; a hostile near-miss chain
  pays the same before its refusal. `schemas/profile/v2/spec.md:188-189`
  (and the proposal's composite-cost paragraph) state the delta elements
  apply "through one per-resolution lookup of the `MAX_MANIFEST_ROWS`-capped
  action table" — the implementation builds no lookup structure, so the
  stated cost shape is product-of-caps linear scans, not one table + lookups.
  Bounded, load-time only, hence minor — but it is the one place the code
  contradicts the contract's own cost sentence, and the fix is cheap.
- **Fix:** in `resolve`/`check_joins`, build one `BTreeSet<&ActionId>` (or
  `BTreeMap<&ActionId, ()>`) from `manifest.actions` per resolution and test
  membership against it; likewise dedupe `touched_actions` with a
  `BTreeSet<&ActionId>` alongside the order-preserving `Vec` instead of the
  `Vec::contains` scan at line 552 (same shape at `merge_touch_order`,
  `chain_ops.rs:684-687`).

### P-2 (nit) — diff compares layouts with a linear find per manifest row

- **Location:** `crates/core/src/chain_ops.rs:250-252`
- **Problem:** for each manifest row, `before.iter().find(...)` and
  `after.iter().find(...)` scan the resolved layouts linearly: O(rows ×
  layout) = at most 256 × 256 ≈ 65 k comparisons per `diff`. Bounded and
  one-shot, so harmless today, but `resolve` already produced both layouts —
  an index (map action → binding, or a merge-walk over the two row-ordered
  vectors) would make it linear with no extra structure.
- **Fix:** build a `BTreeMap<&ActionId, &InputId>` from `before`/`after`
  once, or exploit that both layouts already follow manifest row order and
  walk them with two cursors.

## FYI (no severity, out of specialty)

- `open_link` does `path.try_exists()` followed by `parse_file(path)` — two
  syscalls per link where the open alone would do; the `try_exists` also
  leaves a small stat→open TOCTOU window (correctness, not perf).

## Verdict

No blockers, no majors. The budgets the design promises (32 documents,
256 per delta list) are enforced at parse and pinned by tests that run in
0.00 s; the only deviation from the design's own cost statement is the join
scan in P-1, bounded and load-time, carried as minor.
