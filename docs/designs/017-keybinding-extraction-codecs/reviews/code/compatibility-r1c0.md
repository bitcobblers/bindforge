# Code review — compatibility seat, round 1 pass c0

**Change under review:** `git diff fc28ab94f313d079222d61438bf5b9074a1b8039...HEAD -- ':(exclude)docs/designs'`
(64 files, +5986/−80: `crates/codecs` (new), `crates/discovery` `ScanRecord`/`scan()` seam,
`apps/discovery-cli` `bindings` face + renderer generalization, `schemas/bindings/v1/`,
`schemas/README.md`, `schemas/manifests/v1/spec.md`, harness carve-outs).

**Inputs used:** that diff, `02-proposal.md`, `docs/adrs/017-keybinding-extraction-codecs.md`,
the runner's gate record, my own previous findings (none — first pass). No other review
report or transcript was opened.

**Specialty applied:** `references/compatibility.md` only. Out-of-specialty observations are
the FYI lines at the end and carry no severity.

## Evidence

- Gate at 2026-11-24T06:32:09.463Z, green: `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`
  all exit 0.
- Re-ran one leg myself: `cargo test -p bindforge-codecs --locked --offline` → 64 tests over
  10 targets, every leg `test result: ok.`; `--test paths` → 6 passed including
  `symlinked_userdirs_are_skipped_and_never_anchored`, so the one platform-scoped assertion
  in Test strategy #17 really runs on the Linux leg rather than being `cfg`-ed out
  (checklist item 7). The gate record's single rendered `test result: ok. 0 passed` line is
  the doc-test/lib leg, not the whole suite.
- **Missing evidence, recorded:** the `x86_64-pc-windows-msvc` compile leg (`.github/workflows/check.yml:42-46`,
  and proposal Rollout 9's "the Windows compile-check leg must stay green with `crates/codecs`
  added") is **not** among the legs the runner captured, and is not re-runnable in this
  checkout (`cargo check -p bindforge-codecs --all-targets --target x86_64-pc-windows-msvc`
  → `can't find crate for core` — target not installed here). Static read: nothing in
  `crates/codecs/src/**` touches a platform-only API; the only `std::os::unix` uses in the
  change are the two `#[cfg(unix)]` test bodies, and the `#[cfg(windows)]`/`#[cfg(not(windows))]`
  plugin-selection arms in `bindings.rs:169-178` mirror `main.rs:152-165`. I therefore have a
  concrete read that the leg is likely green, and no run that shows it — not blocking, since
  the property (the crate builds for the Windows target) is not one the *change* contradicts.

## Checklist walk

1. **Platform code matches the Compatibility matrix.** Partly. The CLI face's plugin
   construction, `--steam-root` override and its `cfg(windows)` selection are a faithful copy
   of `list`'s (`bindings.rs:169-178` vs `main.rs:152-165`), and no `crates/codecs` source is
   platform-specific — so Flatpak/Snap, Proton and Linux rows have real code paths. Two rows
   are not matched by the code: the NTFS/exFAT row's named degradation (finding C) and the
   Windows row's portability premise, where the root-relative templates are unguarded and
   the document they produce is platform-divergent (finding B).
2. **Node floor.** Not applicable — the change touches no `tools/orchestration` file; the
   `npm run check` leg is green and unshifted.
3. **Paths.** Built with `Path::join` everywhere (`paths.rs:118-126`, `user_trees`'
   `userdata/<dir>/<appid>`, the CLI's argv paths); no string concatenation of path text.
   But two path-handling assumptions are unmet/hidden: names re-materialized from a lossy
   `String` (finding A) and the case-exact grammar test (finding D).
4. **Guarded platform imports.** Satisfied: the only platform-only import in the change
   (`std::os::unix::fs::symlink`, twice) is inside `#[cfg(unix)]` test bodies; nothing
   platform-only is reachable from a library load path.
5. **Locale/layout reality.** Non-ASCII *binding* names survive: the token bounds are octet
   counts and the only rejects are control characters and edge spaces
   (`unreal_ini.rs:461-476`, `snapshot.rs:351-372`), and the `ÅÄÖ` UTF-16 case is pinned
   (`tests/read.rs:66-75`). Non-UTF-8 *file* names — legal on ext4, ordinary on the
   exFAT/NTFS libraries the matrix calls supported — do not: finding A.
6. **Native dependencies.** None introduced. The `Cargo.lock` delta is the
   `bindforge-codecs` stanza plus the `discovery-cli` edge; `serde_json` was already resolved
   (pinned by `tests/zz_api_probe.rs:116-136`), so the advisory gate and the Windows leg gain
   no new build.
7. **Platform-scoped tests.** Satisfied for what ships: 64 codecs tests green on Linux, the
   symlink assertions `#[cfg(unix)]` with a `#[cfg(not(unix))]` early-return arm in
   `tests/paths.rs`; nothing is skipped everywhere. Minor plan divergence: finding E.

## Findings

### A. major — `crates/codecs/src/families/unreal_ini.rs:194-243` — the locate walk re-materializes paths from a lossy string, so non-UTF-8 names become invented candidates

`locate` keys its byte-ordered walk on `entry.file_name().to_string_lossy()` (line 197) and
then rebuilds the child path as `dir.join(&name)` (line 217) instead of using `entry.path()`.
For an entry whose name is not valid UTF-8 — legal on every Linux filesystem the project
supports, and the normal state of a library carried on exFAT/NTFS, which the Compatibility
matrix has a *supported* row for — the joined path names something that never existed:

- A grammar-shaped file (line 226 passes on the lossy name) then falls into the `Err(_)` arm
  at line 240-242, which is written for "a match the reader refuses is located on path
  grammar alone". `read::open_capped` returns `UnreadableFile` for a *stat* failure, not a
  content refusal, so a path the walk invented is pushed as a located candidate; `extract`
  then re-opens it (`read_and_decode`, line 516), fails, and refuses the whole game —
  `extraction-refused`, exit 5 — with no entry lost-and-found and no route around it. That
  also puts a never-existent relative path into a candidate set, against §The first family's
  "extract reads only located files" reading of the file layout.
- A non-UTF-8 *directory* name is queued at line 222 as a path that does not exist, its
  `read_dir` fails at line 190, and the whole subtree is silently skipped — the entries under
  it are simply absent, with no budget-skip counter to account for them.

Neither outcome is reachable through any fixture, because every fixture name is ASCII.

**Fix:** keep `entry.path()` for both the queue and `candidates.push`, and keep the lossy
`String` only as the sort key (or sort on `entry.file_name()`'s `OsStr` bytes, which is the
walk's actual byte order). Then split the `Err` arm: locate-on-path-grammar for
`FileOverBudget`/`UndecodableText` (the refusals §Read discipline contemplates) and
skip-and-count for `UnreadableFile`, so an entry the OS will not stat never becomes a
candidate.

### B. minor — `crates/codecs/src/families/unreal_ini.rs:506-511` — `provenance.files` carries platform-native separators, so the same layout does not state the same document

`tree_relative` is `strip_prefix(..).to_string_lossy()`, i.e. whatever separator the host
`join` used. On the `x86_64-pc-windows-msvc` host the face runs (matrix row "Windows host":
"the root-relative trees resolve and are stat'd" — and `<root>/userdata/<steamid>/<appid>` is
exactly the layout a Windows Steam install has, with cloud-synced UE configs under
`remote/<Project>/Saved/Config/…`), the contract document prints
`Saved\Config\WindowsNoEditor\Engine.ini`, while the spec, the fixtures
(`schemas/bindings/v1/fixtures/valid/*.json`) and the byte-order sort are `/`-based. That
breaks the rule §`bindings/v1` states for this very field ("the portability rule: the same
layout states the same document on any machine") and re-orders `provenance.files` (`\` = 0x5C
sorts apart from `/` = 0x2F). The read side already knows: `snapshot.rs:305-308` counts depth
as `.max(file.matches('/').count() + 1)` — an explicit `/`-only assumption with no matching
normalization on the producing side, so a `\`-spelled document's depth is under-counted on
Linux.

**Fix:** build the contract spelling once —
`components().map(|c| c.as_os_str().to_string_lossy()).join("/")` — and use it for the
document, the sort and refusal messages; leave absolute probe paths native (§The CLI face:
Local paths already scopes those to the operator's terminal). A `provenance.files`
`/`-spelling pin in `tests/honesty.rs` or the loader round-trip keeps it from rotting.

### C. minor — `crates/codecs/src/paths.rs:127-136` — the matrix's named degradation for a symlinked/broken prefix is not the status the shipped route reaches

The comment at lines 133-135 states "A17's broken-prefix case degrades to files-absent", and
the Compatibility matrix's NTFS/exFAT row promises it as
`family-confirmed-files-absent` (exit 4). Shipped behavior: `proton_trees` skips a template
root that is a link, so the tree is absent; `unreal_ini::detect` returns `None` when nothing
is located (`unreal_ini.rs:73-90`); and `appids()` is empty (`unreal_ini.rs:67-71`), so no
`Evidence::AppidTable` claim exists to produce status 4 at all — a game with a broken prefix
and no `userdata` tree answers `claim-refused` (exit 3). The safety half of the row ("never a
hang or an out-of-tree read") does hold, and `tests/paths.rs` pins the anchoring rule; only
the status name/code differs from what the matrix and the comment promise.

**Fix:** either state the degradation correctly (matrix row → `claim-refused`, exit 3, with
§The registry's reason: a probe that locates nothing does not claim; add it to the §Rollout 8
ledger hand-off) or give the row a route (an appid-table claim, which `017-R4`'s data would
create). Do not leave a code comment promising a status no shipped path produces.

### D. nit — `crates/codecs/src/families/unreal_ini.rs:251-258` — the case-exact grammar test is an unflagged filesystem assumption

`grammar_match` compares `Saved`, `Config` and the `.ini` suffix byte-exactly. That is right
for UE's own spelling on Linux, but on the case-insensitive filesystems of a row the matrix
keeps live, a `saved/config` or `X.INI` tree is silently out of the grammar, and nothing in
`grammar_match`'s comment, `schemas/bindings/v1/spec.md` or `02-proposal.md` §The first
family says the grammar is case-exact. Checklist item 3 asks for exactly this flag.

**Fix:** one sentence in `grammar_match`'s doc comment (and, if the format should survive a
move to case-insensitive homes, one in `spec.md`'s Bounds/Shape) naming the match as
case-exact by design — or match the extension case-insensitively and say so.

### E. nit — plan-vs-implementation drift in the platform-scoped fixtures

§The first family names a `symlinked-prefix` fixture tree whose links are "made at test time
behind `#[cfg(unix)]` with a `#[cfg(not(unix))]` early return"; no such tree ships (the
fixture set is `steam-userdata`, `proton-prefix`, `bom-utf16`, `defaults-only`,
`bindings-absent`, `broken-tuple`) — the coverage moved to temp-dir links in
`tests/paths.rs:139-184` (which has the `cfg(not(unix))` arm) and
`tests/unreal_ini.rs:182-223` (which has none, so the Windows compile leg sees no second
link arm). Coverage is equivalent on Linux and the compile leg is unaffected, so this is
recorded as drift from the plan's fixture list, not a gap.

**Fix:** name the shipped shape in `03-build-notes.md` (links made at test time in temp
roots, no committed `symlinked-prefix` tree), and add the `#[cfg(not(unix))]` arm to the
`unreal_ini` link test for symmetry with the `paths` one.

## FYI (out of specialty, no severity)

- `unreal_ini.rs:146-170`: the contract's `game` value is re-derived from the tree root's
  components (`position("userdata")` / `position("compatdata")`) rather than carried from
  `GameContext::lookup_key`, so a root whose own path contains an earlier `userdata` or
  `compatdata` component mis-derives the appid or refuses the extraction outright.
- `apps/discovery-cli/src/bindings.rs:296-310`: whenever the status is non-ok, `--probe --json`
  emits `"claims": []` and discards the claims the run actually made, which is the one thing
  probe mode exists to show.
- `schemas/README.md`'s new aggregate bullet counts one open per located file; locate opens
  every path-grammar match for the content test and `extract` re-opens the located ones, so
  the read volume per game is up to 2× the stated 544 MiB.

## Verdict

`request-changes` — one open major finding (A). B–E are minor/nit and do not gate the branch
on their own.
