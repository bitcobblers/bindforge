# Plan

## Problem

`apps/discovery-cli` prints the ADR-022 table unaligned: cells are unpadded,
columns do not line up vertically, and no header separator exists. Plugin
failures also never reach a clean error path: `main.rs` wraps
`engine.discover_all().unwrap()`, while `DiscoveryEngine::discover_all`
swallows plugin errors (prints `Error from plugin …` and returns `Ok`), so
the CLI can neither exit non-zero on a plugin failure nor name the failed
plugin in its own voice. The `--json` shape must stay byte-stable.

## Chosen approach

Presentation-only, confined to `apps/discovery-cli`:

1. **Aligned table** — new private `render_table(&[DiscoveredGame]) -> String`
   in `main.rs`. Columns stay Slug, Display Name, Install Path, Platform.
   Column width = max of `chars().count()` over the header cell and that
   column's body cells. Each row renders, per column, `| ` + cell padded right
   to the column width; a separator row of `width + 2` dashes per column sits
   under the header, so every rendered line shares identical `|` byte
   offsets. Empty results print the header block (header + separator) only —
   no "(no games found)" copy.
2. **JSON unchanged** — keep compact `serde_json::to_string` (byte-stable;
   existing pins stay green, still parseable by `jq`). Its `unwrap()` also
   becomes a one-line stderr message + non-zero exit, so no panic path
   remains for any in-scope failure.
3. **Clean error path** — the CLI stops routing through `DiscoveryEngine`
   (it hides plugin errors behind `Ok`) and holds its own
   `Vec<Box<dyn DiscoveryPlugin>>`, registering the same plugins identically
   (`from_host` vs cfg-gated `from_roots`); aggregation appends in
   registration order, matching the engine. `fn main() -> ExitCode`; on the
   first `plugin.discover()` `Err`, print one line
   `error: plugin <name> failed: <DiscoveryError as Debug>` to stderr and
   return `ExitCode::FAILURE`. `crates/discovery` is untouched.

## Files to touch

- `apps/discovery-cli/src/main.rs` — plugin loop + `ExitCode` error path;
  `render_table`; compact JSON kept; no `unwrap()` on in-scope failures.
- `apps/discovery-cli/tests/cli_tests.rs` — `test_cli_list_format`: replace
  the exact unpadded header string with padded-header/alignment assertions;
  keep slug, display-name, decoy and exactly-four-row checks; the
  `test_cli_list_json_*` tests stay byte-unchanged.
- `apps/discovery-cli/tests/cli_output_polish_tests.rs` (new) — alignment,
  header-only-empty and forced-failure pins listed below.

## Test plan

`cargo test --workspace --locked` must show:

- Existing `test_cli_list_json_*` and the T-002 dual-plugin tests pass
  unmodified: JSON keys and spellings (`SteamLinux`, null `codec_id`)
  byte-stable; missing root still prints `[]` with empty stderr.
- New alignment test: for `list --steam-root <fixture>`, every stdout line
  has the identical list of `|` byte offsets; line 2 is the `|---|…|`
  separator; exactly header + separator + four fixture rows.
- New empty test: `list --steam-root /bindforge-no-such-steam-root` prints
  exactly header + separator and no body row.
- New failure test: a temp root whose `steamapps/libraryfolders.vdf` is
  non-UTF-8 makes `discover()` return `PluginError` (the §7 error path);
  `list --steam-root <tmp>` exits non-zero, prints one stderr line naming
  `steam-linux`, leaves stdout empty, and emits no `panicked at` text.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets
  -- -D warnings` pass.

## Done criteria

- `discovery list` renders an aligned table with a header separator;
  `--json` stays a single compact JSON document with byte-identical pins.
- A forced plugin failure yields a one-line stderr message naming the plugin
  and a non-zero exit, pinned by a test, with no panic output.
- The gates pass: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace --locked`.
