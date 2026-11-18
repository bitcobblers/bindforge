# Build Notes

## Progress

- **Completed Behaviors**:
    - `crates/discovery/src/keyvalues.rs` (new, §1/§4): hand-rolled KeyValues reader —
      quoted and bare keys/values, `{}` sections in declaration order, `//` comments,
      leading BOM, LF/CRLF, `\\` + `\"` unescapes, case-insensitive first-match
      `get_str`/`get_section`, positioned `KeyValueError { line, message }` for
      unbalanced braces, unterminated strings, `[` conditionals and `#base`, and the
      `MAX_NESTING_DEPTH = 16` / `MAX_DOCUMENT_BYTES = 1 MiB` caps (exported, per the
      repo's parse-budget naming). No new dependency; `Cargo.lock` unchanged.
    - `crates/discovery/src/steam_linux.rs` (new, §2–§7): `SteamLinuxPlugin` with
      `from_host()` (`HOME` via `var_os`, never the deprecated `home_dir`) and
      `from_roots()`; pure `host_candidates(home, env)` with the §2 candidate order and
      the absolute-only `XDG_DATA_HOME` / `SNAP_USER_DATA` handling; `libraries_in_root`
      (root always included as a library, numeric-keyed VDF `path`s, relative values
      joined to the VDF's directory); the §2 canonical path rule (canonicalize →
      drop unresolvable / lacking `steamapps` → sort → de-dup) applied to roots, to each
      root's libraries and to the cross-root library set inside `discover()`; `parse_manifest`
      with the `name`-absent/empty fallback to `installdir` and the
      `.`/`..`/absolute/`/` refusals; per-file skip-and-continue, one `Err` path (§7);
      output sorted by `(slug, install_path)`; AppID never read.
    - `apps/discovery-cli`: `list` gained the repeatable `--steam-root`; `DummyPlugin`
      and its now-unused imports are deleted; the printing code is untouched (§8).
    - `crates/discovery/src/lib.rs`: exports `keyvalues` and `steam_linux` alongside the
      existing five modules.
- **Remaining Work**:
    - Nothing left in T-001's design scope in the source tree.
    - Two pre-existing assertions need the test engineer (see *Test Contract Conflicts*);
      this stage may not edit test files, so they are routed to `test-update`.
    - Rollout step 5 (a contributor's real `discovery list` / `list --json` output from a
      machine with Steam) is not performed here: no such machine is available in this
      run, and design review already flagged pasting raw host files into a checked-in
      artifact. It stays an open, human step.
- **Relevant Paths and Interface Decisions**:
    - `crates/discovery/src/{keyvalues.rs,steam_linux.rs,lib.rs,game.rs}`,
      `apps/discovery-cli/src/main.rs`.
    - The public surface is exactly §1's list; the scan loop, composition helper, skip
      counting and the §7 message builder are private.
    - `Platform` now carries `#[serde(rename_all = "kebab-case")]` (see *Deviations* 1).
    - A file that is merely absent is not counted as a skipped file; unreadable, oversize
      and non-UTF-8 files are (see *Deviations* 3).
- **Last Validation Result**: `cargo clippy --workspace --all-targets -- -D warnings`
  passed; `cargo fmt --all -- --check` and `cargo test --workspace --locked` fail only
  inside test files this stage does not own — see *Gate Posture*.
- **Next Concrete Action**: `test-update` for `crates/discovery/tests/steam_linux_tests.rs`,
  `apps/discovery-cli/tests/cli_tests.rs` (secondary-library `install_path`) and the
  rustfmt pass over those files plus `crates/discovery/tests/keyvalues_tests.rs`; then
  re-run the three cargo legs.

## Deviations from the design

1. **`Platform` gained a serde attribute** (`crates/discovery/src/game.rs`), although
   *Context* states "no change to `DiscoveredGame`, `Platform`".
   `apps/discovery-cli/tests/cli_tests.rs:93` asserts `platform == "steam-linux"`, the
   form ADR-022 documents and `Platform`'s own `Display` already prints, while serde's
   default emitted `"SteamLinux"`. Only the `#[serde(rename_all = "kebab-case")]`
   attribute was added: the variants, the five `DiscoveredGame` fields and the JSON key
   set are unchanged, so the wire form now agrees with `Display` instead of contradicting it.
2. **How an unterminated string is detected** (§4 leaves the mechanism open): a quoted
   value that reaches a newline before its closing quote is refused as `unterminated
   string`, positioned at the line the string *opened* on. A scan that instead runs to the
   next quote would close the string on a later line and report the wrong position, which
   `keyvalues_tests.rs::rejects_unterminated_string_with_positioned_error` pins.
   Consequence: multi-line quoted values are refused; none is known in these two file
   kinds, and the accepted value ends up identical to what §4 specifies.
3. **"Absent" is not a skip** (§7 lists "a missing or malformed `libraryfolders.vdf`"
   among the things skipped): an absent file is treated as a normal degraded scan and is
   *not* counted in §7's skipped count, while unreadable, oversize and non-UTF-8 files
   are. Counting an absent file would report the §7 error for "Steam installed, nothing
   under it", which §7 rules out. A malformed or unreadable VDF is still counted.
4. **The skip count is threaded through a private helper**: `libraries_in_root` keeps
   §1's signature and delegates to a private counting variant, so a malformed
   `libraryfolders.vdf` can reach §7's counter from inside `discover()`.
5. **Parse-budget constant placement**: `MAX_DOCUMENT_BYTES` and `MAX_NESTING_DEPTH` are
   exported from `keyvalues.rs` (no `bindforge-core` edge exists for `crates/discovery`),
   and the size cap is applied to `metadata` before the contents are allocated, per
   `repo-practices` ("cap input before they allocate it").
6. **One extra `installdir` refusal**: an interior NUL byte. §5's rule exists because
   the value is appended to an emitted path that a later consumer opens, and a NUL is not
   a name any platform can open; no documented case changes posture because of it.
7. **`install_path` keeps the library prefix** — `install_path` for a manifest found in
   library `l` is `l/steamapps/common/<installdir>` with `l` the library that held the
   manifest (§5, A2, §2's "canonical library prefix"), which is what two of the
   pre-existing assertions dispute. See *Test Contract Conflicts*; this is not a change of
   design, it is the design.

## Test Contract Conflicts (why `testsNeedUpdate` is true)

Two assertions fail, both the same single mistake: they build the expected
`install_path` of the **secondary library's** game from the fixture *root* instead of
from the library its manifest sits in, which contradicts A2 ("the game directory is
`<library>/steamapps/common/<installdir>`"), §5's composition rule, test strategy #3
(`libraries_in_root` returns `<root>` **and** `<root>/secondary` — that test passes here)
and the failing test's own comment ("`install_path` is `<canonical library>/steamapps/common/<installdir>`").

- `crates/discovery/tests/steam_linux_tests.rs:373-385`
  (`discover_output_honours_the_game_output_contract`): the `(slug, install_dir)` table
  uses one `root` for all four rows. `unknown/secondary-title` needs
  `<root>/secondary/steamapps/common/Secondary Title`, i.e. a per-row library
  (`""`, `""`, `""`, `"secondary"`).
- `apps/discovery-cli/tests/cli_tests.rs:82-101` (`test_cli_list_json_format`): the
  `(slug, display_name, install_dir)` table has the same shape; the fourth row's `want`
  must be `<root>/secondary/steamapps/common/Secondary Title`.
- Everything else in both files passes as written: the four slugs and display names, the
  `(slug, install_path)` order, `platform == "steam-linux"`, `codec_id == null`, the
  five-field JSON shape, the table header, the `[]` case, the decoy and compatdata
  exclusions, and all 22 other `steam_linux_tests` / all 15 `keyvalues_tests` cases.
- Separately, the three test files are not rustfmt-clean (18 `Diff in` hunks), so
  `cargo fmt --all -- --check` fails on them. They need `rustfmt` in the same round; this
  stage's own five source files are rustfmt-clean.

Bending the product instead — composing a secondary library's paths from the primary
root — would emit a non-existent directory for every game in every secondary library
(the whole reason `libraryfolders.vdf` exists), so it was not done.

## Gate Posture

Legs run in this stage's shell against the working tree (`--offline` added only because
this sandbox has no network; `Cargo.lock` shows no diff in `git status`).

- `cargo fmt --all -- --check` — **fails**; every reported file is a test file, e.g.
  ```
  Diff in <repo>/apps/discovery-cli/tests/cli_tests.rs:50:
  Diff in <repo>/crates/discovery/tests/keyvalues_tests.rs:25:
  Diff in <repo>/crates/discovery/tests/steam_linux_tests.rs:9:
  ```
- `cargo clippy --workspace --all-targets -- -D warnings` — **passes**:
  ```
  Checking discovery v0.1.0
  Checking discovery-cli v0.1.0
  Finished `dev` profile [unoptimized + debuginfo] target(s)
  ```
- `cargo test --workspace --locked` — **fails** on the conflict above (all other
  workspace suites pass):
  ```
  test discover_output_honours_the_game_output_contract ... FAILED
  panicked at crates/discovery/tests/steam_linux_tests.rs:380:9
   left: Some("<repo>/crates/discovery/tests/fixtures/steam/minimal/secondary/steamapps/common/Secondary Title")
   right: Some("<repo>/crates/discovery/tests/fixtures/steam/minimal/steamapps/common/Secondary Title")
  test result: FAILED. 22 passed; 1 failed
  ```
  and, from `cargo test -p discovery-cli --locked`:
  ```
  test test_cli_list_json_format ... FAILED
  panicked at apps/discovery-cli/tests/cli_tests.rs:96:9
   left: String("…/minimal/secondary/steamapps/common/Secondary Title")
   right: "…/minimal/steamapps/common/Secondary Title"
  test result: FAILED. 2 passed; 1 failed
  ```
  `<repo>` and `…` mark elided absolute-path prefixes; no path or credential is dropped
  from the record beyond that elision.
