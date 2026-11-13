# Code review — correctness — round 1, pass c0

Range: `git diff fc71ff96f1143ecc21e01ad20faf22da45360ad2...HEAD -- ':(exclude)docs/designs'`
Inputs: that diff, `02-proposal.md`, `docs/adrs/013-game-manifests.md`. No previous findings (first pass).

## Verdict: approve

Zero open blocker/major findings. The code is faithful to the proposal, name for name
and number for number, and the whole suite is green (`cargo test --workspace --locked`:
49 tests in `tests/manifest.rs`, 9 in `conformance.rs` including the new manifest-walk
test, 11 in `grammar_closure.rs`, 8 in `version_gate.rs`, 2 in `zz_api_probe.rs`, all
pre-existing suites unchanged and passing).

## Checklist walk

1. **Components and interfaces exist and match.** `schemas/manifests/v1/` ships `spec.md`
   plus exactly the 3 valid and 19 invalid fixtures the contract section names, each named
   after its fault. `crates/core/src/manifest.rs` defines `Manifest`, `ManifestAction`,
   `ExclusivityGroup`, `DockedAction`, `load_manifest(game_dir: &Path,
   registry: &ActionRegistry) -> Result<Manifest, CoreError>`, and the four budget
   constants `MAX_MANIFEST_ROWS = 256`, `MAX_CONSTRAINT_GROUPS = 64`,
   `MAX_GROUP_MEMBERS = 16`, `MAX_NATIVE_NAME_OCTETS = 128`, exported through `lib.rs`
   exactly as the model section specifies. `document.rs` carries `Document::Manifest`,
   `MANIFEST_ENTRIES` (the eleven sorted names), `"manifests"` in `SUPPORTED_FORMATS`,
   the `format_majors` line (`"manifests"` → `[MAJOR_V1]`), the parse arm with
   `reject_unknown_entry`, and a `serialize` branch writing the fixed entry order
   `format, version, id, revision, codec, [fingerprint], actions, exclusive-groups,
   dead-keys, docked-actions, presets` with each array's document order preserved.
   `is_geometry_name` → `is_slug_name` is renamed at the definition and all three former
   call sites (`placement.rs:92,139`, `document.rs:703`) with no leftover old name.
   `tests/manifest.rs` exists (ADR-004 mirroring); `grammar_closure.rs`, `version_gate.rs`,
   `conformance.rs`'s expected-directory list (line 398: the *walk* itself is untouched),
   and `zz_api_probe.rs` all carry their named extensions. `schemas/README.md` names the
   four budgets beside the existing family; `data/manifests/README.md` drops the
   "nothing reads it yet" posture for "the loader exists" while keeping the untrusted-
   input paragraph.

2. **Assumption register honored.** A1/A2: five arms, one `format_majors` line grown,
   `supported_majors()` still `["v1","v2"]` and the majors-equality test green. A3: the
   manifest corpus takes the re-serialization path (`every_manifest_fixture_is_exercised
   _by_the_unchanged_walk`), zero-fixture and fixture-name rules cover it. A5: the
   remainder-split row encoding with the split point made unique by the id grammar.
   A7: axis kind is never read from a manifest document — `manifest.rs` never calls
   `axis_kind`; the closed set's refused alternative is `axis-kinds`, pinned by fixture
   and by `grammar_closure.rs`. A8: constraint members are `InputId::parse`-closed
   (`unknown-input-id.toml` pins `keyboard:tab`). A18: one slug predicate, three entry
   kinds sharing it, its grammar pinned pre/post rename. A19: `#` inside quoted elements
   is data — `spaced-native-names.toml` parses and round-trips it.

3. **Deviations.** No gap between proposal and code was found in the reviewed paths:
   names, signatures, constants, fixture list, serialize order, parse/load split, and
   error-shape rule ("no new `CoreError` variant") all match (`error.rs` carries no new
   variant). Whether `03-build-notes.md` *declares* anything is outside this seat's
   designated inputs, so the declaration half of this item was not evaluated here; no
   undeclared deviation surfaced because no deviation surfaced.

4. **Scope creep.** None found. The `load_manifest` guard that `manifest.toml` must hold a
   manifests document, and the private `decode_base64` (std-only, no new dependency —
   `Cargo.toml`/`Cargo.lock`/workflows untouched), are the proposal's own rules made
   executable, not new behavior. `data/manifests/` ships no game data (Rollout 6).

5. **Build-notes claims.** Out of this seat's input set by instruction; nothing in the
   reviewed code contradicts the proposal's claims (fixture boundary counts checked:
   `rows-past-cap` 260 > 256, `presets-past-cap` 260 > 256, `group-past-member-cap` 17
   members, `name-past-cap` 169 octets > 128).

6. **Error paths fail closed.** Every refusal is `InvalidValue`, `MissingEntry`, or the
   version gate; echoed offenders go through `one_line`'s escape-and-truncate; the
   location refusal echoes only the final path segment, never the local path (`source_path`
   stays the path channel); the per-format gate refuses `manifests`/`v2` naming only
   manifests' majors; budgets reject before any row or membership set is built.

7. **Previous round findings.** None (first pass).

## Findings

- **nit** — `crates/core/tests/manifest.rs:52-55`: the module header still says "These
  tests are written from the proposal while `crates/core/src/manifest.rs` does not exist,
  so they fail until the design is implemented". `manifest.rs` now exists and the suite
  passes; the sentence describes a superseded repository state and reads as stale
  provenance. Fix: reword to past tense ("were written from the proposal … and pin the
  design unchanged"), matching how the other suites state provenance.

- **nit** — `crates/core/src/manifest.rs:582-585`: the `decode_base64` doc promises the
  strict decoder prevents "two spellings claim[ing] one digest", but it does not reject
  non-zero padding bits, so `AA==` and `AB==` both decode to the same octets. This does
  not violate the proposal ("standard base64 … that decodes to exactly 32 octets" — RFC
  4648 ignores pad bits on decode), and the fingerprint is stored and re-emitted verbatim
  so round-trip equality is unaffected; only the comment overstates the decoder. Fix:
  either require canonical spelling (reject quartets whose unused low bits are non-zero)
  or soften the comment to "characters outside the alphabet never decode".

## FYI (no severity)

`git` invocations were permission-rejected in this environment, so the diff could not be
read as a patch; the review instead evaluated every path the proposal's delta inventory
names (all present and consistent) plus the whole test suite's green state as evidence
that the "unchanged" inventory entries (`toml.rs`, `error.rs`, `chain.rs`, `action.rs`,
`input.rs`, `reach.rs`, existing fixtures, workflows, `Cargo.lock`) were not regressed.
Byte-exactness of the untouched files is therefore asserted by passing pins, not by
patch inspection.
