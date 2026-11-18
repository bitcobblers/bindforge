# Code review — testing seat — round 1, pass c0

- Scope reviewed: `git diff 4caa688933b2969f569f6126dddb713180c7c410...HEAD -- ':(exclude)docs/designs'`
  (22 files, +1639/−33). Product paths under review: `crates/discovery/src/{keyvalues.rs,steam_linux.rs,lib.rs,game.rs}`,
  `crates/discovery/tests/{keyvalues_tests.rs,steam_linux_tests.rs}`,
  `crates/discovery/tests/fixtures/steam/**`, `apps/discovery-cli/src/main.rs`,
  `apps/discovery-cli/tests/cli_tests.rs`. Harness paths in the same diff: `.opencode/agents/code/{coder,test-engineer}.md`,
  `tools/orchestration/src/pipeline/flows.ts`.
- Inputs used: that diff, `02-proposal.md`, `docs/tasks/T-001-steam-linux-discovery.md`, the runner's gate
  record, and the executed read-only checks below. No earlier review report or transcript was opened.
  Method note for the deed audit: one repo-wide content grep (pattern `SteamWindows|steam-windows`, used to
  check whether any in-repo consumer depends on the PascalCase serialization) incidentally surfaced matches
  inside `docs/designs/**/reviews/**` and `docs/designs/**/transcripts/**`. Those files were not opened,
  listed or read as inputs; the finding below rests solely on the diff, `02-proposal.md`,
  `crates/discovery/tests/game_tests.rs`, `crates/discovery/src/game.rs` and the grep of source/schema paths.
- Checklist applied: `references/testing.md` only.

## Gate evidence (checklist #6)

Supplied, from the runner's gate at 2026-11-18T04:11:25Z — all four legs green:
`npm run check` (exit 0), `cargo fmt --all -- --check` (exit 0),
`cargo clippy --workspace --all-targets -- -D warnings` (exit 0), `cargo test --workspace --locked` (exit 0).

The single `test result: ok. 0 passed; …` line the renderer captured is ambiguous on its own — read alone it
could mean the new tests never ran. I re-ran the suite to resolve that coverage question:
`cargo test --workspace --locked` (executed by me, exit 0, green) reports per target
`tests/keyvalues_tests.rs … test result: ok. 14 passed; 0 failed`,
`tests/steam_linux_tests.rs … test result: ok. 23 passed; 0 failed`,
`tests/cli_tests.rs … test result: ok. 3 passed; 0 failed`, with the pre-existing
`{game,plugin,registry,slug}_tests.rs` legs also green. The captured tail is one of the workspace's several
zero-test targets (`src/lib.rs` / `src/main.rs` unittests and the three doc-test legs each print
`0 passed`). So the gate is genuinely green and 40 new tests run; the supplied tail is a rendering artifact,
not a finding. `Cargo.lock` and both `Cargo.toml` files are absent from the diff, which is the §10 /
Rollout #3 "dependency posture held" check.

No required gate fails, and no check I needed was unavailable.

## Checklist walk-through

1. **Test-strategy coverage by citation.** Items #1–#9 all have tests carrying a
   `// proves: Test strategy #N` citation: #1 → `keyvalues_tests.rs` (14 tests, including the `#base` and
   16/17-level cap cases); #2 → `steam_linux_tests.rs:69-152`; #3 → `:158-195`; #4 → `:197-207` and `:420-443`;
   #5 → `:213-317`; #6 → `:360-418`; #7 → `:449-492`; #8 → `:332-358`; #9 →
   `apps/discovery-cli/tests/cli_tests.rs:36,75,140`. Item #10 is the gate item itself and is evidenced by the
   green legs above plus the untouched `Cargo.lock`, not by a test. **Pass.**
2. **Unit + integration layers as promised.** The crate's grammar/selection/manifest units and the fixture-root
   plugin integration both live in `crates/discovery/tests/` per the repo's existing
   `<module>_tests.rs` convention (§9), and the CLI layer runs the built binary through
   `env!("CARGO_BIN_EXE_discovery-cli")` (§8/§9, A19). **Pass.**
3. **Revision rounds.** No previously rejected blocker/major exists for this seat (first pass). **N/A.**
4. **Determinism.** No wall-clock, no network, no ports, no filesystem writes in any new test; every plugin and
   CLI test is pinned to the read-only fixture tree through `env!("CARGO_MANIFEST_DIR")`, `from_roots` and
   `--steam-root`, and `host_candidates` is driven by an injected env closure — no test reads `$HOME`,
   `$XDG_DATA_HOME` or `$SNAP_USER_DATA`, and no test calls `from_host()`, so the `ubuntu-latest`-no-Steam
   claim in #10 holds. `read_dir` order is handled by an explicit sort in both the code and the assertions.
   The one environment-shaped literal, `--steam-root /bindforge-no-such-steam-root`
   (`cli_tests.rs:140-151`), is a nonexistent absolute path and deterministic. **Pass.**
5. **Names describe behavior.** All 40 names read as behavior/contract statements
   (`parse_manifest_rejects_unusable_installdir`, `discover_of_all_broken_root_returns_plugin_error_naming_root_without_contents`,
   `host_candidates_ignores_relative_xdg_data_home`). **Pass.**
6. **Gate evidence.** Assessed and corroborated as above.
7. **Untestable/untested additions.** Several additions and design statements have no pin — findings below.

## Findings

### F1 — major — `crates/discovery/src/game.rs:8` (with `apps/discovery-cli/tests/cli_tests.rs:121`)

The diff adds `#[serde(rename_all = "kebab-case")]` to `Platform`, which re-spells
`DiscoveredGame.platform` on the machine-readable channel from `"SteamLinux"` to `"steam-linux"`
(and, since the derive has no `alias`, `Deserialize` no longer accepts the old spelling). The design forbids
this: `02-proposal.md:14-15` "no change to `DiscoveredGame`, `Platform`, …", `:35` Non-goals "Any change to
`DiscoveredGame`", `:262` "the printing code is untouched, so the table header and JSON shape are unchanged".
The testing problem is where the change is guarded: the only assertion of the new spelling in the repository is
`cli_tests.rs:121`, written against the post-change behavior — the base `cli_tests.rs` asserted only `[]`
(visible in this diff's removed lines), so nothing in the base would have flagged it, and
`crates/discovery/tests/game_tests.rs` (the data-model test, untouched by this diff) contains no serde
assertion at all, so the package that owns `Platform` pins neither direction of its wire form. Consequence:
the suite cannot distinguish a ratified rename from an accidental derive edit — a revert or a further rename in
`game.rs` is caught only by a downstream CLI test, and nothing tests that previously emitted JSON still
deserializes. Note the new spelling does match `docs/adrs/022-game-discovery.md:34` and the existing `Display`
impl, and no in-repo consumer of the PascalCase form exists (grep over `schemas/`, `data/`, `*.rs`): so the
rename may well be the right call, but it is currently carried as an untested side effect of a test expectation
rather than as a stated, pinned change.

Fix: make the contract explicit at its own layer and deliberate in scope. Either drop the derive attribute from
this additive task and assert `"SteamLinux"` at `cli_tests.rs:121`, or keep the ADR-022-aligned kebab form and
add to `crates/discovery/tests/game_tests.rs` a cited test that pins both directions
(`serde_json::to_value(Platform::SteamLinux) == "steam-linux"` plus a round-trip, and the old spelling's fate
stated via `#[serde(alias = …)]` if prior output must keep deserializing) — the proposal's `:262` "JSON shape
… unchanged" sentence then needs correcting in the same change.

### F2 — minor — `crates/discovery/tests/steam_linux_tests.rs` (no test); property at `crates/discovery/src/steam_linux.rs:61-71`

§2's canonical statement ("the library paths of all selected roots as **one set** per `discover()` call … no
library is ever scanned twice, **including by two roots sharing one**") has no test. Test strategy #3 pins
de-duping only inside one root (`"0" → ".."`, `steam_linux_tests.rs:161-171`). The single error the design
designed against here — hoisting `canonical_paths` so libraries are de-duped per root instead of per call —
leaves every current test green while duplicating rows for a `~/.steam/root` + real-root host. Related surface
also unexercised: §8's `--steam-root` is documented as repeatable, and all three CLI tests pass it once.

Fix: one unit test with `from_roots([fixture("minimal"), fixture("minimal/secondary")])` asserting exactly the
same four rows (no duplicate `unknown/secondary-title`), and one CLI test passing `--steam-root` twice over the
same fixture tree with an unchanged four-row table.

### F3 — minor — `crates/discovery/src/steam_linux.rs:89` sort key; `crates/discovery/tests/fixtures/steam/`

§5's two statements "Identical slugs from two libraries stay two rows" and the `install_path` **tie-break** of
the `(slug, install_path)` sort are untested: the fixture yields four distinct slugs, so
`discover_output_honours_the_game_output_contract` (`steam_linux_tests.rs:408-417`) only ever exercises the slug
component, and no fixture produces a duplicate slug at all. A change to a slug-only sort, or to
de-duplicating by slug, is invisible to the suite.

Fix: add a checked-in `appmanifest_900006.acf` to `minimal/secondary/steamapps/` carrying
`"name" "Example Game"` (same name, different library), and extend #8's expected set to five rows with
`unknown/example-game` appearing twice in `install_path` order — this pins both the two-rows rule and the
tie-break in one fixture entry.

### F4 — minor — §6 selection policy (`crates/discovery/src/steam_linux.rs:198-247`, `scan_library` / `discovered_game`); fixture set

§6 is an explicit accepted cost — "No `StateFlags` bit test and no non-game filter, so Proton builds and the
Steam Linux Runtime appear in `discovery list` as rows (A13)" — and the Risks table rates its likelihood High.
Nothing pins it: every fixture manifest carries `"StateFlags" "4"` and every fixture name looks like a game, so
a `StateFlags` bit test or a name denylist introduced later would keep all 40 tests green. The design's Test
strategy has no item for this either, so this is a gap the test plan itself left, not an implementation slip.

Fix: add one fixture manifest with a non-ready `StateFlags` (e.g. `"StateFlags" "6"`) and a runtime-shaped
`installdir`, and assert in the #8 integration test that it is still emitted — a cited `// proves: Chosen
design §6` test that says "no filtering here on purpose".

### F5 — minor — `crates/discovery/src/keyvalues.rs:21,77` and `crates/discovery/src/steam_linux.rs:285-312`

The 1 MiB cap exists twice (`keyvalues::MAX_DOCUMENT_BYTES` at parse time, `read_text_file`'s pre-read
`metadata` check) and neither is covered by a test: no test parses an oversize document, and no fixture comes
near the size. §4 makes these two named constants the whole work bound per file (A16) and §7 puts the
skipped-file count in the error message precisely because these caps bite — yet flipping the comparison or
moving the constant breaks nothing.

Fix: a cited unit test that `parse(&oversize)` returns `KeyValueError { line: 1, .. }` (a 1 MiB + 1 byte quoted
token is enough), and one line asserting `MAX_DOCUMENT_BYTES` equals the repo's shared parse budget constant that
§4 says it restates, so the two cannot drift apart.

### F6 — minor — `crates/discovery/src/steam_linux.rs:152-160` (`libraries_in_root_with_skips` absolute branch)

The `path.is_absolute()` arm — the shape every real `libraryfolders.vdf` uses, and the
Compatibility-matrix row "libraries on arbitrary mounts (`/run/media/…`, `/mnt/…`, ZFS) — Yes" — is never
exercised: the fixture VDF carries only `".."` and `"../secondary"`, and the design itself explains why
(§9: "a checked-in fixture cannot contain this machine's absolute paths"). Risk is muted because
`PathBuf::join` with an absolute argument replaces the base, but the promised behavior is still unverified,
and the relative-only fixture means a change in how a library path is resolved would pass silently.

Fix: use the escape hatch §9 already sanctions for a writable tree — build a one-library VDF under
`env!("CARGO_TARGET_TMPDIR")` whose numeric block points at an absolute path inside that tmpdir (with a
`steamapps/` directory so the canonical-path rule keeps it) and assert `libraries_in_root` returns it; the
tmpdir route also gives F2's symlink-alias de-duping a cheap home later.

### F7 — nit — `crates/discovery/src/steam_linux.rs:273-280` (`usable_install_dir`)

The `contains('\0')` refusal is an addition beyond §5's enumerated set (absent, empty, `.`, `..`, absolute,
`/`) and is the only guard in that function with no test: `parse_manifest_rejects_unusable_installdir`
(`steam_linux_tests.rs:110-119`) covers the five documented cases and not the NUL one.

Fix: add `"\0"` (and, cheaply, `" ./ "`-style whitespace-only) to the existing `for bad in [...]` list, so the
extra refusal is a stated expectation rather than an unclaimed branch.

### F8 — nit — `crates/discovery/src/steam_linux.rs:75-87` (§7 message content)

§7 specifies the single error message as "names the selected roots **and the count of skipped files** and
nothing else". `discover_of_all_broken_root_returns_plugin_error_naming_root_without_contents` asserts the root
name and the no-contents half, but never the count — the very field that distinguishes "Steam present,
everything under it unreadable" from "nothing installed" in the engine's swallowed `eprintln!` (A14). A bug
that reports zero skips (e.g. counting only one of the two skip sites) would pass.

Fix: assert the message contains the expected skip count (`2` for the `all-broken` fixture: an unbalanced-brace
manifest and an unterminated-string manifest).

## FYI (no severity, outside this seat)

Two harness files ride in this product diff: `.opencode/agents/code/{coder,test-engineer}.md` flip three
`transcripts` read/glob/grep allows to deny, and `tools/orchestration/src/pipeline/flows.ts` changes the light
flow's `stageTimeoutMs` from `15 * 60_000` to `12 * 60_000`. On the testing side that timeout knob is not an
unpinned addition — it realigns the source with a pre-existing pin at
`tools/orchestration/tests/pipeline/flows.test.ts:104` (`expect(cfg.stageTimeoutMs).toBe(12 * 60_000)`), and
`npm run check` includes `vitest run` and is green — so this is a scope-hygiene observation only.

## Verdict

`request-changes`: one open major (F1). The suite itself is in good shape — every numbered Test strategy item
is cited, the fixture design delivers the decoy, fallback, mixed-case, malformed and escaping cases the design
promised, and every test is hermetic and order-independent.
