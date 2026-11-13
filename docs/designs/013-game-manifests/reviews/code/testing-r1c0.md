# Code review — testing seat, round 1 pass c0

Diff: `git diff fc71ff96f1143ecc21e01ad20faf22da45360ad2...HEAD -- ':(exclude)docs/designs'`
Judged against: `docs/designs/013-game-manifests/02-proposal.md`, `docs/adrs/013-game-manifests.md`.
Inputs: the diff, the proposal, the ADR. First testing pass; no prior findings to re-check.

## Checklist evaluation

1. **Test strategy citations — pass.** Every numbered item the 013 proposal's Test
   strategy promises is pinned by a `proves:`-cited test:
   - #1 (row encoding): `tests/manifest.rs:377-539` — split-at-first-space, spaces in
     names, `#`/`"`/`\` data, no-space, empty remainder, trailing space, `\u0007`
     control char, name past 128 octets (plus the exact-cap accept), malformed head.
   - #2 (uniqueness/disjointness): `tests/manifest.rs:620-776` — all eight refusals
     named, plus one-member and MAX_GROUP_MEMBERS+1 groups.
   - #3 (identity/version rules): `tests/manifest.rs:778-936` — id/codec/preset slug
     breaks incl. 64-octet, revision 0/negative/string, fingerprint bad-algorithm/
     non-base64/31-and-16-octet, per-format v2 gate; v2-gate case duplicated in
     `version_gate.rs:212-240`.
   - #4 (load-time validation + budgets): `tests/manifest.rs:938-1174` — unregistered
     id refused then accepted once the registry lists it, `Remnant-2/`+`other/`
     location refusals with no path echo, 257 rows / 257th preset / 65th group.
   - #5 (round-trip): `tests/manifest.rs:1176-1336` — eleven-entry doc, deliberate
     document-order preservation, punctuation-bearing names byte-exact.
   - #6 (integration walk): `tests/conformance.rs` header + the new
     `every_manifest_fixture_is_exercised_by_the_unchanged_walk`; the generic
     walks (`every_valid_fixture_re_serializes…`, single-line-message, zero-fixture,
     case-unique-name rules) all loop `format_dirs()`, so the new corpus is covered.
   - #7 (closed set, untouched formats, rename): `tests/grammar_closure.rs`
     (manifests in the per-format unknown-entry loop, exactly-defined-entries accept,
     `axis-kinds` refusal, rename-behaviour case) and `tests/version_gate.rs`.
   - #8 (platform): not a code test; its Linux leg I ran myself (below). The
     Windows leg is the proposal's contributor task, as written.
2. **Layers — pass.** Unit (`tests/manifest.rs`) and integration
   (`conformance.rs`/`grammar_closure.rs`/`version_gate.rs`/`zz_api_probe.rs`) both
   exist, per the delta inventory.
3. **Revision rounds — pass.** The design r1 major (presets under
   `MAX_MANIFEST_ROWS`, no fifth constant) has `refuses_a_preset_array_past_max_manifest_rows`
   (`tests/manifest.rs:1105-1116`) citing `Revision log r1`, plus the
   `presets-past-cap.toml` fixture.
4. **Determinism — pass.** No wall-clock in any assertion (the pre-existing
   `common::ScratchDir` uses time only to name unique scratch dirs), no network, no
   live server, no ports. All scratch fixtures are in-tree or scratch-built.
5. **Names describe behavior — pass.** `refuses_an_input_id_that_is_both_a_dead_key_and_a_docked_input_naming_the_offender`
   style throughout; no step-named tests.
6. **Gates — green** (this repo's gates are cargo, not npm; the proposal's Test
   strategy #8 names the same three). Tails pasted below.
7. **Untestable additions — findings.** Three reachable branches of the new loader
   and decoder carry no test (findings 1–3 below); nothing shipped is a knob without
   a consumer — every budget constant and public name is exercised by
   `manifest.rs`/`zz_api_probe.rs`, and the fingerprint's grammar-only status is the
   design's own (grammar tests exist; production consumption is ADR-017 by design).

## Findings

- **minor — `crates/core/src/manifest.rs:223-228`: the non-manifest-document branch
  of `load_manifest` is untested.** If `manifest.toml` holds a valid document of a
  *different* format (`format = "profile"`, say), the loader answers
  `InvalidValue("`manifest.toml` must hold a manifests document")` — no test drives
  it, so a regression that dropped the guard (or the wrong-format case falling
  through) would pass. Fix: one case in `tests/manifest.rs`'s load block — write a
  `VALID_MANIFEST`-adjacent profile document into a correctly-named scratch game dir
  and assert the `InvalidValue` refusal naming `format`.
- **minor — `crates/core/src/manifest.rs:586-627`: the hand-rolled strict-base64
  decoder's strictness branches have no test.** The doc-comment promises "the length
  a multiple of four, `=` only at the end of the final quartet … never decodes
  through a permissive fallback", but no test or fixture pins a body whose length is
  not a multiple of 4, one with `=` inside a quartet, or an empty body after
  `sha256-` (lines 588, 597-602, 606-608 are only exercised by accident on the
  happy paths). With no third-party decoder to guard this, the contract claim is
  unpinned. Fix: extend
  `refuses_a_fingerprint_whose_body_is_not_base64_naming_the_offender` with
  `"sha256-A=A="`-style and un-padded-length bodies, or add the equivalent
  assertions beside it.
- **nit — `crates/core/src/manifest.rs:230-235`: the non-UTF-8 directory-segment
  branch is untested.** Reachable on Linux by constructing an `OsString` from
  non-UTF-8 bytes; low value, so a nit rather than a request. Fix if convenient:
  one `load_manifest` case with a non-UTF-8 scratch dir name.
- **nit — `crates/core/tests/manifest.rs:52-55`: stale suite header.** It still
  claims the tests are "written from the proposal while `crates/core/src/manifest.rs`
  does not exist, so they fail until the design is implemented" — false now. Fix:
  drop or rewrite that sentence at the next touch.

## Gate runs (Linux, this machine)

- `cargo fmt --all -- --check` — no output (clean).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — clean.
- `cargo test --workspace --locked` — all green. Tail:

```
running 8 tests
test core_declares_exactly_the_major_versions_present_under_schemas ... ok
test rejects_a_document_declaring_an_unsupported_major_naming_the_supported_majors ... ok
test rejects_a_manifest_document_declaring_a_major_the_manifests_format_does_not_ship ... ok
test rejects_a_document_that_declares_no_version_naming_the_supported_majors ... ok
test the_invalid_corpus_ships_a_fixture_for_a_major_the_format_does_not_ship ... ok
test the_invalid_corpus_ships_a_missing_version_fixture ... ok
test the_invalid_corpus_ships_a_manifest_fixture_for_a_major_the_format_does_not_ship ... ok
test rejects_a_document_declaring_a_major_that_only_another_format_ships ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/zz_api_probe.rs (target/debug/deps/zz_api_probe-7f2ede0)
test the_adr_013_names_resolve_at_the_crate_root ... ok
test the_adr_012_names_resolve_at_the_crate_root ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests bindforge_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Per-suite counts for the touched files: `tests/manifest.rs` 49 passed,
`conformance.rs` 9, `grammar_closure.rs` 11, `version_gate.rs` 8, `zz_api_probe.rs` 2.

## FYI (out of specialty, no severity)

The new `every_manifest_fixture_is_exercised_by_the_unchanged_walk` re-asserts what
the generic conformance walks already cover for the new directory; kept value is the
"refuses at `parse` alone / registry-free corpus" claim, which the generic walks do
not state. Not a request.

## Verdict

approve — zero blocker/major; four test-coverage findings (2 minor, 2 nit) for the
next round's coder to fold in.
