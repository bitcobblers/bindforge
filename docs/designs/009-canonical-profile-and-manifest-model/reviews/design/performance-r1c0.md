# Performance review r1 (panel seat c0, full document)

verdict: request-changes

Scope: whole of `02-proposal.md`. Inputs: the proposal, the ADR, the Revision
log, `01-research.md` (for the checklist's baseline quote). No prior
performance findings; every finding below is `new`.

## Findings

- [major][new] Chosen design → Profiles are chains, "Id → file, never id →
  path" (with Adversarial posture and Rollout step 2) — "`crates/core/src/index.rs`
  walks `data/profiles/` (bounded depth and entry count, each file size-checked
  by the shipped budget)" and "Chain and id resolution add no new budget: they
  reuse `MAX_CHAIN_DEPTH` and the tree-walk bounds" / Rollout 2 "No new
  dependency, no budget change". There is no shipped tree-walk bound to reuse:
  the research baseline names the envelope as per-document only — "`MAX_DOCUMENT_BYTES
  = 1 MiB` checked before allocation, `MAX_NESTING_DEPTH = 16`, …
  `MAX_CHAIN_DEPTH = 32`" (01-research, Chains/patches/transforms; confirmed by
  `schemas/README.md` "Input budget and chain bound" and the three public
  constants in `crates/core/src/lib.rs`). So the index's aggregate exposure is
  the product of the 1 MiB per-file cap and an *unnumbered* entry count: total
  parse time and resident bytes at load are unbounded by anything the design
  pins, and the claim that the design "add[s] no new budget" is false — the
  depth/entry-count limits are new budget. This is the checklist's bounded-memory
  item (translation tables/caches) and its startup-cost item (file reads at
  boot) at the one place the design adds an unbounded read loop. Fix: name the
  walk bounds as public constants in `index.rs` beside the shipped ones
  (`MAX_INDEX_ENTRIES`, `MAX_INDEX_DEPTH`, re-exported from `lib.rs` like
  `MAX_DOCUMENT_BYTES`), give them seed-scale headroom in the same voice A24
  uses (the seed is one base + one patch chain), and correct the two claims:
  Adversarial posture becomes "no change to the per-document budget; the tree
  walk adds `MAX_INDEX_ENTRIES`/`MAX_INDEX_DEPTH`", Rollout 2 drops "no budget
  change" for that sentence, and Test strategy #1 pins an over-entry-count and
  over-depth walk case next to the over-size and over-nesting cases.

- [minor][new] Chosen design → Action taxonomy, API line —
  "`Taxonomy::load(dir: &Path) -> Result<Taxonomy, CoreError>`" while the data
  lives at `schemas/taxonomy/v1/actions.toml` and the same directory ships
  `spec.md` and `fixtures/`, which Rollout 1 requires to carry a valid+invalid
  pair plus "the refused-TOML-construct cases". A dir-taking loader has no
  stated file-selection rule, so the boot read set is undefined: a
  discover-every-`*.toml` implementation reads the fixture files on every load
  (and would fold fixture ids into the taxonomy). `placement` is the mirror
  case — its format dir ships no data file at all (`data/placements/*.toml`
  is the content), so a dir-shaped loader has nothing pinned to find. Fix:
  take the document path, not the directory — `Taxonomy::load(path: &Path)`
  pinned to `actions.toml`, mirroring the shipped `load_reach_tables`, which
  names its per-hand file and needs no discovery; state in
  `taxonomy/v1/spec.md` that `fixtures/` is harness input and never load
  input.

- [minor][new] Chosen design → Manifests state game facts, "Drift mechanism"
  (with Test strategy #13) — "The `cargo test` gate resolves every profile in
  `data/profiles/` against the manifest revision it pins and re-runs
  `validate`". Resolution needs an `Index`, and the `Index` is a full parse of
  the whole `data/profiles/` tree, so the gate's cost is (index-building tests)
  × (parse of every document) once `model_walk.rs`, `manifest_drift.rs` and
  the conformance walk are separate binaries with per-test `Index` construction.
  The API already supports build-once — `resolve(index: &Index, root, rung)` —
  so nothing forces the repeated cost; it is just unstated. Fix: one sentence
  in Rollout 5 — the drift and model-walk suites share a single index built
  once per test binary (a shared fixture), so the gate stays linear in the
  number of files, not in the number of tests.

## Checklist coverage

1. Input-path latency budget — **nothing to charge**: v1 ships no keystroke/
   event path ("Any input-event API (injection, capture), any UI" is a
   Non-goal), so no per-event allocation, rebuild, or synchronous IO exists in
   scope. Not a gap; see FYI for the hand-off.
2. Structure costs — the only quadratic shapes are `placement::validate`
   ("the offset vectors between source keys equal the offset vectors between
   the target keys", i.e. all-pairs correspondence) and the cross-hand
   coordinate agreement check; both are bounded by the admitted keyboard name
   set, which the Authoring bar requires fully graded (≈100 keys), so worst
   case is ~10⁴ cheap comparisons per validate. No unbounded n in
   bindings × devices × platforms. No finding.
3. Memory growth — the reach tables (2 × admitted keys), taxonomy (ids, added
   only by review PR), `Resolved` (≤ taxonomy ids) are structurally bounded;
   the `Index` is the one unbounded table → finding 1.
4. Startup cost — per-file reads go through the shipped 1 MiB pre-read cap
   (A6), which is the right guard; the taxonomy's dir-shaped loader is the
   open question → finding 2.
5. Caching/invalidation — v1 ships no cache. The `Index` is a fresh snapshot
   per process and the drift check re-derives everything from files, so
   invalidation is by construction, not by key: correct, and nothing to fix.
6. Baseline gap — A24 answers research Q6 against the baseline and is marked
   `unverified` with its basis, so the conclusion is explained. One arithmetic
   note: A24 counts "the shipped profile fixtures are under 1 KiB" and
   "≈ 45 ids × ≈ 40 bytes" for the taxonomy, and calls the manifest "tens of
   lines", but a manifest carries role-qualified entries per bindable action
   (`action:<id>`, `default:<id>`, plus `exclusive:`/`docked` membership), so
   it is ~3× the taxonomy, not "tens of lines". The conclusion survives by
   orders of magnitude, so no finding; the per-format-budget conclusion is the
   right one for *documents* — the aggregate is finding 1.

## FYI (no severity)

- `apply_policy` returns `PolicyOutcome { base: Resolved, … }` and
  `relocate-cluster` runs `placement::validate` per invocation, so a policy
  stack re-validates the same placement and clones the base map per
  transform. Bounded (≤ taxonomy ids × stack length) and off any event path —
  worth a memo, not a change, in v1.
- The ADR-012 apply path inherits the index/resolution cost and the same
  missing aggregate bound. Deferring it is consistent with the ADR's "Not
  decided here: … the hub service and voting (ADR-014)", so it is not a
  finding here; finding 1's constants are what that seat needs.
- `Ctx { taxonomy, reach: &ReachTables, placements: &Placements }` — `taxonomy`
  is the only field without a `&`; ownership shape is an API-style question for
  another seat.
