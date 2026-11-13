# Performance review r1 (dissent pass c1)

verdict: approve

Scope: the blob diff `cc78a21 → 7b2ddc6` of `02-proposal.md`, then the full current document.
Checklist applied: `references/performance.md` (latency budget, O(n²) in unbounded n, bounded memory, startup cost, cache invalidation, research-baseline gap).

## Round-1 findings — closure check

- **[major] no shipped tree-walk bound to reuse — CLOSED.** §3 now names the walk bounds as
  public constants (`MAX_INDEX_ENTRIES`, `MAX_INDEX_DEPTH` in `index.rs`, re-exported from
  `lib.rs`), rejected as `IndexTooLarge` / `IndexTooDeep`, with seed-scale headroom in A25,
  which states the aggregate bound explicitly: "their product is the only aggregate bound on
  bytes read at load, since the shipped cap is per-file". Both false claims are corrected —
  Adversarial posture now reads "The per-document budget is unchanged and `MAX_CHAIN_DEPTH`
  is reused as is; the tree walk adds the two public constants", Rollout 2 says "no
  per-document budget change". Test strategy #1 pins the over-entry-count and over-depth
  walk cases beside over-size and over-nesting. The added detail that the size cap applies
  to bytes read from the opened file, not `metadata` length, also bounds per-entry read
  time, so total boot read cost is `MAX_INDEX_ENTRIES × 1 MiB` — bounded, as the checklist
  requires.
- **[minor] `Taxonomy::load(dir)` with no file-selection rule — CLOSED.** It is now
  `Taxonomy::load(path: &Path)` pinned to `schemas/taxonomy/v1/actions.toml`, mirroring
  `load_reach_tables`, with `spec.md` stating `fixtures/` is harness input and never load
  input, and Test #2 asserting "`load` reads `actions.toml` and never `fixtures/`". The
  boot read set for the taxonomy is a single fixed path. (The `placement` mirror case is not
  fully closed — see finding 3; it is a narrower residue in newly added text, not a
  re-report of the closed finding.)
- **[minor] drift/model-walk gate cost (index per test function) — CLOSED.** Rollout 5
  builds the suites "sharing one `Index` per test binary" and Test #11 states the reason:
  "The walk and drift suites build one `Index` per test binary and share it, so the gate is
  linear in files, not tests." The drift mechanism names its read as "a read of the manifest
  index", so dependent-profile cost is one index build plus a per-profile chain walk.
  `validate` re-runs per dependent profile, which is linear in profiles with a constant
  per-manifest row count — no unstated superlinearity.

## Findings

- [minor][new] Chosen design → Manifests state game facts (with Test strategy #8) —
  "`validate(manifest, &Taxonomy)` enforces: … every `exclusive` and `docked` member appears
  in the `action` rows" — the r1 rewrite replaced the role-qualified quoted keys
  (`"action:<id>"`, a keyed map) with the nested-array rows `action = [[id, name]]`, so the
  action table is a list, not a lookup. "Appears in the `action` rows" is therefore a scan
  per `exclusive`/`docked` member, and the only bound on the row count is the shipped
  per-file 1 MiB (A24, A25) — an adversarial manifest with thousands of `action` rows and
  thousands of `exclusive` members makes validation quadratic (≈10⁹ comparisons) inside a
  document the budget accepts. Nothing in the design pins the indexing, so the cost is
  invented mid-coding. Fix: one clause in §5, echoed in Test #8's voice — `validate` builds
  the `action`-row id set once as a `BTreeSet` and does membership lookups, so validation is
  `O(rows log rows)` in the document.
- [minor][new] Chosen design → Transforms → `relocate-cluster` — "the matched actions'
  current inputs in `base` are isometric to the named placement's source-key set (same offset
  vectors, per `placement::validate`'s shape rule over `<hand>`'s table)" — "same offset
  vectors" between two sets, read literally as `placement::validate`'s shape rule states it
  ("the offset vectors between source keys equal the offset vectors between the target
  keys"), is an all-pairs comparison: `O(k²)` per `apply_policy` invocation, with a
  `ReachTable` lookup per pair. For a placement document `k` is the pair count (4 in the
  shipped example) and the cost is nothing; this new call site sets `k` to every action
  matching `<action-prefix>`, bounded only by the base's entry count under the 1 MiB cap.
  Not the keystroke path (there is no event API here), so the cost is bounded-input, not
  per-event, hence minor. Fix: state the linear form in the shape rule both call sites
  reference — compare one anchor's offset plus the sorted coordinate multiset of the two
  key sets, `O(k log k)` — so the precondition's cost is pinned in the text that both
  `validate` and `relocate-cluster` cite.
- [minor][new] Files created → `schemas/README.md` row and Rollout 1 (the data-homes rule),
  read against Chosen design → Profiles are chains → "Id → file, never id → path" —
  "`data/` holds content discovered by walking" — the data-homes rule makes every `data/`
  tree walked content, but the two new constants are scoped to the two walks that name them
  (`data/profiles/`, `data/manifests/`). `data/placements/*.toml`, which is what the
  transforms' `Ctx { … placements: &Placements }` is built from, and `data/policies/` are
  discovered by a walk with no entry-count bound named and no pinned loader signature — the
  same aggregate-exposure plus unscoped-read shape that round 1 closed for the other two
  trees, now in text this revision added. Fix: extend §3's bound clause to name every walked
  `data/` tree (placements, policies) under the same `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH`,
  and name the placements loader (`Placements::load(dir)`) beside `Taxonomy::load` and
  `load_reach_tables`, so the read set behind `Ctx` is a pinned path rule rather than an
  implementation choice.

## Checklist coverage notes

- Latency budget (item 1): no event path in scope (input-event API is a Non-goal); the
  remaining cost sites are load-time and CI-time — covered by findings 1–3.
- Bounded memory (item 3): the Index's resident bytes are now the product of the two named
  public constants and the per-file cap, stated in A25; the graded-set scoping in §2 shrank
  the authored reach corpus rather than growing it, and the `off-reach` default keeps
  `class_of`/`rank` total without a second table.
- Startup cost (item 4): taxonomy and reach tables are fixed-path loads; profiles and
  manifests are bounded walks; placements/policies are the residue in finding 3.
- Caching (item 5): no cache is claimed, so no invalidation rule is owed; the one shared
  artifact is the per-binary `Index` in the test gate, and the drift cases are fixture-local,
  so they do not mutate the shared instance.
- Baseline (item 6): A6/A24/A25 quote the shipped per-file 1 MiB, 16-level, 32-link guards
  and state the aggregate consequence; the 32-link unit is now stated as one `derives` link
  (base + 32 patches, 33 documents) consistently in §3, Risks, and Test #5. No unexplained
  gap between the design's numbers and the baseline.

## FYI (no severity)

- Removing the policy-level `mouse-swap` opt-in and resting the ADR's "default off" on
  "a policy enables a transform by listing it" is a semantics reading of ADR Decision 4 —
  for the compatibility/correctness seats, not this one.
- `fork`/`merge` now ship as the `patch/v1/spec.md` contract with implementation and suite in
  the ADR-012 slice; the ADR places `apps/cli` in ADR-012, so this is consistent with the
  source document, and scope is not this seat's verdict.
- Test #3's "a key outside a hand's table reads as that hand's `off-reach`" makes every
  placement verdict depend on which keys the graded set happens to cover — a data-authoring
  consequence for the grading seat.
