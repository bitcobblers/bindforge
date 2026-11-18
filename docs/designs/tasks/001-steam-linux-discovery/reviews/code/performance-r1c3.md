# Code review — performance seat, round 1, pass c3 (ratification)

- Change under review: `git diff 752fd5abc9e4e88b57a98ef1c23c31daa3983c13..HEAD -- ':(exclude)docs/designs'`
- Scope as given: 3 files, +9/−7 — `apps/discovery-cli/tests/cli_tests.rs`, `crates/discovery/src/game.rs`, `crates/discovery/src/steam_linux.rs`. Commits in range (excluding docs): `77e24c72`, `752096e9`, `c9fb37b7`.
- Design read against: `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md` (full current document) and `docs/tasks/T-001-steam-linux-discovery.md`.
- Pass type: ratification. Task: verify the post-approval revisions did not break the performance properties I approved in round 1. Not a re-review of the whole branch.

## Incremental changes, performance assessment

1. **`crates/discovery/src/game.rs` — `#[serde(rename_all = "kebab-case")]` removed from `Platform`** (revert of the unauthorized rename). Serde container attributes are compile-time derive directives; removing one changes only the generated variant-name literal ("SteamLinux" instead of "steam-linux"). No runtime cost is added or removed on any path. No performance impact.
2. **`apps/discovery-cli/tests/cli_tests.rs` — platform assertion string `"steam-linux"` → `"SteamLinux"`**. Test-only string comparison against parsed JSON. No performance impact.
3. **`crates/discovery/src/steam_linux.rs` — §7 error guard gains `!self.roots.is_empty() &&`** (now `steam_linux.rs:77`). This is a `Vec::is_empty` — an O(1) pointer comparison, evaluated first with `&&` short-circuiting, on the cold post-scan path. It strictly adds no measurable work; if anything it shortens the false-path evaluation when roots are empty. No performance impact. (Whether the clause is logically redundant given `skipped` can only be non-zero after roots are scanned is a correctness-seat question, not a performance one.)

Nothing else is in the incremental diff. No hot path, loop body, filesystem call pattern, allocation, or data structure changed.

## Prior findings — re-check status

Both r1 nits are **unchanged by the revisions**; the only delta is line numbers shifted +2 below the `:70` hunk:

- **Nit (carried, unchanged) — `crates/discovery/src/steam_linux.rs:68`** (rule applied twice per library): `libraries_in_root_with_skips` still ends with `canonical_paths(libraries)` at `:162` (realpath + `steamapps` `is_dir` + BTreeSet sort/de-dupe per root), and `discover()` still runs `canonical_paths` again over the concatenated set at `:68` (`canonical_paths` body at `:322`). Each library pays two `std::fs::canonicalize` calls and two `steamapps` `is_dir` stats per scan. Fix unchanged: since every per-root set is already canonicalized/sorted/de-duped, `discover()` only needs to merge the per-root outputs (e.g. extend a `BTreeSet`, or sort + `dedup` the concatenated `Vec`) instead of re-canonicalizing; behavior of the §2 canonical-path rule is preserved because its output is idempotent.
- **Nit (carried, unchanged) — `crates/discovery/src/steam_linux.rs:218`** (manifest stat'd twice before open): `scan_library` still calls `path.is_file()` at `:218`, then `read_text_file` stats again via `std::fs::metadata` for the size cap at `:291`, then `std::fs::read` at `:299`. Hundreds of appmanifests on a real install still means hundreds of redundant stats. Fix unchanged: one `metadata` call in `scan_library` supplying both the `is_file` test and the `len()` cap (pass `&Metadata` or the size into `read_text_file`), or drop the pre-stat and let the capped `metadata` inside `read_text_file` reject non-files.

Neither nit was addressed by the revisions; neither was made worse. Both remain advisory (per-file redundant syscalls in a scan that is inherently I/O-bound and per-file-bounded), consistent with what I approved in round 1.

## Performance checklist

1. Event/keystroke path — **N/A**: no event loop here; the only path is the synchronous `discovery list` scan, which the proposal deliberately keeps (per-file bounds, §4; wall-clock budget explicitly deferred, *Deferred / out of scope*).
2. Complexity — no new quadratic or repeated-scan patterns in the incremental diff; the two known redundant-work nits above are unchanged.
3. Unbounded growth — no cache, registry, or array growth introduced by the revisions; scan sets are bounded by selected roots and libraries as before.
4. Startup — no boot-path change; the revisions add no file reads or parses.
5. Cache invalidation — **N/A**: no caching exists or was added (proposal: no persistence/caching, Non-goals).
6. Benchmark-like test / timing note — none in the diff or proposal to run; the design defers the performance-budget harness, so there is no timing claim to verify.

## Gate evidence

Runner gate at 2026-11-18T04:41:08.659Z, green: `npm run check` (exit 0), `cargo fmt --all -- --check` (exit 0), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), `cargo test --workspace --locked` (exit 0, `test result: ok. 0 passed; 0 failed; ...` for the captured leg). No performance behavior claim in the incremental diff depends on this evidence; cited only to confirm the revised code compiles and tests pass.

## FYI (out of specialty, no severity)

Reverting the serde rename restores the JSON `platform` value to `SteamLinux` while `Platform::Display` still prints `steam-linux`, so the table and `--json` now spell the platform differently — aligns with the proposal's additive/no-shape-change constraint; compatibility seat's call.

## Verdict

**approve** — the post-approval revisions (serde-attribute revert, test string update, one O(1) guard clause) introduce no performance regression; both r1 performance nits survive unchanged and remain non-blocking.
