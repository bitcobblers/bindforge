# performance review r1 (c3 ratification)

verdict: approve

Scope: diff `ec0ea47..2aae9c9` of `02-proposal.md`, plus the full current
document. Ratification check only: whether the post-approval revisions broke
anything the performance seat approved or made a cited section false.

## Diff-by-diff performance check

- **Fixture list** (`generated-by-bidi-char.toml` added): one more invalid
  single-document fixture; parse-time refusal, no cost-model change. Clean.
- **"Mount at any rung" rewrite** ("a consumer mounts a prefix by
  `load_chain`ing a lower link as head, or reads the documents singly with
  `parse_file`"): mounting a prefix walks a *shorter* chain (still ≤
  `MAX_CHAIN_DEPTH` documents, ≤ 1 MiB each); `parse_file` is one capped
  document. The composite cost paragraph ("32 MiB", "`MAX_CHAIN_DEPTH × 3 ×
  MAX_PATCH_DELTA_ELEMENTS` delta elements through one per-resolution lookup
  of the `MAX_MANIFEST_ROWS`-capped action table") still covers every mount
  path the rewrite opens. Nothing the rewrite adds is unbounded. Clean.
- **`diff` bullet** (collision rule "against every link id of `a`", explicit
  `derives = [a's head id]`): id-collision checking is a scan over ≤ 32 link
  ids; no cost change. Clean.
- **`merge` bullet** (collision rule "against `p1` and `p2`"; conflict example
  flipped to "lower `swap X Y` with both bound in the base, upper `clear X`:
  the net needs the `swap` before the `clear`"; identity-cancelling
  composition "refuses as the empty-patch case"): merge inputs are two single
  patches, each ≤ 3 × 256 elements, so composition cost is unchanged; the
  flipped example is the representable-direction-correct case under the fixed
  `set → clear → swap` order (it matches the determinism rules, so no cost
  model rests on a wrong example); the identity refusal removes the only path
  where `merge` could emit an empty patch that a later load would reject —
  refusal at merge is strictly cheaper for the consumer than a write-then-
  refuse round-trip. Clean.
- **Test strategy #5** (round-trip becomes a file-level `load_chain` move;
  flipped conflict pair example; identity-cancellation case added): test
  costs are corpus-sized; consistent with the bounded model. Clean.
- **Revision log rewrite**: summarization only; it still records the r1
  decisions, and does not claim a cap clause for `diff`/`merge` outputs that
  the text does not carry. Clean.

## Findings

- [nit][re-raised] Chain operations → `diff`/`merge` — "`diff` … a `set` per
  differing action, a `clear` per action `b` unbinds" and "`merge` … composes
  the two delta lists in chain order … over every action either touches" —
  unchanged by this diff: the bullets still carry no output-cap clause.
  Substance remains closed: the action join plus
  `MAX_MANIFEST_ROWS = 256 = MAX_PATCH_DELTA_ELEMENTS` provably cap the
  output for loadable inputs, and a merge over parse-valid, action-join-
  invalid patches can emit a 257th element only to be refused at the output's
  own parse ("a 257th element in any delta list"). Not a regression — the
  revisions neither fixed nor broke it. Fix (unchanged): one clause per
  bullet that output lists are capped because touched actions are rows of the
  shared pin's `MAX_MANIFEST_ROWS`-capped action table, and that `merge`
  takes its inputs to be loadable against the shared pin (or refuses a
  composition exceeding a list).

## Checklist status

1. Event-path latency: no per-event work introduced; resolution and chain
   ops are load-time. OK.
2. O(n²) over unbounded n: chain ≤ 32 links, lists ≤ 256, action table ≤ 256
   rows; merge over two single patches. OK.
3. Memory growth: composite ≤ 32 documents / 32 MiB, stated (A21). OK.
4. Startup cost: one `load_chain` walk, bounded and stated. OK.
5. Caching: no caches in this design. OK.
6. Research baseline: evaluated in r1; the revisions change no expected
   number. (01-research.md is outside this pass's named inputs; no cost-model
   claim moved, so no re-evaluation needed.)
