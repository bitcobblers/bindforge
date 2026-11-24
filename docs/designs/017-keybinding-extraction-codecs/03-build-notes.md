# 03 — Build notes (ADR-017 keybinding extraction & codecs)

Stage: implement (full flow), fix round r1 (code-review findings), then fix
round r2 (gate reproduction).
Source: `02-proposal.md`. Prior attempt `implement-r0` timed out with the
src side complete but unverified; `implement-r1c0` verified it against
every behavior the committed suites encode, fixed clippy/fmt, and closed
the spec-amendment obligations. The `implement-r1` fix pass resolved the
open findings of code review round 1 (`compatibility-r1c0`, `security-r1c0`,
`performance-r1c0`). This attempt (`implement-r2`) reproduces the red gate at
2026-11-24T07:24:45.594Z and reports what it found; see "Round-2 fix pass".

## Round-2 fix pass (this round)

The runner's gate at 2026-11-24T07:24:45.594Z is red on the three cargo legs
and green on `npm run check` (verified again in this shell: 62 files / 698
tests passed). All four legs were re-run here; the evidence is in
"Gate posture (r2)".

What the red legs share: every failing byte lives in one file the test
engineer owns — `crates/codecs/tests/unreal_ini.rs`, as extended by the
test owner's commit `9f6c01a3` ("test: pin round-1 codec fixes in the
codecs suites"), which landed after the green gate at 06:32:09Z.

- `cargo fmt --all -- --check`: the only three diffs are in
  `crates/codecs/tests/unreal_ini.rs` (at its lines 291, 342 and 454 — the
  line-wrapping of three statements the rustfmt 2024 style re-wraps).
- `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace --locked`: both stop at the one error `E0382`
  (borrow of moved value `canonical`) inside that same test target;
  `crates/codecs/tests/common/mod.rs:153` is only the note that points at
  the harness's owning `library: PathBuf` parameter, not a second defect.
- Nothing else is red: `cargo clippy --workspace --lib --bins -- -D
  warnings` finishes clean, and the other seven `crates/codecs` targets
  (`context`, `determinism`, `honesty`, `paths`, `read`, `registry`,
  `snapshot`, `zz_api_probe` — including the test owner's two new round-1
  pin groups in `registry.rs` and `snapshot.rs`) plus every
  `bindforge-core`, `discovery`, `transforms` and `discovery-cli` target
  run green.

Because the src is not what the gate blames, no implementation change was
made — and none would move these legs: rustfmt's diff and an `E0382` in a
test body are both inside a test file, which this stage must not edit. To be
certain the new pins do not hide a src gap, the three behaviors they pin
(the `#[cfg(unix)]` byte-named locate pin, the control-byte echo pin, and
the open-cap pin) were replayed against the shipped `crates/codecs` src by a
throwaway harness (`target/scratch-r2`, detached from the workspace by its
own `[workspace]` table, gitignored — deviation 1 again): the byte-named
file and the file under the byte-named directory both locate at their real
paths and the two documents fold to `MoveForward/W` + `TurnRight/MouseX`;
the `UnparsableEntry` echo renders
`Saved/Config/Weird\u001bDir/Engine.ini:2: Name= must carry a quoted value`
on one control-free line that still names the file; the stating file past
`MAX_CANDIDATE_FILES` quiet grammar matches claims nothing while the same
file one slot inside the cap locates. `0 failures`. The src already satisfies
every pin the red target carries, so the round's only honest output is the
test-update hand-off below.

## Round-1 review fixes (round r1)

Compatibility seat:

- **A (major, locate walk invented candidates from lossy names) — fixed.**
  `unreal_ini::locate` now walks `DirEntry::path()` (the bytes the OS
  named) for the queue and the candidate push, and keeps the lossy
  `String` as the byte-sort key only (the reviewer's first fix option;
  `OsStrExt` byte keys were avoided as platform-only). The `Err` arm is
  split: `FileOverBudget`/`UndecodableText` locate on path grammar alone
  (as §The first family states), `UnreadableFile` is skip-and-count — a
  path the OS will not stat never becomes a candidate.
- **B (minor, host-native separators in `provenance.files`) — fixed.**
  `tree_relative` builds the contract spelling once, `/`-joined component
  by component; the document, the byte-order sort and the refusal echoes
  all carry it. Absolute probe paths stay host-native (§Local paths).
- **C (minor, broken-prefix comment promised a status no route produces) —
  fixed.** The `paths.rs` comment now states the shipped answer: a broken
  or symlinked prefix locates nothing, the probe claims nothing, and the
  face answers `claim-refused` (exit 3); `family-confirmed-files-absent`
  (exit 4) belongs to appid-table claims (`017-R4`'s data would be the
  first to create one). The proposal's Compatibility-matrix row wording
  ("named degradation" = exit 4) is therefore *not* what ships; the
  shipped status is exit 3 — see the ledger hand-off below.
- **D (nit, unflagged case-exact grammar) — fixed.** `grammar_match`'s doc
  comment and `schemas/bindings/v1/spec.md` (Shape) now name the match as
  byte-exact (case-exact) by design and state that a case-folded tree on
  a case-insensitive filesystem is outside the grammar.
- **E (nit, plan-vs-implementation fixture drift) — recorded.** No
  committed `symlinked-prefix` fixture tree ships: the link coverage is
  made at test time in temp roots (`tests/paths.rs`, which carries the
  `#[cfg(not(unix))]` arm, and `tests/unreal_ini.rs`, which does not —
  listed under test-update items).

Security seat:

- **1 (major, loader accepted escaping `provenance.files` spellings) —
  fixed.** The loader now refuses any `files` entry that is not a plain
  `/`-separated sequence of `Normal` components: `Prefix`/`RootDir`/
  `CurDir`/`ParentDir` components and raw spellings carrying `\` or `:`
  are refused naming `files`. Stated in the spec's Bounds bullet; pinned
  through the corpus by the new
  `schemas/bindings/v1/fixtures/invalid/files-traversal-files.json`
  (`every_invalid_schema_fixture_is_refused_naming_its_key` covers it).
  Windows-drive, UNC, backslash-dotdot and `./` spellings additionally
  verified through a scratch harness (see Gate posture).
- **2 (minor, unescaped path echoes on the `error: ` line) — fixed at the
  source:** `ExtractionRefusal`'s `Display` renders every `path` field
  through `bindforge_core::escape_and_truncate`, so every sink (the CLI's
  `error: ` line included) gets a single-line, control-free, truncating
  message. Clean paths render unchanged, so no pinned echo changes.

Performance seat:

- **1 (major, open cap gated on the located set) — fixed.** `locate`
  counts `opened` independently of locating and refuses to open past
  `MAX_CANDIDATE_FILES` per tree; the README/spec "the product caps the
  opens too" now holds of the code.
- **2 (minor, located files read twice) — fixed by the reviewer's option
  (b), honesty in the docs:** `schemas/README.md` and
  `schemas/bindings/v1/spec.md` now state
  `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES ×
  MAX_BINDING_FILE_OCTETS` (1,088 MiB) per game, `--probe` paying one
  pass and the extracting face two. (Carrying decoded text on
  `UserCandidates` would change the proposal's type shape; not done.)
- **3 (minor, removal lines scanned the whole working set) — fixed.**
  `Fold::remove` builds the key once and returns when `seen` (maintained
  in lockstep, so membership is exact) does not hold it; only a removal
  of a held pair pays the `retain`.
- **4 (minor, listing collected before the budget applied) — fixed in the
  test-safe half plus the reviewer's alternative:** the walk tests the
  entry budget at the top of the queue loop (an over-budget walk enters
  no further directory), and `lib.rs` documents beside
  `MAX_LOCATE_ENTRIES_PER_TREE` that the constant bounds *examined
  entries*, not the transient per-directory listing (the sorted listing is
  what keeps the examined set deterministic; a pre-sort `take` would make
  it readdir-order-dependent).
- **5 (nit, pass 2 re-ran a declined pass-1 family's `detect`) — fixed.**
  `claim_with` remembers the ids whose `detect` ran in pass 1 and skips
  them in pass 2 (detect is pure over `ctx`, so the skip is equivalent).
- **6 (nit, k records walked identical trees) — fixed.** The bindings
  face memoizes the claim per `(appid, library, roots)` across the record
  loop; §Determinism's tree-identity dedup still merges the groups.
- **7 (nit, decode copied the whole buffer) — fixed.** Added
  `read::decode_owned(Vec<u8>)` (strict-UTF-8 path moves the buffer);
  both read paths use it; the borrowed `decode(&[u8])` seam stays for the
  suites and delegates.

Out-of-severity FYI lines (compat FYI 1–3): the aggregate double-read (3)
is fixed with finding perf-2. The `game_key` re-derivation from tree
components (1) and the `--probe --json` empty-`claims` on a non-ok status
(2) are untouched — they carry no severity and each implies a design
question (carry the lookup key on `UserCandidates`; widen the status
document) outside this fix round.

## Progress (r1c0, unchanged)

- **Completed:** `crates/codecs` (lib, bin-facing API, all modules and the
  `unreal-ini` family), discovery `ScanRecord`/`steam_common` scan refactor
  with per-root library ownership and dedupe, plugin `scan_records()`/
  `roots()` wrappers, `schemas/bindings/v1` contract tree + fixtures,
  `apps/discovery-cli` `bindings` subcommand (statuses/exit codes 0/1/3/4/5,
  `--probe`, `--json`), `schemas/manifests/v1/spec.md` and `schemas/README.md`
  amendments, `Cargo.lock` (exactly the designed delta). The test-file
  compile items listed below were since closed by the test owner; the gate
  has been green since.

## Test-update items (owner: test-update)

Blocking the gate now (round r2) — all three are in
`crates/codecs/tests/unreal_ini.rs`, and each is the file's own defect, so a
src change cannot discharge them:

1. **`E0382` at `crates/codecs/tests/unreal_ini.rs:273`** (in
   `byte_named_directories_and_files_locate_at_their_real_paths`):
   `context(appid, vec![canonical.clone()], canonical)` moves `canonical`,
   which the `expect` closure at line 281 borrows afterwards. The compiler's
   own fix applies: pass `canonical.clone()` as the third argument (the
   sibling tests at lines 201 and 380 already clone; the `single_store`
   helper does not need it because nothing reads `canonical` afterwards).
2. **Three rustfmt diffs** in the same file — lines 291
   (`let docs = UnrealIni.extract(&claim).expect(…)`), 342
   (`assert_eq!(message.lines().count(), 1, …)`) and 454
   (`root.write(&format!("{tree}/f{:02}.ini", MAX_CANDIDATE_FILES - 1),
   stating)`): each exceeds the 100-column 2024-style limit and must be
   re-wrapped; `cargo fmt --all` over the file is the whole fix.
3. The same file's formatting is what keeps `cargo fmt --all -- --check`
   red, so items 1 and 2 must land together for all three cargo legs to go
   green at once.

Carried from r1c0 — all resolved (the gate's `cargo test --workspace
--locked` leg runs every suite green):

- `crates/codecs/tests/*`, `crates/discovery/tests/scan_record_tests.rs`,
  `apps/discovery-cli/tests/cli_bindings_tests.rs` compile errors and the
  two harness generalizations (`JSON_FIXTURE_FORMATS`,
  `SOURCE_KEY_TREES`) — done.

New pins this pass would lock in (behaviors verified by a scratch harness
only, and by the new invalid fixture for the traversal case):

- **Open-cap independent of locating (perf-1):** N grammar-matching `.ini`
  files that do *not* state the section plus one that does, sorted past
  `MAX_CANDIDATE_FILES`, asserting the stating file is not located (the
  existing `a_tree_locates_at_most_the_candidate_cap` fixture makes the
  opened and located counters indistinguishable).
- **Traversal refusals beyond the shipped fixture:** the `C:/…`, UNC,
  `..\..\` and `./` spellings (only `Saved/Config/../../../etc/crontab`
  is a committed fixture now).
- **Non-UTF-8 file and directory names (compat-A):** a `weird\xffname.ini`
  and a `Saved/Config/<d\xffr>/Engine.ini` locate at their real paths and
  extract (needs `std::os::unix` byte-named fixtures behind `#[cfg(unix)]`
  with a `#[cfg(not(unix))]` arm).
- **Escaped refusal echoes (security-2):** a located directory name
  carrying a control byte; assert the `error: ` line is single-line and
  control-free.
- **Symmetry nit (compat-E):** the link test in `tests/unreal_ini.rs` has
  no `#[cfg(not(unix))]` arm; `tests/paths.rs` does.
- **`Fold::remove` no-op path and registry pass-2 skip:** a family whose
  `appids()` contain the appid and whose `detect` counts calls, asserting
  one `detect` per `claim`.

## Deviations from the proposal

1. **Scratch verification instead of new tests.** Test ownership forbids
   authoring test files, so loader/family/CLI behavior — including this
   pass's traversal, non-UTF-8-name, open-cap, removal and escape checks —
   was verified by throwaway harnesses under `target/` (excluded from the
   workspace via their own `[workspace]` tables; not part of the delivery).
2. **Clippy-driven cosmetic edits** in `crates/codecs/src/families/
   unreal_ini.rs`, `read.rs`, `registry.rs` (map-vs-filter_map, range
   `contains`, `copied()`, lifetime elision, redundant closure). Semantics
   unchanged.
3. **`npm ci` refused by the stage sandbox** in r1c0; `npm run check` ran
   against the pre-existing `node_modules` and passed. No
   `tools/orchestration` file is touched by this run.
4. **`017-S1` operator verification** (real Steam install smoke) is a
   manual step the stage cannot execute; scratch CLI runs used synthetic
   Steam roots under `target/` and exercised the same face.
5. **Aggregate read budget restated (perf-2 option b).** §Read discipline
   states the worst case as `(MAX_USER_DIRS_PER_GAME + 1) ×
   MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` (544 MiB) counting one
   read pass; the shipped code reads twice (detect's content test, then
   extract's fold), and the review found the docs must describe the run
   the code performs. `schemas/README.md` and the spec now state `2 ×`
   that product (1,088 MiB); the proposal's sentence counts one pass.
6. **Broken-prefix degradation status (compat-C).** The proposal's
   Compatibility-matrix row promises `family-confirmed-files-absent`
   (exit 4) for the NTFS/exFAT symlinked-prefix degradation; with
   `appids()` empty, no shipped route produces exit 4 for it — the face
   answers `claim-refused` (exit 3), and the code comment now says so.
   Ledger hand-off (Rollout 8): the matrix row's named status is exit 3,
   not exit 4, until `017-R4` supplies appid-table claims.
7. **`MAX_LOCATE_ENTRIES_PER_TREE` documented as bounding examined
   entries, not the transient listing** (the reviewer's explicitly
   offered alternative), plus the budget test hoisted above `read_dir`.
8. **Round r2 lands no product-code change.** The gate's three red legs
   blame `crates/codecs/tests/unreal_ini.rs` alone (one `E0382` and three
   rustfmt re-wraps), a file this stage may not edit; the shipped src was
   shown to satisfy every pin that file carries (see "Round-2 fix pass"),
   so the honest output is the test-update hand-off rather than an
   invented src edit. No `Cargo.lock`, spec or public-surface delta.
9. **Named/public surface grew beyond the proposal's enumerations.** Beyond
   deviations 2 and the Ledger's `decode_owned`, the shipped surface carries
   items the proposal's prose lists did not name: `ExtractionRefusal` ships
   six variants (the §Read discipline four plus `CandidateFilesBudget` and
   `UnreadableFile`, both from the locate/extract seams); `CodecFamily`
   carries a `Sync` supertrait the static `REGISTRY` requires; `registry`
   also exports `claim_with` (the two-pass seam the synthetic family pins
   pass ordering through) and `family(id)` (the face's handle to run the
   claiming family's `extract`); and `serde_json` ships as a load dependency
   rather than the §crates/codecs dev-dependency, on the `catalog` posture.
   None reverses a decision, and the `Cargo.lock` delta stays exactly the
   codecs stanza + the `discovery-cli` edge (no version moves). These are
   recorded as the dated "As-built" note in `02-proposal.md`; that note also
   consolidates the behavioral drifts already listed as deviations 5–7 and
   the security-1 traversal refusal on `provenance.files`.

## Deferred risks (recorded, per proposal)

- `017-R1`–`017-R4` remain open as the proposal states. The
  `schemas/manifests/v1/spec.md` amendment discharges `013-R2` (fingerprint
  byte recipe documented) and notes the codec-registry load check rides on
  `017-R4` (manifests naming unimplemented families stay loadable).
- The ADR-012 vocabulary gap stays open by design (Options); the snapshot
  carries native tokens verbatim.
- `017-S1` operator verification against two locally installed titles
  remains a manual step (A23).

## Ledger

- Unit list as proposed: `codec`, `context`, `registry`, `paths`, `read`,
  `snapshot`, `families/unreal_ini` in `crates/codecs`; `bindings` in
  `apps/discovery-cli`; `ScanRecord` plumbing in `crates/discovery`.
- Public surface delta this pass: one added function,
  `bindforge_codecs::read::decode_owned` (also re-exported at the crate
  root). No type, budget constant, refusal variant or existing signature
  changed; no `Cargo.lock` delta.

## Gate posture (gate at 2026-11-24T06:32:09.463Z, green)

The runner's gate at 2026-11-24T06:32:09.463Z was green on all four legs
(`npm run check`, `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace --locked`, each
exit 0); its single rendered line
`test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;`
is the doc-test/lib leg of that suite, as the compatibility seat's re-run
confirmed. After this pass's changes every leg was re-run in this shell:

- `cargo fmt --all -- --check` → exit 0, no output.
- `cargo clippy --workspace --all-targets -- -D warnings` →
  `Finished `dev profile [unoptimized + debuginfo] target(s)` — clean.
- `cargo test --workspace --locked` → exit 0; 86 `test result: ok.`
  lines, no failing leg; e.g. `crates/codecs/tests/unreal_ini.rs`:
  `test result: ok. 23 passed; 0 failed` (includes
  `a_tree_locates_at_most_the_candidate_cap` and
  `the_locate_walk_stops_at_the_entry_budget` under the new open-cap
  code), `tests/snapshot.rs`: `test result: ok. 8 passed` over the grown
  invalid corpus (the new `files-traversal-files.json` refused naming
  `files`), `discovery-cli` suites incl.
  `one_slug_in_two_libraries_prints_two_probe_groups`: ok.
- `npm run check` (tools/orchestration) → `Test Files 62 passed (62)`,
  `Tests 698 passed | 2 skipped (700)`.
- Scratch harness `cargo run --manifest-path target/scratch-fix/…` →
  `0 failures`: loader refuses `Saved/Config/../../../etc/crontab`,
  `C:/Windows/win.ini`, `\\server\share\x`, `..\..\x`, `./x.ini`,
  `../../etc/passwd` each naming `files` while a plain relative entry
  still loads; `weird\xffname.ini` and `Saved/Config/<d\xffr>/Engine.ini`
  locate at their real byte paths and both fold (no invented candidate,
  no silently skipped subtree); a stating file sorted past 32
  non-stating grammar matches is not located (open cap on opens); 2,000
  removal lines naming absent pairs are no-ops; an ESC byte in an echoed
  path renders as `\u001b` on one control-free line.
- Windows `x86_64-pc-windows-msvc` compile leg: not runnable here (target
  not installed, as the compatibility seat recorded); the only platform
  gates touched are the existing `cfg(windows)` arms in `bindings.rs`.

## Gate posture (gate at 2026-11-24T07:24:45.594Z, red — round r2)

Reproduced leg by leg in this shell, on the tree at `9f6c01a3`. The failing
lines the runner captured are its renderer's; the lines quoted below are the
same ones this shell re-captured, cited line-selectively.

- `cargo fmt --all -- --check` → **exit 1**. The whole failure is three
  hunks in `crates/codecs/tests/unreal_ini.rs` (at its 291, 342 and 454):

  ```text
  -    let docs = UnrealIni.extract(&claim).expect("both byte-named candidates extract");
  +    let docs = UnrealIni
  +        .extract(&claim)
  +        .expect("both byte-named candidates extract");
  ```

  ```text
  -    assert_eq!(message.lines().count(), 1, "the echo must be one line: {message:?}");
  +    assert_eq!(
  +        message.lines().count(),
  +        1,
  +        "the echo must be one line: {message:?}"
  +    );
  ```

  ```text
  -    root.write(&format!("{tree}/f{:02}.ini", MAX_CANDIDATE_FILES - 1), stating);
  +    root.write(
  +        &format!("{tree}/f{:02}.ini", MAX_CANDIDATE_FILES - 1),
  +        stating,
  +    );
  ```

- `cargo clippy --workspace --all-targets -- -D warnings` → **exit 101**.
  One error, in the same test target:

  ```text
  error[E0382]: borrow of moved value: `canonical`
     --> crates/codecs/tests/unreal_ini.rs:281:18
  272 |     let canonical = root.canonical();
      |         --------- move occurs because `canonical` has type `std::path::PathBuf`, which does not implement the `Copy` trait
  273 |     let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical));
      |                                                                       --------- value moved here
  281 |     let expect = |relative: &std::path::Path| -> PathBuf {
      |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ value borrowed here after move
  283 |         canonical.join(stripped)
      |         --------- borrow occurs due to use in closure
  help: consider cloning the value if the performance cost is acceptable
  273 |     let claim = unreal_claim(&context(appid, vec![canonical.clone()], canonical.clone()));
      |                                                                                ++++++++
  error: could not compile `bindforge-codecs` (test "unreal_ini") due to 1 previous error
  ```

  (The note's second site, `crates/codecs/tests/common/mod.rs:153`, is the
  harness's `library: PathBuf` parameter — a note about the callee's
  ownership, not a second defect, and a file the same ownership rule puts
  with the test engineer.)

- `cargo test --workspace --locked` → **exit 101**. Identical diagnosis: the
  same `error[E0382]: borrow of moved value: canonical` at
  `crates/codecs/tests/unreal_ini.rs:281:18` ending in

  ```text
  error: could not compile `bindforge-codecs` (test "unreal_ini") due to 1 previous error
  ```

- `npm run check` (tools/orchestration) → **exit 0, green**:
  `All matched files use Prettier code style!` and
  `Test Files 62 passed (62)` / `Tests 698 passed | 2 skipped (700)`.

Green legs this round isolated the blame to the one test target:

- `cargo clippy --workspace --lib --bins -- -D warnings` →
  `Finished `dev` profile [unoptimized + debuginfo] target(s)` — the shipped
  src is clippy-clean.
- `cargo test --workspace --locked --exclude bindforge-codecs` → every
  `bindforge-core`, `discovery`, `transforms` and `discovery-cli` target
  reports `test result: ok.`; no failures.
- `cargo test -p bindforge-codecs --locked --test context --test determinism
  --test honesty --test paths --test read --test registry --test snapshot
  --test zz_api_probe` → all eight targets `test result: ok.` (incl. the
  round-1 pins the same test commit added to `registry.rs` and
  `snapshot.rs`); only the `unreal_ini` target does not compile.
- Scratch harness `cargo run --manifest-path target/scratch-r2/Cargo.toml`
  → `0 failures`, replaying the three `unreal_ini` pins the broken target
  carries (see "Round-2 fix pass").
