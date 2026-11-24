# Code review — correctness seat — round 1, pass c0

Design: `docs/designs/017-keybinding-extraction-codecs/02-proposal.md`
Source: `docs/adrs/017-keybinding-extraction-codecs.md`
Diff: `git diff fc28ab94f313d079222d61438bf5b9074a1b8039...HEAD -- ':(exclude)docs/designs'`
Checklist: `.opencode/skills/code-review/references/correctness.md` (correctness only)

## Inputs and evidence

- The scoped diff (64 files, +5986/−80), `02-proposal.md`, ADR-017, and the
  runner's gate record (green at 2026-11-24T06:32:09Z: `npm run check`,
  `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
  -- -D warnings`, `cargo test --workspace --locked` all exit 0).
- Run myself: `cargo test -p bindforge-codecs --locked` — every leg green
  (`test result: ok.` for 10 legs, 64 tests total: 3 + 5 + 4 + 6 + 7 + 6 + 8 +
  23 + 2). The gate record's single captured line
  (`test result: ok. 0 passed; 0 failed`) is the empty unittest leg; the new
  suites do execute and pass.
- **Not read, by the caller's scoping:** `03-build-notes.md`, `01-research.md`,
  and every `reviews/` path other than this report. Checklist items 3 and 5
  (deviations declared in `03-build-notes.md`; build-notes claims true) are
  therefore **not evaluable** here. The deviations listed below must be
  declared there; if they are, findings 4–7 close on the record alone.
- No prior-round findings exist (first pass).

## Checklist walkthrough

1. **Components and interfaces named in the proposal exist, wired, matching
   names and signatures.** Satisfied. `bindforge-codecs` lands at
   `crates/codecs` in the existing `crates/*` glob with `lib.rs`, `codec.rs`,
   `context.rs`, `registry.rs`, `paths.rs`, `read.rs`, `snapshot.rs`,
   `families/unreal_ini.rs` and the nine named suites plus `tests/common`. The
   trait is `CodecFamily { id, appids, detect, extract }` with
   `extract(&Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal>`
   (the proposal's revised shape, not `017-C1`'s single-document one);
   `Detection { family, evidence, per_user }`, `UserCandidates { user, files,
   tree_root }`, `Evidence { AppidTable, InstallTreeProbe }`, `GameContext`
   with the six named fields and `from_record`, `TreeKind`/`UserTree`/
   `user_trees`, `read::{open_capped, decode}`, `registry::{REGISTRY, claim,
   family_ids, is_known_family}` and all seven budget constants with the
   proposal's exact values (pinned at `tests/zz_api_probe.rs:39-45`).
   `discovery` gains `ScanRecord { game, root, library }`, `SteamCore::scan()`
   with `discover()` as its projection, and `scan_records()`/`roots()` inherent
   wrappers on both plugins; `DiscoveryPlugin`, `DiscoveredGame`, `COLUMNS`,
   `json_row` and the emit boundary are untouched (`017-L5`). The CLI gains
   `Commands::Bindings { slug, steam_root, probe }`, `bindings.rs` with the
   four statuses on codes 0/3/4/5, and the renderer generalization to a
   `&[String]` header. Contract amendments 1–4 all landed (README two-format
   passage + budget bullet, `JSON_FIXTURE_FORMATS`, the manifests Fingerprint
   semantics and the `codec` row note, `SOURCE_KEY_TREES`). Public surface also
   carries `claim_with` and `registry::family`; both are implied seams (the
   synthetic-family pin, and the face's id→extract lookup), not new behaviour.
2. **Assumption register.** Honored on every row this diff touches: A1 (the
   trait and row shape are byte-untouched), A2 (per-root enumeration with the
   shared dedupe kept, now owner-aware —
   `steam_common.rs:100-145`), A4 (`open_capped`'s pre-allocation length check),
   A5/A20 (no vocabulary join anywhere; `MouseX`-class tokens are accepted
   verbatim), A6 (`codec` bound read as `MAX_SLUG_OCTETS` at `snapshot.rs:189`),
   A7/`013-R3` (narrowed to `family_ids`/`is_known_family`; `load_manifest`
   gains no refusal), A9/A10 (both carve-outs widen to exactly two names, no
   other sweep change), A12 (the `Cargo.lock` delta is the one stanza plus the
   one CLI edge — verified in the lockfile diff and pinned by
   `zz_api_probe.rs:115`), A17 (`steamuser` as a template constant, never a
   glob), A18 (every root searched; no "main root" concept invented), A21
   (`appids()` empty), A22 (link fixtures are `#[cfg(unix)]` with a
   compile-only other arm). No silently changed assumption found.
3. **Deviations declared.** Not evaluable (see above); findings 4–7 are the
   gaps found in code, each needing a `03-build-notes.md` line.
4. **Scope creep.** None found. No apply face, no snapshot written, no
   `apps/cli`, no `toml_edit`, no catalog leg on the bindings face, no
   vocabulary widening, no `data/` content. `list`'s loop, gate, store,
   columns, JSON keys and exit code are unchanged in the diff; the renderer
   refactor is the generalization the proposal asked for.
5. **`03-build-notes.md` claims.** Not evaluable (out of scope).
6. **Error paths exist and fail closed.** Yes, and each is named: the six
   `ExtractionRefusal` variants print their name
   (`codec.rs:158-191`); the loader refuses by key with escaped, single-line
   messages (`snapshot.rs:94-105`); every budget refuses or skips-and-counts
   rather than truncating silently; the four statuses each own an exit code
   above clap's 2; the undiscovered-slug and scan-failure paths keep `list`'s
   one voice and `ExitCode::FAILURE`. Nothing collapses a refusal into an
   empty snapshot. The one law-shaped gap is finding 1.
7. **Previous rounds.** N/A — first pass.

## Findings

### 1. minor — `crates/codecs/src/registry.rs:36-50` — the appid pass can turn a
### family-confirmed game into a non-claim

§The registry is explicit: "An appid-table hit claims **regardless of what the
grammar's files turn out to contain** — that is exactly `017-L2`'s
family-confirmed-but-files-not-found case, and it is the visible one"
(proposal:480-487). `claim_with` implements the opposite rule, and says so:
"The table selects whose `detect` runs; it does not rubber-stamp: a table
family whose own detect declines does not claim" (`registry.rs:37-39`). So a
family whose `appids()` contains `ctx.appid` but which locates nothing returns
no claim at all, and the face answers `claim-refused` (exit 3) — the exact
collapse `017-L2` forbids ("a codec may recognize the game and locate nothing
… never collapsed into an empty entry list or **a non-claim**"). Consequence
today is nil (A21: no shipped family carries appids), so this is latent: it
bites whenever `017-R4` lands appid data and a listed game has never been
launched. Note `tests/registry.rs:169-178` claims to pin this rule but its
context appid (`900998`) is *not* in `AppidOwner::appids()` (`900100`), so
pass 1 skips that family entirely and the declined-detect case is untested.
Faithful to `017-C1`'s "`detect` claims or does not"; not to the proposal's
registry section.
**Fix:** in pass 1, claim on the table hit itself — run `detect` to populate
`per_user`, but treat `None` as a claim with an empty `per_user` carrying
`Evidence::AppidTable` (the face then reports `family-confirmed-files-absent`
through the existing empty-documents path, `bindings.rs:257-259`); or keep the
rule and record it as a declared deviation plus a Revision-log line, and pin
it with a table-listed family whose `detect` declines.

### 2. minor — `crates/codecs/src/families/unreal_ini.rs:99-104` (and
### `apps/discovery-cli/src/bindings.rs:152-159`) — documents are ordered by
### `tree_root` byte order, but the shipped spec's ordering law says
### `user_trees` order

`schemas/bindings/v1/spec.md:85-95` (this diff) states the canonical law:
"documents and their user-tree groups in `user_trees` order", and that "a
conforming serializer emits … arrays in exactly the order above". `user_trees`
order is defined as `TreeKind` declaration order, then root byte order
(`paths.rs:1-12`, proposal:317-319). `extract` instead re-sorts the groups by
`tree_root` alone and ignores kind. For the ordinary case — one Steam root
that is its own library, a game with both a `userdata/<steamid>/<appid>` tree
and a `<lib>/steamapps/compatdata/<appid>/…` tree — the two orders are
opposite: `steamapps/…` sorts before `userdata/…`, so the shipped serializer
emits the Proton document first while the contract says the SteamUserdata one
comes first. `--json` array order and table row order are affected. Both
orders are deterministic, so `017-L4` holds and no test can see this (the
shuffle test at `tests/determinism.rs:48-76` only asserts arrival-order
insensitivity; nothing pins which order is *the* order). The R1 revision entry
("documents deduplicate by tree identity — the `UserCandidates.tree_root`
alone, so §Determinism's order over documents is total") reads as the design
reaching for a total order, but §Determinism's sentence, the one copied into
the shipped spec, was not amended to say tree-root order.
**Fix:** pick one and make the diff self-consistent — either sort so the
emitted order *is* `user_trees` order (carry `TreeKind` on `UserCandidates`,
sort by `(kind, tree_root)`), or amend `schemas/bindings/v1/spec.md` §Ordering
(and a Revision-log line) to name documents ordered by `provenance`-independent
tree-root byte order, since the spec is the third-party conformance text.

### 3. minor — `crates/codecs/src/families/unreal_ini.rs:143-168`, called at
### `108-111` — the snapshot's `game` key is reverse-engineered from the tree
### path, so an out-of-position `userdata`/`compatdata` component turns a good
### extraction into a named refusal

`game_key` scans the located tree's path components for the *first* component
named `compatdata` (taking the next as the appid) or `userdata` (requiring it
to be exactly two components from the end) and refuses with
`ExtractionRefusal::UnreadableFile` — printed as `extraction-refused`, exit 5
— when the guess fails. A Steam root or library directory that merely contains
a component with either literal name (a root at `/data/userdata`, a library
folder named `compatdata`) breaks the reconstruction: for the `userdata` arm
the first match is at the wrong depth and the `components.len() == index + 3`
guard fails; for the `compatdata` arm the next component is not digits. The
authoritative value was in hand at `detect` time (`ctx.lookup_key`), and the
`game` key can only ever be `steam/<ctx.appid>` because `user_trees` builds
both templates from it.
**Fix:** carry the lookup key (or the appid) from `GameContext` onto
`UserTree`/`UserCandidates` when the resolvers build them, and have `extract`
copy it into the document; delete the component-scan heuristic.

### 4. minor (declaration) — `crates/codecs/src/families/unreal_ini.rs:184-245`
### — the locate walk is breadth-first; §The first family says "depth-first in
### byte order"

`locate` drives a `VecDeque` with `push_back`/`pop_front` (its own comment
calls it a "Byte-ordered level walk"), where proposal:498-499 specifies "walks
each `UserTree` **depth-first** in byte order". Determinism and the depth/
entry/`MAX_CANDIDATE_FILES` caps are unaffected; only *which* files a tree
with more than `MAX_LOCATE_ENTRIES_PER_TREE` entries surfaces as candidates
can differ (the first 32 path-grammar matches in walk order). Undeclared
deviations are the blocker-shaped item on this checklist, so it is recorded
here at the severity its behaviour warrants.
**Fix:** record it in `03-build-notes.md` (with the reason level-order makes
the entry budget fail shallow-first, which is the friendlier degradation), or
make the walk depth-first byte order as specified.

### 5. nit — `crates/codecs/src/families/unreal_ini.rs:335-339` — `Fold::clear`
### drops `seen`, so re-admitting an identical pair after a bare `!Bindings=`
### charges the entry budget again

§Read discipline defines the bound as "the **distinct** (`action`, input-token)
pairs a tree's fold has ever **admitted**". `clear()` empties `seen` alongside
`pairs` (keeping only `admitted`), so a file that alternates
`!Bindings=` and `+Bindings=(Name=…,Command=…)` on the same pair admits the
same pair once per cycle, and `admitted` can reach
`MAX_SNAPSHOT_ENTRIES + 1` on a fold holding one pair — a
`SnapshotEntryBudget` refusal where the design's count says the document is
in budget. Fails closed (named refusal, never a wrong snapshot), and needs
~4 097 clear-and-re-add cycles in one ≤ 1 MiB file, so the impact is a
spurious exit 5 in a pathological file.
**Fix:** keep `seen` across `clear()` (clear only `pairs`), or count admissions
into a set of every pair ever seen so "distinct pairs ever admitted" is
literally the counter.

### 6. nit — `apps/discovery-cli/src/bindings.rs:249-268` — `status` is
### aggregated per family, so one record's nothing-to-read is masked by another
### record's entries

§The CLI face step 3 runs `from_record → claim → extract` **per matching
record**, and step 5 exits "the highest-ranked status **it reported**". The
face instead merges every record's claim per family
(`MergedClaim::absorb`, `bindings.rs:118-148`) and extracts once, then asks
whether *any* document has entries (`bindings.rs:257-259`). So a run over two
records of one slug — one whose located store states nothing readable, one
whose store has bindings — reports `ok`/exit 0 where the per-record reading
reports `family-confirmed-files-absent`/exit 4. Output documents are the same
either way (tree-identity dedup makes merge and per-record extraction
equivalent), so this is the status half only.
**Fix:** either derive the files-absent status per record (a record whose own
claim's documents all fold empty) before taking the `max`, or state in
`03-build-notes.md` that status is per family over the merged claim.

### 7. nit — `crates/codecs/Cargo.toml:16-20` — `serde_json` is a load
### dependency, where the proposal's crate layout lists it as a dev-dependency

§`crates/codecs` layout (proposal:214-217) says "dev-dependency `serde_json`
(A12)", but `snapshot.rs`'s loader is a JSON parser, so the crate needs it at
run time; the Cargo.toml comment argues exactly that, and the lockfile cost is
zero (`serde_json` already resolved, A12). The proposal's line is the
internally inconsistent one; the code is right.
**Fix:** one line in `03-build-notes.md` recording the dependency-edge shape
so the deviation is on the record.

## Missing evidence, stated

- `03-build-notes.md` is outside the caller's inputs, so I cannot confirm that
  findings 1, 4, 6 and 7 are declared, nor audit any build-notes claim
  (checklist items 3 and 5). None of them is blocking on that ground alone:
  each finding above identifies either a latent-only behaviour (1), an
  order-of-appearance difference inside a deterministic output (2, 6), a
  contrived-path failure (3), or a documentation gap (4, 5, 7).
- The Windows *compile* leg's half of the new face (the `cfg(windows)` plugin
  selection at `bindings.rs:173-181`, the `cfg(not(unix))` fixture arm) is not
  exercised by any leg I can run; the gate record does not include the
  `x86_64-pc-windows-msvc` leg (A22 says CI runs it). Nothing in the diff looks
  like it would fail there, and a `target/x86_64-pc-windows-msvc` directory in
  the working tree suggests it has been run locally.

## FYI (out of specialty, no severity)

`ExtractionRefusal`'s `path` field is echoed into the CLI's `error: ` line
without `escape_and_truncate`; the values are filesystem paths (which §Local
paths permits this face to print) rather than file contents, so the design's
"every value from a file still goes through `escape_and_truncate`" sentence is
not on its face violated — but a config directory named with a newline would
produce a two-line error line.
