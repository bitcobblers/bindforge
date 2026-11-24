# compatibility review r1 (pass c4 — ratification of the post-approval revisions)

verdict: approve

Scope as given: `git diff a4dda6a8 316ef46c` on `02-proposal.md`, then the full
current document, `docs/adrs/017-keybinding-extraction-codecs.md`, the Revision
log, and this seat's round-1 findings. Read-only verification against
`crates/discovery/src/steam_common.rs`, `crates/discovery/src/steam_linux.rs`,
`crates/discovery/tests/steam_linux_tests.rs`, `tools/orchestration/package.json`
and A15's cited Epic configuration-files page. No other run record opened.

## What the revisions did to this seat's round-1 findings (all confirmed)

- **`roots()` order — closed, correctly.** §Discovery plumbing now reads
  "`roots()` is the selected set as the core holds it — canonicalized,
  existence-filtered, byte order, not the candidate list" and §GameContext
  "`roots: Vec<PathBuf>, // every root the scan resolved, core byte order`".
  Verified against `steam_common.rs:369-381`: `canonical_paths` canonicalizes,
  drops entries that fail to canonicalize or lack a `steamapps/` directory, and
  returns a `BTreeSet` (byte-sorted, de-duplicated) — the wording now states what
  the code holds, and the §Config-path resolvers sentence "the anchors (`roots`,
  `library`) are already canonical from the scan" is now true rather than
  contradicted. The self-contradiction with Test #3 is gone.
- **Cap unit — closed.** `MAX_USER_DIRS_PER_GAME` is now "`SteamUserdata` trees
  kept per game, across all of `ctx.roots`", the resolver keeps "at most
  `MAX_USER_DIRS_PER_GAME` such trees in that enumeration order across all
  roots", the dropped tail is named ("past the 16th tree a game's remaining
  local users are simply absent"), and Test #3's two fixture roots holding 9 and
  8 directories pin the first-16 rule — so `(MAX_USER_DIRS_PER_GAME + 1) ×
  MAX_CANDIDATE_FILES` (544 MiB) is now the true worst case and the figure
  `schemas/README.md` is told to state stands.
- **Dedup key — closed.** `UserCandidates.tree_root` carries tree identity,
  "never rendered into a document", §Determinism dedups "by each document's
  `UserCandidates.tree_root`", §The CLI face says "dedup by tree identity keeps
  one tree stated once", and Test #12 pins both halves (same tree → one
  document; one slug in two libraries → two documents and two `--probe` groups).
  The silent cross-root drop my round-1 finding named is gone, and the contract
  still carries no machine context.
- **`017-C2` ledger hand-off — closed.** Rollout item 8 now lists "`017-C2`'s
  root set", the Context count is "Nine clauses", and the Risks row moved to
  "Nine clauses". The count matches the enumerated ids in both lists (B2, B3,
  C1-bound, C1-shape, C2, C3-inputs, C3-identity, C4, S1), and each has a
  Revision-log entry — including `017-C3`'s document identity, which is a
  recorded divergence from "one document per game per local SteamID" and not a
  silent one.
- **Deck row phrasing — closed.** "the root-relative `userdata/` and Proton
  prefix layout the two rows above claim, the host config home deferred as they
  state" matches §Config-path resolvers' "no resolver reads the process
  environment" posture.
- **`MAX_LOCATE_DEPTH` redefinition** (components in a located file's
  tree-relative path) is coherent across §Read discipline, §The first family's
  prune rule, §`bindings/v1`'s bound, Test #6 and Test #10; it tightens the old
  "levels descended" bound by one level, which only ever narrows the set of
  locatable paths, so no document valid under the earlier reading becomes
  invalid later — forward-compatible.
- Still open, unchanged by this diff: the Flatpak/Snap + Deck test coverage
  (reported below), and the two nits and the `cargo deny` gate note, which this
  pass leaves as recorded in round 1.

## Findings

- [minor][re-raised] **Compatibility matrix** — "Flatpak / Snap Steam | Yes, for
  the root-relative trees" and "Steam Deck | Yes, by the Linux row" (against
  Test strategy #3, "two fixture roots holding 9 and 8 digit directories keep
  the first 16 in that enumeration order"). Checklist item 7 asks for a platform
  test per supported row, and this round re-wrote #3 without adding either piece
  asked for in round 1: no test states that `roots()` returns what
  `SteamCore::with_roots` kept (the seam both rows now rest on), and no
  `ctx.roots` entry in #3 is shaped like a sandboxed install, so the two "Yes"
  rows are covered only by a structural test of the generic union. The
  two-root fixture does strengthen the union mechanism, and no discovery-side
  work is implied — the Flatpak and Snap candidates are already pinned at
  `crates/discovery/tests/steam_linux_tests.rs:288-347` — which is why this
  stays minor. Fix: one clause in #1 or #2 pinning `roots()` to the set the core
  retained, and one `ctx.roots` entry in #3 of the shape
  `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` that yields a
  `SteamUserdata` tree.
- [minor][new] **§`bindings/v1`** — "`files` are tree-relative paths, sorted byte
  order (the portability rule: the same layout states the same document on any
  machine)", tightened this round into "`files` ≤ `MAX_CANDIDATE_FILES`, each
  entry at most `MAX_LOCATE_DEPTH` tree-relative components (§Read discipline's
  table)" and Test #10's new invalid case "a `files` entry over
  `MAX_LOCATE_DEPTH` components". Nowhere is a `files` entry's *spelling* fixed:
  the separator a Windows-produced entry would carry (`\` from a joined
  `PathBuf`, or `Path::display` of one) versus the `/` of a Linux one, and the
  case of components taken from real directory entries. With the bound now
  counted in components, a loader must split the JSON string into components by
  some rule it is not given; and the "byte order" that §Determinism publishes and
  that `013-R2`'s recipe depends on ("the located files' octets concatenated in
  the snapshot's sorted `provenance.files` order") is separator- and
  case-sensitive, so one layout could state two different documents — and hash
  to two fingerprints — on two machines, which is exactly what the portability
  parenthetical promises. Nothing breaks in this run (the Windows row claims
  nothing, `017-R2` owns it), hence minor, but the contract is published now and
  `bindings/v1` ages through majors. I checked for a house precedent to cite:
  no existing tree under `schemas/` (`manifests`, `profile`, `policy`,
  `catalog`, `action-ids`, `keyboard-geometry`) carries a path in a document, so
  there is nothing to inherit. Checklist item 6 (separators and case). Fix: one
  clause in §`bindings/v1` — a `files` entry is the located file's components
  below the tree root joined by `/` (from `Path::components()`, never a
  displayed path), the loader counts and splits on `/` and refuses an entry
  containing `\`, and byte order is over that joined string — replacing today's
  platform-native "tree-relative paths" wording.
- [nit][new] **§Config-path resolvers** — "in `TreeKind` declaration order then
  byte order of the tree's `root` — a total order over the trees, which
  §Determinism relies on" against "for every `root` in `ctx.roots` in `roots()`
  order, one per digit-named `<dir>` in byte order, keeping at most
  `MAX_USER_DIRS_PER_GAME` such trees in that enumeration order across all
  roots" (Test #3 pins the second, "enumerates `ctx.roots` in `roots()` order
  and `<dir>` in byte order"). Those are two orders for one list: root-major
  then `<dir>` equals byte order of the whole tree path only while no canonical
  root is a proper prefix of another, and repeated `--steam-root` values can
  nest with each other's `steamapps/` present, since `canonical_paths` keeps
  both (`steam_common.rs:369-381`). Then which 16 trees the cap keeps and the
  order §Determinism publishes are decided by different rules. Nit because it
  needs nested roots *and* more than 16 userdata directories. Fix: make one
  order do both — enumerate `SteamUserdata` trees by byte order of the tree root
  it builds, which is what §Determinism and the ordering law in
  `schemas/bindings/v1/spec.md` publish — and Test #3's cap case reads as the
  first 16 of that same sequence.

## FYI (no severity)

- §The first family, for the correctness seat: this round's `!` semantics
  contradicts A15's own source. Epic's array-operations table
  (dev.epicgames.com/documentation/unreal-engine/configuration-files-in-unreal-engine)
  gives `!` as "Empty the contents of an array. Any value after `=` is ignored"
  — so `!Bindings=(…)` clears every pair accumulated for the key and adds
  nothing, not "`!Key=<tuple>` removes only the pairs it names", and Test #7's
  new pin ("`!Bindings=(…)` removes just the pairs it names") would enshrine it.
  The same table gives a bare `Key=Value` in an array context the
  reset-then-append meaning, which "Accepted lines: … `Key=Value`" leaves
  undecided. `+` (append-if-absent) and `.` (append-duplicate) read correctly.
- §Read discipline, for the performance seat: "since §The first family caps a
  tree's reads at `MAX_CANDIDATE_FILES` the product also ceilings the *opens*"
  holds for distinct files, not `open()` calls — `detect` opens up to 32
  matches per tree for the content test and `extract` "reads only located files,
  through `read.rs`" re-opens them, so octets read can reach 2 × 544 MiB per
  game; and §The first family's "the walk prunes on nothing but depth and entry
  count" drops round 1's "never descending a directory whose remaining depth is
  under the grammar's four trailing components", so the walk examines strictly
  more entries per tree now.
- Carried from round 1 and untouched by this diff (non-blocking, listed once):
  Rollout item 9 still counts `cargo deny check advisories` among the run's
  gates, where `.github/workflows/check.yml` runs it as the separate ADR-027
  `advisory-gate` job with its own pinned scanner and toolchain.
- Node face is genuinely unaffected: `tools/orchestration/package.json`'s
  `check` is `lint && format:check && test` scoped to that directory, so nothing
  in the matrix or gate list moves, and no Node API is involved
  (checklist items 3 and 4 not applicable — no new external dependency, no
  native module, no packaging story).
- Checklist items 1 (every row has a status; no `unknown`, no empty cell) and 2
  (the design touches no OS input API; §Config-path resolvers states "no
  resolver reads the process environment") are satisfied as written.
