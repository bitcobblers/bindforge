# Code testing checklist

1. Every numbered item in the proposal's Test strategy has at least one
   test citing it (`// proves: Test strategy #N`).
2. Both unit and integration layers exist where the proposal promises
   them.
3. Revision rounds: each previously rejected blocker/major now has a test
   that would have caught it.
4. Tests are deterministic: no wall-clock, no real network, no live
   server, no port collisions.
5. Test names describe behavior, not implementation steps.
6. Assess the runner's supplied gate evidence first. Run additional checks
   only to resolve a specific coverage or correctness question, or when the
   evidence is missing or stale for the reviewed change. For TypeScript use
   the affected package's `npm run check`; for Rust use `cargo fmt --all --
   --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
   `cargo test --workspace --locked`. Never run a write formatter as a reviewer.
   Cite the command, result, and relevant output briefly; distinguish supplied
   evidence from checks you executed. A failing required gate blocks approval;
   an unavailable check is missing evidence, not an invented test failure.
7. Untestable additions (config knobs without consumers, dead branches)
   are findings, not passes.
