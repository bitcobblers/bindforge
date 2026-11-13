# performance review r1c4

Ratification pass over `git diff 7b2ddc6 f752153` of `02-proposal.md`, read against the
full current document, `docs/adrs/009-canonical-profile-and-manifest-model.md`, the
Revision log, and this seat's r1 findings. Scope: did the r2/r3 revisions break what
r1 approved, or make a cited section false.

verdict: approve

## Checklist walk

1. **Per-event cost** — no input-event API in scope (Non-goals), and the diff adds
   none. The new `apply_policy` pre-pass ("validates the whole `stack` against the
   table … before the first invocation runs") is `O(invocations × args)` over a
   document inside the shipped 1 MiB cap, and sits on policy-apply, not on a
   keystroke. Pass.
2. **Structure costs** — the two r1 quadratic shapes (`manifest::validate`'s
   `exclusive`/`docked` → `action`-row membership; `relocate-cluster`'s all-pairs
   reading of `placement::validate`'s shape rule) are **unchanged text** in this diff:
   the §5 rewrite added a consumer sentence, the shape-rule line and the
   `relocate-cluster` line are context lines. Nothing in the diff made them worse,
   and `docked`'s new "only v1 consumer" sentence removes an apply-time reader from
   v1 entirely (Deferred), so the membership scan's blast radius shrank. No new
   quadratic form: the added rules are per-document `InvalidValue` format checks, a
   `game` + `revision` index-key duplicate check, and a per-key admitted-list lookup
   in a fixed seed table — all `O(n)` or `O(n log n)` under the caps.
3. **Memory growth** — one open item, below: the diff widened the walked-`data/`
   read set (`.toml` now named for the `data/` manifest, placement, **and policy**
   documents) while the two index constants stay scoped to the two walks that name
   them.
4. **Startup cost** — the new reach-loader rule (class vocabulary, admitted-list
   membership, file-name/in-file `hand` agreement) runs over the fixed-path seed
   tables, and the `for`-pin → manifest-index lookup is a string key in a bounded
   index. No boot-path regression.
5. **Caching / invalidation** — unchanged: one `Index` per test binary (Test #11),
   the drift gate reads the manifest index; the new `DuplicateId`-on-`game`+`revision`
   rule removes a walk-order-dependent verdict, which helps determinism at no cost.
6. **Research baseline** — the numbers this design quotes are A6/A24/A25 (per-file
   1 MiB, seed-scale trees, "their product is the only aggregate bound on bytes read
   at load"), i.e. research Q6's answer; this diff adds no expected-number claim that
   the baseline does not cover, except the aggregate scope in the finding below.

## Findings

- [minor][re-raised] Files created → `schemas/README.md` row, read against Chosen
  design → Profiles are chains → Id → file — "the guard section's two new index limits
  and their rejections, and the data-homes rule: the contract tree holds lookup tables
  a loader reads at a fixed path, `data/` holds content discovered by walking" /
  "`.toml` for `schemas/` fixtures and for the `data/` manifest, placement, and policy
  documents" — the revision newly pairs the two index limits with the data-homes rule
  in one README cell, and newly names the `data/` placement and policy documents as
  walked content, while the bound sentence those edits sit inside is scoped to exactly
  two walks (`MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH` on `data/profiles/` and
  `data/manifests/`). So the guard text this revision writes states a bound for
  `data/` as a whole that covers two of its four trees, and A25's "their product is
  the only aggregate bound on bytes read at load" no longer names every tree read at
  load: `Ctx { … placements: &Placements }` is built from `data/placements/`, and
  policies from `data/policies/`, by a walk with no entry-count bound named and no
  pinned loader signature beside `Taxonomy::load(path)` / `load_reach_tables`. This is
  the r1 item, not closed and widened by the text that touched it; bounded-input cost
  (memory/read volume), not a keystroke path. Fix: extend §3's bound clause — "every
  walked `data/` tree, `placements` and `policies` included, is under
  `MAX_INDEX_ENTRIES` / `MAX_INDEX_DEPTH`" — and name `Placements::load(dir)` beside
  the two pinned loaders, so the README's guard cell and A25's aggregate-bytes claim
  cover the read set behind `Ctx`. No new mechanism, one clause each.

## FYI (no severity)

- The r1 fix suggestions for the two quadratic shapes (`validate` building the
  `action`-row id set once as a `BTreeSet`; the shape rule stated as anchor-offset
  plus sorted coordinate multiset, `O(k log k)`) remain unpinned in the text. Neither
  is in the diff, so neither is this pass's business; both are cheap to fold into the
  §5 / §4 sentences whenever those are next touched.
- The r3 `seeded-from`-requires-`for` rule and the `UnresolvedAncestor`-on-dangling-pin
  rule each remove a check that previously ran against nothing, so they only reduce
  work; the Revision log's reasons are sound from this seat's angle.
- `generated-by`'s closed producer list and the per-tree format filter are parse-time
  set/table lookups; no cost note needed.
