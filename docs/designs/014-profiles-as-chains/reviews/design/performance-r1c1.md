# Performance review r1 (dissent pass c1)

verdict: approve

Scope: the diff ac9cb8a..ec0ea47 of `02-proposal.md`, plus the full current
document. Inputs: the proposal, the ADR, the Revision log, my r1 findings.
Checklist: `references/performance.md` (latency budget, structure costs,
memory growth, startup cost, caching, research baseline).

## Previous findings — closure check

1. **major — walk shape of `derives` unstated; document-count bound evadable
   through array elements.** CLOSED. The entry row now states "at v2 a
   `patch` carries exactly one element and a `base` none", the invalid
   fixture `derives-two-elements.toml` pins it, `load_chain` step 2 now
   walks "each link's single `derives` element — a v2 chain is one path, not
   a tree", step 3 counts "documents opened, one per link under step 2", and
   the composite is restated as "one resolution opens at most
   `MAX_CHAIN_DEPTH` documents — one per link". The traversal-all-elements
   evasion a non-Rust implementer could construct is now a parse-level
   refusal, and the spec carries the load-level rules. Test strategy #1
   covers the second-element refusal. The fix landed at the contract level,
   which is stronger than my proposed `load_chain`-level clause.

2. **minor — composite stops at bytes/links/per-list count; manifest row cap
   unnamed; diff/merge output caps unstated.** CLOSED in substance. Options
   now names the shipped `MAX_MANIFEST_ROWS = 256` at first use, and the
   composite paragraph adds the derived element total — "applies at most
   `MAX_CHAIN_DEPTH × 3 × MAX_PATCH_DELTA_ELEMENTS` delta elements" (24,576)
   — so resolve's work is fully bounded. With both caps named and equal, the
   diff/merge output bound now follows for loadable inputs (touched/differing
   actions ⊆ the shared pin's `MAX_MANIFEST_ROWS`-capped action table ⇒
   each output list ≤ 256, disjoint output swaps ≤ 128). The explicit clause
   on the bullets is residue, downgraded to a nit below.

3. **minor — quadratic membership lookups; state one lookup structure per
   resolve.** CLOSED. The composite sentence now says the delta elements are
   applied "through one per-resolution lookup of the `MAX_MANIFEST_ROWS`-
   capped action table" — the built-once lookup I asked for is stated once,
   and with the element total and row cap both quantified, even the
   array-representation worst case (~6M cheap comparisons) is a stated,
   bounded number. No new mechanism was added; determinism rule 5 untouched.

4. **minor — chain identity undefined; consumer caches go stale on in-place
   link edits under an unchanged pin.** CLOSED. `load_chain` step 2 now ends
   "which pins the lookup but not the value: a chain's identity is its
   documents' contents" — the exact distinction my fix asked for, in the
   section the spec carries as load-level rules (Rollout 1 lists the
   "directory/pin/load rules" as spec content). The revision log records the
   deferral to 017/018 for caching itself, consistent with the ADR.

## Findings

- [nit][re-raised] Chain operations → `diff`/`merge` — "`diff` … a `set` per
  differing action, a `clear` per action `b` unbinds" and "`merge` … composes
  the two delta lists in chain order … over every action either touches" —
  the bullets still carry no output-cap clause. The substance is gone: with
  `MAX_MANIFEST_ROWS = 256` named and equal to `MAX_PATCH_DELTA_ELEMENTS`,
  every loadable input's touched actions ⊆ the shared pin's action table, so
  outputs are provably within the per-list cap. The only remaining path is
  `merge` over parse-valid, action-join-invalid patches (each can touch up
  to 1024 distinct action ids across its three lists), which can return
  `Ok(Profile)` with a 257th element — caught later by the load's own parse
  rule rather than by `merge`. Fix: one clause per bullet — output lists are
  capped because touched actions are rows of the shared pin's capped action
  table; state that `merge` refuses when the composition would exceed a list
  (or that its inputs are taken to be loadable against the shared pin).

## Checklist disposition

1. Event-path latency: none introduced — resolution is load-time, bounded.
2. Structure costs: all totals now product-bounded (32 × 1 MiB; 32 × 3 ×
   256 elements; ≤ 256 action rows); no unbounded n.
3. Memory growth: Chain ≤ 32 links; per-resolution action lookup bounded by
   the row cap; stated.
4. Startup cost: ≤ `MAX_CHAIN_DEPTH` file opens, each ≤ 1 MiB, one version
   gate and byte budget across every opened source (fixture, `data/`, hub
   bundle, personal directory).
5. Caching invalidation: chain identity = documents' contents; pin equality
   pins against manifest changes only — the stale-cache hole from r1 is
   closed at the contract level; the caching mechanism itself is 017/018's.
6. Research baseline: A21's "32 links × 1 MiB, stated nowhere" is now stated
   in Options, the composite paragraph, and the Risks row, with the added
   element total; no gap between design numbers and the baseline.

## FYI (no severity)

- The `diff` round-trip property ("appended to `a`, it resolves to `b`'s
  layout") breaks when `a` is already at `MAX_CHAIN_DEPTH`: appending the
  output opens a 33rd document and `load_chain` refuses `ChainTooDeep`.
  Corpus pairs must stay depth-capped, or `diff` should refuse at the cap —
  correctness-seat territory.
- `merge`'s output "carries the caller's `id` (`fork`'s collision rule)" —
  `merge` has no `Chain` argument, so the set the id must not collide with is
  undefined; correctness-seat territory.
