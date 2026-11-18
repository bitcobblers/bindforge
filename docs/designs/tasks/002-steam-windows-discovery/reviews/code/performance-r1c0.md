# Code review — performance seat — round 1, pass c0

- Change under review: `git diff d4242107d82c9cf2ef3750615158165b4260f397...HEAD -- ':(exclude)docs/designs'`
- Inputs used: that diff, `02-proposal.md`, `docs/tasks/T-002-steam-windows-discovery.md`, the runner's gate record (green at 2026-11-18T06:26:23Z). No other review artifacts opened.
- Verdict: **approve** (no blocker or major findings).

## Checklist walk

**1. Event/keystroke path.** Not applicable in the interactive sense: this is a
one-shot CLI batch scan (`discovery list`), and synchronous fs/registry access is
the product. The paths that run per call were checked anyway: plugin
construction does its fs work exactly once (`SteamCore::with_roots` →
`canonical_paths`), `discover()` walks each selected root and library once, and
`SteamWindowsPlugin::from_host()` on a non-Windows host is
`from_roots(Vec::new())` (steam_windows.rs §3 branch) — a Linux host pays zero
new fs or registry calls for the second registered plugin. The generic
`SteamCore<T>`/`SteamTarget` split monomorphizes (`T: Copy`, `&impl SteamTarget`
in `parse_manifest`, `&T` in `scan_library`); no dynamic dispatch or extra
allocation entered the scan loop on the Linux path.

**2. Complexity.** `discover()` is linear in roots + libraries + manifests: VDF
parse once per root, `read_dir` once per library, per-file read under the
pre-existing 1 MiB `metadata`-pre-checked cap. Aggregation dedups through
`BTreeSet` (O(n log n)); the `(slug, install_path)` sort is O(g log g).
`host_candidates` is a fixed ≤5-entry list (3 registry pairs + 2 env vars).
`registry_string_value` decodes one bounded `REG_SZ` per call. `identity`-shaped
lookups (root/library dedup) use the BTreeSet, not nested scans. No O(n²) over
any unbounded dimension was introduced.

**3. Unbounded growth.** No caches or registries added. The plugin holds a
fixed, caller-bounded `roots: Vec<PathBuf>` and is single-use per process;
`host_candidates` is bounded; the test fakes' recorded `calls` vectors are
test-scoped. Nothing to document beyond what §5 already states.

**4. Startup.** `discovery list` on Windows probes exactly 3 registry opens and
2 env lookups, then canonicalizes ≤5 candidates; the no-early-stop policy is the
proposal's explicit §5 choice ("a double-listed root costs nothing", collapsed by
the shared canonical-path rule — exercised by
`from_registry_and_roots_collapses_two_views_of_one_root`). On Linux the new
plugin adds no startup reads. Nothing parses more at boot than the scan's own
required inputs.

**5. Cache invalidation.** Not applicable — the change introduces no cache.

**6. Benchmark/timing evidence.** The diff and proposal contain no
benchmark-like test or timing claim, so there is no number to run and quote.
The gate record is green (`cargo test --workspace --locked` exit 0); its single
captured `test result:` tail line reads `0 passed`, which is only the last
suite's line (consistent with a zero-test doctest binary) — the per-binary
totals for the 20+ new tests were not captured. I attempted to re-run the
workspace gate directly to confirm the new suites execute, but the review
sandbox denied `cargo`; since no performance property in the proposal rests on
a measured claim, this is recorded as an evidence-capture gap of the gate
record, not a finding against the code.

## FYI (no action requested)

- `crates/discovery/src/steam_common.rs:88-90` + `:92-97` (with `discover()` at
  `:88-97`): libraries are canonicalized once inside
  `libraries_in_root_with_skips` and the aggregate is canonicalized again in
  `discover()`, so each VDF-listed library pays a second `canonicalize` +
  `steamapps` `is_dir` stat pass. This double pass is moved *verbatim* from the
  pre-change `steam_linux.rs` (visible in the diff's removed lines), it is
  linear with a practically tiny `n` (libraries per host), and pinning the move
  as behavior-identical is exactly the proposal's §1/Risks constraint — so
  rectifying it belongs to a later change, not this one. Not a finding.

## Notes

- The `\\?\` extended-length paths, per-candidate canonicalize, and 3-key probe
  breadth are all proposal-blessed (§5, §7, Risks) and bounded; nothing here
  changes Linux's observable scan cost.
