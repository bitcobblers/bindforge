# BindForge formats contract

`schemas/` is the single home of BindForge's on-disk format contract: one
directory per format (`action-ids`, `keyboard-geometry`, `manifests`,
`policy`, `profile`, `reach-classes`), and under it one directory per major version
(`v1`, `v2`, …). A format's later majors are how a format grows: the
profile-chain vocabulary of the 012-015 series is `profile/v2`, not a sixth
format.
Every product component — the CLI, the desktop app, the hub — validates
documents with these rules, and third-party readers can conform to them
without reading Rust.

The scaffold of this contract tree (ADR-008 design 008) ships `profile/v1` and
`reach-classes/v1`; ADR-011 adds `action-ids/v1`, the id set the rest of the
011-015 series speaks through. ADR-012 adds `reach-classes/v2` — the five-class
reach taxonomy with each hand's grade for every graded input, leaving `v1` and
its placeholder ids untouched — and `keyboard-geometry/v1`, the key grid the
placement predicate checks its shape rule against. ADR-013 adds
`manifests/v1`, the game manifest — game id and data revision, codec family
id, action table, constraints, and preset ids — read from `data/manifests/`
through `load_manifest`, whose registry join and location rule are load-time
rules. ADR-014 adds `profile/v2`: one link of a chain, with its `kind`, its
`for` pin, its three delta lists, and its two provenance labels, read across
files through `load_chain` — whose one-directory rule, pin equality, and
manifest joins are load-time rules of the same shape. `profile/v1` and its
fixtures stay exactly as shipped: the chain's vocabulary is a new major, not a
widened one. ADR-015 adds `policy/v1`: an opt-in stack of transforms — the
closed three row kinds, each one space-joined row in an ordered `transforms`
array, with its tag, hands, grade bound, and closed-vocabulary target list —
read from one file through `load_policy`, whose slug-before-filesystem and
name↔location rules are load-time rules of the same shape. A policy references
no other document by path: the base it applies to arrives from the caller, so
applying a stack is a library call in `crates/transforms`, not a format load.
The conformance harness's `schemas/*/v*/` walk picks every new
directory up with no harness change.

## Versioning rule (canonical)

- One directory per major version: `v1`, `v2`, …
- Every document declares its `format` and its `version`. A document that
  declares no version, or a major `bindforge-core` does not support, is
  rejected with a structured error naming the supported majors. It is never
  defaulted to a version.
- The majors a document may declare are those of **its own format**: a format
  ships a major only when a `schemas/<format>/vN/` directory carries it, and a
  document declaring a major its format does not ship is refused naming that
  format's majors. `reach-classes` and `profile` each have two majors today, so
  their `v2` never widens what an `action-ids`, `keyboard-geometry`, or
  `manifests` reader accepts. A reader that gates on one global list instead
  reads a future-major document with the grammar of the major it happens to
  have — the misreading this rule exists to prevent.
- Minor bumps are additive only.
- A file's presence in `vN/fixtures/valid/` is the definition of
  "supported", and the majors `bindforge-core` declares must equal the
  majors present under `schemas/*/v*/`.
- Fixture file names are lowercase ASCII words joined by `-`, and
  case-unique within a fixture directory.

## Formats

Formats are TOML (ADR-011-015) in a documented subset of TOML v1.1.0 (A20): every
valid contract document is a legal TOML v1.1.0 document, and each version
directory's `spec.md` states exactly which TOML constructs its grammar
admits — and which it refuses. A reader built on a standard TOML parser
therefore accepts more than the contract does, and must reject the rest:
table headers, dotted keys, multi-line arrays, floats, and hex integers are
legal TOML and outside the v1 grammar, and each has an invalid fixture. The
subset is stated here, in the contract, because a narrowing that only lives
in one implementation is not a contract.

**Two formats are JSON, and they are named here so the sentence above is not a
lie about them.** `catalog` (ADR-026, `026-C3`) and `bindings` (ADR-017,
`017-C3`) are the JSON formats. ADR-026 fixes
the on-disk spellings as JSON — `data/catalog/v1/catalog.json` for the seed
corpus, `<data>/bindforge/registry/catalog-v1.json` for a caching instance's
local entries, and the bare-integer `version: 1` form no TOML format shares —
so its fixtures are `.json`, its `spec.md` states a JSON shape rather than a
TOML subset, and its conformance is exercised by its own loader in
`apps/registry` (`corpus.rs`), not by core's TOML `parse`/`rejection` sweep.
ADR-017 adds `bindings`: the extracted key-binding snapshot, whose only
loader is `crates/codecs` (`snapshot.rs`) — the same posture, since a JSON
contract's loader owns its refusals — so its fixtures are `.json` too, and
the harness's TOML parse and rejection loops skip both formats' trees while
its fixture-naming and zero-fixture rules keep covering them.
The harness's `schemas/*/v*/` walk still picks every directory up for the fixture
naming and zero-fixture rules; it skips only the TOML parse path there. The
contract's other JSON documents — the match store `matches.json`, the per-slug
candidate response body, and the `026-C7` cache document
`<data>/bindforge/registry/cache-v1.json` — join them outside the TOML subset,
with the same "the loader that owns the document states its refusals" posture.
`catalog` and `bindings` are also the trees where a discovery lookup key
(`steam/…`, `epic/…`, `local/…`) is legal *content*: it is what those formats
store — the catalog as its corpus, a `bindings` snapshot as the `game` it was
extracted for — so the
hygiene sweep that keeps such spellings out of the shipped corpus skips
`data/catalog/`, `schemas/catalog/` and `schemas/bindings/` and nothing else
(and *not* `data/bindings/`: ADR-017 ships no snapshot under `data/` —
snapshots are printed, never written).

The fixtures are the conformance currency — following the JSON Schema Test
Suite model, every implementer runs the same golden fixtures — and no
machine-checkable schema language is committed.

One vocabulary spans the formats: an **input id** is `<namespace>:<member>`
(`keyboard:*`, `mouse:*`, `gamepad:*`), and its grammar — including the closed
member sets that make an unknown member a refusal — is stated once, in
`schemas/reach-classes/v2/spec.md`, the first format that carries ids. Every
format that carries an id points there rather than restating it.

## Input budget and chain bound (adversarial guard)

Documents arriving from disk, from another user, or from the network are
untrusted at every entry point, and these limits apply before any document
can reach a writer:

- A document larger than 1 MiB (1048576 bytes) is rejected. The cap binds
  *before* the read, not after it: a reader checks a file's length or a
  request's `Content-Length` against the budget instead of allocating the
  whole body and then refusing it. `bindforge-core` exports
  `MAX_DOCUMENT_BYTES` and offers `parse_file`/`parse_reader`, which apply
  the check themselves; the hub's upload policy (ADR-014) carries the same
  requirement onto the network path.
- Tables or arrays nested deeper than 16 levels are rejected
  (`MAX_NESTING_DEPTH`).
- A chain deriving from itself is rejected (`Cycle`); a chain longer than
  `MAX_CHAIN_DEPTH = 32` links is rejected (`ChainTooDeep`) instead of
  recursed into. 32 is an initial value ADR-014 may tune but not remove.
- ADR-013 adds four budgets for the manifest's set-shaped data, named beside
  these: `MAX_MANIFEST_ROWS = 256` bounds each array-shaped manifest entry
  (`actions`, `dead-keys`, `docked-actions`, `presets`),
  `MAX_CONSTRAINT_GROUPS = 64` the `exclusive-groups` array,
  `MAX_GROUP_MEMBERS = 16` one group's members, and
  `MAX_NATIVE_NAME_OCTETS = 128` one game-native action name. Each cap sits
  well above the baselines it was surveyed for, and the shared 1 MiB /
  16-deep budgets still bind first.
- ADR-014 adds two budgets for a profile chain's delta data, named beside
  these: `MAX_PATCH_DELTA_ELEMENTS = 256` bounds each of a `profile/v2`
  link's `set`, `clear`, and `swap` lists — mirroring the `MAX_MANIFEST_ROWS`
  cap that bounds one game's whole action table — and
  `MAX_GENERATED_BY_OCTETS = 128` one `generated-by` provenance label.
  ADR-014 adds no aggregate chain cap: one chain resolution opens at most
  `MAX_CHAIN_DEPTH` documents, so its total read cost is the product already
  stated by the two budgets, `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB).
  `schemas/profile/v2/spec.md` states the composite the same way.
- ADR-015 adds two budgets for a policy stack, named beside these:
  `MAX_POLICY_TRANSFORMS = 64` bounds a `policy/v1` document's `transforms`
  array, and `MAX_RELOCATE_TARGETS = 16` bounds one `relocate-cluster` row's
  target list. ADR-015 adds no aggregate cap either: one application reads at
  most one policy document plus the one base document the chain rules already
  bounded, and evaluates at most `MAX_POLICY_TRANSFORMS × MAX_RELOCATE_TARGETS`
  (64 × 16) predicate moves over the capped loaded tables, geometry, and
  registry. `schemas/policy/v1/spec.md` states the composite the same way, and
  because a policy references no other document by path, a per-document cap
  cannot be defeated by document count.
- ADR-024 adds one budget for the ids themselves: `MAX_SLUG_OCTETS = 128` is
  the shared length bound for every id that rides the slug grammar
  (`[a-z][a-z0-9-]*`) — a manifest `id`, a manifest `codec` or preset id, a
  `profile/v2` `id` or `derives` element, a `policy/v1` `id`, a keyboard
  geometry name, and one action-id segment, which keeps its stricter shape
  rules and borrows only this number. The bound rides beside the grammar at
  every one of these ids: the reference implementation checks a geometry name
  against it beside the grammar check, before the name is joined onto a
  directory, so no rider is enforced more narrowly than `024-B1` enumerates
  it.
  The bound supersedes the DNS-label limit RFC 1035 §2.3.1 publishes, which
  these specs used to state for themselves: each of the four now
  cross-references this bullet instead of carrying a number of its own. The
  value is the sibling of `MAX_NATIVE_NAME_OCTETS = 128` — one legal path
  component on every platform this product names — and the amendment is
  relaxation only: an id valid under the 63-octet text stays valid.

- ADR-025 adds one budget beside it for a value that rides no grammar at all:
  `MAX_DISCOVERY_SLUG_OCTETS = 128` caps the `<source>/<external-id>` discovery
  lookup key a discovery plugin reports for one installed game. It is set equal
  to the two 128 bounds above so the exported family reads as one number, and it
  is explicitly *not* a slug-grammar bound: the discovery key is exempt from
  `[a-z][a-z0-9-]*` — its `/` is the point — it is never a path component, and it
  can never be adopted as a manifest `id` or a profile pin (`025-B1`, `025-L5`).
  The reference implementation checks the key against this cap alone at the
  discovery emit boundary and never against the grammar. ADR-025 also borrows
  `MAX_NATIVE_NAME_OCTETS = 128` a second, non-schema time: it bounds a
  discovered game's `friendly_name`, display metadata that joins nothing, which
  is what makes clamping that value admissible.

- ADR-017 adds seven budgets for keybinding extraction, exported by
  `bindforge-codecs` (`crates/codecs`) rather than by core — core's thirteen
  constants below stay core's:
  `MAX_BINDING_FILE_OCTETS = 1_048_576` bounds one file a codec opens
  (`017-B1`, checked on the file's length before allocation),
  `MAX_SNAPSHOT_ENTRIES = 4096` one snapshot's distinct (`action`,
  input-token) pairs *as admitted* (`017-B2` — a removal frees no room),
  `MAX_CANDIDATE_FILES = 32` one user tree's located candidate set
  (`017-B3`), `MAX_USER_DIRS_PER_GAME = 16` the `SteamUserdata` trees kept
  per game across every resolved root, `MAX_LOCATE_DEPTH = 8` the components
  in a located file's tree-relative path, `MAX_LOCATE_ENTRIES_PER_TREE =
  16384` the entries one locate walk examines below one tree root, and
  `MAX_SNAPSHOT_OCTETS = 4_194_304` one snapshot document the loader reads
  (checked on the file's length before the read). `MAX_USER_DIRS_PER_GAME`
  is the one silent bound: past the 16th located tree a game's remaining
  local users are simply absent from the output, and no refusal names it.
  ADR-017 ships no aggregate constant, one sentence here in the shape
  `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` already uses: one extraction opens
  at most `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` files per
  pass — the `+1` is the Proton prefix tree, and because each tree's *opens*
  are capped at `MAX_CANDIDATE_FILES` independently of how many of them
  locate, the product caps the opens too — and the claim pass reads the
  opened files for its content test while the extraction pass re-reads each
  located file to fold it, so the worst case is
  `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES ×
  MAX_BINDING_FILE_OCTETS` (1,088 MiB) read per game, offline: `--probe`
  pays one pass, the extracting face two, repeating per matching record the
  face walks. `schemas/bindings/v1/spec.md` states the composite the same
  way.

The reference implementation exports all thirteen constants as the public names
for these limits, so an implementer never has to guess which number a
rejection names.

Rejections are structured errors that say what is wrong, in a single bounded
line: a message quoting part of the document — a declared `format`, a declared
`version`, an entry name — escapes control characters and truncates, so one
hostile document cannot forge extra log lines or inflate log volume, and a
local filesystem path never appears in a message shown to whoever sent the
document. The `fixtures/invalid/` corpora are the contract for which
condition produces which rejection.
