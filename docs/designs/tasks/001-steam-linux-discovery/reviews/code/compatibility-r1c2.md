# Code review — compatibility seat, round 1 dissent pass c2

- **Design**: `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md` (read in full at HEAD, 437 lines)
- **Task**: `docs/tasks/T-001-steam-linux-discovery.md`
- **Scope reviewed**: `git diff 77e24c72f9ed0c5347a891337d36029ba9fa4eb6..HEAD -- ':(exclude)docs/designs'`
  → `apps/discovery-cli/tests/cli_tests.rs` (+5/-1), `crates/discovery/src/game.rs` (-3),
  `crates/discovery/src/steam_linux.rs` (+4/-4). Nothing else in scope.
- **Inputs used**: that diff, the current proposal, the task doc, the runner's gate record, my own r1
  findings, and the source/test files the diff's hunks live in (`game.rs`, `main.rs`, `cli_tests.rs`,
  `steam_linux.rs`, `crates/discovery/tests/steam_linux_tests.rs`) plus `docs/adrs/022-game-discovery.md`
  to read the one compatibility claim in the carried major. No other review report or transcript was
  opened, read or grepped for evidence.
- **Gate evidence**: runner gate 2026-11-18T04:41:08.659Z — green; `cargo test --workspace --locked`
  exit 0, `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo fmt --all -- --check`
  exit 0, `npm run check` exit 0.
- **Method note**: I additionally read `git diff main...HEAD -- crates/discovery/src/game.rs
  crates/discovery/tests/game_tests.rs apps/discovery-cli/src/main.rs` to answer my own carried finding,
  whose text was written against the whole-branch diff. For that path pair the range diff is **empty**:
  `crates/discovery/src/game.rs` at HEAD is byte-identical to `main`, and the only in-scope files that
  still differ from `main` are the additive `steam_linux.rs` and the CLI wiring/tests.

## Disposition of this seat's round-1 findings

| r1 finding | c2 status | Why |
|---|---|---|
| **major** — `#[serde(rename_all = "kebab-case")]` on `Platform` re-spells `DiscoveredGame.platform` on the `--json` channel against proposal Context:14-15, Non-goals:35, §8:262 and Rollout:375-376 | **CLOSED — fixed as prescribed (option 1)** | See finding 1 below. |
| **nit** — `usable_install_dir` accepts a backslash; §5:216-218 and Risks:306 and the code disagree about the reject set (NUL refused in code, absent from the doc) | **OPEN, unchanged nit** | Reported again as finding 2. This revision touches neither `usable_install_dir` nor §5. |
| **nit** — §7 error-message tail ("N entries skipped") disagreed with §7:244-246 ("files") and with the code comment | **CLOSED — fixed as prescribed (option 2)** | See finding 3 for the one residual imprecision the chosen option leaves behind. |

### 1. Carried major: closed

`git diff main...HEAD -- crates/discovery/src/game.rs` is empty, and the file at HEAD
(`crates/discovery/src/game.rs:5-9`) carries the bare
`#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]` with no `rename_all` and no
`alias`. So the branch is no longer the author of any wire-format change:

- `apps/discovery-cli/src/main.rs:43` (`serde_json::to_string(&all_games)`) emits
  `"SteamLinux"` / `"SteamWindows"`, exactly as `main` does.
- `Deserialize` accepts exactly what `main` accepted, so nothing previously emitted fails to
  deserialize. Nothing downstream migrates, which is what Rollout:375-377 states.
- The table path (`apps/discovery-cli/src/main.rs:45-53`, printing `game.platform` through the
  untouched `Display` impl at `crates/discovery/src/game.rs:11-18`) is likewise untouched, so
  §8:262's "the table header and JSON shape are unchanged" is now true rather than contradicted, and
  Context:14-15 ("no change to `DiscoveredGame`, `Platform`") and Non-goals:35 hold. No proposal
  amendment was needed under this option, so the Revision log's r0-only content is no longer a gap.
- The coupled expectation moved with the revert, as one statement:
  `apps/discovery-cli/tests/cli_tests.rs:124` now asserts `obj["platform"] == "SteamLinux"`, and the
  runner's green `cargo test --workspace --locked` leg shows the suite passing on that expectation. A
  re-introduction of the derive attribute — or any further rename — now fails that assertion, which is
  the guard the r1 finding asked for at the layer the fix chose.

### 2. Carried nit: still open

`crates/discovery/src/steam_linux.rs:275-282` refuses blank, `'\0'`, `.`, `..`, absolute and
`contains('/')` — and still not `'\'; the doc comment at 270-274 remains the extended NUL rationale.
`02-proposal.md` §5:216-218 and the Risks row at :306 still give the reject set as "absent, empty, `.`,
`..`, absolute, or contains `/`", so the code refuses one thing the spec does not list (NUL) and the
spec's traversal rationale still covers one of two separator spellings. Inert on the only supported
platform — a backslash is an ordinary filename byte on Linux, and the *Compatibility matrix* rows
:315-327 admit no Windows path into this field — so it stays a nit. The incremental diff neither
addressed nor worsened it.

### 3. Carried nit: closed, with one residual

The message tail is back to `{} file(s) skipped as unreadable or malformed`
(`crates/discovery/src/steam_linux.rs:84`) and the code comment at 73-76 now says "at least one file was
skipped", so the printed string, the comment and §7:244-246 are one statement — the option my r1 fix
named ("revert the message tail to `file(s)` if §7's word is the ratified contract"), which also means
nothing about the §7 error surface needs to migrate. Residual: the guard at 77 can be satisfied by
`skipped` increments that are not files — the unreadable-`steamapps`-directory branch at 201-206 and
the unreadable-directory-entry branch at 209-211 increment the same counter with no file skipped — so a
scan whose only skip was a directory prints "1 file(s) skipped". Reported as a nit below, since it is a
wording-only imprecision in the one line the diff reworded and no consumer parses the noun.

## Compatibility checklist walk

1. **Platform code matches the Compatibility matrix** — not re-opened by this diff: none of the three
   changed hunks touches root probing, the candidate list, or the Flatpak/Snap gates. The revert in fact
   removes a place where a non-Linux spelling assumption could have leaked into the machine-readable
   channel. Unchanged from r1.
2. **Node floor** — not applicable: no `tools/orchestration` or JavaScript path in scope; the green
   `npm run check` leg is unaffected by these three files.
3. **Paths built with joins, case-sensitivity assumptions** — unchanged in this diff. Path composition
   stays `PathBuf::join` (`steam_linux.rs:240-243`, `152`, `159`, `201`); no string concatenation was
   introduced. The NUL and separator refusal at 275-282 is finding 2.
4. **Guarded platform imports** — unchanged: no import appears in this diff, and `apps/discovery-cli`
   still imports `SteamLinuxPlugin` unconditionally on a crate that compiles everywhere (Linux-only
   *behaviour*, matching the matrix's "Windows: No" row by absence of a Windows plugin, as at r1).
5. **Locale/layout reality** — unchanged: the revert changes a JSON enum spelling only. Non-ASCII
   `display_name` / `installdir` handling is untouched, and the fixture rows the CLI test asserts
   (`cli_tests.rs:95-115`) are the same four as before.
6. **Native dependencies introduced** — not applicable: this diff adds no dependency, and
   `Cargo.lock`/`Cargo.toml` are not in scope; the green `cargo test --workspace --locked` leg records
   the lock posture holding.
7. **Platform-specific tests guarded, not skipped everywhere** — improved, not regressed:
   `apps/discovery-cli/tests/cli_tests.rs:124` now pins serde's own variant identifier, which is
   platform- and locale-independent, so the `--json` contract test is no longer sensitive to a spelling
   that only some readers expect. `crates/discovery/tests/steam_linux_tests.rs:451-492` stays hermetic
   through `from_roots` + fixtures, so the A18 `ubuntu-latest` CI job runs everything it claims.

## Findings

### Finding 1 — nit — `crates/discovery/src/steam_linux.rs:275-282` read against `02-proposal.md` §5:216-218 and Risks:306
Carried from r1 and untouched by this revision: the `installdir` guard refuses NUL, which neither §5 nor
the Risks row lists, and does not refuse `\`, which the doc's traversal rationale implicitly assumes.
Either direction is defensible; the doc and the code must say the same thing.

**Fix**: add `&& !install_dir.contains('\\')` and extend §5's and :306's reject lists, or state in §5
that the emitted `install_path` is a Linux path by contract; in either case add the NUL clause the code
already enforces to §5:216-218 and Risks:306.

### Finding 2 — nit — `crates/discovery/src/steam_linux.rs:77-88` (reworded clause and comment) read against `02-proposal.md` §7:238-246
Residual of the closed nit: with the message's noun settled to "file(s)", the counter it prints is still
incremented by non-file skips — the unreadable `<library>/steamapps` directory at 201-206 and the
unreadable `read_dir` entry at 209-211 — so a scan whose only skip was a directory prints "1 file(s)
skipped as unreadable or malformed". §7:238-240 itself lists "an unreadable library" as a skip category,
so the counter follows the document; only the printed noun is narrower than what was counted. No test
pins the noun (`crates/discovery/tests/steam_linux_tests.rs:464-492` asserts the message names the root
and leaks no contents), so no consumer can be relying on it either.

**Fix**: keep §7's ratified word and narrow the counting instead — count the directory-level skips into
a separate clause of the message (e.g. "`N file(s), M library(ies) skipped`") and mirror it in
§7:244-246 — or add one word to §7:246 ("skipped entries (files or libraries)") and let the message say
"entries" again. Pick one noun for the string, the 73-76 comment and §7.

## Verdict

**approve.** Zero open blockers and zero open majors: the r1 major is closed exactly as prescribed
(`Platform` at HEAD is identical to `main`, so the `--json` channel carries `SteamLinux` and the
proposal's Context:14-15, Non-goals:35, §8:262 and Rollout:375-377 are all true of the code), and the
remaining two items are nits that change no observable contract. Nothing in the three-file revision
regressed anything this seat approved at r1: no new dependency, no new import, no path-composition
change, no test disabled or platform-skipped, and the runner's gates are green.

## FYI (no severity, not compatibility findings)

- `apps/discovery-cli/tests/cli_tests.rs:121-123` records review-round provenance in shipped test source
  ("the r1 major … was reverted as unauthorized"). The design citation is worth keeping; the round
  numbering and the characterisation are run record, and will read as archaeology to the next
  contributor.
- Pre-existing at `main`, untouched by this branch: nothing in `schemas/` or `crates/discovery`'s own
  tests pins the serialized spelling of `Platform`, and `docs/adrs/022-game-discovery.md:34` gives
  "`steam-linux`, `steam-windows``" as the field's example values while `main`'s JSON emits
  `SteamLinux`. That ADR-vs-wire ambiguity is exactly what let the r1 rename look authorized from inside
  the code; if it is ever resolved, it is a data-model change with its own task, not T-001 churn.
