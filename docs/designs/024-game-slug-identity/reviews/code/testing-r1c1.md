# Code review — testing seat, round 1 dissent pass (c1)

Run: 024-game-slug-identity. Scope: `git diff 0ee2eeeb..HEAD -- ':(exclude)docs/designs'`
(3 commits: `7f616274` docs stage, `f611a604` fix, `818ca620` test), read against the
full current `02-proposal.md` and `docs/adrs/024-game-slug-identity.md`. Inputs: the
diff, the proposal, the ADR, the runner's gate record, my r1 findings. No other
reviews/ or transcripts were opened.

## Gate evidence

Runner gate at 2026-11-18T19:38:10.737Z, green: `npm run check` exit 0, `cargo fmt
--all -- --check` exit 0, `cargo clippy --workspace --all-targets -- -D warnings`
exit 0, `cargo test --workspace --locked` exit 0 (one captured `test result: ok. 0
passed; 0 failed` tail — a zero-unit-test target; consistent with
`apps/discovery-cli/src/main.rs` having no `#[cfg(test)]` module, see F1). No leg
fails; nothing in this pass turns on unexecuted checks. I ran no additional gates —
the open questions below are coverage questions answered by static inspection
(`grep -rn "refused game with invalid slug"` → only `main.rs:99`; `grep -rn
"cfg(test)" apps/discovery-cli crates/discovery/src` → no hits).

## Re-verification of my r1 findings (dissent scope: check these first)

The three commits in this range change `crates/core/src/{document,policy,placement}.rs`,
three `crates/core/tests/*` files, `schemas/README.md` and `schemas/policy/v1/spec.md`.
They implement the load-seam fault-kind split and the README bound rescope — other
seats' r1 findings. None of them touches any location of my three r1 findings, and I
confirm each is unchanged at HEAD.

### F1 — major (open): CLI emit-boundary filter still has zero assertions — r1 finding #1, verbatim unresolved

- Location: `apps/discovery-cli/src/main.rs:88-102` (inline `retain` closure at 89-96,
  `eprintln!("error: refused game with invalid slug: {}", escape_and_truncate(..))`
  at 97-102); promise: Test strategy #9, `02-proposal.md:288-290`.
- Status: not addressed. `grep -rn "refused game with invalid slug"` finds the
  `eprintln` itself and a doc-comment mention at `crates/core/tests/error.rs:87`; no
  test asserts the line at any layer. `grep "cfg(test)"` finds nothing in
  `apps/discovery-cli`; the three subprocess tests in `cli_slug_refusal_tests.rs`
  (lines 138, 181, 233) drive the binary through scratch Steam roots, so every
  invalid row is refused earlier at `steam_common::discovered_game`
  (`crates/discovery/src/steam_common.rs:238-247`) and the main.rs filter never
  executes under test. The suite header (`cli_slug_refusal_tests.rs:5-19`) now
  documents this as deliberate and points the escaped-echo half at `error.rs` — but
  `error.rs` pins `escape_and_truncate` as a renderer (strategy #7), not the CLI's
  named stderr refusal, and #9 remains unamended in the current proposal: "dropped by
  the CLI with one named stderr refusal containing the escaped echo, while valid rows
  still render." Testing-checklist item 3 (each previously-rejected major gets a test
  that would have caught it) is not met: printing to stdout, echoing the install
  path, dropping `escape_and_truncate`, or moving the filter behind rendering would
  each still pass the whole suite, breaking 024-L1's boundary guarantee for
  third-party `dyn DiscoveryPlugin` rows — the only rows this branch exists to catch.
- Fix (unchanged): hoist the filter into a named function in `main.rs` (e.g.
  `fn refuse_invalid_slugs(games: &mut Vec<DiscoveredGame>) -> Vec<String>` returning
  the exact formatted refusal lines) and assert it from a `#[cfg(test)] mod tests`
  inside `main.rs` over hand-built rows — grammar-invalid, one octet past
  `MAX_SLUG_OCTETS`, one valid: valid survives, invalids dropped, each refusal line
  is the named prefix plus the escaped value with no install path. Keep the
  subprocess tests for the end-to-end half.

### F2 — minor (open): collision-pass `skipped` increment remains unasserted — r1 finding #2 unresolved

- Location: `crates/discovery/src/steam_common.rs:323-325`
  (`*skipped += before - games.len()` in private `resolve_collisions`, 278-326);
  `crates/discovery/tests/collision_tests.rs:14-16`.
- Status: not addressed. `resolve_collisions` is still private with no `#[cfg(test)]`
  hook. The refusal test (`collision_tests.rs:158-188`) still keeps two in-bound rows
  alive, so the counter's delta has no observable consequence there: `skipped` is
  read only by the all-rows-refused diagnostic (`steam_common.rs:107-120`), which a
  collision refusal alone can never fire (the byte-order-first name always keeps its
  base). `emission_tests.rs` — which the header now cites as pinning "the counter
  itself" — exercises construction-seam refusals (`discovered_game` → `None`) and
  asserts only the substring "skipped" in the message (lines 149-154, 207-210), never
  a count, and never through the collision pass. Test strategy #4's clause "refused
  and counted" (`02-proposal.md:272`) stays half-pinned for collisions, and the
  header's claim (14-16) overstates it.
- Fix (unchanged): expose `resolve_collisions` (or a thin `pub(crate)` wrapper) under
  `#[cfg(test)]` and unit-test it over a hand-built `Vec<DiscoveredGame>`, asserting
  `skipped` grows by exactly the refused count — or amend the header claim so the run
  record does not assert coverage that does not exist.

### F3 — minor (open): `SLUG_BOUND = 128` retyped on a now-false premise — r1 finding #3 unresolved

- Location: `apps/discovery-cli/tests/cli_slug_refusal_tests.rs:33`, used at 115,
  139-144, 182-183; premise comment at 25-27.
- Status: not addressed. `apps/discovery-cli/Cargo.toml:10` now carries
  `bindforge-core = { workspace = true }` in `[dependencies]`, which Cargo makes
  visible to the package's test targets as-is, so the comment "this crate's tests
  deliberately keep no `bindforge-core` dependency" is false and the bound is
  retyped as a literal where 024-B1 requires it "referenced rather than retyped."
  No Cargo change is even required.
- Fix (unchanged): `const SLUG_BOUND: usize = bindforge_core::MAX_SLUG_OCTETS;` and
  delete the stale two-sentence note at 25-27. The independent `is_slug_shaped`
  black-box oracle may stay.

## The changed code in this range (reviewed for new testing defects)

The three new tests are accepted and well-formed:
- `crates/core/tests/chain_ops.rs:697-713` (`for`-pin head past the bound names the
  bound): cites strategy #6; the "128" assertion cannot be satisfied by the old
  grammar-arm message; the fault kind is pinned because the `parse_refusal` helper
  (chain_ops.rs:311-320) asserts `CoreError::InvalidValue` for every refusal it
  returns; deterministic (in-memory document).
- `crates/core/tests/policy.rs:875-916`: pins kind, bound naming from the exported
  constant, refusal-before-file-read, no path disclosure, message shape.
- `crates/core/tests/placement.rs:333-357`: pins the geometry no-bound as known
  current behaviour, matching `load_geometry` (placement.rs:155-161, grammar-only)
  and the rescoped README bullet and `placement.rs` doc comment; same pinning style
  the proposal sanctions for reserved names (#11).
The README/spec rescope ("the bound stops where a format's spec stops") is consistent
with all three. No wall-clock, network, or port use; names describe behaviour. No new
findings within the testing checklist from this range.

FYI (no severity, out of specialty): the two new core bound-arm messages
(`document.rs` `check_pin`, `policy.rs` `load_policy`) echo the offender whole rather
than through `escape_and_truncate`; the offenders are grammar-valid by arm entry, so
the value is controlled — length-boundedness of those echoes is a security-seat
question, not a testing one.

## Checklist

1 (# citations) — new tests cite their promises; #9 still lacks an assertion for its
stderr-refusal clause → F1. 2 (layers) — unit half of #9 absent → F1. 3 (revision
rounds) — previously-rejected major (F1) has no new test → open. 4 (determinism) —
pass. 5 (names) — pass. 6 (gate evidence) — supplied, green, fresh; no executed
re-runs needed. 7 (dead/untestable additions) — the main.rs filter is dead under test
→ F1; collision counter delta → F2.

## Verdict

request-changes — one open major (F1, unchanged since r1; the fix round did not
address any of this seat's findings), two open minors (F2, F3).
