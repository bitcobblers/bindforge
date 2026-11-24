# Complexity review r1c4 (ratification)

verdict: approve

Scope: `git diff 37bd2f37… 316ef46c…` on `02-proposal.md`, then the full current
document. Ratification lens only: what the post-r1 revisions broke of what this
seat approved, or made false in the sections it cited.

Round-1 closure check (no new findings on these): the `user`-spelling
enumeration is gone and `user` now rides the token grammar (r1 finding 2 —
closed); `roots: Vec<PathBuf>` plus a `roots()` accessor landed, and the
`017-C2` root-set reading is now a registered divergence, so the unregistered
fourth divergence is closed (r1 finding 1 — the part it flagged as
unregistered); the Context/Rollout-8/Risk divergence counts agree at nine
(verified by counting the Context list, Rollout item 8's unit ids and the Risk
row — consistent); the `Cargo.lock` gate is restated as the member's own stanza
and edge and is now true (`Cargo.toml:12` glob, `serde_json` already resolved),
and the added `cargo deny check advisories` leg really is a job in
`.github/workflows/check.yml:83-126` (r1 finding 4 — closed); `user_trees`
dropped the `env` closure and NativeHost, which removes the process-environment
input the round-1 design carried into `017-L4` — a net simplification this seat
endorses. The `steam_common`/`ScanRecord` seam and the `platform` field are the
author's kept choices, now registered; no rebuttal is unsound, so they are not
re-raised.

## Findings

- [minor][regression] Goals, first list, §`GameContext` citation — "`GameContext`
  carries the lookup key, the digit appid, the install path, and the *owning*
  Steam root and library" — the revision replaced `root: PathBuf` with
  `roots: Vec<PathBuf>  // every root the scan resolved, core byte order`, and
  the owning root now exists only on `ScanRecord`, never on the context; this
  Goals sentence (and Context's "lets a codec be told *which* Steam root and
  library own a discovered game") still describes the r1 struct. The one
  sentence in the document that states what the context carries to a codec is
  false against its own struct, and it is the sentence an implementer starts
  from. Fix: "the lookup key, the digit appid, the install path, every resolved
  Steam root and the owning library", and drop "*which* … own" from the Context
  sentence; no struct change.
- [minor][new] §Read discipline and budgets, aggregate paragraph — "since §The
  first family caps a tree's reads at `MAX_CANDIDATE_FILES` the product also
  ceilings the *opens*, so the worst case is `17 × 32 × MAX_BINDING_FILE_OCTETS`
  (544 MiB)" — the R1 rewrite of `detect` makes every path-grammar match open
  "through `read::open_capped` and `read::decode` for that content test", and
  `extract` still "reads only located files, through `read.rs`", so each tree
  costs up to `2 × MAX_CANDIDATE_FILES` opens and up to twice the bytes: the
  ceiling on *opens* is 1,088 and the worst-case read is ~1,06 GiB, not 544 MiB.
  This is the product `schemas/README.md`'s budget bullet is required to state
  (contract-tree amendment 1), so the error ships in prose. Fix: state the
  aggregate as `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` opens,
  or say `detect` hands `extract` the bytes it decoded — which replaces
  "extract reads only located files, through `read.rs`".
- [minor][new] §Read discipline — "`MAX_USER_DIRS_PER_GAME` is the one silent
  bound — past the 16th tree a game's remaining local users are simply absent" —
  the same revision added two more bounds that truncate with no named refusal
  and no status: "a further match is skipped and counted the way an entry past
  `MAX_LOCATE_ENTRIES_PER_TREE` is" (§The first family), and Test strategy 4 now
  files `017-B3`'s refusal away as "`017-B3`'s type invariant, behind the walk's
  skip-and-count", i.e. unreachable on the only path that counts files. So the
  section documents one silent truncation where it now has three, and the one
  budget the ADR names for a located set (`017-B3`) no longer refuses in
  practice — which is the opposite of the same section's own "drift becomes loud
  rather than a silently shorter snapshot". The two new constants are also the
  run's only budgets whose value has no recorded basis at all (`design-time` in
  the Source column, and no surveyed entry count or document size anywhere), so
  nothing establishes that 16,384 entries covers a real Proton prefix rather
  than quietly shortening it. Fix: one word per row in the table's `Bounds`
  column marking which bounds truncate silently and where their count surfaces
  (or make the 33rd match a named refusal), and one clause per design-time
  constant naming the observed value it was sized from.
- [minor][new] §The CLI face, item 1 — "a concretely typed copy of that
  selection, since `scan_records()` and `roots()` are inherent methods and not
  on the trait" — the revision makes the duplication explicit rather than
  incidental: `apps/discovery-cli/src/main.rs:129-142` is the selection (the
  `from_host()` pair when `steam_root` is empty, the `cfg(not(windows))` /
  `cfg(windows)` `from_roots` branch otherwise) and `bindings.rs` now
  re-implements it, which is the second copy of root selection this document's
  own Options rejected option (b) was rejected for ("lets two trees of code
  disagree about which roots exist"), with `017-L5`'s pinned `list` as the thing
  a future drift hits. Fix: one CLI-private helper returning the concretely
  typed plugin pair plus `roots()`, which `list` coerces into its existing
  `Vec<Box<dyn DiscoveryPlugin>>` (byte-identical plugin order and rows, so the
  pinned suites are untouched) and both faces call — replacing the copy, adding
  nothing.
- [nit][new] §Determinism — "across documents, deduplicated by each document's
  `UserCandidates.tree_root` — tree identity, never rendered — the first stating
  it once", next to §`bindings/v1`'s "two trees may share a `user` label … so a
  consumer unions same-`user` documents' entries" — the section's opening claim
  is that this is "One law, stated once in `schemas/bindings/v1/spec.md`", but
  neither clause is statable there: one keys on a Rust field that appears in no
  document, the other is a consumer rule the spec's stated contents (key set,
  bounds, ordering law, refusal names, `user` as personal data) do not include.
  Two contract-shaping rules now live only in proposal prose. Fix: tag the
  dedup clause as the face's rendering rule (§The CLI face item 3 already
  invokes it) and add the same-`user` non-uniqueness to the `spec.md` contents
  list; no shape change.

## Checklist

1. Reuse: `roots()`/`scan_records()` are thin inherent wrappers over state
   `SteamCore` already holds (`steam_common.rs:69-83`), so no new state; the one
   place the revision chose duplication over reuse is the CLI plugin selection
   (finding 4).
2. Goals trace: `tree_root`, `MAX_SNAPSHOT_OCTETS`, `MAX_LOCATE_ENTRIES_PER_TREE`
   all trace to a Goal or a §`017-B`/`017-L` unit; `platform` remains the one
   field with no named consumer (round-1 nit, kept by the author, not
   re-raised).
3. Smaller design: available on the aggregate budget arithmetic (finding 2's
   second option removes a read pass, finding 4's helper removes a copy), but
   nothing here is a 30–50% cut; dropping NativeHost made the document and the
   resolver genuinely smaller this round.
4. New dependencies: none. The `cargo deny check advisories` leg Rollout 9 adds
   is an existing CI job, not a new tool.
5. Deferrable half: still yes (Rollout 1-3 land the crate without the CLI face),
   unchanged from r1.
6. Knobs/abstraction: budget count went 5 → 7 with no new consumer outside the
   crate; `TreeKind` at two variants and one `CodecFamily` implementation remain
   ADR-mandated, not speculative.
7. Value: proportionate. The revision paid for real empirical findings (A18's
   unlabelled main root, A15's `!` operator, the host-config sharing problem)
   and paid down more than it added on the resolver side; what it added on the
   budget and CLI sides is a grounding and duplication problem, not a scope
   problem.

## FYI (no severity)

- Compatibility seat: dropping the `NativeHost` tree means `017-S1`'s "native
  and Proton" seed ships one of the two contexts its own terminology names; the
  divergence is registered and handed for `amended-by`, and the deferral is
  consistent with the ADR's reservations, so this seat does not re-open it.
- Correctness seat: Test strategy 12's "one slug in two libraries … yield two
  documents" is the case that makes `user` a non-unique label in the contract;
  see the §Determinism nit for where that rule is written down.
