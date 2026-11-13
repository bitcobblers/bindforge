# Code review — compatibility seat, round 1 pass c0

Range: `git diff 1af12026893971772c5ac8384ceae348d63cd064...HEAD -- ':(exclude)docs/designs'`
Judged against: `docs/designs/015-transforms-as-preconditioned-policies/02-proposal.md`, `docs/adrs/015-transforms-as-preconditioned-policies.md`.

## Verdict: approve

Zero blocker, zero major findings. All seven compatibility checklist items evaluated; none could not be evaluated.

## Checklist evaluation

1. **Platform code vs the Compatibility matrix — pass.** The diff contains no OS-specific code
   at all: no `cfg(target_os…)`, no OS input API, no filesystem-behavior branches. That is
   exactly what the matrix's Product-artifacts row claims ("touches no OS input API … parsed
   documents in, parsed documents out"). The Linux-CI row is real: `.github/workflows/check.yml`'s
   `rust` job runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D
   warnings`, `cargo test --workspace --locked`; the `crates/*` workspace glob admits
   `crates/transforms` with no CI change, and no CI file was touched. macOS "no (v1)" adds
   nothing. Verified locally: all three gate commands green (all pre-existing suites plus
   `policy.rs` 26, `apply.rs` 27, both `zz_api_probe.rs` targets; 0 ignored tests).

2. **Toolchain floor — pass.** `crates/transforms/Cargo.toml` inherits `rust-version = 1.85` /
   `edition = 2024` from `[workspace.package]` rather than restating a higher floor. Manual scan
   of every new source and test file for APIs newer than 1.85: only long-stable std
   (`BTreeMap`/`BTreeSet`/`VecDeque`, `strip_suffix`, `u32::try_from`, `matches!`, inline format
   args, `SystemTime`, atomics); no let-chains or other post-1.85 features. std-only, and the
   new crate's only dependency is `bindforge-core`, as the matrix's Toolchain-floor row states.

3. **Paths — pass.** `load_policy` slug-checks `id` *before* any filesystem question, so a
   separator, `..`, or an absolute path — which `Path::join` would let replace `dir` outright —
   never reaches the join; the joined component is `format!("{id}.toml")` over a lowercase-ASCII
   slug. The rule is pinned by `a_policy_id_outside_the_slug_grammar_is_refused_before_any_file_is_read`
   and by the core `zz_api_probe.rs` case, and refusals name no local path. Scratch-dir helpers
   join only literal test-authored names. Case-sensitivity assumptions are explicit and enforced
   (see 5).

4. **Guarded platform imports — pass (nothing to guard).** No platform-only module is
   introduced anywhere in the diff.

5. **Locale/layout reality — pass.** Policy ids, file stems, tags, and target inputs are ASCII
   grammars (`[a-z][a-z0-9-]*`, closed input vocabulary); uppercase and non-ASCII spellings refuse
   at parse (`id-uppercase.toml`, `tag-uppercase.toml`) rather than surviving into file names —
   precisely the discipline the matrix's Windows row relies on, keeping a policy directory
   case-unique on a case-insensitive filesystem. `TagPattern::matches` is byte-exact with no case
   folding, preserving 011's posture. Non-QWERTY reality is handled as data: contiguity and
   legality read the loaded grid, and the ANSI-US-only floor (A20) is stated in the matrix and in
   the transforms test-common doc; no code or claim depends on a layout beyond the shipped table.

6. **Native dependencies — pass.** `Cargo.lock`'s diff is exactly one entry, the new workspace
   package with a single dependency edge — matching the matrix's "Cargo.lock gains exactly one
   package" and Test strategy #10. No build scripts, no native tooling, no CI matrix change
   needed or made. `cargo test --workspace --locked` resolves and passes with the lockfile
   unchanged, which pins the claim.

7. **Platform test claims — pass.** The Linux leg (the merge PR's gate) is automated and green —
   verified by running the three gate commands. Test strategy #11's Windows leg is honestly
   scoped in the proposal as a one-time contributor run, not an automated test; the diff adds no
   platform-gated tests and none are silently skipped (`0 ignored` across the workspace). The
   cross-platform bite the Windows row names — slug↔file-stem case collisions — is guarded by the
   pre-existing `fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique` walk,
   which is directory-driven and therefore now covers `schemas/policy/v1/` with no change: its 36
   fixture names are all lowercase dash-joined and case-unique.

## Notes

- The unchanged conformance walk picking up `policy/v1` (A8/A12) was independently confirmed:
  the only harness edit is the expected-directory list, and the walk's zero-fixture, case-unique
  name, round-trip, and one-line-truncating-refusal rules now run over the new corpus —
  `conformance.rs` 9/9 green including `every_invalid_fixture_is_rejected_with_a_structured_error`.
- Version-gate compatibility for old readers holds both directions: pre-015 core refuses
  `format = "policy"` as `UnknownFormat` (fail-closed), and new core's per-format gate refuses a
  `policy` document declaring `v2` naming policy's own majors while `supported_majors()` stays
  unchanged — pinned by the two new `version_gate.rs` cases.

FYI (out of specialty, no severity): `crates/transforms/tests/common/mod.rs` is a new file not
named in the proposal's delta inventory (which listed only `apply.rs` and `zz_api_probe.rs` under
that tests directory); it mirrors the established `tests/common` shape in `crates/core`.
