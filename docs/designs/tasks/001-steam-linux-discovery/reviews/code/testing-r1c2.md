# Code review — testing seat, round 1, dissent pass c2

- Change under review: `git diff 77e24c72f9ed0c5347a891337d36029ba9fa4eb6..HEAD -- ':(exclude)docs/designs'`
  (3 files: `apps/discovery-cli/tests/cli_tests.rs` +4/−1, `crates/discovery/src/game.rs` −3,
  `crates/discovery/src/steam_linux.rs` +4/−4), read against the full current `02-proposal.md`.
- Inputs used: the incremental diff, current source and tests at HEAD, `02-proposal.md`,
  `docs/tasks/T-001-steam-linux-discovery.md`, the runner's gate record, and my r1 findings.
- Verdict: **approve** (zero open blocker/major; the r1 major is resolved as prescribed; five minors
  and three nits remain open, none gating).

## Gate evidence (checklist 6)

Runner-captured, green, from the code under review (gate 2026-11-18T04:41:08Z):
`npm run check` exit 0; `cargo fmt --all -- --check` exit 0;
`cargo clippy --workspace --all-targets -- -D warnings` exit 0;
`cargo test --workspace --locked` exit 0 (`test result: ok.`). The revision's only behavior-visible
change — the CLI JSON assertion `obj["platform"] == "SteamLinux"` (cli_tests.rs:124) — is exercised by
the green `cargo test` leg, and the reverted §7 message string is compiled in the green clippy/test
legs. The evidence is current for this revision and consistent with my source reads; I executed no
additional checks (my open questions are all coverage-absence, which re-running cannot resolve, and no
leg is missing or stale).

## Revision vs. my r1 findings

**Resolved — the r1 major (Platform serde rename vs. proposal Context:14-15, Non-goals:35, §8:262).**
The revision took exactly the first branch of my prescribed fix: `#[serde(rename_all = "kebab-case")]`
and its doc comment are removed from `crates/discovery/src/game.rs` (HEAD game.rs is byte-identical to
main's — `git diff main...HEAD -- crates/discovery/src/game.rs` is empty, so the net branch again touches
no data model, per the additive-task rule), and `apps/discovery-cli/tests/cli_tests.rs:124` now asserts
`obj["platform"] == "SteamLinux"` for all four JSON rows, with a comment citing §8/Context. Checklist
item 3 is satisfied: this per-row assertion would have failed the r1 revision and fails any future
serialized-shape change, so the contract has a guard at the layer where it is user-visible (the CLI's
`--json` output), matching §8:262's "table header and JSON shape are unchanged". With the derive gone
there is no crate-level serde attribute left to pin, and `Platform` serde behavior is now unchanged code
outside this task's test strategy — I withdraw the `game_tests.rs` half of the r1 fix as no longer
applicable. Closed.

**Narrowed — r1 nit on §7 message wording drift (`entries` → `file(s)`).** The revision reverted the
message to `{} file(s) skipped as unreadable or malformed` (steam_linux.rs:84), restoring §7:246's
"count of skipped files" and the count-bearing noun. The drift aspect is closed; the no-test-pins-the-
count aspect stays open (finding N2 below).

**Open — the other five minors and the two other nits are untouched by the revision** (the steam_linux.rs
hunk is comment + string only; no test file besides cli_tests.rs changed; no fixture changed). Re-listed
below with line numbers verified at HEAD.

## Checklist walk

1. Numbered Test strategy items #1–#10 all retain citing tests (`// proves:` comments throughout);
   none regressed. The open findings below concern design properties that are *not* in the numbered
   list (§2 cross-root set, §5 tie-break, §6 cost, §4 cap, absolute `path`) plus §7's count.
2. Unit + integration layers both present (crate tests + CLI integration), unchanged this round.
3. Revision round: the r1 major now has a test that would have caught it (cli_tests.rs:124). ✔
4. Determinism: unchanged; new/kept tests use fixtures and a fake path, no clock/network/home probing.
5. Test names still describe behavior; the changed assertion lives inside
   `test_cli_list_json_format`.
6. Gate evidence: green and current, cited above. No failing leg.
7. Untestable additions: the r1 `!self.roots.is_empty()` redundancy (finding N3) is still present.

## Open findings

### Minor

1. **Cross-root "one set per `discover()` call" is untested.**
   Location: `crates/discovery/src/steam_linux.rs:61-71` (cross-root de-dup at line 68);
   `crates/discovery/tests/steam_linux_tests.rs:32,454,465` and `apps/discovery-cli/tests/cli_tests.rs:84,150`
   (every call passes exactly one root).
   Problem: §2's canonical statement ("no library is ever scanned twice, including by two roots sharing
   one") has no test. Dropping the `canonical_paths(libraries)` call at line 68 (keeping per-root dedup at
   line 162) keeps every test green while duplicating rows for e.g. a `~/.steam/root` plus real-root host.
   Fix: unit test with `from_roots([fixture("minimal"), fixture("minimal/secondary")])` asserting exactly
   the same four rows with no duplicate `unknown/secondary-title`, plus one CLI test passing `--steam-root`
   twice over the same fixture tree asserting the unchanged four-row table.

2. **`(slug, install_path)` tie-break and §5's "identical slugs stay two rows" are untested.**
   Location: `crates/discovery/src/steam_linux.rs:91`; `crates/discovery/tests/fixtures/steam/` (four
   distinct slugs only).
   Problem: unchanged from r1 — the fixture yields four distinct slugs, so only the slug component of the
   sort key is exercised; a slug-only sort or slug de-duping passes the suite.
   Fix: add a checked-in `appmanifest_900006.acf` under `minimal/secondary/steamapps/` with `"name"
   "Example Game"` (same name, other library) and extend the #8 integration expectation to five rows with
   `unknown/example-game` twice in install_path order.

3. **§6's accepted cost (non-game/Proton/runtime rows emitted) is unpinned.**
   Location: `crates/discovery/src/steam_linux.rs` (`scan_library`/`discovered_game`) and the fixture set
   (every manifest carries `"StateFlags" "4"`, e.g. `minimal/steamapps/appmanifest_900001.acf:6`).
   Problem: unchanged from r1 — adding a StateFlags bit test or name denylist keeps all tests green,
   silently reversing the §6 policy decision (Risks: likelihood High).
   Fix: add one fixture manifest with a non-ready `StateFlags` (e.g. `"6"`) and a runtime-shaped
   `installdir`, and assert in the #8 integration test that it is still emitted, cited
   `// proves: Chosen design §6`.

4. **The 1 MiB per-file cap (§4/A16) has zero test coverage on both legs.**
   Location: `crates/discovery/src/keyvalues.rs:21,77` (`MAX_DOCUMENT_BYTES`);
   `crates/discovery/src/steam_linux.rs:287-298` (`read_text_file` size gate).
   Problem: no test parses an oversize document and no test reads a file above the cap; inverting either
   comparison breaks no test, though §4 makes these caps the entire per-file work bound. Correction to my
   r1 wording: the drift half of that finding was overstated — steam_linux.rs:290 already restates the
   parse-budget constant by reference (`const MAX_FILE_BYTES: u64 = keyvalues::MAX_DOCUMENT_BYTES as u64`),
   so the two constants cannot drift; the remaining gap is pure coverage. (Note: the cap is not in the
   numbered Test strategy, so this is §4-coverage, not checklist-1.)
   Fix: cited unit tests — `parse` on a 1 MiB + 1 byte document returns a `KeyValueError`, and a
   `read_text_file`-leg (or §7 error-count) expectation for an oversize manifest via the
   `env!("CARGO_TARGET_TMPDIR")` escape hatch.

5. **The absolute-`path` arm of library selection is never exercised.**
   Location: `crates/discovery/src/steam_linux.rs:154-160` (`path.is_absolute()` branch);
   `crates/discovery/tests/fixtures/steam/minimal/steamapps/libraryfolders.vdf` (only `".."` and
   `"../secondary"`).
   Problem: unchanged from r1 — this is the shape every real `libraryfolders.vdf` uses and the Compatibility
   matrix's "libraries on arbitrary mounts — Yes" row; no test uses `env!("CARGO_TARGET_TMPDIR")`, and a
   change in library-path resolution would pass silently.
   Fix: per §9's sanctioned escape hatch, build a one-library tree under `CARGO_TARGET_TMPDIR` with a
   `libraryfolders.vdf` whose numeric block carries that absolute path, and assert `libraries_in_root`
   returns it as-is.

### Nit

6. **`contains('\0')` refusal in `usable_install_dir` is unstated and untested.**
   Location: `crates/discovery/src/steam_linux.rs:277`;
   `crates/discovery/tests/steam_linux_tests.rs:113` (`for bad in ["", ".", "..", "/abs/path", "sub/dir"]`).
   Problem: unchanged from r1 — a guard beyond §5's enumerated rejection set with no test case.
   Fix: add `"\0"` (and a whitespace-only value) to the existing `for bad in [...]` list in
   `parse_manifest_rejects_unusable_installdir`.

7. **§7's skip count and message wording are still unpinned (wording-drift half now resolved).**
   Location: `crates/discovery/src/steam_linux.rs:84`;
   `crates/discovery/tests/steam_linux_tests.rs:464-492`.
   Problem: the revision reverted the r1 wording drift, restoring §7:246's "count of skipped files". But
   no test pins the count, the `file(s)` noun, or any message wording beyond the root name — a bug
   reporting the wrong skip count still passes.
   Fix: in `discover_of_all_broken_root_returns_plugin_error_naming_root_without_contents`, assert the
   message contains the expected skip count (the all-broken fixture's manifest count) and pin the fixed
   message tail, cited `// proves: Chosen design §7`.

8. **The `!self.roots.is_empty()` clause remains an untestable addition (checklist 7).**
   Location: `crates/discovery/src/steam_linux.rs:77`.
   Problem: unchanged by this revision (the comment above it was re-worded, the clause kept) — `skipped`
   is only incremented inside the `if !self.roots.is_empty()` scan block at lines 61-71, so `skipped > 0`
   already entails a selected root; the roots-empty + skipped>0 combination is unconstructible, so no
   test can exercise the clause, while the comment claims it spells out a §7 clause.
   Fix: drop the redundant clause (the scan-block guard plus `skipped > 0` already encode the three §7
   clauses) or make the roots-selected precondition real (e.g. count skipped roots) so a test can pin
   each clause.

## FYI (out of specialty, no severity)

- The new comment at cli_tests.rs:121-123 cites review-round history ("the r1 major … was reverted as
  unauthorized") inside test source; design-clause citations (as elsewhere in this suite) would age better.

## Verdict

approve — 0 blocker, 0 major, 5 minor, 3 nit (all carried or narrowed from r1; the sole r1 major is
resolved exactly as prescribed and guarded by cli_tests.rs:124).
