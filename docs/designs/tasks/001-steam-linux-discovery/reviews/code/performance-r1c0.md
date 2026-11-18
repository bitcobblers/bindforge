# Code review — performance seat, round 1 pass c0

- Change under review: `git diff 4caa688933b2969f569f6126dddb713180c7c410...HEAD -- ':(exclude)docs/designs'` (source and test paths: `crates/discovery/src/{keyvalues.rs,steam_linux.rs,lib.rs,game.rs}`, `crates/discovery/tests/*`, `apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_tests.rs`, plus harness files noted in the FYI).
- Inputs used: the diff, `02-proposal.md`, `docs/tasks/T-001-steam-linux-discovery.md`, the runner's gate record, and my own read-only check runs. No other review reports or transcripts opened.

## Checklist walkthrough

1. **Event/keystroke path** — N/A as such: the change is a one-shot CLI scan where synchronous fs is inherent, not a keystroke/event path. Within that path the parser is single-pass over bytes, `Section::find` is a linear scan only over a small file's children, and allocations are one `Vec<u8>`/`String` per token — proportionate to files the design caps at 1 MiB and which in practice run to tens of lines. Two small redundant-work items are filed as nits (findings 1–2).
2. **Complexity** — pass. No O(n²) on unbounded input: per file the reader is O(bytes); per scan the work is O(roots + libraries + manifests); the `apps` maps of real `libraryfolders.vdf` files (hundreds of entries) are walked once, and only numeric-keyed blocks are probed (`steam_linux.rs:184-197`). Sorting is O(g log g) on games (`steam_linux.rs:91`). `BTreeSet` is used exactly where sort+de-dup by identity was intended (`steam_linux.rs:320-332`).
3. **Unbounded growth** — N/A/pass. No caches, registries or growing collections are introduced; the plugin holds only its canonicalized `roots`, and the §7 error message lists the selected roots and a count only.
4. **Startup** — pass. `from_host()` probes the fixed 10-candidate list (realpath + one `steamapps` stat each, failing fast on absent paths); `discover()` reads each root's VDF at one fixed path and enumerates `steamapps/` non-recursively, so boot reads only what the listing needs and never opens the compatdata look-alikes.
5. **Cache invalidation** — N/A. There is no cache (persistence/caching is an explicit proposal non-goal).
6. **Benchmark-like checks** — none exist to run: the proposal deliberately bounds work per file rather than wall-clock (q17, *Deferred* "Scan-latency budgets and a benchmark harness"), and no timing test is in the diff. As a sanity run I executed the gates myself: `cargo test --workspace --locked` is green — `steam_linux_tests` "test result: ok. 23 passed; 0 failed; … finished in 0.00s", `keyvalues_tests` green, `cli_tests` "3 passed … finished in 0.00s". The runner's gate `0 passed` tail is the final (empty doctest) leg of an exit-0 run, not an empty suite. `discovery list --steam-root <minimal fixture>` returns the four rows immediately.

## Findings

1. **[nit] `crates/discovery/src/steam_linux.rs:66,68` (with `:160`) — every library path is canonicalized and `steamapps`-gated twice per scan.** `discover()` extends `libraries` from `libraries_in_root_with_skips`, which already runs the full canonical path rule (`canonical_paths` at `:160`: `realpath` + `is_dir` + sort + de-dupe per root), then runs `canonical_paths` again over the concatenation. Each library therefore pays two `realpath` calls and two `steamapps` stats every scan. Impact is small (single-digit libraries per host), so this is polish, not a defect. Fix: on the `discover()` path collect raw root+`path` candidates from the VDF and apply the canonical path rule once at the outer level; keep `libraries_in_root` itself applying the rule for its public/test contract.
2. **[nit] `crates/discovery/src/steam_linux.rs:216` + `:289` — two stats per manifest before the open.** `scan_library` calls `path.is_file()` (stat), then `read_text_file` calls `std::fs::metadata(path)` for the size cap (stat again), then `std::fs::read` (open). Fix: use `entry.file_type()` (served from `d_type` without a syscall on ordinary Linux filesystems) for the directory-entry check and pass `entry.metadata().len()` into `read_text_file` for the size cap — one fewer `stat` per manifest, ~hundreds saved on a large real library.

No blocker or major findings.

## Strengths (performance-relevant)

- The 1 MiB cap is applied to `metadata().len()` *before* the file is allocated into memory (`steam_linux.rs:287-297`), and the depth cap prevents runaway recursion — work is bounded per file as the proposal promises.
- No recursion below `steamapps/` and one fixed VDF path per root: the scan cost never touches the compatdata trees, and canonicalize+de-dupe guarantees a shared library is scanned once.
- The CLI test rewrite replaces `cargo run` subprocess launches (workspace lock + compile per test) with `CARGO_BIN_EXE_*` direct execution, cutting test-suite wall time; both test binaries finish in 0.00 s here.

## FYI (out of specialty, no severity)

The diff also contains changes outside the proposal's stated scope: `.opencode/agents/code/{coder,test-engineer}.md` (transcript access flipped allow→deny) and `tools/orchestration/src/pipeline/flows.ts` (light-flow `stageTimeoutMs` 15→12 min) — harness configuration, not product performance; flagging only that the proposal (§10 "no change beyond the listed files") does not mention them.

## Verdict

approve (0 blockers, 0 majors, 2 nits).
