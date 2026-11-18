# Code review — performance seat, round 1 pass c5 (ratification)

## Scope and inputs

- Pass: ratification (c5). I approved round 1 with no open findings on this
  seat; this pass checks only that the revisions landed after that approval
  did not break the performance properties I approved.
- Diff: `git diff 0ee2eeeb7992f470561763e45e56261f503e64ae..HEAD -- ':(exclude)docs/designs'`
  (17 files, +713/−69). Read against `02-proposal.md` (including its
  Revision-log "Escalation repair (after code r1c4)" entry, which is what this
  diff implements) and `docs/adrs/024-game-slug-identity.md`.
- Gate record (runner, 2026-11-18T21:37:49.945Z, green): `npm run check`,
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked` all exit 0.
- Checklist: `references/performance.md` only.

## What the post-approval revisions are, performance-wise

The escalation repair is: (1) the CLI's inline emit-boundary filter extracted
into a named `refuse_invalid_slugs` with an in-binary test module; (2) the
grammar-vs-bound fault-kind split in `load_policy`, `check_pin` (document.rs)
and the new `load_geometry` bound check; (3) the gate-pinned corpus test suite
`crates/core/tests/ledger_annotations.rs`; (4) new boundary tests in
`chain_ops.rs`, `placement.rs`, `policy.rs`; (5) in-source unit tests for the
private collision pass in `steam_common.rs`; (6) doc-comment, spec and README
wording amendments plus the `013-L5` ledger line; (7) test-side re-pins.

## Checklist walk

1. **Event/keystroke path — no hot path touched.** The CLI extraction is
   behaviour-preserving: one `retain` pass over the run's rows, an `O(|slug|)`
   `is_slug_name` plus an `O(1)` `len()` check per row, and the
   `format!`/`escape_and_truncate` allocation only on the invalid branch —
   the same work the inline version did (it cloned the slug and escaped after
   the retain; now the escape happens in-closure on the same invalid rows
   only). No new allocation on the valid-row path; no fs or network added to
   any emit path.
2. **Complexity — no new superlinearity.** `load_geometry` gains a single
   `name.len() > MAX_SLUG_OCTETS` compare (`O(1)`, before the join, so the
   refused name never reaches `parse_file`); `load_policy` and `check_pin`
   split the pre-existing `!is_slug_name(id) || id.len() > MAX_SLUG_OCTETS`
   condition into two sequential `if`s — identical total work, different fault
   naming. The `resolve_collisions` pass itself is untouched by this diff (the
   diff only adds a `#[cfg(test)]` module beside it), so the per-plugin
   claimed-slug-set probing I approved in round 1 is unchanged.
3. **Unbounded growth — nothing new.** `refuse_invalid_slugs` returns a
   `Vec<String>` bounded by the run's row count (as before); no new cache or
   registry. `ledger_annotations.rs` allocates sentence chunks over checked-in
   corpus files, but only inside `cargo test`.
4. **Startup — boot reads unchanged.** The added checks are pure string
   predicates before existing file joins; `ledger_annotations.rs` runs only in
   the test suite and reads a handful of markdown files per test run — test
   cost, not boot cost.
5. **Cache invalidation — N/A.** This change ships no caches.
6. **Benchmark-like claims — none added by these revisions.** The proposal's
   collision-probe cost claim ("no measurable cost today; no baseline exists;
   none claimed") is unchanged and its subject (the collision pass) is not
   touched by this diff beyond added tests; nothing here to run.

## Cross-checks

- Test-side only: `SLUG_BOUND` in `cli_slug_refusal_tests.rs` now references
  the exported constant (the crate already depends on `bindforge-core`);
  negligible compile note, no runtime surface.
- Doc/spec/README/ledger edits carry no code path.
- The gate record is green across all four legs, including the workspace test
  suite that now carries the new `ledger_annotations` gates and the
  in-binary CLI filter tests — the extraction's behavior pinning is enforced.

## Findings

None. The post-approval revisions preserve the asymptotics and allocation
behavior of everything approved in round 1; the only added runtime work in
product code is one `O(1)` length compare beside each existing grammar check.

## Verdict

approve — zero open blocker/major findings on this seat.
