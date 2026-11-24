# Code review — security seat, round 1 dissent pass (c1)

- Seat: security (single specialty: `references/security.md`)
- Pass: c1 dissent — scope is the last revision, checked first against *my*
  round-1 findings
- Diff scope: `git diff a8fd09ec6d6b8210574447f9b68e7cc199f73153..HEAD --
  ':(exclude)docs/designs'` (15 files, +504/−44)
- Inputs used: that diff, `docs/designs/017-keybinding-extraction-codecs/02-proposal.md`,
  `docs/adrs/017-keybinding-extraction-codecs.md`, the runner's gate record, my
  round-1 findings. Source files were opened only to establish findings
  (`crates/codecs/src/{snapshot,codec,read,families/unreal_ini}.rs`,
  `apps/discovery-cli/src/{bindings,main}.rs`, `crates/core/src/error.rs`,
  `crates/codecs/tests/snapshot.rs`, `schemas/bindings/v1/fixtures/**`). No
  review report or transcript was opened.

## Disposition of my round-1 findings

### Finding 1 (major) — `provenance.files` accepted traversal and foreign absolute spellings: **resolved**

The loader now refuses, naming `files`, anything that is not a plain
`/`-separated sequence of `Component::Normal`:
`crates/codecs/src/snapshot.rs:305-323` adds `foreign_spelling` (any `\` or
`:` anywhere) plus `only_ordinary` (every component `Normal`), on top of the
pre-existing empty/leading-`/` check at `snapshot.rs:299-304`.

I re-derived the closure rather than trusting the test names:

- POSIX spellings out of the tree need a `ParentDir` (`..`) or `CurDir` (`.`)
  component — both refused; a `..` can only arise as a whole component when
  splitting on `/`, so `"a/../b"`, `".."`, `"./x.ini"` and
  `"Saved/Config/../../../etc/crontab"` are all refused. Names that merely
  *contain* dots (`"...x"`, `"..b"`) are `Normal` and are not traversal.
- Windows absolute/UNC spellings reach a Linux loader only as raw text and are
  refused by the character test (`C:/…`, `C:Windows`, `\\server\share`,
  `..\..\x.ini`, streams `x.ini::$DATA`), and a forward-slash UNC
  (`//server/share`) is caught by the leading-`/` check. So the grammar is
  safe for a Windows-side consumer of a Linux-authored document too, which is
  the platform the design's Windows compile leg (`A22`) implies.
- The device/cur-dir/root/prefix `Component` kinds are all non-`Normal`, so the
  single `all(Normal)` predicate closes the remaining classes.
- Nothing else in the crate re-enters a `files` string as a path: `provenance`
  is only ever *emitted* (`crates/codecs/src/families/unreal_ini.rs`,
  `apps/discovery-cli/src/bindings.rs:335-339` prints the absolute located
  paths in probe mode, which §The CLI face's Local-paths scoping covers), so the
  loader is the boundary the fingerprint/apply consumers (`017-R1`) will read
  through.

The fix matches what I asked for in all three parts: the refusal names `files`
like every other refusal; `schemas/bindings/v1/spec.md` Bounds now states it
("no leading `/`, no `.` or `..` component, and no `\` or `:` anywhere … any
other spelling is a refusal naming `files`"); and the invalid-fixture corpus
gained `schemas/bindings/v1/fixtures/invalid/files-traversal-files.json`
(`Saved/Config/../../../etc/crontab`), which `expected_key`
(`crates/codecs/tests/snapshot.rs:289-311`) resolves to `files`, so
`every_invalid_schema_fixture_is_refused_naming_its_key` pins the refusal name.
The additional `crates/codecs/tests/snapshot.rs:177-222` sweep covers drive,
UNC, backslash, parent-, current-directory and absolute spellings.
I ran the legs myself: `cargo test -p bindforge-codecs --locked --test snapshot`
→ `test result: ok. 9 passed; 0 failed`, both named tests present;
`--test unreal_ini` → `27 passed; 0 failed`. Valid fixtures are unaffected
(the shipped family's `tree_relative` emits `Normal` components only,
`unreal_ini.rs:548-555`).

### Finding 2 (minor) — raw `ExtractionRefusal` Display onto the operator's stderr: **resolved**

The fix took my second option: every `path` field is rendered through
`bindforge_core::escape_and_truncate` inside `Display`
(`crates/codecs/src/codec.rs:158-205`: `FileOverBudget`, `UndecodableText`,
`UnparsableEntry`, `UnreadableFile`; the two budget variants carry no path), so
the single remaining raw sink, `apps/discovery-cli/src/bindings.rs:305`
(`eprintln!("error: {message}")`), can no longer emit ESC/OSC/second-line bytes
regardless of which caller prints a refusal. `escape_and_truncate`
(`crates/core/src/error.rs:190-204`) escapes `\` and every control character and
truncates at `MESSAGE_VALUE_CHARS`, so composed lines stay single-line and
control-free. The `UnparsableEntry` `detail` field was already escaped at
construction (`unreal_ini.rs:469`, `:475` via `short` = `escape_and_truncate`),
so no field is left raw. The pin I asked for exists and passes:
`crates/codecs/tests/unreal_ini.rs`
`a_control_byte_in_an_echoed_path_is_escaped_and_stays_one_line` (a located
directory named `Weird\x1bDir`, asserting one line, no control characters, and
that the refusing file is still named).

## Security checklist walk

1. **Boundary validation of external input** — `load_snapshot` is the whole
   read face for a shareable document: closed key sets, per-key bounds,
   metadata-length check before the read, and now a closed path grammar. Pass.
2. **Path handling** — no traversal or absolute path survives the loader (above);
   candidate paths come only from `DirEntry::path()` under canonicalized anchors,
   links are never descended or anchored, and no shell string interpolation
   exists anywhere in the diff. Pass; one over-broad-refusal nit below.
3. **Injection** — the grammar test is octet comparison
   (`unreal_ini.rs:272-286`), tuple splitting is one linear pass
   (`split_top_level`); no regex, no shell, no eval. Terminal injection is now
   closed at the Display. Pass.
4. **Secrets/PII** — no credential or host identity in the diff; the fixtures'
   SteamID is a corpus value, and `spec.md` still states `user` is
   launcher-local identity a sharing surface treats as personal data. Pass.
5. **New dependencies** — none in this range; `Cargo.lock` is untouched, no
   advisory surface added. Not applicable.
6. **Filesystem writes** — still none outside test temp roots; every product
   open is `OpenOptions::new().read(true)` (`crates/codecs/src/read.rs:33-38`),
   `017-L1` holds. Pass.
7. **Clean rejection** — refusals name their key/variant, are single-line and
   control-free; the budget caps (`MAX_CANDIDATE_FILES` counted on opens, the
   entry budget stopping the walk *before* entering another directory) are
   enforced before allocation. Pass, with the minor signal-loss finding below.

Budget docs (`schemas/README.md:200-213`,
`schemas/bindings/v1/spec.md` aggregate sentence) were re-checked for honesty
rather than numbers: the two-pass doubling to 1,088 MiB *read* is what the code
now does, and nothing buffers more than one `MAX_BINDING_FILE_OCTETS` file at a
time, so the corrected sentence documents rather than hides a limit.

## Findings

### minor — `crates/codecs/src/families/unreal_ini.rs:263-265`: an unreadable grammar match is now a silent skip, where the code already has the loud path

`locate` used to make *every* read refusal a candidate, so `extract` refused it
by name. The new arm routes `ExtractionRefusal::UnreadableFile` (stat/open
failure — `EACCES`, `EOVERFLOW`, a dangling or exotic entry) into `Err(_) => {}`:
the file is never located, so nothing ever names it. The realistic case is not
exotic: Proton prefixes created under `sudo` hold root-owned `*.ini` files a
normal user's extraction cannot open, and the face then reports
`claim-refused` (exit 3) or `family-confirmed-files-absent` (exit 4) — statuses
whose meaning is "there is no binding store here" — instead of
`extraction-refused` (exit 5) naming the file it was not allowed to read. For an
artifact whose whole provenance posture exists to make drift loud rather than
wrong (`ADR-017` Consequences; `017-L1`'s named-refusal discipline), a
permission failure that is indistinguishable from absence removes the operator's
only signal that the snapshot is *incomplete*, and an incomplete snapshot is
what manifest authoring and `017-R1` consume. No test pins this behaviour (the
only nearby pin, `a_reader_refused_match_is_located_and_refused_by_name`, covers
`FileOverBudget`/`UndecodableText`), and `extract` already maps `UnreadableFile`
to the relative-path form (`unreal_ini.rs:567-571`), so the loud path is
implemented and simply no longer reached. The fail-closed direction is right —
this is signal loss, not an escape — hence minor, not major.
Fix: keep the unreadable match as a candidate so `extract` names it (the
pre-revision behaviour, and the one the §The first family sentence still
describes), or, if skipping is required, count it and make the skip observable —
a skip-and-count line in `--probe` output plus a `#[cfg(unix)]` test that a
mode-`000` located match either refuses by name or is reported as skipped, so
the status cannot silently mean "no bindings".

### nit — `crates/codecs/src/snapshot.rs:311` with `crates/codecs/src/families/unreal_ini.rs:548-555`: the new `:`/`\` refusals are wider than POSIX name legality, so the family can produce a document its own loader refuses

The loader now rejects any `files` entry containing `:` or `\`, but on Linux
both are legal *inside* a name, and `grammar_match` only compares `Saved`,
`Config` and the `.ini` suffix, so a located `Saved/Config/Linux/Game:Game.ini`
or a `d\dr` platform directory yields a contract spelling the loader refuses
naming `files`. Round 1's compat fix (locate and `tree_relative` now carry raw
OS-named bytes, pinned by
`byte_named_directories_and_files_locate_at_their_real_paths`) widened exactly
this channel, so `every_document_the_family_produces_passes_the_loader` stays
green only because no fixture names a file that way. Failure is fail-closed —
loud refusal, never an out-of-tree join — so this is a nit, not a hole. Fix:
make the two producers agree by refusing the *name* at locate time (skip and
count such a match the way an over-cap match is skipped, so a document never
carries an unloadable spelling) or escape the component before joining, and
state which in `schemas/bindings/v1/spec.md`'s Bounds bullet.

## FYI (no severity, not findings)

- `apps/discovery-cli/src/bindings.rs:197` still renders a plugin failure with
  `{error:?}` through the raw `refused()` sink (`main.rs:103-106`); that line is
  unchanged, shared with `list`, and `DiscoveryError` carries plugin-authored
  strings — pre-existing pattern, outside this diff's changed lines, noted only
  because my round-1 fix text mentioned it.
- `crates/codecs/src/lib.rs:51-59` now documents that one `read_dir` listing is
  materialized and byte-sorted before `MAX_LOCATE_ENTRIES_PER_TREE` trims it;
  the per-directory transient is unbounded. Owned by the performance seat.

## Evidence

- Gate record (green at 2026-11-24T07:31:52Z): `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked`, `npm run check`.
- Run by me on the code under review:
  `cargo test -p bindforge-codecs --locked --test snapshot` → `9 passed; 0 failed`
  (`every_invalid_schema_fixture_is_refused_naming_its_key`,
  `the_loader_refuses_escaping_files_spellings_naming_files`,
  `refusal_messages_are_single_line_and_control_free` all present);
  `--test unreal_ini` → `27 passed; 0 failed` including
  `a_control_byte_in_an_echoed_path_is_escaped_and_stays_one_line` and
  `byte_named_directories_and_files_locate_at_their_real_paths`.
- Missing evidence, recorded: no leg runs a Windows test binary (`A22`), so the
  cross-platform reading of the new `files` grammar is established by reasoning
  over the refused character set, not by a executed Windows leg; the minor
  finding's scenario (root-owned prefix files) has no fixture either way.

## Verdict

Both of my round-1 findings are fixed as prescribed and pinned by passing
tests. What remains is one minor signal-loss regression introduced by this
revision and one nit, neither of which admits an out-of-tree read, an injection
channel, a write, or a silent escape. **approve**.
