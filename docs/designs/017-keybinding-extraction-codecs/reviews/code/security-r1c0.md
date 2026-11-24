# Code review — security seat, round 1 pass c0

Design: `docs/designs/017-keybinding-extraction-codecs/02-proposal.md`
Source: `docs/adrs/017-keybinding-extraction-codecs.md`
Scope: `git diff fc28ab94f313d079222d61438bf5b9074a1b8039...HEAD -- ':(exclude)docs/designs'`
(64 files, +5986/−80: `crates/codecs/**`, `apps/discovery-cli/{src,tests}/**`,
`crates/discovery/**`, `crates/core/tests/**`, `schemas/**`, `Cargo.lock`)
Inputs used: the diff, `02-proposal.md`, ADR-017, the runner's gate record. No
other review file was opened.

**Verdict: request-changes** — one major, one minor.

## Checklist walk

1. **External input validated at the boundary** — *partially met*. Shared
   binding files are untrusted and are read only through `read::open_capped`
   (`crates/codecs/src/read.rs:21-52`): `fs::metadata` length against
   `MAX_BINDING_FILE_OCTETS` before allocation, then `take(limit + 1)` so the
   read itself stays bounded, then a non-lossy decode (`read.rs:58-92`) that
   refuses unpaired UTF-16 surrogates and BOM-less invalid UTF-8 as
   `UndecodableText`. Budgets land as named refusals, never empty snapshots.
   `serde_json` parsing goes through a closed key set at every level
   (`crates/codecs/src/snapshot.rs:148-164, 229-236, 269-276`) and no value is
   ever evaluated. The gap is the path-shaped field — Finding 1.
2. **Path handling** — *not met for the loader boundary* (Finding 1). The
   resolver side is clean: every path a codec can reach is built from
   constants plus validated digits — `GameContext::from_record` admits only a
   `steam` token with a 1–20 ASCII-digit appid
   (`crates/codecs/src/context.rs:49-59`), `user_trees` keeps only digit-named
   real directories from `read_dir` (`crates/codecs/src/paths.rs:45-49, 85-99`),
   and the Proton template is six literal joins off the owning library
   (`paths.rs:117-126`). I verified the refactor did not de-canonicalize the
   anchors: `libraries_in_root_with_skips` still ends in `canonical_paths`
   (`crates/discovery/src/steam_common.rs:165-180`), so `ScanRecord.library` —
   the Proton anchor — is canonical as §Config-path resolvers claims. No
   shell, no `Command::new`, no string-interpolated command anywhere in the
   diff.
3. **Injection** — met, with the escape gap at Finding 2. No regex is
   introduced, so there is no backtracking surface; `split_top_level`
   (`crates/codecs/src/families/unreal_ini.rs:481-497`) and the line fold are
   single-pass over a size-capped buffer. File names never come from document
   content.
4. **Secrets and PII** — met. No credential, token or host username appears in
   source, fixtures or messages. `user` (a local SteamID) and the `--probe`
   absolute paths are PII-shaped and are handled as such: `user` is a token
   with a control-char-free grammar (`snapshot.rs:351-375`), and
   `schemas/bindings/v1/spec.md:110-115` names it, and probe paths, personal
   data for any sharing surface — which is the scoping §Local paths decided and
   which nothing in the diff contradicts (no upload, no cache, no write).
5. **New dependencies** — met. `crates/codecs/Cargo.toml` adds no external
   package beyond `serde_json` (already resolved), and the `Cargo.lock` delta is
   exactly the two edges the proposal promised: the `bindforge-codecs` stanza
   and the `discovery-cli` → `bindforge-codecs` line. No resolved version moves,
   so no new advisory surface enters and `cargo deny`'s set is unchanged.
6. **Filesystem writes** — met (N/A as designed, `017-L1`). The only open in
   product code is `OpenOptions::new().read(true)` (`read.rs:33-38`); grepping
   the new sources for `fs::write`/`File::create`/`create_dir`/`remove_file`/
   `rename` returns only doc comments and that one read. The writes in
   `apps/discovery-cli/src/matches.rs` are the pre-existing store, untouched by
   this diff.
7. **Clean rejection of malformed/hostile input** — met apart from Finding 2.
   Every refusal is a named variant with a single-line `Display`
   (`crates/codecs/src/codec.rs:158-191`); the snapshot loader's refusals are
   escaped, control-free and truncating (`snapshot.rs:95-105`), pinned by
   `crates/codecs/tests/snapshot.rs:89-109`. Walks stop at
   `MAX_LOCATE_ENTRIES_PER_TREE` and prune at `MAX_LOCATE_DEPTH`
   (`unreal_ini.rs:203-224`), links are skipped at the entry, at the tree root
   and at the tree anchor (`paths.rs:76-99`, `unreal_ini.rs:207-214`), and
   `tests/paths.rs:141-177` plus `tests/unreal_ini.rs:184-221` pin that a link
   is never descended, read or anchored. No stack trace reaches a user: the
   face prints `status:` + `error:` lines and its own exit code.

## Findings

### 1. major — the `bindings/v1` loader accepts `provenance.files` entries that escape the tree

**Location:** `crates/codecs/src/snapshot.rs:296-319` (the `files` loop; the
whole shape check is `file.is_empty() || file.starts_with('/')` at line 299 and
the component count at 305-308)

**Problem.** The loader is the boundary validator for a document the contract
treats as shareable, and it is the only place `bindings/v1` is checked. Its
`files` refusal knows two shapes: empty and POSIX-absolute. Everything that
denotes *outside the tree* by another spelling loads:

- `"../../../../etc/passwd"` → `Path::components()` is
  `[ParentDir, ParentDir, ParentDir, Normal]` = 4 ≤ `MAX_LOCATE_DEPTH` (8);
- `"Saved/Config/../../../etc/crontab"` → 5 components, passes;
- `"C:/Windows/win.ini"`, `"\\\\server\\share\\x"` and `"..\\..\\x"` pass on
  every platform and are absolute or traversal spellings to the Windows
  resolver this run compiles for (A22's compile leg, `steam_windows.rs`);
- `"./x.ini"`, whose `CurDir` component Rust's `components()` drops, so the
  count under-reads the raw spelling.

Grep confirms no `ParentDir`, `..`, or `is_absolute` check exists anywhere in
`crates/codecs`, and the invalid corpus has only the depth case
(`schemas/bindings/v1/fixtures/invalid/files-depth-files.json`), so the
conformance currency never exercises traversal and the green gate cannot
establish the property. `schemas/bindings/v1/spec.md:45, 69-70` asks only for
"tree-relative paths" bounded in component count, so code and spec are jointly
under-specified here.

**Why it matters now, not later.** This run itself makes those entries
executable data: the `schemas/manifests/v1/spec.md:128-136` amendment it ships
defines the fingerprint as "the located config files' octets concatenated in
the extracted snapshot's sorted `provenance.files` order". ADR-017's named
consumers of a snapshot — manifest authoring, the apply tool (`017-R1`), the
UI's import — must therefore resolve those strings against a tree root. With no
refusal, a hostile or doctored document that passes validation states files
outside the tree it claims to have read, and `017-L3`'s provenance claim stops
being checkable by any consumer that trusts the loader. Catching it here is the
cheap moment: once `bindings/v1` ages it is a corpus-wide contract question.

**Fix.** Refuse any `files` entry that is not a plain `/`-separated sequence of
`Normal` components: reject `Component::Prefix`/`RootDir`/`CurDir`/`ParentDir`
(keeping the current `files`-naming refusal), and reject a raw spelling
containing `\\` or `:` so the Windows resolver cannot be fed an alternate
absolute form. State the rule in `schemas/bindings/v1/spec.md`'s Bounds bullet
and add `schemas/bindings/v1/fixtures/invalid/files-traversal__files.json` so
`every_invalid_schema_fixture_is_refused_naming_its_key` pins the refusal name;
the family's own `tree_relative` output is unaffected, so no valid fixture or
document changes.

### 2. minor — the extraction-refusal line echoes filesystem paths unescaped

**Location:** `apps/discovery-cli/src/bindings.rs:293`
(`eprintln!("error: {message}")`), fed by
`crates/codecs/src/codec.rs:158-191` (the `Display` impls interpolate `path`
raw) and `crates/codecs/src/families/unreal_ini.rs:506-511`
(`tree_relative`, `to_string_lossy` over `read_dir` names; also
`unreal_ini.rs:109-111`, which echoes `tree_root.display()`).

**Problem.** Every path a refusal echoes is assembled from directory and file
names the located tree chose. On Linux a file name may contain ESC or any other
control byte, so a game's own config tree — the input class this design calls
untrusted — can push raw escape sequences onto the operator's terminal through
the `error: ` line (cursor movement, OSC title/clipboard set, color resets that
hide the following output). The same values on stdout are already safe: the
table goes through `cell()` (`apps/discovery-cli/src/main.rs:348-356`), which
escapes any control byte, and `--json` output is escaped by `serde_json`. The
extraction half therefore breaks the discipline this run states for its own
messages — `schemas/bindings/v1/spec.md:104-106`: "Every message is
single-line, control-free and truncating; echoed values pass through the shared
`escape_and_truncate` renderer" — and the risks row "bounded escaped echoes"
only holds for stdout. Note the loader refusals do obey it
(`snapshot.rs:95-105`, pinned by `tests/snapshot.rs:89-109`), and
`UnparsableEntry`'s *detail* is escaped via `short()`
(`unreal_ini.rs:499-502`); it is the path fields that are not.

**Fix.** Escape at the sink or at the source: either
`eprintln!("error: {}", bindforge_core::escape_and_truncate(&message))` at
`bindings.rs:293` (and in `refused`, if the scan-failure line should keep the
same guarantee), or have `ExtractionRefusal`'s `Display` render every `path`
field through `escape_and_truncate`. Add one fixture route whose directory
name carries a control byte and assert the `error:` line is single-line and
control-free, so the guarantee is pinned rather than intended.

## FYI (out of specialty, no severity)

`crates/codecs/Cargo.toml` ships `serde_json` as a load dependency while
§`crates/codecs` of the proposal says "dev-dependency `serde_json`" (the lock
delta is unaffected); and `Fold::remove`'s full `pairs.retain`
(`unreal_ini.rs:329-333`) makes a hostile 1 MiB file's removal lines cost O
(admitted pairs) each — performance seat's to weigh.

## Gate record

The runner's 2026-11-24T06:32:09Z gate is green on all four legs
(`npm run check`, `cargo fmt`, `cargo clippy -D warnings`, `cargo test
--workspace --locked`). Nothing in it speaks to either finding: no fixture or
assertion in the diff exercises a `..` path in `provenance.files`, and none
routes a control byte through an `error: ` line. I ran no build or test
commands myself, only read-only inspection.
