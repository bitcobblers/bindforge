# correctness review r1 (dissent pass c1)

verdict: request-changes

Scope: the blob diff `cc78a21 → 7b2ddc6` of `02-proposal.md`, then the full
current document. Inputs used: the proposal, `docs/adrs/009-...md`, the
Revision log, my r0 findings. Checklist: `references/correctness.md`.

## Round-1 findings — closure check

- [closed] Round-trip split (major). Encoding rules → Round-trip now reads
  "Every format's `fixtures/valid/` re-serializes to an equal value, which is
  exactly what the unmodified harness does to every format (A8) — `reach-classes`
  alone is load-asserted"; Test strategy #1 names all six round-tripping formats
  (policy included) and puts `actions.toml`/`{left,right}.toml` with their
  loaders; Rollout 2 ships the `serialize` arms. Re-verified against
  `crates/core/tests/conformance.rs:63` (the exemption is a hardcoded
  `dir.format == "reach-classes"` branch) — the text now matches the harness,
  and the Goals' "no harness change" claim holds.
- [closed] `seeded-from` inside `validate(manifest, &Taxonomy)` (minor). Moved
  to `resolve` (§3) and out of Test #8; Test #12 adds the dropped-preset drift
  case. (Residual edge case reported below, #5.)
- [closed] `[.<patch>]` revision (minor). Encoding rules now: "A document's own
  revision is a `<major>.<minor>` string in `revision`, and the only document
  that carries one is a `manifest/v1`"; §3 states a profile carries no
  `revision`; Options and Test #9 ("dotted-revision is `BadPin`") agree. Every
  pinnable revision is nameable now.
- [closed] "contiguous cluster" (minor). Now "the matched actions' current
  inputs in `base` are isometric to the named placement's source-key set (same
  offset vectors, per `placement::validate`'s shape rule over `<hand>`'s table)"
  — a predicate over shipped coordinates, and Test #10 asserts it.
- [closed] `MAX_CHAIN_DEPTH` unit (nit). "The unit is one `derives` entry, so
  `MAX_CHAIN_DEPTH = 32` accepts 32 links — a base plus 32 patches, 33
  documents"; carried into the Risks row and Test #5. Re-checked
  `crates/core/src/chain.rs:27` (`derives.len() > MAX_CHAIN_DEPTH`) — the
  arithmetic is now right.

## Findings

- [major][new] Chosen design → Transforms (policy entry set) + Rollout 3 + Test
  strategy #10 — "Each invocation must match its registry entry's declared
  arity and the grammars of its arguments (hand ∈ `left|right`, a placement id,
  a trailing-`*` action pattern); a mismatch is the shipped `InvalidValue {
  key, message }`, one `fixtures/invalid/` case per shape" and Test #10 "is
  rejected at parse (`InvalidValue`), before `apply_policy` runs". The check
  needs the registry table, and the design puts that table on the far side of
  the one-way dependency edge: Rollout 3 creates the "registry — each entry
  carrying its declared arity and argument grammars" in `crates/transforms`
  (`src/registry.rs`, §4's file list), whose only edge is
  `bindforge-transforms → bindforge-core`. The rejections are pinned as
  `schemas/policy/v1/fixtures/invalid/` cases, and the harness rejects fixture
  text through `bindforge_core::parse` alone
  (`crates/core/tests/common/mod.rs:237-243`, `conformance.rs:138-144`), so
  core's reader must decide arity and the `left|right` grammar without the
  registry — as written, the rule is unimplementable, and implementing it
  duplicates the registry in core (two sources for one fact). Fix: declare the
  v1 registry table (the three names, their arity, and their argument
  grammars) in `crates/core` — `crates/core/src/registry.rs`, added to the
  Files-created row's core list and Rollout 2 — with `bindforge-transforms`
  implementing the entries against it; Test #10's parse clause and the
  per-shape fixtures then need no change, and the registry keeps one home.
- [minor][new] Chosen design → Input vocabulary — "`schemas/README.md` carries
  the admitted keyboard-name list — the single enumeration the graded set below
  and Test strategy #3 iterate". Test #3 as revised iterates the graded set
  ("every key in the graded set is graded in both hands, and a key outside a
  hand's table reads as that hand's `off-reach`"), and the graded set is defined
  the other way round — "every key the shipped placements and the seed
  manifest's `default` rows reference, extended only by review". Nothing
  consumes the admitted list any more: input-id validity is grammar-only
  (`<part>` = lowercase words joined by `-`), and an ungraded key reads as
  `off-reach`, so `keyboard:banana` is a valid, total, ungraded input. The r0
  clause that gave the list a check ("every admitted keyboard name is graded in
  both hands") was replaced, and the new attribution is false. Fix: state the
  list's enforcement in the reach-classes loader rules — "a `keyboard:` key
  outside the admitted list is `InvalidValue`" — and describe the list as the
  finite set the graded set draws from, dropping "and Test strategy #3".
- [minor][new] Chosen design → Adversarial posture — "New `CoreError` variants,
  each pinned by a fixture: … `MergeConflict` … `IndexTooLarge`,
  `IndexTooDeep`". Two of these have no fixture to pin them. `MergeConflict`'s
  rule ships as `patch/v1/spec.md` text only — §3 defers `fork`/`merge` and
  Test #7 defers "fork/merge's suite … to the ADR-012 slice" — so no v1 fixture
  or test produces it. `IndexTooLarge`/`IndexTooDeep` are tree-walk conditions:
  the harness walks fixtures one file at a time, so an over-wide/over-deep walk
  cannot be a fixture document; Test #1 exercises them from a synthetic tree.
  Fix: qualify the sentence — "each pinned by a fixture, except `MergeConflict`
  (its suite lands with `fork`/`merge`) and the two index limits (pinned by
  Test strategy #1's walk)".
- [minor][new] Chosen design → Profiles are chains → Id → file — "`data/manifests/`
  gets the same treatment keyed on each document's declared `game` + `revision`,
  so a `for` pin is an index lookup". The collision case is unstated: two
  manifest documents declaring the same `game` + `revision` (their `id`s free to
  differ) leave the pin lookup ambiguous, and the drift gate (§5, Test #12)
  resolves every profile against that key, so an ambiguous key makes the gate's
  verdict non-deterministic. The id case has a rule ("`DuplicateId` fires on
  two documents declaring one id"); the manifest key does not. Fix: one clause —
  "two documents declaring one `game` + `revision` is `DuplicateId`" — and a
  fixture in Test #9.
- [minor][new] Chosen design → Profiles are chains (`resolve`) + Manifests —
  "`resolve` also rejects a `seeded-from` the pinned manifest does not declare,
  naming the profile id". The check runs against a pin that may be absent —
  "`for` … absent = a profile not bound to a game", `Resolved { …, manifest:
  Option<ManifestPin> }`, and the drift gate resolves each profile "against the
  manifest revision it pins" — so an unpinned profile carrying `seeded-from`
  names a preset no manifest is consulted against, and it passes CI silently,
  which is exactly the dangling case the move to `resolve` was meant to catch.
  Fix: state the precondition at parse — "`seeded-from` requires `for`; a
  `seeded-from` with no `for` is `InvalidValue`" — no new mechanism, and mirror
  it in Test #6.
- [minor][new] Files created (`schemas/README.md` row) + Adversarial posture —
  the row lists what lands in the README as "format list, the canonical
  encoding rules, the admitted keyboard-name list …, `version`/`revision`, and
  the data-homes rule", and the design adds "two new public constants beside
  the shipped ones — `MAX_INDEX_ENTRIES`, `MAX_INDEX_DEPTH`" plus
  `IndexTooLarge`/`IndexTooDeep`. The shipped contract section those belong to
  says "The three constants are the reference implementation's public names for
  these limits" and "The `fixtures/invalid/` corpora are the contract for which
  condition produces which rejection"
  (`schemas/README.md:64,72-73`) — after this design there are five limit
  constants, and the two new rejections have no contract home (they are not
  fixture-expressible, see above). Fix: add the two limits and the two
  rejections to the README's guard section in that Files-created row, and say
  so in Rollout 1.
- [nit][new] Chosen design → Manifests — "then the three nested-array rows of
  the Encoding rules: `default` = `[id, input]` (the emphasis fact, contract 1)"
  and "The entry set stays the shipped closed allow-list — no prefix rule
  (Risks)". Two pointers are stale: §1's canonical emphasis statement cites the
  same row as "a manifest `default` row (contract 5)" and says the taxonomy
  (contract 1) carries no emphasis, and the prefix-rule Risk row the citation
  points at was deleted in this revision. Fix: cite contract 5 (or "§1") for the
  emphasis row, and drop "(Risks)".
- [nit][new] Chosen design → Profiles are chains → Id → file — "File names
  follow the shipped fixture-name **stem** rule (lowercase ASCII, `-`-joined),
  the extension stated separately (`.bindprofile` for profile/patch content,
  `.toml` for `schemas/` fixtures)". The enumeration omits the `data/` documents
  that use `.toml`: `data/manifests/<game>/manifest.toml` (§5),
  `data/placements/*.toml` (§2), `data/policies/` (§4). Fix: "`.toml` for
  `schemas/` fixtures and for the `data/` manifest, placement, and policy
  documents".

## FYI (no severity)

- `relocate-cluster` over an input the hand's table does not grade has no
  coordinates, so isometry is undefined; the text routes "precondition failure"
  to `Skipped`, which plausibly covers it, while `placement::validate` names
  `UnknownInput` for the same shape. Which one a `mouse:*` cluster member
  produces is the compatibility seat's call.
- Removing the policy-level `hand`/`mouse-swap` entries does not cut the ADR's
  "default **off**" / opt-in requirement: the ADR's own frame is "handedness is
  a stack of independent decisions", and each decision is now a stack entry, so
  listing-is-opt-in reads consistent to me.
