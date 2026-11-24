# `bindings/v1` — the extracted key-binding snapshot (JSON)

Status: version 1, major 1. The format is introduced by ADR-017 (design 017,
`017-C3`); its loader-owns-its-refusals posture is the one `catalog` keeps
(`schemas/catalog/v1/spec.md`), and it is the contract tree's second JSON
format (`schemas/README.md`).

A `bindings/v1` document states **the bindings a game's own files state**:
the (`action`, input-token) pairs a located user config carries, both halves
verbatim, with the evidence and the files they came from. A snapshot is
pre-taxonomy by definition — the action-id and input-id joins are the
manifest's (`013-C4`, `017-L3`) — so this format performs no translation and
validates nothing against the ADR-012 closed member sets.

One document per game per located user tree. Two trees may share a `user`
label; nothing but `entries` separates their documents, and a consumer
unions same-`user` documents' entries. Documents are machine-local output:
`discovery bindings` prints them and nothing in this tree writes them
(`017-R1`).

## Shape

One JSON object. The top-level key set is closed at exactly:

| key | value |
|---|---|
| `format` | the string `"bindings"` |
| `version` | the bare integer `1` (the `catalog` spelling — never a string) |
| `codec` | the claiming family's registry id |
| `game` | the game's discovery lookup key (`steam/<external-id>`) |
| `user` | optional — the context label; absent when the family's storage is not user-scoped |
| `entries` | array of entry objects (below), the pairs the located files state |
| `provenance` | object `{"evidence": …, "files": […]}` |

An entry object's key set is closed at `{"action", "inputs"}`:

| key | value |
|---|---|
| `action` | the native action (or command) name, verbatim |
| `inputs` | array of the located file's own input tokens, verbatim |

`provenance.evidence` is the claiming pass's evidence token —
`appid-table` or `install-tree-probe` — which is what lets an artifact say
which family claimed a game and on what basis (`017-C5`).
`provenance.files` are tree-relative paths, spelled `/`-joined component by
component whatever the host separator (the portability rule: the same layout
states the same document on any machine). The
`Saved/Config/<one directory>/<name>.ini` grammar the shipped `unreal-ini`
family locates is byte-exact — case-exactness is by design, and a
case-folded tree on a case-insensitive filesystem is outside the grammar,
never silently half-read.

Input spelling is canonical: `inputs` are the located file's own tokens, and
`bindings/v1` carries no ADR-012 input ids anywhere (§Input spelling, design
017). A token the closed vocabulary cannot spell — `MouseX`, `TAB`,
`Gamepad_RightX` — is stated anyway, because the snapshot reports the file.

## Bounds

- `codec` rides the slug grammar at `MAX_SLUG_OCTETS` (the bound the manifest
  `codec` entry already rides, `024-B1`), and is a family id from
  `bindforge-codecs`' registry.
- `game` is at most `MAX_DISCOVERY_SLUG_OCTETS`.
- `action`, every `inputs` element, and `user` are 1–
  `MAX_NATIVE_NAME_OCTETS` octets — deliberately the *manifest's*
  native-name bound, so an unjoinable action never appears. `user` rides
  that same token grammar rather than a closed list of the labels the
  shipped resolvers build, so another launcher context's name costs no
  format major.
- No string in the document carries an ASCII control character or a leading
  or trailing space (the manifest rule, kept so renderers need no second
  escape path).
- The (`action`, input-token) pair total is at most `MAX_SNAPSHOT_ENTRIES`;
  this bounds `entries` and every `inputs` array together.
- `provenance.files` is at most `MAX_CANDIDATE_FILES` entries, each at most
  `MAX_LOCATE_DEPTH` tree-relative components. Each entry is a plain
  `/`-separated sequence of ordinary path components — portable spelling on
  every host — and nothing that denotes a location outside the tree: no
  leading `/`, no `.` or `..` component, and no `\` or `:` anywhere (the
  Windows spellings of an absolute or UNC path); any other spelling is a
  refusal naming `files`.
- The serialized document is at most `MAX_SNAPSHOT_OCTETS`; a loader checks
  the file's length against that bound **before** reading it.

The constants are exported by `bindforge-codecs` (`crates/codecs`) beside the
rest of this format's budgets; the aggregate posture — one extraction opens
at most `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` files per pass,
the claim pass opens and reads them for its content test and the extraction
pass re-reads each located file to fold it, so the worst case is
`2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES ×
MAX_BINDING_FILE_OCTETS` octets (1,088 MiB) read per game, `--probe` paying
one pass and the extracting face two — is stated in `schemas/README.md`.
`MAX_USER_DIRS_PER_GAME` is the one silent bound: past the 16th located user
tree of a game, that game's remaining local users are simply absent from
this format's output — no refusal names it.

## Ordering (canonical)

One law (§Determinism, design 017 — cited, never restated elsewhere):
documents and their user-tree groups in `user_trees` order; within a
document, `entries` sorted by `action` byte order, each entry's `inputs`
deduplicated and sorted byte order, `provenance.files` sorted byte order;
across documents, a tree is stated once (deduplicated by tree identity —
never rendered); each file's entries are folded in byte order of its
relative path. Nothing reads wall-clock time or the process environment, and
no directory-return order reaches a document. A conforming serializer emits
keys in the order this spec declares them and arrays in exactly the order
above; the shipped serializer is `serde_json::to_string` (compact,
struct-declared key order).

## Refusals

An unknown key, a missing required key, or a value outside its bound is a
refusal **naming the key**. The loader's refusal set is its own
(`crates/codecs/src/snapshot.rs`); the extraction refusals that keep a
family from ever emitting a non-conforming document are named in design
017 §Read discipline: `FileOverBudget`, `UndecodableText`,
`UnparsableEntry`, `SnapshotEntryBudget`. Every message is single-line,
control-free and truncating; echoed values pass through the shared
`escape_and_truncate` renderer. An oversized document is refused on its
length check naming `MAX_SNAPSHOT_OCTETS`; a pair total past
`MAX_SNAPSHOT_ENTRIES` names `entries`.

## Privacy

`user` is launcher-local identity — a local SteamID directory name or the
constant `steamuser`. A sharing surface treats it, and any absolute path in
probe output, as personal data; the contract document itself carries only
tree-relative paths.

## Fixtures

`fixtures/valid/` and `fixtures/invalid/` are the conformance currency:
every valid document loads byte-stably through the shipped loader, and every
invalid one is refused naming its key — the fixture's own file name promises
which key (`<shape>[-<key>]…json`, the final `-`-segment names it). Fixture
file names are lowercase ASCII, `-`-separated, case-unique.
