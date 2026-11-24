# Code review — compatibility seat, round 1 dissent pass (c1)

- **Run**: `docs/designs/017-keybinding-extraction-codecs/` (ADR-017)
- **Seat**: compatibility (one specialty; `references/compatibility.md`)
- **Scope**: `git diff a8fd09ec6d6b8210574447f9b68e7cc199f73153..HEAD -- ':(exclude)docs/designs'`
  (15 files: `crates/codecs/src/{codec,families/unreal_ini,lib,paths,read,registry,snapshot}.rs`,
  `crates/codecs/tests/{registry,snapshot,unreal_ini,zz_api_probe}.rs`,
  `apps/discovery-cli/src/bindings.rs`, `schemas/README.md`,
  `schemas/bindings/v1/spec.md`, `schemas/bindings/v1/fixtures/invalid/files-traversal-files.json`)
- **Pass type**: dissent c1 — this pass checks the findings of my round 1 first;
  new items only where the changed code makes them true.
- **Inputs used**: that diff; `02-proposal.md` (full current document);
  `docs/adrs/017-keybinding-extraction-codecs.md`; the runner's gate record; my round-1 findings.
  No review report or transcript was opened.
- **Verdict**: **approve** — 0 blocker, 0 major, 1 minor, 3 nit.

## Gate evidence

The runner's gate at 2026-11-24T07:31:52Z is green on all four legs. The one captured
`test result: ok. 0 passed; 0 failed;` line is a single leg's tail, so I re-ran the
crates myself: `cargo test -p bindforge-codecs --locked` → all legs green
(27 + 9 + 7 + 7 + 6 + 5 + 4 + 3 + 2 passed, 0 failed across the binaries). The five
tests this pass added all *execute* on the Linux leg —
`byte_named_directories_and_files_locate_at_their_real_paths`,
`a_control_byte_in_an_echoed_path_is_escaped_and_stays_one_line`,
`the_open_cap_bites_on_matches_that_never_locate`,
`removals_of_absent_pairs_are_no_ops`,
`the_loader_refuses_escaping_files_spellings_naming_files`,
`pass_two_does_not_re_run_a_family_the_appid_pass_already_ran`. No missing-evidence
finding: the round-1 fix's claims are covered by tests that actually run here.

## Round-1 findings: disposition

| R1 finding | Status | Evidence |
|---|---|---|
| **A (major)** — locate walk rebuilt child paths from `file_name().to_string_lossy()`, so a non-UTF-8 name invented a path that never existed (grammar-shaped file → recorded candidate → whole game `UnreadableFile`/exit 5; non-UTF-8 directory → whole subtree silently skipped) | **Resolved** | `unreal_ini.rs:233` now `let path = entry.path();`; the lossy name survives only as the sort key (`210-217`) and as the `child_relative` grammar component (`231-232`); `candidates.push(path)` gets the OS-named path. `Err` arm split at `257-265`: `FileOverBudget`/`UndecodableText` locate on path grammar alone, everything else is a skip. Pinned by `tests/unreal_ini.rs::byte_named_directories_and_files_locate_at_their_real_paths` (`weird\xffname.ini` and a `d\xffr` platform directory), which asserts both files locate *at their real paths* and fold to the right pairs — that test fails against the round-1 code. The `a_reader_refused_match_is_located_and_refused_by_name` pin still passes, so the "never unseen" half survived the split. |
| **B (minor)** — `tree_relative` emitted platform-native separators into `provenance.files`, so a Windows host stated `Saved\Config\…` and re-sorted the array (`\`=0x5C vs `/`=0x2F), breaking the field's own portability rule | **Resolved in code** | `unreal_ini.rs:541-554` builds the contract spelling component-by-component and joins with `/`; `schemas/bindings/v1/spec.md` now states the rule ("spelled `/`-joined component by component whatever the host separator"); `--probe` keeps absolute host-native paths, as §The CLI face says. The new loader rule (`snapshot.rs:305-323`) additionally refuses `\` and `:` anywhere, so a host that ever regressed to native separators is refused loudly rather than shipping a second spelling. No pin runs for the separator itself (see nit 2) — the code and the contract carry it. |
| **C (minor)** — the `paths.rs` comment and the Compatibility matrix's NTFS/exFAT row promised `family-confirmed-files-absent` (exit 4); shipped behavior is `claim-refused` (exit 3) | **Partly resolved** — carried as minor 1 below | Comment corrected (`paths.rs:133-141`); the proposal's matrix row (line 628) still reads exit 4. |
| **D (nit)** — case-exact `Saved`/`Config`/`.ini` match was an unstated assumption | **Resolved** | Doc comment `unreal_ini.rs:274-279` names the match byte-exact/case-exact by design and states the case-folded consequence; `spec.md:32-37` records it as a locate-time contract fact. |
| **E (nit)** — `unreal_ini`'s link test had no `#[cfg(not(unix))]` arm; plan drift on `symlinked-prefix` | **Resolved in code** | `tests/unreal_ini.rs:225-230` adds the compile-only arm, and both new unix-only tests carry matching arms. The build-notes half is outside my inputs. |

## Checklist walk

1. **Platform code matches the Compatibility matrix** — the Windows row's promise
   ("the root-relative trees resolve and are stat'd", documents portable) now holds
   with the `/`-joined spelling; the Linux/Proton/Flatpak rows are unchanged by this
   diff. The NTFS/exFAT row is the one row still contradicted by shipped behavior
   (minor 1).
2. **Node floor** — not applicable: no `tools/orchestration` path in the diff.
3. **Paths** — the diff's central change is exactly this item: walk with
   `DirEntry::path()`, keep the lossy rendering as a key only, normalize the
   contract spelling, refuse foreign spellings at the loader, and flag the
   case-sensitivity assumption in both the code comment and `spec.md`. One asymmetry
   the new loader rule opens on the producer side (nit 2).
4. **Guarded platform imports** — the only platform-specific imports added
   (`std::ffi::OsString`, `std::os::unix::ffi::OsStringExt`) sit inside
   `#[cfg(unix)]` tests, each with a `#[cfg(not(unix))]` sibling, so the
   `x86_64-pc-windows-msvc` compile leg (A22) keeps compiling.
5. **Locale/layout reality** — raw-byte names on ext4 now locate and extract at
   their real paths; contract spelling of such a component is the deterministic
   `to_string_lossy` of it (U+FFFD), which is stable across runs and across the two
   places the codec renders it (`tree_relative`, `game_key`), so `017-L4` is not
   disturbed. Non-ASCII action/input tokens are untouched.
6. **Native dependencies** — none introduced; `Cargo.lock` delta stays the
   `bindforge-codecs` stanza plus the CLI edge (`zz_api_probe` pins it, still green).
7. **Platform-scoped tests** — the three unix-only pins are guarded symmetrically
   and execute on the runner (verified above). Nothing is skipped everywhere.

## Findings

### minor 1 — the NTFS/exFAT row still promises a status the code cannot produce on that route
**Location**: `crates/codecs/src/paths.rs:133-141` (the corrected comment), read
against `02-proposal.md:628` (Compatibility matrix, "Steam library on NTFS/exFAT with
symlinked prefixes") and `apps/discovery-cli/src/bindings.rs:267-271`.
**Problem**: the fix corrected the comment but re-described the matrix instead of
correcting it: the current proposal row still states that "a broken prefix reads as
`family-confirmed-files-absent` (exit 4)". Shipped behavior is `claim-refused`
(exit 3): an unanchorable prefix yields no `UserTree`, so `unreal_ini::detect`
returns `None` (`unreal_ini.rs:86-88`) and no claim exists to be statused 4 — and no
shipped family carries appids, so the appid-table route the comment points at
(`paths.rs:139-141`, "`family-confirmed-files-absent` belongs to appid-table claims")
is not the only route either: `bindings.rs:269-271` reaches exit 4 from *any* claim
whose documents all fold empty, which an `install-tree-probe` claim can do. The row's
safety half (no hang, no out-of-tree read) holds, so this is a record-accuracy item,
not a behavior defect.
**Fix**: one-line amendment to the proposal's matrix row — "a broken prefix locates
no grammar, so the face answers `claim-refused` (exit 3); `family-confirmed-files-absent`
(4) belongs to a claim that extracts nothing" — or add the row to the §Rollout 8
ledger hand-off list with that wording, and drop the appid-table-only rationale from
the `paths.rs` comment so it cites the empty-fold route instead.

### nit 1 — the `UnreadableFile` skip swallows permission failures, turning a loud exit 5 into a silent gap
**Location**: `crates/codecs/src/families/unreal_ini.rs:257-266` with
`crates/codecs/src/read.rs:27-47`.
**Problem**: the split my round-1 fix asked for is implemented one class wider than
needed. `open_capped` collapses every OS error — `NotFound` (a config the game is
mid-rewrite on), `PermissionDenied`, and an I/O error on a mounted library — into
`UnreadableFile`, and the walk's `Err(_) => {}` arm discards all of them. A
grammar-shaped `Engine.ini` that exists but cannot be read (a Proton prefix created
under another uid, a mode-000 or root-squashed file on an NTFS/exFAT library — the
same layout the matrix's second row claims to serve) is now neither located nor
refused: the game extracts a snapshot whose `provenance.files` simply omits it, so a
partial reading of the game's binding surface is stated as the whole one. Before this
diff that layout answered `extraction-refused` (exit 5), which is what `017-L1`'s
"named refusals exist to make drift loud rather than wrong" asks for.
**Fix**: keep the skip for the vanished-entry case only — carry the
`std::io::ErrorKind` through (or map `ErrorKind::NotFound` to `UnreadableFile` and
every other kind to a refusal the walk does not swallow), so `NotFound` stays a skip
and `PermissionDenied`/other I/O failures keep refusing the game by name; or, if no
new name is wanted, count grammar matches dropped this way and print the count under
`--probe`.

### nit 2 — the producer can state a `files` spelling the loader's new rule refuses
**Location**: `crates/codecs/src/snapshot.rs:305-323` (new rule) with
`crates/codecs/src/families/unreal_ini.rs:280-287` and `541-554`.
**Problem**: `grammar_match` constrains only `Saved`, `Config`, the `.ini` suffix and
the depth, so a component the game's own filesystem chose may contain `:` or `\` —
both legal octets in an ext4 name, and reachable in the Proton prefix's
`Documents/My Games/<project>` component as well as in a platform directory or a file
named `foo:bar.ini`. `tree_relative` joins such a component into
`provenance.files` verbatim (lossy only for invalid UTF-8, never for these two
octets), so the document the shipped family emits for that layout is refused by the
family's own loader naming `files` — the §Test strategy #11 claim that "the extraction
and contract halves cannot disagree" becomes conditional on file names, and only on
the platform whose names allow those bytes. Nothing asserts the boundary: the honesty
pin checks only `!stated.starts_with('/')` (`tests/honesty.rs:138-142`).
**Fix**: make the boundary producer-side too — reject a grammar match whose relative
components contain `\` or `:` (the same documented silent-non-claim posture as the
case-exactness note just added), or state in `spec.md` that a located name carrying
those octets is outside what `bindings/v1` can spell; and add an assertion over
produced `files` entries (no host separator other than `/`, no `:`) — buildable on
the Linux leg, since a name containing `\` or `:` is constructible there, which also
pins `tree_relative`'s component-join behavior that no leg otherwise exercises.

## FYI (out of specialty, no severity)

- `unreal_ini.rs:113-114` sorts the located files by `PathBuf` (component-wise)
  while `spec.md`'s ordering law and the fold order speak of *byte order of the
  relative path*; with two components where one is a prefix of the other and the next
  octet is below `/` (a `My Games`/`My Games 2`-shaped pair in one prefix), the two
  orders differ, which moves both the printed `files` order and the fold order that
  `-`/`!` removal semantics depend on. Pre-existing (unchanged by this diff) and the
  determinism seat's law.
- `apps/discovery-cli/src/bindings.rs:227-237` memoizes a claim on
  `(appid, library, roots)`, which is exactly what the *shipped* resolvers read; the
  cached `Detection` is nonetheless handed to a trait object whose contract gives it
  the whole `GameContext`, so a `017-R3` family that probes `install_path` would be
  answered from another row's walk. Keying on the full context, or naming the
  invariant at the cache, keeps that a non-issue.
- The tightened `files` rule did not add the "no ASCII control character in any
  string" check `spec.md:70-72` states, which `snapshot.rs:367-395` applies to
  tokens but not to `files`; the round-1 escape work is on the `Display` side only.
