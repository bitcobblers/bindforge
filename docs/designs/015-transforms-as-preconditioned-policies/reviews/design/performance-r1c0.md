# performance review r1

verdict: approve

## Findings

None. Every performance checklist item was evaluated against the whole
document (r0, first pass for this seat; the revision log records no prior
decisions to respect or contradict).

## Checklist coverage (evaluation basis)

1. **Input-path latency / per-event cost** — clean. Nothing in the delta
   inventory touches the event path: `resolve`, `chain_ops`, `placement`,
   `reach` are listed Unchanged, and the Compatibility matrix pins
   "Existing formats, fixtures, loaders, `resolve` — yes (untouched)". The
   transform stack runs only at authoring/apply time (`crates/transforms::apply`,
   CLI deferred to ADR-017), and the "Transform output" option deliberately
   materializes a base document instead of computing inside `resolve`, keeping
   014's load/resolve cost byte-unchanged. No new keystroke-path allocation,
   table rebuild, or synchronous IO.
2. **Structure costs (O(n²) over unbounded n)** — clean. The "Budget and cost"
   section bounds one application to "≤ 64 rows × ≤ 16 predicate moves over the
   capped loaded tables, geometry, and registry" via `MAX_POLICY_TRANSFORMS = 64`
   (mirrors `MAX_CONSTRAINT_GROUPS`) and `MAX_RELOCATE_TARGETS = 16` (mirrors
   `MAX_GROUP_MEMBERS`). Tag matching is a linear scan of the loaded registry
   (byte-exact prefixing), the match set is sorted once per row (O(m log m)),
   rook-adjacency contiguity is over ≤16 cluster cells, and the untouchable
   guard checks membership against the manifest's already-capped constraint
   rows. Worst case is rows × registry size × capped manifest entries — all
   factors document-bounded by `MAX_DOCUMENT_BYTES`. No quadratic product of
   unbounded dimensions.
3. **Memory growth** — clean. No caches or translation tables are introduced;
   inputs are the already-bounded loaded documents, and outputs are a single
   base whose `set` list is in-place over the source (bounded by the source
   document) plus ≤64 `TransformReport` entries, each optionally carrying one
   `PlacementCheck` whose vectors are bounded by the row's ≤16 pairs.
   The TF2 exec-chain lesson (per-file caps defeated by document count) is
   addressed explicitly: a policy references no other document by path.
4. **Startup cost** — clean. `load_policy` reads exactly one `≤ MAX_DOCUMENT_BYTES`
   file with the slug-before-filesystem rule; the conformance walk gains one
   directory (30 fixtures) with no harness change.
5. **Caching / invalidation** — clean. No caches are added. The derived-artifact
   invalidation story is 014's pin equality: `apply` entry-refuses a base whose
   `for_manifest` differs from the joined manifest, and a stale patch refuses
   at load naming the link, so a regenerated base cannot silently change what
   an old patch means. Registry-growth staleness is handled by re-evaluating
   the match rule per apply and reporting `match-count` — a visible no-op,
   not silent divergence.
6. **Research baseline gap** — evaluated against the baselines the proposal
   itself quotes (the per-prompt input set for this pass does not include
   01-research.md; the proposal's assumption register cites the shipped
   baseline directly). A13 (nine shipped budget constants, 014's
   composite-statement posture), A6 (manifest caps the new constants mirror),
   A24 (five severity classes → bound ≤ 4), and A3 (40 shipped registry ids)
   are internally consistent: every new number in the design (64, 16, ≤4,
   nine→eleven constants) is a mirror of a shipped cap, not a fresh estimate.
   No unexplained gap between expected cost and the quoted baseline.

## FYI (no severity)

- Options → "Transform output": the materialize-vs-compute-in-`resolve` choice
  is also what keeps the resolve hot path cost frozen — worth one clause in
  the Risks table if the compatibility seat wants it (their call, not mine).
