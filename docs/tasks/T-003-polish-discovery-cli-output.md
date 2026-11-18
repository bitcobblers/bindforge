---
doc_type: task
status: completed
date: 2026-11-18
type: enhancement
functional_area: application
associated_adr: ADR-022
depends_on: [T-001-steam-linux-discovery, T-002-steam-windows-discovery]
flow: light
---

# T-003: Polish discovery CLI output formatting

`apps/discovery-cli` already implements the ADR-022 bootstrap contract:
`discovery list` prints a pipe-delimited table of discovered games and
`discovery list --json` prints a JSON array of `DiscoveredGame` objects that
piping tools can consume. The output shape is right but rough:

- The table is not aligned. Each row is `| a | b | c | d |` with unpadded
  cells, so columns do not line up vertically across rows and the header is
  not separated from the body.
- On a plugin error the CLI panics: `engine.discover_all().unwrap()` dumps a
  `main.rs` panic message instead of a clean error and exit code.

**Changes (presentation only, in `apps/discovery-cli`):**

1. Align the default table. Column width is the maximum cell display width of
   header and rows in that column; cells are left-padded inside the existing
   `| … |` frame; a header separator row (e.g. `|---|`) sits under the header.
   Column set and order stay Slug, Display Name, Install Path, Platform.
   Empty results print the header only (no "(no games found)" copy unless the
   proposal decides otherwise).
2. Keep the `--json` output contract byte-stable: a single JSON array on
   stdout whose keys and value spellings are exactly today's
   (`slug`, `display_name`, `install_path`, `platform` spelled `SteamLinux`,
   `codec_id`). Whether it stays compact or becomes
   `serde_json::to_string_pretty` is a design choice; it must remain a
   single valid JSON document parseable by `jq` either way. No other flags
   (`--jsonl`, colors, etc.) are in scope.
3. Replace the `unwrap()` on `discover_all()` with an error path: print a
   one-line message naming the failed plugin to stderr and exit non-zero.
   No panic text on any in-scope failure.

**Non-goals:** no changes to `crates/discovery` behavior or to
`DiscoveredGame`'s serde shape; the kebab-case table tag vs PascalCase JSON
spelling split is deliberate (T-001 as-built record) and stays.

Done when:
- `discovery list` renders an aligned table with a header separator; tests in
  `apps/discovery-cli/tests/` pin column alignment (all rendered rows share
  identical `|` offsets) and the header-only empty case.
- The existing `test_cli_list_json_*` and `test_cli_list_format` pins still
  pass; if the format test's exact-string expectation must change for
  padding, update it to assert alignment rather than the old unpadded
  strings, keeping the JSON expectation byte-identical.
- A forced plugin failure (injection seam or `--steam-root` on a path whose
  scan errors) yields a stderr message and non-zero exit, pinned by a test;
  no panic output.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
  -- -D warnings`, and `cargo test --workspace --locked` pass.
