# Code review — compatibility seat, round 1 dissent pass (c1)

- Run: `docs/designs/tasks/001-steam-linux-discovery/` (T-001, Linux Steam discovery plugin)
- Seat specialty: **compatibility** (`.opencode/skills/code-review/references/compatibility.md`)
- Scope under review (as given by the runner): `git diff 752fd5abc9e4e88b57a98ef1c23c31daa3983c13..HEAD -- ':(exclude)docs/designs'`,
  read against the current `02-proposal.md`.
- Pass type: dissent c1 — the last revision was written to answer this seat's round-1 findings; those were
  re-checked first, and new items are raised only where the changed code makes them true.
- Inputs used: this diff; `02-proposal.md`; `docs/tasks/T-001-steam-linux-discovery.md`; the runner's gate
  record (green at 2026-11-18T04:31:16Z); this seat's round-1 findings as quoted by the runner; plus the
  surrounding source lines needed to establish a concrete finding (`crates/discovery/src/game.rs`,
  `crates/discovery/src/steam_linux.rs`, `crates/discovery/tests/game_tests.rs`,
  `crates/discovery/tests/steam_linux_tests.rs`, `apps/discovery-cli/src/main.rs`,
  `apps/discovery-cli/tests/cli_tests.rs`, `docs/adrs/022-game-discovery.md`, `schemas/`), and
  `git diff main...HEAD` used only to attribute the `Platform` serde edit to this branch. No review report
  or transcript was used as review input.

## What the revision under review actually changes

`git diff --stat` for the range: one file, `crates/discovery/src/steam_linux.rs`, +6/-4, entirely inside
`discover()`'s §7 error path (steam_linux.rs:73-89):

1. the `Err` guard gains the explicit first clause of §7 — `!self.roots.is_empty() && games.is_empty() && skipped > 0`;
2. the `PluginError` message's tail is reworded from `N file(s) skipped as unreadable or malformed` to
   `N entries skipped as unreadable or malformed`.

Compatibility assessment of the revision itself:

- **Behaviour is a no-op.** `skipped` is incremented only inside `libraries_in_root_with_skips`
  (steam_linux.rs:66), `scan_library` (steam_linux.rs:69) and the helpers they call, and all of those are
  reached only under the pre-existing `if !self.roots.is_empty()` guard at steam_linux.rs:61. So the new
  clause cannot suppress or add an `Err` in any reachable state; it restates §7's first clause in code.
  The `Ok(vec![])` behaviour for a nonexistent `--steam-root` (proposal §7:240-242, Test #7,
  steam_linux_tests.rs:454-457) is unaffected, and no serialized type is touched.
- **No serialized surface changes.** `DiscoveredGame` and `Platform` are not in this range; the message
  string is the payload of `DiscoveryError::PluginError`, which reaches a user only through the engine's
  swallowed `eprintln!` on stderr (A14, proposal §7:246-248) and never enters the stdout `--json` stream
  (`apps/discovery-cli/src/main.rs:40-43`). Nothing in the repo parses that string; the only assertion over
  it (steam_linux_tests.rs:471-489) checks "names the root" and "leaks no contents", both of which hold for
  either wording — consistent with the runner's green `cargo test --workspace --locked` leg.
- **Checklist items 2, 3, 4, 5, 6, 7**: not applicable to this range — no `tools/orchestration` path, no
  new path arithmetic, no platform-gated import, no ID/format change, no dependency (the branch adds no
  `Cargo.lock` entry), and no test platform-guard change. Item 1 is not implicated by the hunk: the change
  adds no OS-specific code and the Compatibility matrix is unchanged by it.

So the revision introduces one documentation/contract divergence (finding C1-3 below) and answers none of
this seat's round-1 findings.

## Compatibility checklist walk-through

1. *Platform code matches the Compatibility matrix* — no change in this range; the matrix rows for
   Windows/macOS/Deck ("No") remain satisfied by the plugin having no non-Linux probe path. Not
   re-litigated: no code in this range touches platform policy.
2. *Node floor* — N/A, no JS/TS path in the range.
3. *Paths built by joins* — the range adds no path arithmetic; the existing `join`-based composition and
   canonical path rule are unchanged. See C1-2 for the one separator-class gap carried from round 1.
4. *Guarded platform imports* — N/A (no conditional import in this range).
5. *Locale/layout reality* — no format or ID change in this range; the serialized-spelling item this
   checklist covers is still open as C1-1.
6. *Native dependencies introduced in the diff* — none; `Cargo.lock` is not in the branch diff at all
   (A15, Rollout #3 held). Pass.
7. *Platform-specific tests guarded, not skipped everywhere* — no test in this range changed its platform
   precondition; the §7 test (steam_linux_tests.rs:463-492) is hermetic through `from_roots` + fixtures and
   ran green on the `ubuntu-latest`-style gate the runner captured (A18). Pass.

## Findings

### C1-1 — **major** (carried, still open): `Platform`'s serialized form is re-spelled by this branch, against the design's compatibility statements, with no alias and no schema

- **Location**: `crates/discovery/src/game.rs:7-8` (`#[derive(… Serialize, Deserialize)]` +
  `#[serde(rename_all = "kebab-case")]` on `Platform`), with the coupled expectation at
  `apps/discovery-cli/tests/cli_tests.rs:121`.
- **Status check against the revision**: **not addressed.** The range under review does not touch
  `game.rs`; `git diff main...HEAD -- crates/discovery/src/game.rs` shows this branch added the attribute
  (with a new doc comment at game.rs:5-6 asserting the kebab form is "the … name `Display` below prints and
  ADR-022 documents"), and no `#[serde(alias = …)]` was added for the prior spelling. `apps/discovery-cli/src/main.rs:43`
  serializes the enum through serde (`serde_json::to_string(&all_games)`), so the `--json` channel emits
  `"steam-linux"`/`"steam-windows"` where the base enum's default identifiers gave
  `"SteamLinux"`/`"SteamWindows"`, and `Deserialize` now refuses the old spelling.
- **Still contradicts the current document** (all three statements are present, verbatim, in the
  `02-proposal.md` this pass reads): Context:14-15 "The task is additive: no change to `DiscoveredGame`,
  `Platform`, …"; Non-goals:35 "Any change to `DiscoveredGame`, the trait, …"; §8:262 "The printing code is
  untouched, so the table header and JSON shape are unchanged" — reinforced by Rollout:376-377 "no output
  format, schema or data-model change, so nothing downstream migrates". The Revision log (412-437) records
  no decision authorizing a serialized-form change.
- **Still unpinned at its own layer**: `crates/discovery/tests/game_tests.rs` (1-19) is the data-model test
  and contains no serde assertion; the only place in the repo asserting any serialized spelling of
  `platform` is `cli_tests.rs:121`, written against post-change behaviour. A repo-wide search finds no
  `schemas/` spec or fixture that defines the platform spelling (the `schemas/` "alias" hits are unrelated
  profile/policy prose), so no version umbrella covers the rename, and `docs/adrs/022-game-discovery.md:34`
  documents only the field's kebab *name* in prose, untouched by this branch.
- **Why major, not blocker**: the kebab spelling is the ADR-022-aligned direction and `Display`
  (game.rs:14-21) already printed it, so the human table is unchanged; the design's own compatibility claim
  about the machine-readable channel is the violated requirement, and the blast radius is the `--json`
  handoff §7:242,248 reasons about.
- **Fix** (unchanged from round 1): either drop `#[serde(rename_all = "kebab-case")]` from this additive
  task and assert `"SteamLinux"` at `cli_tests.rs:121`, or make the break deliberate and survivable — keep
  the kebab form, add `#[serde(alias = "SteamLinux")]` / `#[serde(alias = "SteamWindows")]`, pin both
  spellings in a serde round-trip test in `crates/discovery/tests/game_tests.rs`, record the serialized form
  in ADR-022 or a `schemas/` spec, and correct §8:262 (and Context:14-15 / Non-goals:35) in the same change.

### C1-2 — **nit** (carried, still open): `usable_install_dir` still covers one of two separator spellings

- **Location**: `crates/discovery/src/steam_linux.rs:275-282`.
- **Status check**: **not addressed.** The guard now refuses blank, `'\0'`, `.`, `..`, absolute and `contains('/')`;
  a backslash is still not refused (the doc comment at 270-274 was extended to justify the NUL clause instead).
  §5:216-218 and the Risks row at :306 still list the reject set as "absent, empty, `.`, `..`, absolute, or
  contains `/`", so the code-exceeds-spec delta (the NUL refusal) and the spec's reject list remain
  unaligned in both directions. Inert on the only supported platform (`\` is an ordinary filename byte on
  Linux), hence still a nit.
- **Fix**: add `&& !install_dir.contains('\\')` and extend §5's and :306's lists, or state in §5 that the
  emitted `install_path` is a Linux path by contract; either way, align §5 with the NUL clause the code
  already enforces.

### C1-3 — **nit** (new, created by this revision): the §7 message wording the code prints no longer matches §7's stated message contract

- **Location**: `crates/discovery/src/steam_linux.rs:83-88` (message tail reworded to
  `{} entries skipped as unreadable or malformed`) read against proposal §7:244-246, which still says
  `msg` "names the selected roots and the count of skipped **files** and nothing else".
- **Problem**: the reworded string is the §7 message, and §7 is the only description of that string that any
  consumer of the stderr debug line has. The two now disagree on the unit being counted. The code is the
  more truthful of the two — the same counter also absorbs a malformed `libraryfolders.vdf`
  (steam_linux.rs:173-176) and a library whose `steamapps` directory cannot be read (steam_linux.rs:201-206),
  which are not files — so this is not a behaviour defect and nothing needs to migrate: the comment at
  steam_linux.rs:73-76 now says "file or library" while the string says "entries" and §7 says "files".
  No test pins either wording (steam_linux_tests.rs:471-489), which is why the green gate does not see the
  divergence.
- **Fix**: amend §7:246 to "the count of skipped entries (files, libraries or VDFs)" so the document names
  what the code counts — or, if §7's word is the ratified contract, revert the tail to `file(s)`. Pick one;
  the string, the code comment and §7 should be one statement.

## Verdict

`request-changes`: one **major** compatibility finding from round 1 (C1-1, the serialized `Platform` form
against the design's unchanged-JSON-shape statements) is still open and was not the target of this
revision; two nits carried and one new nit is created by the revision's rewording. The revision's own
substance — restating §7's root-selected clause in code — is behaviour-preserving and compatibility-neutral.
