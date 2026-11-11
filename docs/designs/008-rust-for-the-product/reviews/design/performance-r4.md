# performance review r4

verdict: approve

Scope: `docs/designs/008-rust-for-the-product/reviews/design/editorial-r4.diff` (four hunks: Goals bullet, Gates CI paragraph, Risks mitigation cell, Rollout 5 bullet). Inputs: `02-proposal.md`, `docs/adrs/008-rust-for-the-product.md`, the Revision log. Prior performance findings: none (editorial phase runs one language pass per round).

## Findings

None. All four hunks are grammar/voice edits with no change to any performance-relevant behavior, number, or obligation:

- **Goals bullet** — "CI running both toolchains' gates, and a Rust section in `repo-practices`." Parallel-structure rewrite of the previous sentence; the two CI jobs and their costs are unchanged, and the Gates paragraph's "satisfying the ADR's 'CI must run both'" (ADR-008 Consequences: "CI must run both") still holds.
- **Gates paragraph** — "so it commits to the only recurring look at the lockfile:" replaces "so it keeps the only recurring look at the lockfile it commits:". The scan target is still pinned verbatim in the same sentence ("against the committed `Cargo.lock`"), the weekly/manual-dispatch cadence and never-a-PR-check placement are untouched, and the rationale for excluding per-PR advisory scanning from the PR gate (live network advisory DB would make unchanged PRs flaky — a CI-cost argument) survives intact.
- **Risks table** — the mitigation cell flips passive to active voice ("Codec conformance fixtures pin `toml_edit`'s known non-preservation gaps (A21)..."). Same obligation, same timing (`crates/adapters` landing, ADR-012); fixture-checking stays a CI-time cost, not a runtime path.
- **Rollout 5** — added `and` before "tests-under-each-crate". Pure list grammar.

## Checklist coverage (against the diff; unchanged text cited as context)

1. **Input-path latency budget** — not touched by the diff. The design's parse budgets (1 MiB, 16 nesting levels, `MAX_CHAIN_DEPTH = 32`, typed `Cycle`/`ChainTooDeep` instead of recursion, Formats contract "Adversarial guard") remain stated and unmodified.
2. **O(n²) over unbounded n** — nothing in the diff adds or removes structure-cost language; the walk in the conformance harness is CI-time over fixture directories.
3. **Memory growth** — parse/depth budgets bound decoder memory; unchanged by the diff.
4. **Startup cost** — unchanged; gate commands and toolchain pin identical.
5. **Caching invalidation** — the sqlx offline cache stays deferred to ADR-014 with its `--check` honesty job named in Options considered; consistent with ADR-008's Decision clause 2 (hub stack), so not re-opened. `Cargo.lock` + `--locked` determinism untouched.
6. **Research baseline numbers** — the diff changes no number and no expected-performance claim; no gap introduced between design and baseline.

## FYI (no severity)
- The Risks-cell rewrite incidentally reads clearer about who owns the pin (`Codec conformance fixtures` as subject); if anything this is a wording improvement worth keeping, for the editor's record.
