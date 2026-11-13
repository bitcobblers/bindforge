# Code review — performance seat — round 1, ratification pass c2

Scope: `git diff 4844669384fd046eebfd47960283e178990617a4..HEAD -- ':(exclude)docs/designs'`
(fix stage `f299913` + test-update stage `aaf0796`), read against the current
`02-proposal.md` and ADR-014. This pass checks only that the post-approval
revisions did not break what the performance seat approved in round 1.

Inputs: the diff, `02-proposal.md`, `docs/adrs/014-profiles-as-chains.md`, my
round-1 findings. No other review reports consulted.

## Verdict: approve

## Round-1 finding status

1. **minor — manifest action join was product-of-caps linear scans** (`check_joins`).
   **Resolved, and resolved as the fix prescribed.** `chain_ops.rs:545-549` now
   builds one `BTreeSet<&ActionId>` from `manifest.actions` per resolution and
   tests membership with `rows.contains(action)` (chain_ops.rs:552). Cost shape
   is now: one set build of at most `MAX_MANIFEST_ROWS` = 256 entries (~2k
   comparisons) plus O(log 256) per touched action — that is the proposal's
   composite-cost paragraph (02-proposal.md:421-426) and the spec's
   "one per-resolution lookup of the `MAX_MANIFEST_ROWS`-capped action table"
   (schemas/profile/v2/spec.md:192-196, A21). Hostile near-miss chains pay the
   same bounded shape before refusing. Row order untouched, so determinism rule
   5 is unaffected. Verified by the suite: `cargo test -p bindforge-core --test
   chain_ops` — 74 passed, 0 failed, including
   `resolves_a_chain_of_exactly_max_chain_depth_opened_links` and
   `rejects_a_delta_list_past_max_patch_delta_elements` (finished in 0.00s).
2. **nit — `diff` compares layouts with a linear find per manifest row**
   (chain_ops.rs:264-266). **Unchanged by the fixes** — still at most
   256×256 ≈ 65k comparisons per call, bounded by `MAX_MANIFEST_ROWS`, load-time
   only. Not broken; remains open as a nit.

## What the later fixes did to the performance profile

- `load_chain` head/ancestor split (chain_ops.rs:371-389): one *fewer* fs
  syscall on the head (the `try_exists` probe is gone; a missing head surfaces
  from `parse_file`'s `Io`). Ancestors keep the probe. No new fs on any path.
- `emit_composition` partner handling (chain_ops.rs:785-799): the `continue` on
  an already-emitted partner replaces a condition inside the refusal check;
  `exchanged` is a `BTreeSet`, so membership is O(log s). Complexity unchanged.
- `merge` provenance inheritance removed: allocation removed, none added.
- `serialize_profile_link` writes `kind`/`for` unconditionally (document.rs):
  one `map_or`, negligible; still one string build per document.
- New tests are scratch-file IO in test processes only. No benchmark-like test
  or timing note exists in the suite (checklist item 6: nothing to run — the
  suite's max-depth/max-delta budget tests pass at 0.00s, corroborating the
  join cost above).

No new caches or registries (checklist item 3), no startup-path change (item 4),
no cache-invalidation surface added (item 5), no event-path change (item 1).

## Open findings (both carried, unchanged — non-blocking)

- **nit** — `crates/core/src/chain_ops.rs:576-597, 694-705`: the residual half
  of the round-1 fix suggestion was not taken — `touched_actions` dedupes with
  `Vec::contains` and `merge_touch_order` with `iter().any`, so the per-link
  dedupe is O(d²) with d ≤ 3×`MAX_PATCH_DELTA_ELEMENTS` = 768 (worst ≈ 0.3M
  string comparisons per link, load-time only). Bounded by the parse budgets;
  a `BTreeSet` alongside the order-preserving `Vec` would finish the shape.
- **nit** — `crates/core/src/chain_ops.rs:264-266`: `diff` still does a linear
  find over `before`/`after` per manifest row (≈65k comparisons at the caps).
  A map or two-cursor walk over the row-ordered vectors makes it linear.

## FYI (out of specialty, no severity)

The `CoreError::Cycle` `Display` text changed ("(`Cycle`)" removed,
error.rs:149) — whether that string is frozen under A15 belongs to the
compatibility seat.
