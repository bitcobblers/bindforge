# Code review — correctness seat, round 1, dissent pass c2

- Run: `001-steam-linux-discovery` (task flow, light)
- Seat: correctness (checklist: `references/correctness.md` — faithfulness to the design)
- Scope as instructed: `git diff 77e24c72..HEAD -- ':(exclude)docs/designs'`
  (`apps/discovery-cli/tests/cli_tests.rs`, `crates/discovery/src/game.rs`,
  `crates/discovery/src/steam_linux.rs`), read against the full current
  `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md` and
  `docs/tasks/T-001-steam-linux-discovery.md`.
- Pass type: dissent c2 — scope is first the two findings I carried from r1c0,
  and only defects the changed code makes true.
- Inputs used: the diff, `02-proposal.md`, `T-001-steam-linux-discovery.md`, the
  runner's gate record, my quoted r1c0 findings, plus my own read-only
  `cargo test` runs. No `reviews/` path other than this report was opened.

## Verdict: approve

Zero open blocker and zero open major findings. Two nits, neither blocking.

## Re-check of my r1c0 findings

### 1. MAJOR (r1c0) — unauthorized `#[serde(rename_all = "kebab-case")]` on `Platform` → CLOSED

`752096e9` reverts the attribute and its doc comment; `c9fb37b7` re-pins the
serialized spelling in the CLI test.

- `git diff main..HEAD -- crates/discovery/src/game.rs` is **empty**: `game.rs`
  is byte-identical to main, so `Platform` serializes by serde's default
  variant name (`"SteamLinux"`) exactly as it did before T-001, and `Display`
  still prints `steam-linux` (main's own code, untouched).
- The proposal's three clauses that forbade the change are therefore satisfied
  with the document unamended — Context:14 ("no change to `DiscoveredGame`,
  `Platform`, …"), Non-goals:35, §8:262 ("the printing code is untouched, so
  the table header and JSON shape are unchanged"). The Revision log's
  r0-only entries are now consistent rather than contradictory.
- `apps/discovery-cli/tests/cli_tests.rs:124` now asserts
  `obj["platform"] == "SteamLinux"`, so the test documents main's shape
  instead of depending on the deviation. T-001 is purely additive on the data
  model again, as Context:14 and the task's additive scope require.
- No expectation elsewhere depended on the kebab-case spelling:
  `crates/discovery/tests/{steam_linux,plugin,game}_tests.rs` compare
  `Platform::SteamLinux` as a value, never as a serialized string.
- Evidence, run by me at HEAD (`cargo test -p discovery-cli --test cli_tests
  --locked`): `test result: ok. 3 passed; 0 failed` — including
  `test_cli_list_json_format`, the test that carries the new assertion. Full
  `cargo test --workspace --locked`: legs of 14 / 1 / 1 / 5 / 23 / 0 / 3 pass,
  0 failures across every leg. This also explains the gate record's lone
  captured `test result: ok. 0 passed` line: it is the tail of a zero-test
  (doc-test) leg, not the suite's total.

Checklist #2 (assumption register): A17's "seam is unchanged" holds again.
Checklist #3/#4 (undeclared deviation / scope creep): the one undeclared
deviation I found at r1c0 is gone, so the unverifiable question of whether
`03-build-notes.md` declared it no longer matters.

### 2. NIT (r1c0) — `usable_install_dir` refuses interior NUL beyond §5's set → STILL OPEN, unchanged

`crates/discovery/src/steam_linux.rs:275-282` is not touched by this diff; the
justifying comment at :270-274 is unchanged either. Still harmless hardening
beyond §5's stated rejection set (absent, empty, `.`, `..`, absolute,
contains `/`). Carried as a nit, unchanged.

## New findings from this pass's diff

### NIT — `crates/discovery/src/steam_linux.rs:84` — the skip count is labelled "file(s)" but one of its inputs is a directory

`752096e9` restored §7's wording, changing `{} entries skipped …` to
`{} file(s) skipped …`. The wording now matches §7:246 ("the count of skipped
files") literally, which is the right call. But the counter it prints is not
file-only: `scan_library` at :201-206 adds `1` when
`read_dir(<library>/steamapps)` fails, and `canonical_paths`' gate at :328
accepts a `steamapps` directory that exists but is unreadable (`is_dir()` is
true for mode `000`), so an unreadable *library* is reported as a skipped
*file*. §7's first paragraph lists "an unreadable library" as its own
skip-and-continue case, and §7's trigger clause says "at least one file was
skipped" — so in that one case the count's category, and the trigger's stated
witness, are both slightly wider than the message claims.

Not behavioural: the error still fires, still names the roots, still carries
no contents, and no test asserts the wording
(`steam_linux_tests.rs:460-475` checks only that the message names the root
and carries no file contents). No test or caller parses the string.

Fix: count file skips and directory skips into separate counters and print
"{} file(s)/{} dir(s) skipped", or count only files into the §7 counter and
let an unreadable library be reported separately; the `entries` wording of
`77e24c72` was also acceptable here, so either wording or counting is enough.

## Checklist walk-through

1. **Components/interfaces exist and match** — nothing in this diff adds or
   renames an interface; §1's surface is unchanged by it. Not applicable
   beyond confirming `game.rs` reverted to main.
2. **Assumption register honored** — A17 (unchanged seam, five fields,
   `Platform::SteamLinux`) holds again after the revert. Pass.
3. **Deviations declared** — the one undeclared deviation is reverted, so no
   undeclared gap remains in the paths under review. Pass.
4. **Scope creep** — none: this diff only removes unauthorized code and
   re-pins one assertion; the `--steam-root` seam, `DummyPlugin` removal and
   the §7 path are what §8 and §7 asked for.
5. **Build-notes claims** — not checkable from permitted inputs; no longer
   load-bearing, since the deviation it might have declared is gone. Recorded
   as missing evidence, not a blocker: no required property of the change
   itself depends on it.
6. **Error paths exist and fail closed** — §7's single `Err` path is intact
   and its message now restates §7:244-246 clause-for-clause (roots selected,
   zero games, skipped count, root names, no contents). The one nit above is
   about the count's label, not about the path failing open.
7. **Previous round's blocker/major addressed** — yes: the r1c0 major is
   closed by reverting to main's `game.rs` plus the `SteamLinux` assertion,
   verified green by my own test runs.

## Gate posture

Runner gate at 2026-11-18T04:41:08.659Z: `npm run check`, `cargo fmt --all --
--check`, `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo test --workspace --locked` all exit 0. My independent `cargo test
--workspace --locked` and `cargo test -p discovery-cli --test cli_tests
--locked` at HEAD are green (0 failed in every leg).

## FYI (no severity, out of specialty)

`apps/discovery-cli/tests/cli_tests.rs:121-123` records review-round history
in shipped source ("the r1 major … was reverted as unauthorized"); the
proposal citation in the first two lines is the part that will still read well
in two years.
