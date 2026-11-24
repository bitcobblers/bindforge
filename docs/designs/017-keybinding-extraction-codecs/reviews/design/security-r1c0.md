# Security review r1 (pass c0, full panel)

verdict: request-changes

Scope: whole document. Inputs used: `02-proposal.md`, `docs/adrs/017-keybinding-extraction-codecs.md`,
the Revision log, plus read-only inspection of `crates/discovery/src/{steam_common.rs,keyvalues.rs}`,
`crates/core/src/{error.rs,toml.rs}`, `apps/discovery-cli/src/main.rs`, `Cargo.lock`,
`schemas/README.md`, `schemas/catalog/v1/spec.md`, `schemas/*/v*/spec.md` budget passages.

## Findings

- [major][new] §Config-path resolvers (closing paragraph) + §Read discipline —
  "Candidate paths are never canonicalized: the anchors (`root`, `library`) are already canonical
  from the scan, and §Read discipline never descends a link, so no resolved path is needed to keep a
  walk inside a tree." and "Locate walks take `DirEntry::file_type()` — which does not follow
  symlinks — so a link is never descended and never read as a file" — the no-link-descent rule
  governs the walk *below* a `UserTree` root; it says nothing about the root itself, which is built
  by `join`. For `SteamUserdata` the `<dir>` component is a `read_dir` name filtered only by the
  1–20-digit rule ("<ctx.root>/userdata/<dir>/<ctx.appid>, one per digit-named `<dir>`"), so a
  digit-named **symlink** under `userdata/` is accepted and `read_dir`/the existence test follow it;
  for `ProtonPrefix` the whole `pfx/drive_c/users/steamuser` chain is filesystem content, and the
  design itself records that "prefix trees are held together by symlinks" (A17). A tree-anchor
  symlink therefore relocates the walk outside the context's trees, which contradicts `017-C2`'s
  "locates nothing outside the context's trees" (re-quoted in §GameContext: "locates nothing outside
  the trees `user_trees` returns") — the containment claim that the design uses to justify skipping
  canonicalization. Test strategy #6 pins link handling *inside* the walk only ("a symlinked
  directory and a symlinked `.ini` in `symlinked-prefix` are neither descended nor read"), so as
  written the implementation and its suite both miss the anchor. Fix: in §Config-path resolvers,
  state that a `UserTree` is returned only when its own final component is not a symlink —
  `DirEntry::file_type()` on the enumerated `userdata/<dir>` entries, `symlink_metadata` on a
  template root — and that a symlinked tree root is skipped and counted exactly like a symlinked
  directory in the walk; replace "no resolved path is needed to keep a walk inside a tree" with the
  qualified claim (links are not followed anywhere, including at the anchor), and extend the
  `symlinked-prefix` fixture/#6 with a symlinked tree root.

- [major][new] §`bindings/v1` — "Bounds, stated once here and cited elsewhere: … `entries` ≤
  `MAX_SNAPSHOT_ENTRIES`; `files` ≤ `MAX_CANDIDATE_FILES` per document." and "Unknown key, missing
  required key, or a value outside its bound is a refusal naming the key." — two of the format's
  repeated fields have no count bound and the format has no document-size bound: one entry's
  `inputs` array is unbounded (only each token's octets are), so `entries` × `inputs` is
  unbounded, and nothing in the five-constant budget family or in §`bindings/v1` states a byte cap
  on a document the loader parses. The repo's contract-tree posture is the opposite:
  `schemas/README.md` §"Input budget and chain bound (adversarial guard)" says documents "arriving
  from disk, from another user, or from the network are untrusted at every entry point" and "A
  document larger than 1 MiB … The cap binds *before* the read"; every existing format's `spec.md`
  restates that sentence (profile/v1, profile/v2, policy/v1, manifests/v1, action-ids/v1,
  reach-classes/v1+v2) and every array-shaped field carries a named bound (`MAX_MANIFEST_ROWS`,
  `MAX_CONSTRAINT_GROUPS`, `MAX_GROUP_MEMBERS`, `MAX_PATCH_DELTA_ELEMENTS`,
  `MAX_POLICY_TRANSFORMS`, `MAX_RELOCATE_TARGETS`). The nearest precedent — `catalog`, the JSON
  format this run's README amendment copies — states `MAX_CATALOG_DOCUMENT_OCTETS` and "At load, the
  check rides `fs::metadata` *before* the read … so an oversized file is never buffered".
  `bindings/v1` is designed to travel (manifest authoring, the apply tool, UI import), so this is
  the format most likely to be fed a document from elsewhere, and `bindings/v1` ages through majors:
  a refusal added later breaks already-valid documents, so the hole becomes permanent at
  implementation. Test strategy #10's invalid corpus names no over-size or over-count-`inputs` case,
  and #15's README pin names only the five constants. Fix: add one named per-entry input bound
  (`MAX_INPUTS_PER_ENTRY`, in the §Read discipline budget table, the README budget bullet and the
  aggregate sentence) and state in §`bindings/v1` that the `snapshot.rs` loader applies a
  `MAX_DOCUMENT_BYTES`-style `fs::metadata` length check before parsing (the A4 posture §Read
  discipline already uses) — one invalid fixture for each in #10, replacing nothing already stated.

- [minor][new] §The first family — "collecting `*.ini` files whose path ends
  `Saved/Config/<one directory>/<name>.ini` and whose text carries a
  `[/Script/Engine.PlayerInput]` section with at least one `Bindings=` line" — deciding the claim
  means opening and decoding game-owned files during `detect`, but the design routes only `extract`
  through the reader ("`extract` reads only located files, through `read.rs`") and never says
  `detect`'s locate test goes through `read::open_capped`/`read::decode`, nor what happens when a
  path-grammar match is over budget or undecodable at locate time. Left open, the natural reading is
  a silent skip at locate, which reports a hostile-or-drifted over-budget binding store as
  `claim-refused`/`family-confirmed-files-absent` — the same collapse `017-L1` refuses for the
  post-claim case ("a claimed game with an unreadable binding file is a different fact from a game
  no codec claimed"), and it is also the only place a codec could allocate a file's whole body
  before `MAX_BINDING_FILE_OCTETS` is consulted. Fix: one clause in §The first family stating that
  locate reads candidates through `read::open_capped` + `read::decode` (which is what makes the
  `bom-utf16` fixture locatable at all), and that a path-grammar match that is over budget or
  undecodable claims and refuses by name (`FileOverBudget`/`UndecodableText`) instead of being
  skipped.

- [nit][new] §`bindings/v1` — "`user` (the context label, absent when the family's storage is not
  user-scoped)" / "`user` one of the three spellings above" — the spec's stated content list (key
  set, bounds, ordering law, refusal names) never records that one key is a persistent personal
  identifier (the local SteamID) in a document whose whole purpose is to be carried to other tools
  and machines. Nothing in this run ships it anywhere, so nothing is violated; the note is for the
  downstream owner. Fix: one sentence in `schemas/bindings/v1/spec.md` naming `user` as
  launcher-local identity that a sharing surface must treat as personal data.

- [nit][new] §Local paths (scoping) — "`--probe` mode exists to name files, so it prints absolute
  paths in probe output and tree-relative ones in the contract document" — the scoping argument is
  sound against the rule it cites (`schemas/README.md`'s no-path clause is scoped to "a message shown
  to whoever sent the document"), but probe lines also carry the OS username and the SteamID, and
  the Risks row frames the exposure only as a reviewer expecting the no-path discipline. Fix: add
  that one disclosure to the same paragraph (probe output is local-operator output, not safe to paste
  into a public report) rather than changing the behaviour.

## Checklist coverage

1. Trust boundaries — game config files, launcher metadata (unchanged scan) and CLI/env input are
   classified; the operator-`--steam-root` path stays operator-owned. Gap: the snapshot loader's own
   input is never classified (finding 2) and locate-time reads are unclassified (finding 3).
2. Parsing untrusted data — caps, BOM handling and closed key sets are present for game files;
   document-size and array-count bounds for the new JSON format are missing (finding 2).
3. Path handling — verified `canonical_paths` (`crates/discovery/src/steam_common.rs:369`) makes both
   anchors canonical, so the design's premise holds; appid is digit-gated; no path reaches a shell
   string. Gap at the walk anchor (finding 1).
4. Secrets/PII — no network leg on the face (a privacy improvement over `017-C4`'s wording); echoes
   go through `escape_and_truncate` (`crates/core/src/error.rs:190`) and `user` is digit-gated, so no
   control-character injection into the table or log. Notes: findings 4 and 5.
5. Dependencies — no new external package; locked `serde 1.0.229` / `serde_json 1.0.154` checked
   against RustSec/GHSA: no advisories on either crate (the 2024 stack-overflow advisory is
   `serde-json-wasm`, a different crate). Not applicable as a blocker; `Cargo.lock`-unchanged is
   pinned by Test #16 and Gate 9.
6. Privileges — read-only, no device access, no writes, no network; least privilege holds
   (`017-L1`'s never-write plus the bytes-unchanged test).
7. Fail-closed — named refusals on distinct exit codes, nothing applied, highest-ranked status wins;
   locate-time ambiguity is the one non-closed seam (finding 3).

## FYI (no severity)

- §Read discipline writes `FileOptions::new().read(true)`; the workspace's std spelling is
  `std::fs::OpenOptions` (correctness seat — the never-write pin itself is sound).
- `MAX_SNAPSHOT_ENTRIES = 4096` is 16× `MAX_MANIFEST_ROWS = 256`, so a snapshot can state more
  actions than any manifest may hold; the join is lossy in the other direction (correctness/
  complexity seat).
- §`bindings/v1`'s spec content list omits the shared `MAX_NESTING_DEPTH` (16) sentence every other
  `spec.md` restates; serde_json's own 128-level limit is a different number, so the new format's
  nesting posture is unstated rather than wrong (folded into finding 2's evidence, not its fix).
