# Code review — compatibility seat, round 1 correction pass 2 (ratification)

Scope as given: `git diff 39d4fcb589fd14ce2c81ea01572c84dbd4b28d39..HEAD -- ':(exclude)docs/designs'`,
read against the full current `docs/designs/015-transforms-as-preconditioned-policies/02-proposal.md`
and `docs/adrs/015-transforms-as-preconditioned-policies.md`. Files in scope:
`crates/transforms/src/apply.rs`, `crates/transforms/src/lib.rs`,
`crates/transforms/tests/apply.rs`, `schemas/policy/v1/spec.md`.

Prior verdict held no findings, so this pass checks only that the two landed commits
(`fix(transforms): close the duplicate-binding guard gap in the stack`, plus the test-update
commit) did not break what round 1 approved.

## What the revision changes

Two behaviors, both contract-visible:

1. **A fourth entry refusal** — `refuse_entry` (`crates/transforms/src/apply.rs:273-283`,
   `duplicated_action` at `:301-306`) refuses, as `CoreError::InvalidValue { key: "set" }`,
   a base whose `set` binds one action twice; `schemas/policy/v1/spec.md:163-169` states it.
2. **Occupancy asks every holder** — `Layout::holders_of` (`:357-367`) and
   `Stack::foreign_holder` (`:535-549`) replace the first-answer `action_on` read in the
   `relocate-cluster` target check; `spec.md:195-200` states it.

No public API, dependency, loader, core, or CI surface moved: the diff touches no `Cargo.toml`
or `Cargo.lock`, adds no `pub` item (`crates/core/tests/zz_api_probe.rs` and
`crates/transforms/tests/zz_api_probe.rs` are untouched and still pass), and adds no
`CoreError` variant.

## Checklist walk

1. **Platform code vs. the proposal's Compatibility matrix** — no OS-specific code is added;
   the new rules read only parsed values (base `set` list, loaded registry, manifest), so
   no matrix row is contradicted. The matrix's "Third-party reader, non-Rust | yes (data
   only) — `policy/v1/spec.md` + fixtures are the contract, load rules and stack semantics
   included" row is the row this diff actually loads: both new rules are stated in `spec.md`,
   and each is stated with its reason and its cross-reference to `profile/v2`'s
   "duplicates are data" rule, so a non-Rust implementer is told the formats answer
   differently on purpose. Two completeness gaps in that statement are findings 1 and 2.
2. **Toolchain floor** — floor is declared once, `rust-version = "1.85"`, edition 2024
   (`Cargo.toml:16-17`). Everything the diff adds is far below that floor: `BTreeSet::insert`
   returning `bool`, iterator `find`/`filter`/`map`, RPIT-in-inherent-method with an elided
   return lifetime (stable 1.65; and the edition-2024 capture rules are what make
   `holders_of`'s `impl Iterator<Item = &ActionId>` capture `&self`). `cargo check --workspace
   --all-targets` and `cargo clippy --workspace --all-targets -- -D warnings` are clean.
3. **Paths** — the diff adds no path handling. The only filesystem work in the change is in
   the test helpers, untouched: `ScratchDir` still builds its path from
   `std::env::temp_dir().join(format!(...))` with a pid + counter + nanos nonce
   (`crates/transforms/tests/common/mod.rs:96-111`), and the two new labels
   (`relocate-occupied-second-holder`, `entry-duplicate-action`) are lowercase-ASCII,
   hyphen-joined and case-unique — the Windows row of the matrix's discipline holds.
4. **Guarded platform imports** — nothing imported; the crate stays std-only and
   `bindforge-core`-only, which is what the matrix's toolchain row promises.
5. **Locale/layout reality** — the new refusal echoes `action.as_str()`, and `ActionId`'s
   grammar is exactly two segments of `a-z`, `0-9`, `-`, ≤ 63 octets each
   (`crates/core/src/action.rs:38-47`), so the echo is ASCII and bounded, and
   `CoreError`'s Display puts the whole message through `one_line`
   (`crates/core/src/error.rs:146-148,172-186`) — the escape-and-truncate posture (A14)
   survives the new message, exactly as it does for the three older entry refusals. Neither
   new rule consults the grid or the tables, so the "Keyboard geometry | yes (ansi-us only)"
   floor (A20) is neither widened nor relied on: a non-ANSI grid answers these two questions
   identically.
6. **Native dependencies** — none introduced; `Cargo.lock` is outside the diff, consistent
   with the matrix's "gains exactly one package" (the transforms crate, already shipped).
7. **Test platform claims** — the two new tests are plain `#[test]`s with no `cfg` gate and no
   `#[ignore]`, run in the ordinary suite, and read only repo-relative contract files through
   `CARGO_MANIFEST_DIR` and the shared scratch dir — the same shape as every round-1 case, so
   Test strategy #11's "green on the Linux CI leg, suite runs once on a Windows contributor
   machine" claim still covers them. Gate evidence run in-place at this HEAD:
   `cargo test --workspace --locked` — every suite green, `0 ignored` everywhere
   (`crates/transforms/tests/apply.rs`: 29 passed, up 2; `conformance.rs`: 9 passed, walk
   unchanged; core suites unchanged: `policy.rs` 26, `chain_ops.rs` 74, `grammar_closure.rs` 15,
   `version_gate.rs` 11, both `zz_api_probe.rs` 5 + 2). Nothing previously approved turned red:
   the test diff is additive — no round-1 assertion was edited or removed.

Cross-format claims the diff makes about shipped behavior, verified rather than trusted:
`resolve` folds by action in a `BTreeMap` so a duplicated action is last-writer-wins
(`crates/core/src/chain_ops.rs:167-186`, `schemas/profile/v2/spec.md:81-83,175`), and it keeps
two actions that share one input, both appearing in the resolved layout
(`chain_ops.rs:190-199`) — so the doc comments' two premises ("resolve settles a duplicated
action last-writer-wins" and "an input may serve several actions, exactly as `resolve` keeps
them") are true of the shipped format, and `schemas/profile/v2/fixtures/valid/set-twice-last-element.toml`
is the valid fixture the new `duplicated_action` doc cites.

## Findings

### 1. minor — `schemas/policy/v1/spec.md:195-200` (code: `crates/transforms/src/apply.rs:535-549`)

The amended occupancy clause says "every action holding that key is asked, not one answer at a
time", but never says **which** holder the report names when a key has more than one foreign
holder. Core answers with the first foreign holder in source-base element order
(`foreign_holder` = `holders_of(target).find(|h| !matched.contains(*h))`). Before this diff the
rule was a single `action_on` read, so "the first holder in element order" was the only
possible answer and the omission cost nothing; the diff turns it into a selection rule, and
selection rules in this contract are load-bearing — `spec.md:179-181` promises that "the
offender a report names [is] fixed by the data", `apply.rs:29-31` calls the offender part of the
determinism surface, and the matrix's third-party row makes `spec.md` the whole contract. A
conforming non-Rust reader may legitimately name the byte-order-smallest foreign holder and
still claim to implement `policy/v1`. Nothing pins core's choice either: the new test
(`crates/transforms/tests/apply.rs:316-347`) needles only the target key
(`assert_offender_names(offender, &["keyboard:s"], ...)`, `:340`) and never asserts `combat.fire`
as the named holder, and its base has exactly one foreign holder.

Fix: in the `occupied-target` clause state the selection rule in one clause — "the named holder
is the first action, in the source base's `set` element order, that holds the target and sits
outside the match set" — and pin it by adding (or extending a case to) a target held by two
foreign actions with `assert_offender_names(offender, &["keyboard:s", "combat.fire"], ...)`
naming the expected one; mirror the sentence in `foreign_holder`'s doc comment.

### 2. nit — `schemas/policy/v1/spec.md:163-169` (code: `crates/transforms/src/apply.rs:301-306`)

"The refusal names the duplicated action" is ambiguous for a base that duplicates more than one
action, which the new rule makes reachable: core names the first duplicate in `set` element
order, while `crates/transforms/src/lib.rs:37-40` and `apply.rs:29-31` attribute the crate's
determinism to sorted containers. The test
(`crates/transforms/tests/apply.rs:1133-1162`) duplicates exactly one action, so no case pins
the multi-duplicate answer either. Two readers can both conform and disagree about which id the
player is told.

Fix: state the tie-break in the bullet ("the first duplicated id in `set` element order", or
byte-order-smallest if that is what is meant) and add one base duplicating two ids asserting the
named one; keep the code doc and the spec sentence worded alike.

## Out of seat (FYI, no severity)

- The shipped entry-refusal set is now four; the proposal's Chosen design → The stack still
  enumerates three (its revision log records only the r1 compatibility-matrix edits). The
  design doc is outside this diff's scope, so this is a note for the run's record, not a change
  request here.
- `apply` and `resolve` now answer differently for one and the same base document (resolve
  settles a duplicated action last-writer-wins, `apply` refuses it). `spec.md:163-169` explains
  why, so the contract is coherent; ADR-017's CLI should surface the refusal rather than
  normalize the base on the player's behalf.
- The new invariant covers duplicated *actions*; one input carrying several *actions* is still
  read as a first answer by `swap_polarity` (both ids on one key ⇒ the exchange is a no-op
  reported `Applied`) and by `swap_mouse` via `action_on` — acknowledged in the code's own
  comment at `apply.rs:347-349`, and correctness/security territory, not mine.

## Verdict

**approve.** The revisions did not break the round-1 compatibility surface: no platform-specific
code, no toolchain-floor or dependency change, path and case-uniqueness discipline intact,
message-echo posture intact, all gate commands green locally with nothing skipped, and every
previously-pinned assertion still passing. The two findings are contract-completeness gaps in
the sentences this diff added, not behavior that regresses the approved design.
