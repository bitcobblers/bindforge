# Build notes — design 013 (game manifests) implementation

## What landed

Exactly the proposal's delta inventory (Chosen design → Delta inventory):

- New: `crates/core/src/manifest.rs` (the typed model, the parse-level rules,
  the four budget constants, the fingerprint grammar with an in-tree strict
  standard-base64 decoder, and `load_manifest` with the location rule and the
  registry join), and `schemas/manifests/v1/` — `spec.md` plus the
  `fixtures/valid/` and `fixtures/invalid/` corpora named in the proposal
  (3 valid, 19 invalid).
- Edited: `crates/core/src/document.rs` (the `manifests` arm, `MANIFEST_ENTRIES`,
  `SUPPORTED_FORMATS`, the `format_majors` line, the `serialize` branch, and
  the `is_geometry_name` → `is_slug_name` import/call), `crates/core/src/lib.rs`
  (module, re-exports of `Manifest`, `ManifestAction`, `ExclusivityGroup`,
  `DockedAction`, `load_manifest`, and the four budgets, public-surface doc),
  `crates/core/src/placement.rs` (the predicate renamed `is_slug_name` at its
  two remaining call sites; grammar untouched), `schemas/README.md` (format
  list, the four new budget constants beside the existing exported limits),
  `data/manifests/README.md` (the loader exists; the untrusted-input posture
  paragraph untouched).
- `git status` confirms nothing else changed: no test file, no `Cargo.toml`,
  no `Cargo.lock`, no workflow (std-only, no new dependency, A10).

## Gate evidence (run in the implementation shell)

`cargo fmt --all -- --check` — clean (no output, exit 0), and
`cargo clippy --workspace --all-targets -- -D warnings`:

```
Checking bindforge-core v0.1.0 (/home/mike/repos/bindforge/crates/core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s
```

(the only finding in the first pass, one `clippy::useless_format` in
`manifest.rs`, was fixed rather than waived).

`cargo test --workspace --locked` — every target green; the relevant lines:

```
Running tests/conformance.rs        test result: ok. 9 passed;  0 failed
Running tests/grammar_closure.rs    test result: ok. 11 passed; 0 failed
Running tests/manifest.rs           test result: ok. 49 passed; 0 failed
Running tests/version_gate.rs       test result: ok. 8 passed;  0 failed
Running tests/zz_api_probe.rs       test result: ok. 2 passed;  0 failed
```

with the pre-existing suites (`action`, `action_ids`, `chain_guard`,
`gate_contract`, `input`, `parse_budget`, `placement`, `reach_tables`,
`reach_v2`) all passing unchanged. The `supported_majors`/tree majors
equality case passes untouched, exactly as A2 predicted.

## Deviations from the proposal

1. **Fixture element counts.** `rows-past-cap.toml` carries 260 `actions`
   rows and `presets-past-cap.toml` 260 preset ids, not exactly 257; the cap
   is what the fixture pins and the exact count past it is not part of the
   contract. `name-past-cap.toml` carries a 169-octet name (measured at HEAD;
   any length past the 128 cap pins the same single fault). (The authoring shell intermittently
   refused scripted file generation, so these corpora were written directly
   with headroom over the cap instead of an exact 257.)
2. **`unknown-input-id.toml` places the unknown id in `dead-keys`.** The
   proposal names the fixture but not the entry; `dead-keys` is the entry
   ADR-015's target lookup reads first, and it names the Deferred item (a key
   like `tab` is outside the closed vocabulary) at its most load-bearing.
3. **`spaced-native-names.toml` carries three rows,** not the single
   `"`/`\`/`#`-bearing name the proposal calls out: the extra rows pin
   interior multi-space and parenthesized names, both of which the name
   grammar accepts. Additive; the pinned case is included verbatim.
4. **`string_entry`/`string_value` in `document.rs` became `pub(crate)`** so
   the `manifest` arm reuses them instead of restating them — mechanical,
   inside the file the delta inventory already edits.
5. **The location-rule refusal echoes the directory's last path segment and
   the document's `id`** (e.g. `` `Remnant-2` is not the manifest's id
   `remnant-2` … ``). The proposal forbids echoing a local *path*, which it
   does not — the scratch-path-absence is pinned by the test — and naming the
   two spellings is how a contributor sees the mismatch.

All message *sentences* beyond what the tests pin (they assert the offender
appears, not the wording) are the implementation's, following the 012
precedent's phrasing. No rule, variant, budget value, fixture name, or entry
deviates from the proposal.

## As-built addendum (2026-11-13)

The record above covers the implement stage (commit `500e9e4`); the
"nothing else changed" line holds for that commit alone. The test half of
the delta inventory landed one stage earlier, at test-authoring (commit
`756a40b`). Completing the record for the whole change set
(`git diff fc71ff96...HEAD`, 34 code files outside `docs/designs/`):

- **Tests landed:** `crates/core/tests/manifest.rs` — new, 49 cases pinning
  Test strategy #1–#5; `conformance.rs` — `manifests/v1` added to the
  expected-directory list (the *walk* untouched) plus
  `every_manifest_fixture_is_exercised_by_the_unchanged_walk`, 9 cases;
  `grammar_closure.rs` — `manifests/v1` in the per-format unknown-entry
  loop plus three cases (exactly-defined-entries accept, `axis-kinds`
  refusal, the `is_slug_name` rename's behaviour pinned), 11 cases;
  `version_gate.rs` — the manifests `v2` refusal and the manifests
  `major-not-shipped` fixture case, 8 cases; `zz_api_probe.rs` —
  `the_adr_013_names_resolve_at_the_crate_root`, 2 cases. The gate
  evidence above already shows these final counts; the round-1c0 reviewers
  re-ran all three gate commands on the final tree and got the same
  results.
- **Verified against the tree:** the fixture corpus is exactly the
  proposal's list (3 valid, 19 invalid, each named after its fault);
  `rows-past-cap.toml` carries 260 rows, `presets-past-cap.toml` 260 ids,
  `group-past-member-cap.toml` 17 members, `name-past-cap.toml` a
  169-octet name (Deviation 1 corrected above — the original 140 was a
  miscount). The delta inventory's Unchanged entries (`toml.rs`,
  `error.rs`, `chain.rs`, `action.rs`, `input.rs`, `reach.rs`, every
  pre-existing fixture, `Cargo.toml`, `Cargo.lock`, both workflows) appear
  nowhere in the diff range.
- **Code review (round 1, pass c0):** all five seats — correctness,
  testing, compatibility, security, performance — approved with zero open
  blocker/major. No further candidate round followed, so the open
  minor/nit findings are recorded, not folded in: untested
  `load_manifest` non-manifest-document guard and strict-base64 branches;
  no non-ASCII native-name case in corpus or suite; the decoder
  accepting non-canonical pad bits contrary to its comment; budgets
  applied after array materialization (transient, bounded by the 1 MiB
  budget); a stale test-authoring header in `tests/manifest.rs`; and
  `spec.md`'s "The loader enforces monotonicity forward" sentence, which
  overstates `load_manifest` — the loader enforces only the location rule
  and the registry join, monotonicity being review- and pin-enforced per
  the proposal.
- Deviations 1–5 above re-verified against the diff; no additional
  undeclared rule, variant, budget value, entry, or fixture-name drift
  found. The As-built note in `02-proposal.md` records the same drift
  against the design sections it touches.
