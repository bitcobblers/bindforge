# Complexity review r1 (seat c0, full panel, scope: whole document)

verdict: request-changes

Value check (checklist #7): the scale is proportionate to what the feature is —
ADR-009 is the first product model and mandates five contracts in `schemas/`,
so five format dirs, five core modules and the `crates/transforms` edge are all
traceable to a Goal. Reuse is good and verifiable: the design routes every new
format through the shipped `parse`/`parse_file`/`parse_reader`
(`crates/core/src/document.rs:94-150`), reuses `resolve_chain` +
`MAX_CHAIN_DEPTH` (`crates/core/src/chain.rs:18-26`) and `load_reach_tables`
(`crates/core/src/reach.rs:103-131`) unchanged, adds **no dependency** (core
stays depends-on-nothing; `serde` in `[workspace.dependencies]` stays unused by
members), and the Options section defers three-way merge, geometry-as-prose and
`based-on` aliases. Nothing in `tools/` (it is the orchestration pipeline, no
reusable library) is being reinvented. The surplus is small, local, and mostly
in *published entries with no consumer* and *authored-data volume*, which is
why the fixes below are subtractions, not additions.

## Findings

- [major][new] Chosen design → Transforms produce bases (`schemas/policy/v1/`) —
  "`hand` (`left`/`right`, required)" — this required entry has no consumer
  anywhere in the design. Every transform that needs a hand already receives it
  in its own invocation (`relocate-cluster <action-prefix> <hand>
  <placement-id>`), the other two (`swap-polarity`, `swap-mouse-buttons`) take
  no hand, `Ctx { taxonomy, reach, placements }` carries none, and Test
  strategy #11 exercises stack order, preconditions and the mouse gate — never
  the policy's `hand`. `hand` is not in the ADR's policy sketch either (ADR-009
  Decision 4 spells the hand inside `relocate-cluster(move.steer-*, →right, …)`).
  Because a format's entry set is closed and minor bumps are additive only
  (`schemas/README.md:22`), an unreadable **required** entry shipped in v1
  costs a major bump to remove, and it becomes ambiguous for a policy with two
  relocate invocations for opposite hands. Fix: delete `hand` from the
  `policy/v1` entry table and from the example; the invocation argument is the
  only spelling the registry reads.

- [major][new] Chosen design → Manifests state game facts — "then role-qualified
  quoted keys: `action:<id>` = the game-native action name (the join),
  `default:<id>` = … `exclusive:<group>` = id array" — this is the one place
  the design relaxes the shipped closed-entry-set invariant into a prefix rule
  (the Risks row admits it). The shipped mechanism is a flat allow-list
  (`PROFILE_ENTRIES` + `reject_unknown_entry`, `crates/core/src/document.rs:39-43,156-170`);
  the relaxation means the manifest reader needs an enumerated-prefix path, a
  distinct unknown-prefix rejection, its fixture, its spec clause and Test #1's
  "unknown-prefix role key" case — permanent machinery to keep the entry set
  open, chosen without a recorded alternative (Options considered covers id
  encoding, not role prefixes). Fix: encode the same three roles as nested
  arrays, which the design already accepts at nesting level 2 (A13; used by
  `stack` and `swap`):
  `action = [["move.steer-left", "Steer Left"]]`,
  `default = [["combat.melee", "keyboard:f"]]`,
  `exclusive = [["ui", "ui.map", "ui.inventory"]]`.
  Every row is `[id, value…]`, the entry set stays closed, `validate` keeps the
  same checks, and the prefix enumeration, the unknown-prefix error path, its
  fixture and the Risk row all disappear.

- [major][new] Chosen design → Reach classes and placements (Authoring bar) +
  Test strategy #3 — "both hands graded over the full admitted keyboard name
  set" and "every admitted keyboard name is graded in both hands" — the graded
  corpus is the design's single largest authored artifact (class + row + col
  for every admitted name, twice), and most of it has no consumer: the reach
  bound reads grades for cluster source/target keys, the emphasis reading reads
  the grade of a game's `default:` input, and Test #3's asymmetry claim needs a
  handful of mirrored keys. Keys no shipped placement touches and no game
  defaults to (function keys, navigation, lock keys) are graded only to satisfy
  the coverage rule, and each extra row must also pass the "grades
  non-decreasing in grid distance from that hand's home-graded keys" check, so
  the CI surface grows with the corpus. Fix (keeps the authored-grading
  methodology the r0 log defends; shrinks what it authors): state a **graded
  set** — the keys the shipped placements and manifest defaults reference,
  extended by review — and treat a key absent from a hand's table as
  `off-reach` for that hand instead of `UnknownInput`, which also makes the
  emphasis reading total for any game default. Drop the full-set coverage
  clause from the authoring bar and from Test #3, keeping the coordinate-
  agreement and monotonicity checks over the graded subset.

- [minor][new] Chosen design → Transforms produce bases — "`mouse-swap`
  (boolean, default **false** — the ADR's opt-in)" plus
  "`swap-mouse-buttons()` — accepted in a stack only when the policy declares
  `mouse-swap = true`, else the document is rejected (`MouseSwapNotOptedIn`)" —
  a knob whose only consumer is the gate on its own transform: a policy that
  lists `swap-mouse-buttons()` has already opted in, in the same document,
  explicitly. The ADR's "default **off**" is satisfied by stack membership (the
  transform is absent unless the author writes it). Fix: drop `mouse-swap` and
  `MouseSwapNotOptedIn`; keep "a policy opts in by listing
  `swap-mouse-buttons()`" as the sentence in `policy/v1/spec.md`. This removes
  an entry, a `CoreError` variant, a fixture and Test #11's mouse clause.

- [minor][new] Chosen design → Profiles are chains — "`revision` (a
  `<major>.<minor>` string)" as an entry on `profile` and `patch` — nothing
  reads a profile's or patch's own revision: chain identity is `id` +
  `derives`, the pin's half is the **manifest's** revision, and Rollout's
  defaults line "revision absent = unpinned" describes the `for` entry, not
  this one (unlike the manifest, where the drift mechanism consumes it).
  Fix: keep `revision` on `manifest/v1` only, and drop it from `profile/v1` and
  `patch/v1`; a per-document revision is additive, so it can join in a minor
  bump when a consumer exists. Correct the defaults line to
  "`for` absent = not bound to a game".

- [minor][new] Chosen design → Profiles are chains (Id → file, never id → path) —
  "`index.rs` walks `data/profiles/` (bounded depth and entry count, each file
  size-checked by the shipped budget)" while the same section says "Chain and
  id resolution add no new budget" — the depth/entry-count limits are a new
  pair of tunables with no named constant and no named rejection, against the
  shipped convention that "the three constants are the reference
  implementation's public names for these limits, so an implementer never has
  to guess which number a rejection names" (`schemas/README.md:64-65`). At v1
  the walked tree is in-repo reviewed content, not untrusted input: untrusted
  documents arrive through `parse`/`parse_file`/`parse_reader`, which already
  carry the 1 MiB and 16-level caps. Fix: drop the two walk bounds and cite the
  per-file size budget, or name both constants and the `CoreError` variant they
  produce, so no limit ships unnameable.

- [minor][new] Chosen design → Profiles are chains + Rollout 5 — "`fork(index,
  root, rung) -> PatchDoc`" and "`merge(base, a, b)` … overlap is
  `MergeConflict`", shipped with Test strategy #8, while the CLI front end for
  `diff`/`fork`/`merge` is Deferred to ADR-012 — the merge/fork surface has no
  v1 consumer beyond its own tests, and merge is the design's only
  multi-document algebra. Fix: keep the *rule* in `patch/v1/spec.md` (chain
  order, per-action single touch, disjointness for a legal merge) so the format
  contract stays complete, ship `resolve` + `diff` in the first slice, and land
  `fork`/`merge` plus Test #8 in the ADR-012 slice that has the caller. This
  keeps the first merged slice thin without changing any contract.

- [nit][new] Files created by this design — taxonomy data in
  `schemas/taxonomy/v1/actions.toml` and reach data in
  `schemas/reach-classes/v1/{left,right}.toml`, but placements, policies,
  manifests and profiles under `data/` — two homes for reviewed data with no
  stated rule, which Rollout 6 then has to explain in a `repo-practices` line.
  Fix: one sentence in `schemas/README.md` — the contract tree holds lookup
  tables a loader reads at a fixed path; `data/` holds content discovered by
  walking — and the split stops looking accidental.

## FYI (no severity)

- `resolve_chain` returns the chain **self-first** and validates only the
  `derives` list of one document (`crates/core/src/chain.rs:26-41`); `resolve`
  applies documents deepest-ancestor-first and must detect
  `UnresolvedAncestor` per hop, so the reuse needs a stated reversal and a
  per-link guard. Correctness seat's call.
- The authoring bar, Test #3 and the input-vocabulary section all depend on the
  `schemas/README.md` keyboard-name list, which does not exist yet; the size of
  the graded corpus (finding 3) is undefined until that list is written.
- The round-trip split extends the re-serialization exemption
  (`schemas/reach-classes/v1/spec.md:52-58`, reach-classes only) to `taxonomy`
  and `placement`; the shipped profile spec promises re-serialization for valid
  fixtures, so the widening is a contract edit with third-party reader
  consequences. Compatibility seat's ground.
