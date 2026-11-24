# Code review — testing seat — round 1, pass c0

Change under review: `git diff fc28ab94f313d079222d61438bf5b9074a1b8039...HEAD -- ':(exclude)docs/designs'`
(64 files, +5986/−80). Design: `docs/designs/017-keybinding-extraction-codecs/02-proposal.md`;
source ADR: `docs/adrs/017-keybinding-extraction-codecs.md`. Specialty applied: `references/testing.md` only.

## Verdict

**approve** — zero blocker and zero major findings. The proposal's Test strategy has 17 numbered
items; all 17 are cited by at least one test in the diff, unit and integration layers are both
present, the required gates are green, and no test depends on wall-clock, a live service or a fixed
port. Six findings follow: four `minor` coverage gaps and two `nit`s. None of them withholds
approval, because each names a clause whose *required* property is pinned elsewhere or whose
absence cannot produce a false artifact.

## Gate evidence

Supplied by the runner (gate at 2026-11-24T06:32:09.463Z, green — cited, not re-derived):
`npm run check` exit 0; `cargo fmt --all -- --check` exit 0;
`cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo test --workspace --locked` exit 0
(the single `test result: ok. 0 passed; …` line the renderer quoted is the crate's doc-test leg —
`cargo test -p bindforge-codecs` ends with `Doc-tests bindforge_codecs … 0 passed`, so the excerpt is
the empty doc leg, not an empty suite).

Checks I executed, to resolve the specific coverage question "do the new suites actually run in the
workspace, or are they dead files?":

- `cargo test -p bindforge-codecs --locked` → green; `tests/unreal_ini.rs` reports
  `23 passed; 0 failed`, `tests/zz_api_probe.rs` `2 passed`, and every new target
  (`context`, `determinism`, `honesty`, `paths`, `read`, `registry`, `snapshot`, `unreal_ini`,
  `zz_api_probe`) ran.
- `cargo test -p discovery-cli --locked` → green; `cli_bindings_status_tests` `7 passed`,
  `cli_bindings_tests` `10 passed`, and the four `list` suites named by Test strategy #14
  (`cli_output_polish_tests`, `cli_json_mode_tests`, `cli_catalog_gate_tests`,
  `cli_slug_refusal_tests`) ran unmodified — none of them appears in the diff.

## Test strategy coverage (checklist item 1)

| # | Where it is cited | Read |
|---|---|---|
| 1 `ScanRecord`/`scan()` | `crates/discovery/tests/scan_record_tests.rs:34,73,121,152` | covered: ownership, shared-library single scan owned by the first root, `discover()` as the exact projection in `(slug, install_path)` order, `roots()` canonicalized/filtered/byte-ordered |
| 2 `from_record` | `crates/codecs/tests/context.rs:30,62,83` | covered, incl. `local/x`, `epic/…`, `steam/`, non-digit, 21-digit → `None`, and the 1/20-digit extremes |
| 3 `user_trees` | `crates/codecs/tests/paths.rs:29,62,104,118,135,184` | covered incl. the 9+8-digit two-root 16-kept/17th-dropped cap ("counted" half unobservable — see finding 6) |
| 4 budgets at the seams | `crates/codecs/tests/read.rs:17,27`; `tests/unreal_ini.rs:261,282,311,578`; `tests/snapshot.rs:111` | covered, incl. `UserCandidates::push` refusing the 33rd and both `MAX_SNAPSHOT_ENTRIES` breach shapes |
| 5 `read::decode` | `crates/codecs/tests/read.rs:47,56,65,76,95`; `tests/unreal_ini.rs:610` | covered, UTF-8 / UTF-8-BOM / UTF-16LE / UTF-16BE twins extract identically |
| 6 locate walk | `crates/codecs/tests/unreal_ini.rs:37,84,105,124,152,180,225`; `tests/paths.rs:135` | covered; the named `symlinked-prefix` fixture tree does not exist (finding 1) |
| 7 the fold | `crates/codecs/tests/unreal_ini.rs:333,359,378,398,419,442,463,493,512,534,557` | covered clause by clause, `MouseX`/`Left`/`Gamepad_RightX`/`TAB` verbatim and unrefused |
| 8 extraction honesty | `crates/codecs/tests/honesty.rs:45,78,147,180` | covered over the fixture documents |
| 9 registry | `crates/codecs/tests/registry.rs:89,108,139,159,181,215` | covered; pass ordering pinned both slice arrangements |
| 10 snapshot loader | `crates/codecs/tests/snapshot.rs:13,41,64,86,136,202` + 3 valid / 16 invalid fixtures | covered for every case the item names; some implemented refusals have no fixture (finding 3) |
| 11 family ⇄ loader agreement | `crates/codecs/tests/snapshot.rs:177` | covered (byte round-trip) |
| 12 determinism | `crates/codecs/tests/determinism.rs:27,43,78,138,179`; `apps/discovery-cli/tests/cli_bindings_tests.rs:99,154,402` | covered: repeated extraction, `per_user` reversed/rotated, tree-identity dedup, two libraries one label → two documents, table and `--json` byte-identical across subprocess runs |
| 13 CLI face | `cli_bindings_tests.rs:48,99,167,203,228,272,318`; `cli_bindings_status_tests.rs:45,67,96,129,154,168,193` | covered for all four statuses on 0/3/4/5 by real fixture routes, the status document, the probe table/JSON, the no-catalog posture and the bytes-unchanged check; the probe-mode *failure* arms are untested (finding 4) |
| 14 `list` untouched | `cli_bindings_tests.rs:353` + the four named suites green and absent from the diff | covered |
| 15 contract-tree pins | `crates/codecs/tests/registry.rs:215` (the `013-R3` half); `crates/core/tests/common/mod.rs:428` + `conformance.rs:235,259,360` (harness skip vs. zero-fixture/fixture-name coverage); `crates/discovery/tests/boundary_tests.rs:111,124` (sweep carve-out) | mostly covered; the `schemas/README.md` statements no #15 names have no test (finding 5) |
| 16 api probe / lockfile | `crates/codecs/tests/zz_api_probe.rs:26,110` | covered; the "only delta" wording is pinned by presence, and the diff itself confirms it: `Cargo.lock` is +11/−0 (the `bindforge-codecs` stanza and the `discovery-cli` edge), no resolved version moved |
| 17 platform scoping | `crates/codecs/tests/paths.rs:139,177` (`#[cfg(unix)]` link fixtures + compile-only `#[cfg(not(unix))]` arm); `cli_bindings_tests.rs:272` | covered; temp roots use `std::env::temp_dir()` as in `crates/{core,discovery,transforms}/tests/common`, so the #17 "…and `target/`" wording is a repo-wide looseness, not new drift |

## Checklist

1. **Citations** — pass: all 17 items have a `// proves: Test strategy #N` citation in the diff, and each
   citation sits on a test that exercises the clause it names (spot-checked clause by clause for #3–#10).
2. **Layers** — pass: unit suites mirror `src/` per ADR-004 (`tests/{registry,context,paths,read,snapshot,unreal_ini,determinism,honesty,zz_api_probe}.rs`) plus the subprocess integration face.
3. **Revision rounds** — n/a: this is the first pass; no prior blocker/major to re-pin.
4. **Determinism** — pass: no `sleep`, no wall-clock assertion (the only clock use is the temp-dir
   nonce at `crates/codecs/tests/common/mod.rs:53`, never an input to an assertion), no fixed ports
   (`FakeCatalog` binds `127.0.0.1:0`, `apps/discovery-cli/tests/common/mod.rs:185`), `run_cli` strips
   `BINDFORGE_CATALOG_URL`/`HOME`/`XDG_DATA_HOME`/`APPDATA` so the no-network and no-store pins are
   structural rather than asserted against ambient state. Walk-order assertions derive their
   expectations from the canonical roots' own byte order, so `mkdtemp` naming cannot flip them.
5. **Names** — pass: names state behaviors (`a_library_two_roots_share_is_scanned_once_owned_by_the_first`,
   `a_command_fully_removed_leaves_no_entry`), not implementation steps.
6. **Gate evidence** — pass, and the two commands I ran (above) resolve the only open coverage question.
7. **Untestable additions** — four findings below: one refusal variant, one refusal family in the
   loader, and two CLI arms that no test reaches.

## Findings

### Minor

1. **`minor` — the `symlinked-prefix` fixture the design names is absent, and the broken-prefix
   status it exists to pin is unpinned.**
   Location: `crates/codecs/tests/fixtures/unreal/` (fixture trees present: `steam-userdata`,
   `proton-prefix`, `bom-utf16`, `defaults-only`, `bindings-absent`, `broken-tuple`);
   `apps/discovery-cli/tests/cli_bindings_status_tests.rs:96-127`.
   Problem: §The first family lists seven fixture trees including `symlinked-prefix`, and Test
   strategy #6 names "a symlinked *tree root* in `symlinked-prefix`". The link *properties* are
   pinned by test-time symlinks in `crates/codecs/tests/paths.rs:135-175` and
   `crates/codecs/tests/unreal_ini.rs:180-223`, so the unit clause holds — but the scenario those
   fixtures encode (the Compatibility matrix row "Steam library on NTFS/exFAT with symlinked prefixes
   … reads as `family-confirmed-files-absent` (exit 4)" and Risks row 3, "a broken prefix degrades to
   files-absent") is reached by no test. Exit 4 is pinned only by a route where a located file removes
   its own pairs (`cli_bindings_status_tests.rs:103`), which is "files present, nothing stated", not
   "family confirmed, files absent". Under `crates/codecs/src/paths.rs:127-136` a template root that
   is itself a link is dropped, so no tree, no located file and no claim results — which
   `apps/discovery-cli/src/bindings.rs:224-235` reports as `claim-refused` (3). The promised
   degradation for the documented environment is therefore both unpinned and, on the code as
   written, a different status; this seat raises the missing coverage, and the behavior itself is for
   the correctness seat.
   Fix: add the `symlinked-prefix` tree (its links created at test time behind `#[cfg(unix)]` with the
   `#[cfg(not(unix))]` early return, as §The first family already specifies) and one CLI subprocess test
   that runs `bindings <slug>` against a Proton prefix whose `steamuser` root is a link, asserting the
   named status and its exit code — whichever of 3/4 the design settles on.

2. **`minor` — `ExtractionRefusal::UnreadableFile` is a named refusal no test reaches.**
   Location: `crates/codecs/src/codec.rs:152` and its `Display` arm at `codec.rs:186`; produced at
   `crates/codecs/src/read.rs:27,36,45` and `crates/codecs/src/families/unreal_ini.rs:109`.
   Problem: every other variant of the public refusal enum has a test that asserts it by name
   (`FileOverBudget` — `tests/read.rs:27` and `tests/unreal_ini.rs:311`; `UndecodableText` —
   `tests/read.rs:76,95`; `UnparsableEntry` — `tests/unreal_ini.rs:534,557`; `SnapshotEntryBudget` —
   `tests/unreal_ini.rs:578`; `CandidateFilesBudget` — `tests/unreal_ini.rs:282`). Grepping the whole
   workspace for `UnreadableFile` returns only `src/` hits, so the branch at
   `unreal_ini.rs:109` (`game_key()` declining a tree root that spells neither template) is a dead
   branch in test terms while being part of the published refusal surface the CLI prints by name.
   Fix: one test that calls `UnrealIni.extract` on a hand-built `Detection` whose
   `UserCandidates.tree_root` spells neither resolver template (the `registry.rs:32` `/synthetic/tree`
   shape already exists) and asserts `matches!(refusal, ExtractionRefusal::UnreadableFile { .. })`
   and that the printed line names the variant.

3. **`minor` — four loader refusals that `schemas/bindings/v1/spec.md` states have neither fixture nor
   direct test.**
   Location: `crates/codecs/src/snapshot.rs:167` (`format` other than `"bindings"`), `:136` (body that
   is not a JSON document), `:299` (a `provenance.files` entry that is absolute), `:199`/`:352` (an
   empty `game` / empty token), `:226`/`:266` (an entry or `provenance` that is not an object);
   corpus: `schemas/bindings/v1/fixtures/invalid/` (16 fixtures).
   Problem: spec §Fixtures makes the two corpora "the conformance currency: … every invalid one is
   refused naming its key", and §Shape/§Privacy state the rules these arms enforce ("`format` | the
   string `"bindings"`", "`provenance.files` are tree-relative paths", "the contract document itself
   carries only tree-relative paths"). Each of the cases Test strategy #10 enumerates *is* covered
   (and the two over-bound cases are generated in `tests/snapshot.rs:111,136` because they are too
   large to check in), so nothing here is a required-clause hole — but a third-party implementer
   running this corpus cannot distinguish "absolute `files` is legal" from "it is refused", and a
   later edit that deletes one of these arms keeps the suite green.
   Fix: add the matching `fixtures/invalid/` documents (`files-absolute-files.json`,
   `format-not-bindings-format.json`, `game-empty-game.json`, `document-not-json.json`), named per the
   convention `tests/snapshot.rs:239` reads, and they are picked up by the existing
   `every_invalid_schema_fixture_is_refused_naming_its_key` loop with no new test body.

4. **`minor` — the `--probe` failure arms of the new face are untested, and two of its status arms are
   unreachable.**
   Location: `apps/discovery-cli/src/bindings.rs:237-242` (probe with no claims → `claim-refused`),
   `:299-303` (the `--probe --json` status document gains `"claims": []`), `:250-254`
   (`registry::family()` returning `None`), `:270-272` (`status == Ok && documents.is_empty()`).
   Problem: `cli_bindings_status_tests.rs` pins 0/3/4/5 through the non-probe route only, so the
   reachable probe-mode failure — `bindings <slug> --probe` on a game no family claims, and its
   `{"status":"claim-refused","claims":[]}` document — has no assertion; `--probe --json` is tested
   only for `ok` (`cli_bindings_tests.rs:203`). The other two arms cannot be reached through the
   shipped registry (a `Detection.family` always names a registered family, and a family whose
   extraction returned documents with no entries has already been ranked
   `family-confirmed-files-absent` at `bindings.rs:257`), so no test can exist for them as written.
   Fix: add one probe-mode status test per failure kind (table mode exit 3 + one `status:` line;
   `--probe --json` parsing to `status`/`claims`), and either drop the two unreachable arms or leave a
   one-line comment naming the invariant that makes them defensive-only, so checklist "dead branch"
   pressure does not accumulate against a later seat.

5. **`minor` — Test strategy #15's `schemas/README.md` clauses are the only pins in the run with no
   test behind them.**
   Location: `schemas/README.md` (the "Two formats are JSON" passage, the new seven-budget bullet with
   its aggregate product, and the `data/catalog/`, `schemas/catalog/`, `schemas/bindings/` hygiene
   sentence); no test in the diff reads that file — grepping `MAX_BINDING_FILE_OCTETS`,
   `MAX_SNAPSHOT_OCTETS`, `MAX_USER_DIRS_PER_GAME` and `bindforge-codecs` across `crates/core/tests`
   returns nothing.
   Problem: #15 names these statements as contract-tree pins, and the repo has precedent for pinning
   exactly this kind of text — `crates/core/tests/ledger_annotations.rs:129-151` asserts that the
   README bullet states `MAX_SLUG_OCTETS = 128` and every rider, "pinned against drift". Here, an edit
   that reverts the passage to one JSON format, or deletes the codecs budget family and its aggregate
   sentence, leaves `JSON_FIXTURE_FORMATS`, the conformance walk, the boundary sweep and every Rust
   suite green, because the harness const and the sweep encode the *skips*, not the *statement*.
   Fix: one test in the spirit of `ledger_annotations.rs` — read `schemas/README.md`, assert the
   two-format passage names `catalog` and `bindings`, assert the budget bullet states the seven
   `MAX_*` names with their values and the `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES`
   product, and assert the hygiene sentence names `schemas/bindings/`.

### Nits

6. **`nit` — the "and counted" half of Test strategy #3 and #6 has no observable, so no test asserts
   it.** Location: `crates/codecs/src/paths.rs:76-99` and `crates/codecs/src/families/unreal_ini.rs:203-231`;
   tests `crates/codecs/tests/paths.rs:118,135` and `crates/codecs/tests/unreal_ini.rs:225`.
   Both places the proposal says a skip is "skipped **and counted**" (the resolver links, an entry past
   `MAX_LOCATE_ENTRIES_PER_TREE`, a path match past `MAX_CANDIDATE_FILES`); the code skips, increments
   a local `examined` counter, and surfaces no count anywhere — neither in a refusal, nor in `--probe`
   output. The tests therefore pin the safety property that matters (the walk stops, the hidden
   candidate is never located, every run ends in a status) and nothing more. Fix: either surface the
   counts (a probe column or a budget-named line, which would make them testable) or hand the operator
   the wording correction in the Revision log so the ledger does not carry a fact the tree cannot show.

7. **`nit` — two dead helpers in the new harness.** Location: `crates/codecs/tests/common/mod.rs:101`
   (`TempRoot::files_below` — no target calls it; the byte-snapshot comparisons are done by
   `apps/discovery-cli/tests/cli_bindings_tests.rs:295 tree_bytes` instead) and
   `crates/codecs/tests/honesty.rs:36-43` (`honest_games`, marked `#[allow(dead_code)]`, whose list —
   `steam-userdata`, `proton-prefix`, `broken-tuple` — matches neither of the loops that actually run
   two lines below and would suggest `broken-tuple` is honesty-audited when it only ever refuses). The
   module-level `allow(dead_code)` is justified for genuinely per-target helpers; these two are dead in
   every target. Fix: delete both, or point `files_below` at the never-write check in the codec suites
   and use `honest_games()` from the loops that enumerate fixture games.

## FYI (out of specialty, no severity)

`§The first family` says `detect` walks "depth-first in byte order"; `crates/codecs/src/families/unreal_ini.rs:187-245`
is a breadth-first queue with byte-sorted names per level. Output order and the budget's "shallow noise
consumes the budget" reading are unaffected, which is why `the_locate_walk_stops_at_the_entry_budget` holds —
flagging only so the correctness seat can decide whether the prose or the code is the fact.
