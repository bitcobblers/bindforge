# correctness review r1

verdict: request-changes

Scope: whole document (`docs/designs/tasks/001-steam-linux-discovery/02-proposal.md`).
Inputs used: the proposal, `docs/tasks/T-001-steam-linux-discovery.md`, the Revision
log, and read-only inspection of the code the proposal cites
(`crates/discovery/src/{engine,plugin,game,slug,lib,registry}.rs`,
`apps/discovery-cli/{src/main.rs,Cargo.toml}`, `Cargo.lock`, `rust-toolchain.toml`,
`.github/workflows/check.yml`) plus webfetch of the assumption-register URLs
(A1, A2, A3, A4, A5, A8, A9-versioned, A12). `01-research.md` was not opened (not in
this pass's allowed inputs); every claim it carries was checked against the
proposal's own primary sources instead.

## Findings

- [blocker][new] *Chosen design §3 Library selection* — "- `<root>/steamapps` itself, always." —
  the entry `libraries_in_root` returns for the root-as-library inclusion is off by one
  directory level relative to the "library" concept every other section uses. §3's own
  closing line ("`appmanifest_*.acf` files are read only as entries directly inside a
  selected library's `steamapps/`"), §5's composition
  (`install_path = l.join("steamapps").join("common").join(m.install_dir)`), A2, and the
  Goals bullet ("a guaranteed scan of each root's own `steamapps/`") all require a library
  entry to be a directory that *contains* `steamapps/`; §2's canonical path rule drops any
  entry that lacks one. As written, the returned entry `<root>/steamapps` needs
  `<root>/steamapps/steamapps`, so it is either dropped by the gate or scanned at the wrong
  place, the root's own `appmanifest_*.acf` files are never read, and the design's central
  defence against the silent-zero bug (A12) is inert. Verified against A2/A3's cited source:
  `steamlocate-rs/src/library.rs` `Library::from_dir` reads `path.join("steamapps")` and
  `resolve_app_dir` = `path/steamapps/common/<installdir>`, i.e. the VDF `path` is the
  library root. Concrete failure: on a real single-library install (`~/.local/share/Steam`,
  no secondary libraries) `discovery list` emits zero rows, contradicting T-001's Done-when
  "Running `discovery list` on a Linux machine with Steam correctly identifies installed
  games"; and Test strategy #8 ("returns exactly four games") and #3 ("the primary library
  ... [is] returned") are unsatisfiable as specified. Fix: make the inclusion "`<root>`
  itself, always" and state, in §3, that the entries `libraries_in_root` returns are library
  roots that each contain a `steamapps/` directory — the same sentence then makes §2's
  `steamapps` gate and §5's join agree.

- [major][new] *§9 Fixtures* — "# modern shape: \"0\" → \".\", \"1\" → \"../secondary\"" —
  the fixture's `"0"` entry contradicts §3's join rule ("otherwise joined to the directory
  holding the VDF"), which is `<root>/steamapps`, so `"."` resolves to `<root>/steamapps`
  and not to `<root>`. Only `"1" → "../secondary"` is written against that base (it yields
  `<root>/secondary`, which the tree does have). Consequently the `"0"` entry contributes
  nothing that survives the `steamapps` gate, and §9's claim that "the `\"0\" → \".\"` entry
  exercises root/library de-duping" plus Test strategy #3's "adds no duplicate" assertion
  test nothing. Fix: with the join base left as the VDF's directory, write the fixture's
  modern-shape entries as `"0" → ".."` and `"1" → "../secondary"`; `".."` then canonicalizes
  to the same directory as §3's root inclusion, which is exactly the de-duping the section
  claims to exercise, and the primary library's four-manifest `steamapps/` becomes reachable.

- [major][new] *§7 Failure policy* vs *Test strategy #7* — "The single `Err` case: at least
  one root was selected but **no** library under any selected root could be scanned." —
  the stated predicate is about *libraries* being unscannable, and §7's skip list already
  puts "a manifest that fails to parse" on the continue side. Test strategy #7 pins the
  opposite: "a root that has a `steamapps` directory whose entries are all unparseable
  returns exactly one `Err(DiscoveryError::PluginError(_))`". In that scenario the library
  *was* scanned successfully and yielded zero games, so §7's predicate returns
  `Ok(vec![])` and the design's own test fails; §7's error text ("the count of files skipped
  as unreadable or malformed") points the other way again. The alternative reading — "a
  selected root emitted no games" — would make a genuinely empty install an error, which
  §7's "nothing installed" sentence forbids. This matters beyond the test: Risks row 4 names
  Test #7 as the mitigation for "a degraded scan prints nothing", so whichever side an
  implementer silently changes, the mitigation is lost. Fix: restate the predicate as a
  count over candidate files, e.g. "at least one library was selected, at least one
  `appmanifest_*.acf` was found under it, and every one of them was skipped as unreadable,
  non-UTF-8 or unparseable, and no game was emitted", and keep "no manifests at all" on the
  `Ok(vec![])` side; both §7's sentence and Test #7 then describe the same condition.

- [major][new] *§2 Root selection* and *A9* — "`home` comes from `std::env::home_dir()` (A9)" —
  on the toolchain this repo pins (`rust-toolchain.toml` channel `1.85.0`), `std::env::home_dir`
  still carries `#[deprecated(since = "1.29.0")]`: the versioned reference
  (doc.rust-lang.org/1.85.0/std/env/fn.home_dir.html) shows the deprecation banner, and the
  1.85.0 release notes say the deprecation would be removed "in a subsequent release"
  (crates.io's `home` crate records it was undeprecated in 1.87). The call site therefore
  emits a `deprecated` warning, and A18's gate — `.github/workflows/check.yml:31`
  `cargo clippy --workspace --all-targets -- -D warnings` — turns it into a hard failure, so
  §10's "the gates pass" and Test strategy #10 are false as written. A9 is marked `verified`
  against the *unversioned* doc page, which no longer shows the deprecation and so does not
  support the pinned-toolchain fact the design actually depends on. Fix: replace the sentence
  with `std::env::var_os("HOME")` (documenting that the passwd-entry fallback is not used on
  the pinned floor), or keep `home_dir()` behind a named `#[allow(deprecated)]` with a
  one-line reason; either way add the pinned-toolchain deprecation status as its own
  assumption-register row rather than folding it into A9.

- [minor][new] *§9 Fixtures* vs *Test strategy #3 and #7* — "Read-only, checked-in, one tree
  used by both packages" — three roots the test strategy asserts are not in the fixture tree,
  and §9 rules out building them at runtime ("No test writes to the filesystem"): #3's "a root
  with no `steamapps` directory yields an empty list", #3's "a root whose `libraryfolders.vdf`
  is absent still yields its own `steamapps`", and #7's "a root that has a `steamapps`
  directory whose entries are all unparseable". The last one is the only pin for §7's single
  error path. Fix: extend the §9 tree with two more roots under `fixtures/steam/` — e.g.
  `empty/` (a directory with no `steamapps`) and `broken/steamapps/` holding two truncated
  `appmanifest_*.acf` — and cite them from #3 and #7; no new mechanism is needed.

- [nit][new] *§1 Modules and public surface* — "Every `pub` item above is exercised by a test
  in *Test strategy*" — `SteamLinuxPlugin::from_host()` is not exercised by any test (by
  design: Test #10 says "no test reads the developer's home directory"), and `name()` returning
  `"steam-linux"` is asserted nowhere. Fix: scope the sentence to "every `pub` item except
  `from_host()`, which is the thin wrapper over `host_candidates` + the probe that Test #5
  pins", or add a `name()` assertion to Test #8.

- [nit][new] *§9 Fixtures* — "the crate's tests directly, the CLI's through
  `env!(\"CARGO_MANIFEST_DIR\")` (A19)" — A19 gives a test the root of *its own* package, so
  from `apps/discovery-cli` the shared tree is reached by a `../../crates/discovery/tests/…`
  hop that the section never states; an implementer who reads it as "the fixture root" gets a
  path that does not exist. Fix: name the join explicitly
  (`env!("CARGO_MANIFEST_DIR") + "/../../crates/discovery/tests/fixtures/steam/minimal"`).

## FYI (no severity)

- `keyvalues::KeyValueError { line, message: &'static str }` is a new `pub` type with no
  `Display`/`std::error::Error`/`Clone`; *Deferred* covers `DiscoveryError`'s taxonomy but not
  this one. Compatibility/complexity seat's call. (compatibility)
- §5 emits an `install_path` composed from user-editable files without checking that it stays
  under the library, and *Deferred* hands validation to "the codec's own design"; the §5
  rejections (`.`/`..`/absolute/`/`) plus §2's canonicalization are the only in-scope control.
  Security seat's call. (security)
- Test strategy labels items 1–7 "Unit" while §9 places them in `crates/discovery/tests/*.rs`,
  which Cargo builds as integration tests with public-API-only access. Consistent with the
  repo's existing `slug_tests.rs` layout; the label is what misleads. (complexity)

## Checklist coverage

1. Assumption register — A1, A2, A3, A4, A5, A8, A12, A14, A15, A17, A18 re-checked against
   their cited bases and the repo: all supported (32 `[[package]]` entries in `Cargo.lock`;
   `crates/discovery/Cargo.toml:6-7` is `serde` only; `engine.rs:20-33` swallows plugin `Err`
   into one `eprintln!`; `check.yml:16-33` matches A18; `DummyPlugin` has exactly the three
   references in `apps/discovery-cli/src/main.rs`). A6, A7, A16, A20 are correctly marked
   `unverified`. A9's basis does not support the pinned-toolchain fact §2 depends on — filed
   above.
2. Source spot-check — fetched `steamlocate-rs/src/{app,library,locate/linux}.rs`,
   `ValveSoftware/steam-for-linux#8653`, `docs.rs/aurelia-steam` `library_folder_paths`,
   `docs.rs/steamie-core` `vdf`, and the 1.85.0/1.100.0 `home_dir` pages. All live and all
   support the rows that cite them; A8's candidate list matches `locate/linux.rs` item for
   item including the `$SNAP_USER_DATA` default.
3. Internal contradictions — three filed (§3 vs §5/A2/§2; §7 vs Test #7; §9 vs Test #3/#7).
4. Missing edge cases — empty file, absent Steam, absent `libraryfolders.vdf`, unreadable
   file, name collisions, symlink aliases and the `"0"` duplicate, non-UTF-8, size/depth caps,
   and mid-write truncation are each covered somewhere in §2/§4/§5/§7; no additional gap found.
5. Interfaces — the same names and shapes are used consistently across §1, §5 and Test
   strategy, apart from the "library" directory level filed as the blocker.
6. Source-document coverage — T-001's four plugin clauses, both CLI Done-when items and the
   unit-test item are each decided. N/A: nothing in the task is left undecided.
7. Revision rounds — first pass; the Revision log's 13 r0 entries each map to live text and
   none of the findings above is a rejected one re-reported.
Missing evidence: none blocking beyond the filed items. `01-research.md` line references
(99-102, 103-106, 180-192, 319-324, 416-420, etc.) could not be verified in this pass because
the file is outside the caller's allowed inputs; nothing in the argument above depends on them.
