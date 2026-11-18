# Code review — testing seat, round 1 dissent pass c2

- Design: `docs/designs/024-game-slug-identity/02-proposal.md` (+ `docs/adrs/024-game-slug-identity.md`)
- Scope reviewed: `git diff 818ca62094aacfed640ec7320fe3d293a235646d..HEAD -- ':(exclude)docs/designs'`
  (11 files: `apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_slug_refusal_tests.rs`,
  `crates/core/src/placement.rs`, `crates/core/tests/placement.rs`,
  `crates/discovery/src/steam_common.rs`, `crates/discovery/tests/collision_tests.rs`,
  `schemas/README.md`, four `schemas/*/v1|v2/spec.md`)
- Pass type: dissent c2 — the range under review is the fix round written against this seat's r1 findings.
- Inputs used: the diff, the proposal, ADR-024, the runner's gate record, my r1 findings, my own r1 report.
  No other `reviews/` file opened.

## Gate evidence

Runner gate at 2026-11-18T19:51:47Z: `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` — all exit 0.
The captured `test result: ok. 0 passed; 0 failed` line is the tail of the last leg
(the three doc-test legs), not the workspace total.

Checks I executed myself, to settle the specific coverage question my r1 F1 raised (do the new
in-source test modules actually compile and run?):

- `cargo test --workspace --locked` → completed, every leg `test result: ok`; the
  `discovery-cli` bin unit-test leg reports `test result: ok. 5 passed; 0 failed`, listing
  `tests::a_valid_row_survives_and_draws_no_refusal` and `tests::valid_rows_survive_while_the_invalid_ones_of_the_same_run_refused`.
- `cargo test -p discovery --locked --lib` → `test result: ok. 2 passed; 0 failed`, listing
  `steam_common::tests::a_name_with_no_in_bound_candidate_grows_skipped_by_its_own_row_count` and
  `steam_common::tests::suffix_assignment_and_shared_names_refuse_nothing_and_count_nothing`.

No required gate failed; no required gate unavailable.

## r1 findings — re-check

### F1 (major) — CLI emit-boundary filter untested. **Resolved.**

`apps/discovery-cli/src/main.rs:115-127` now hoists the filter into
`fn refuse_invalid_slugs(games: &mut Vec<DiscoveredGame>) -> Vec<String>` returning the exact
`error: refused game with invalid slug: <escape_and_truncate(slug)>` lines; `main` only
`eprintln!`s them (main.rs:81-84). `apps/discovery-cli/src/main.rs:206-313` is a `#[cfg(test)] mod
tests` over hand-built rows, matching the fix I prescribed:

- `a_valid_row_survives_and_draws_no_refusal` — valid row survives, no refusal.
- `a_grammar_invalid_row_is_dropped_with_the_named_escaped_line` — `Bad Slug` dropped, line is the
  named prefix plus the value.
- `a_refusal_echo_is_escaped_and_never_carries_a_second_line_or_path` — `evil\nslug` renders as
  `evil\u000aslug` (matches `crates/core/src/error.rs:197-201`, control chars as `\u{:04x}`), and a
  `paths-must-not-leak-here` canary asserts no install path in any line.
- `a_slug_one_octet_past_the_shared_bound_is_refused_not_truncated_into_a_row` — `MAX_SLUG_OCTETS + 1`
  octets, bound leg exercised independently of the grammar leg, echo is the shared renderer's output
  (`escape_and_truncate` cuts at `MESSAGE_VALUE_CHARS = 80` + `…`, so a raw echo would not match).
- `valid_rows_survive_while_the_invalid_ones_of_the_same_run_refused` — mixed run: the two valid rows
  survive in order, three refusals in input order.

The mutation classes I named in r1 (printing to stdout instead of returning the line for stderr,
echoing `install_path`, dropping `escape_and_truncate`, moving the filter behind `render_table` /
`serde_json::to_string`) now fail these tests at the filter itself. Confirmed executed and green
(5 passed) by my own run above. `cli_slug_refusal_tests.rs:20-25` records the split honestly: the
in-source unit half plus this suite's end-to-end half, with the absent injection seam stated
(lines 8-14).

### F2 (minor) — collision-pass `skipped` delta unpinned; header overstated coverage. **Resolved.**

`crates/discovery/src/steam_common.rs:430-508` is a `#[cfg(test)] mod tests` using `super::*`, so the
private `resolve_collisions` (steam_common.rs:278) is pinned without widening the public surface.
`a_name_with_no_in_bound_candidate_grows_skipped_by_its_own_row_count` asserts `skipped == 2` for the
two rows of the refused name whose base sits exactly on `MAX_SLUG_OCTETS` — i.e. the counter grows by
the refused *row* count, not the refused-name count — plus the survivor set and who keeps the base.
`suffix_assignment_and_shared_names_refuse_nothing_and_count_nothing` is the negative leg: suffixed
and shared-name rows count zero and the assignments are pinned per display name. Both run green in my
own `cargo test -p discovery --locked --lib` run. The overclaim at `collision_tests.rs:14-16` is
replaced with text that points at the new in-source module and describes `emission_tests.rs`
accurately (construction-seam refusals). Matches the first branch of the fix I offered.

### F3 (minor) — `SLUG_BOUND` retyped at 128 on a false "no core dependency" premise. **Resolved.**

`apps/discovery-cli/tests/cli_slug_refusal_tests.rs:34-37` is now
`const SLUG_BOUND: usize = bindforge_core::MAX_SLUG_OCTETS;` with the note corrected; the stale
two-sentence "no `bindforge-core` dependency" paragraph is deleted (diff at lines 25-27 of the old
file). The black-box grammar oracle is untouched, as agreed. No literal `128` bound remains in that
suite's assertions.

## Testing checklist walkthrough

1. **Strategy coverage.** Design 024 #4 and #9 — the two clauses this seat found thin in r1 — are now
   pinned at their own seams and cited (`collision_tests.rs:4`, `:14-19`;
   `cli_slug_refusal_tests.rs:5`, `:20-25`; `crates/core/tests/placement.rs:323-333` re-cites the
   README bound bullet's narrower-enforced-scope clause, which `schemas/README.md:131-139` does state,
   so the pin's premise is real). No strategy item lost coverage in this round. See nit T1 for the
   per-test marker convention.
2. **Both layers where promised.** Now present: unit (in-source `main.rs`, `steam_common.rs`) beside
   integration (`cli_slug_refusal_tests.rs`, `collision_tests.rs`, `emission_tests.rs`).
3. **Revision rounds have a catching test.** Both r1 majors/minors above now have the test that would
   have caught them; verified executed, not just present.
4. **Determinism.** The new tests build rows in memory — no wall-clock, no network, no server, no
   ports, no filesystem (the `install_path` value is never opened, it is only a leak canary). The
   unchanged subprocess suites use per-label temp roots with `process::id()`.
5. **Names describe behavior.** `a_slug_one_octet_past_the_shared_bound_is_refused_not_truncated_into_a_row`,
   `a_name_with_no_in_bound_candidate_grows_skipped_by_its_own_row_count`, etc. No `test_step_2` style names.
6. **Gate evidence.** Assessed first; green; I ran only the two cargo test legs needed to confirm the
   new modules execute, and distinguished them above.
7. **Untestable additions.** None: every added production function in range
   (`refuse_invalid_slugs`) has direct tests; the rest of the range is doc/comment text plus one
   re-cited existing pin.

## Open findings

No blockers, no majors. Two nits, neither of which the fix round introduced as a defect:

- **T1 (nit)** — the two new in-source test modules carry no `// proves: design 024's Test strategy
  #N` marker at the test, while the repo convention has ~450 such markers; the citations currently
  live only in the neighbouring suites' module headers.
- **T2 (nit)** — the `main()` call site of `refuse_invalid_slugs` is unpinned. This is the shape this
  seat prescribed in r1 (function-level unit tests plus the existing subprocess half), so it is not
  reopened as a major; recorded so the residual is on the record rather than discovered later.

## Verdict

`approve` — the r1 major and both minors are closed with tests I verified compile, run, and pass; the
remaining items are nits.
