# Code review — testing seat — r1c0

- Run: T-003 polish discovery CLI output (light flow)
- Pass: round 1, c0 (full panel), specialty: testing
- Scope: `git diff 092bdc29a602fd9711bf77eb039e9e1391a50969...HEAD -- ':(exclude)docs/designs'`
  → `apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_tests.rs`,
  `apps/discovery-cli/tests/cli_output_polish_tests.rs` (3 files, +467/−23).
- Inputs used: the diff, `02-proposal.md`, `docs/tasks/T-003-polish-discovery-cli-output.md`,
  the runner's gate record, and read-only inspection of `crates/discovery/src/steam_common.rs`
  (untouched by the diff) and `.github/workflows/check.yml` to ground the failure test's premise.
  No review reports or transcripts opened.

## Gate evidence (supplied by the runner)

Gate at 2026-11-18T12:55:29Z, green, matching this branch's code commit (`72f3b7fc` is
HEAD; the diff contains exactly its three files):

- `npm run check` — exit 0
- `cargo fmt --all -- --check` — exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0
- `cargo test --workspace --locked` — exit 0. The quoted `test result: ok. 0 passed` line
  is only the tail (one suite's summary); see the executed check below for the full picture.

## Checks executed by this seat

- `cargo test -p discovery-cli --locked` — green. All 8 discovery-cli tests pass, including
  all three new pins in `cli_output_polish_tests.rs` (`..._columns_align_under_a_separator`,
  `..._header_block_when_no_games_are_found`, `..._reports_a_plugin_failure_on_one_stderr_line`),
  the updated `test_cli_list_format`, the untouched `test_cli_list_json_format` and
  `test_cli_list_json_empty_for_missing_steam_root`, and both T-002 dual-plugin tests.
  This resolved my one coverage question (whether the gate's "0 passed" tail meant the new
  file was running — it is).

## Test strategy coverage (checklist #1, #2)

The proposal's Test plan is bulleted (not numbered); each bullet maps to a citing test:

1. *Existing `test_cli_list_json_*` + T-002 tests pass unmodified, JSON byte-stable* —
   the diff leaves all these tests byte-unchanged (verified in the diff hunks) and they
   pass in the gate and in my run. `test_cli_list_format` keeps its slug, display-name,
   decoy and exactly-four-row checks (cli_tests.rs:110–137) and its
   `// proves: Test strategy #9 … as polished by T-003 §1` cite.
2. *New alignment test* — `cli_output_polish_tests.rs:88` pins identical `|` byte offsets
   on every line, line 2 as a dashes-and-pipes-only separator whose runs fill exactly the
   columns, exactly header + separator + four rows, per-column width = widest cell
   (header included) in `chars().count()` plus one leading space, and exactly the four
   fixture rows.
3. *New empty test* — `cli_output_polish_tests.rs:167` pins exit 0, empty stderr, exactly
   two lines (header + aligned separator), no body row, no prose copy.
4. *New failure test* — `cli_output_polish_tests.rs:211` pins `ExitCode::FAILURE`
   (`code() == Some(1)`), empty stdout, exactly one stderr line starting
   `error: plugin steam-linux failed: ` and containing `PluginError`, no `panicked at`,
   and no engine `Error from plugin` voice. Premise grounded in the untouched crate:
   `crates/discovery/src/steam_common.rs:101–114` returns `PluginError` when a root is
   selected, zero games are found and ≥1 file is skipped; a non-UTF-8
   `steamapps/libraryfolders.vdf` (bytes `ff fe 00 01`) triggers the skip, and
   `steam_linux.rs:28` names the plugin `steam-linux`. The CLI's `{:?}` on
   `DiscoveryError` renders `PluginError(..)`, matching the assertions.
5. *fmt/clippy gates* — green in the supplied record.

Layers: the proposal promises integration-level CLI pins only (private `render_table` in
`main.rs`); no unit layer was promised, so none is required — **not applicable**.
Revision-round checklist item #3 — not applicable, first pass.

## Determinism and naming (checklist #4, #5)

- No wall-clock, network, live server or ports. The failure test builds a pid-scoped temp
  root (`bindforge-t003-unreadable-root-<pid>`), removes any stale copy up front, and
  cleans up *before* its asserts so an assertion failure cannot leak the tree. Tests pass
  `--steam-root` everywhere, so nothing probes the host. Fixture tree is read-only and
  shared safely. `test_cli_list_format` was already host-free.
- Names describe behavior (`…_columns_align_under_a_separator`,
  `…_header_block_when_no_games_are_found`, `…_reports_a_plugin_failure_on_one_stderr_line`).
  `test_cli_list_format`'s legacy name is required to stay by the task itself.
- The new tests compile for `x86_64-pc-windows-msvc` under CI's compile-only `cargo check`
  (std-only APIs); CI runs tests on ubuntu-latest only, where the pinned plugin name and
  scan behavior hold.

## Findings

- **nit** — `apps/discovery-cli/src/main.rs:80–81`: the JSON serialization `Err` arm
  (stderr line + `ExitCode::FAILURE`) has no pin and cannot be exercised through the CLI's
  surface, since serializing `Vec<DiscoveredGame>` is infallible in scope. This is the
  checklist-#7 category (defensive branch with no test). Fix: acceptable as-is given the
  proposal demands "no panic path remains" and the test plan lists no pin for it; if ever
  made reachable (injection seam), add a pin then. No change requested.
- **nit** — `apps/discovery-cli/tests/cli_tests.rs:95–98`: the header-padding assertion
  (`cell.chars().count() > column.chars().count() + 1`) holds only because, in the current
  fixture, every column's widest cell is a body cell; if a future fixture change made a
  header the widest cell of its column, a correct renderer would fail this assertion. The
  alignment property is already pinned rigorously and data-independently in
  `cli_output_polish_tests.rs`, so a fix is optional: assert padding relative to the
  column widths recomputed from the observed rows rather than a strict `>` against the
  name length.
- **nit** — `apps/discovery-cli/tests/cli_output_polish_tests.rs:263`: the failure test
  hard-codes `error: plugin steam-linux failed:` and the steam-linux root-scan error path,
  so it is Linux-runner-specific. That matches the proposal's own Test-plan wording and
  the existing suite's convention (e.g. `cli_tests.rs:189` pins `SteamLinux`), and CI runs
  tests on ubuntu-latest only, so this is consistency only; a future cross-platform CI
  would need `#[cfg(unix)]` or a platform-derived plugin name.

## FYI (out of specialty, no severity)

Proposal §1 says the separator is "width + 2 dashes per column"; the implementation writes
`width + 1` dashes after each column's `|` (`main.rs:142–149`), which is what actually keeps
every rendered line's `|` byte offsets identical — the binding property in the same sentence.
The tests pin the offsets/gap-fill property (which holds), not the literal dash count, so the
consistent reading wins; the proposal's arithmetic phrasing is the loose side.

## Verdict

approve — zero blocker or major findings; all three Test-plan behavior bullets are pinned
by citing tests, the promised unchanged pins are byte-unchanged and green, and determinism
holds. The three nits require no action to approve.
