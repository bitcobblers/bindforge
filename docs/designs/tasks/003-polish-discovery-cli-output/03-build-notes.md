# Build notes — T-003 polish discovery CLI output

## Progress

- **Done:** all three proposal increments are implemented and the whole cargo
  gate is green in this worktree.
  1. Aligned table: private `render_table` / `row_cells` / `column_widths` /
     `render_separator` / `render_row` in `apps/discovery-cli/src/main.rs`. Column width = max `chars().count()` of
     the header cell and that column's body cells; each row renders per column
     `| ` + cell padded right to the width, closed by a final `|`; a dash
     separator sits under the header and fills the same column gaps, so all
     rendered lines share identical `|` byte offsets. Empty results print the
     header block only.
  2. JSON unchanged: still one compact `serde_json::to_string` document; its
     `unwrap()` is now a one-line stderr message + `ExitCode::FAILURE`.
  3. Clean error path: `main() -> ExitCode`; the CLI holds its own
     `Vec<Box<dyn DiscoveryPlugin>>` (same `from_host` / cfg-gated `from_roots`
     registration, aggregation appends in registration order) and no longer
     routes through `DiscoveryEngine`, so the first `plugin.discover()` `Err`
     prints `error: plugin <name> failed: <DiscoveryError as Debug>` to stderr
     and returns `ExitCode::FAILURE`. `crates/discovery` untouched.
- **Tests (light flow — this stage owns them):**
  - `apps/discovery-cli/tests/cli_tests.rs`: `test_cli_list_format` now asserts
    the padded header (the old unpadded `| Slug | … |` string must be *absent*),
    per-column header padding, shared `|` offsets across all six lines, and
    keeps the slug / display-name / decoy / exactly-four-rows checks.
    `test_cli_list_json_*` are byte-unchanged.
  - `apps/discovery-cli/tests/cli_output_polish_tests.rs` (new): alignment
    (identical `|` offsets on every line, line 2 = separator whose dash runs
    fill each column, column gap = widest cell + one leading space, exactly
    header + separator + four rows); header-only empty table for
    `/bindforge-no-such-steam-root` (2 lines, empty stderr, no prose copy);
    forced failure via a temp root whose `steamapps/libraryfolders.vdf` is
    non-UTF-8 → exit code 1, stdout empty, exactly one stderr line
    `error: plugin steam-linux failed: PluginError(…)`, no `panicked at`, no
    engine `Error from plugin` line. The temp tree is created and removed by
    the test itself.
- **Remaining work:** none for this stage.
- **Relevant paths:** `apps/discovery-cli/src/main.rs`,
  `apps/discovery-cli/tests/cli_tests.rs`,
  `apps/discovery-cli/tests/cli_output_polish_tests.rs`.
- **Interface decisions:** `render_table(&[DiscoveredGame]) -> String` is
  private to `main.rs` (no new public API, `crates/discovery` untouched); the
  plugin name in the error line comes from `DiscoveryPlugin::name()`.
- **Last validation:** `cargo fmt --all -- --check` (clean), `cargo clippy
  --workspace --all-targets -- -D warnings` (clean, no warnings emitted),
  `cargo test --workspace --locked` (exit 0; discovery-cli 8 tests pass across
  three integration binaries, every other crate's suites still green).
- **Next concrete action:** none — hand back to the runner for its gate and
  commit.

## Deviations from the proposal

- **Separator dash count is `width + 1`, not `width + 2`, per column.** The
  proposal's two §1 clauses are mutually exclusive as literally written: with
  rows rendered per column as `| ` + cell padded to `width` (which the proposal
  states explicitly, and which the Test-plan alignment test depends on), the gap
  between two consecutive `|` delimiters is `width + 1` characters, so a
  `width + 2`-dash run per column cannot keep the delimiters at the same byte
  offsets. The proposal's stated invariant — "every rendered line shares
  identical `|` byte offsets" — is what the Test plan pins, so the separator
  fills exactly the gap (`|` + `width + 1` dashes per column, a `width + 2`
  character segment counting its leading delimiter). Rendered shape for the
  fixture: `| Slug                    | Display Name    | … |` over
  `|-------------------------|-----------------| … |`. The alignment test pins
  the invariant and that each dash run fills its column, not a literal dash
  count.
- **JSON serialization failure text** is `error: failed to serialize games:
  <serde_json::Error>` — the proposal fixed only "one-line stderr message +
  non-zero exit" for this path, not its wording.
- **Test helpers duplicated** (`cells`, `pipe_offsets`) in the two integration
  files rather than shared through a `tests/common` module, to keep the
  proposal's "Files to touch" list exact.
- The failure test names `steam-linux` unconditionally, matching the proposal
  and the pre-existing `cli_tests.rs` suite, which already pins
  `platform == "SteamLinux"` without a cfg guard.
- No gate record was supplied by the runner for this run, so no "Gate posture"
  section is required; the legs below were executed here.

## Gates run in this worktree

- `cargo fmt --all -- --check` — no output, clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — finished, no
  warnings.
- `cargo test --workspace --locked` — exit 0. `discovery-cli`: `cli_tests.rs`
  3 passed, `cli_dual_plugin_tests.rs` 2 passed, `cli_output_polish_tests.rs`
  3 passed; `crates/discovery` and the core/transforms suites unchanged and
  green.
- `npm ci && npm run check` in `tools/orchestration` was not run: this change
  touches no orchestration or schema file.

## Operator follow-up (post-run, 2026-11-18)

Two defects were raised against the shipped output and fixed directly on this
branch by the operator (outside the runner):

1. **Cells sat flush against the frame.** The as-shipped row rendered per
   column as `| ` + cell padded to `width` + `|`, so any cell as wide as its
   column abutted the delimiter (`| Slug|`, `| steam-linux|`). `render_row`
   now writes `| ` + cell padded to `width` + ` |` and `render_separator`
   fills `width + 2` dashes per column; the shared-`|`-offsets invariant is
   unchanged. The alignment test's column formula moved from
   `gap = widest + 1` to `gap = widest + 2`, and new assertions pin that every
   header and body cell is framed by one space on each side. This supersedes
   the "Separator dash count is `width + 1`" deviation above.
2. **`discovery --help` did not list `--json`.** The flag lived on the `list`
   subcommand, so only `list --help` showed it. It is now a clap
   `global = true` flag on the root `Cli`: listed by `discovery --help` and by
   `discovery list --help`, accepted before or after the subcommand
   (`discovery --json list …`). The JSON document itself is byte-unchanged.

New test: `test_help_lists_the_json_flag_at_every_level`
(`cli_output_polish_tests.rs`) pins both help levels and the pre-subcommand
position. Gates re-run after the fix: `cargo fmt --all -- --check` clean,
`cargo clippy --workspace --all-targets -- -D warnings` clean,
`cargo test --workspace --locked` exit 0 (discovery-cli now 9 tests across
three integration binaries: `cli_tests` 3, `cli_dual_plugin_tests` 2,
`cli_output_polish_tests` 4).
