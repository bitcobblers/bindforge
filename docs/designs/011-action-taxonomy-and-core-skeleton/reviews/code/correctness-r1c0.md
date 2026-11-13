# Code review — correctness (round 1, seat c0)

Diff reviewed: `4aeb57c2…HEAD`, `':(exclude)docs/designs'` (30 files, +1860/−37).
Inputs: that diff, `02-proposal.md`, `docs/adrs/011-action-taxonomy-and-core-skeleton.md`,
`03-build-notes.md` (for the deviation/claims checklist rows). No other seats' reports read.
This seat's first pass; checklist item 7 (prior-round findings) is vacuous.

## Checklist

1. **Components and interfaces named in the proposal exist, wired, with the named names.** ✓
   `crates/core/src/action.rs`: `AxisKind` (three values, `name()`/closed `from_name()`
   mirroring `Hand`), `ActionId::parse`/`as_str`, `ActionRegistry` (`version`, `ids()`,
   `contains`, `axis_kind -> Option<AxisKind>`), `load_action_registry` via `parse_file`
   with the version-equals-directory check. `document.rs`: `ACTION_IDS_ENTRIES`, the
   `action-ids` arm in `parse` (closed entry set checked first), `Document::ActionIds`,
   the `SUPPORTED_FORMATS` entry, the `serialize` arm writing the three arrays back.
   `lib.rs`: `mod action;`, the four re-exports, and the public-surface list updated.
   `grammar_closure.rs`: `action-ids` joins the per-format list and the shipped
   `unknown-entry.toml` fixture satisfies it.

2. **Assumption register honored.** ✓ A5: 63-octet segment bound is RFC 1035's number.
   A10: every axis array is one line. A11: both `data/` directories carry READMEs.
   A12: withdrawal is major-directory route only, stated in `spec.md`; no alias machinery.
   A17: nothing in the diff *acts on* axis kind — it is recorded, graded, and asserted only.
   A9: no workflow change in the diff. `registry.toml` matches the proposal's enumeration
   byte-for-byte in content: 40 ids, 4 `discrete-lateral` (exactly the four
   `move.steer-*`/`move.strafe-*` endpoints), 1 `continuous-lateral`, 35 `none`, namespaces
   8/3/8/3/8/6/4.

3. **Deviations declared in `03-build-notes.md`.** ✓ The seven recorded deviations cover
   every gap I found between proposal and code: `ids()` in id order over `BTreeMap` rather
   than document order (with the true rationale that `toml::read_document` yields a
   `BTreeMap` and `toml.rs` is unchanged), `Vec<ActionId>` returns, `registry_from_entries`
   in `action.rs` as `pub(crate)`, the public `version` field, the private
   `MAX_SEGMENT_OCTETS`, the extra non-major-directory refusal, and the `schemas/profile/v1/
   spec.md` + `schemas/README.md` ADR-014 retarget beyond the delta inventory. No undeclared
   deviation found.

4. **Scope creep.** ✓ Nothing behavioral beyond the proposal. The only outside-inventory
   edit is the one-line lineage change in `schemas/profile/v1/spec.md`, declared as
   deviation 7 and docs-only.

5. **Build-notes claims checked against the code.** ✓ Spot-checked and true: array counts
   4/1/35; 3 valid + 14 invalid fixtures, each one fault named after it (I re-read all 17);
   no new `CoreError` variant (`action.rs` uses only `InvalidValue`/`MissingEntry`); no
   `Cargo.toml`/`Cargo.lock`/workflow change; `conformance.rs` and `common/mod.rs` untouched;
   round 2's two test-file blockers are gone from the current `tests/action.rs`
   (`fn entry<'a>` at :65; the loop array at :251-258 is one-element-per-line); the eight
   formatter sites round 3 tabulated match the current file text at all eight locations;
   `common/mod.rs:33` `#![allow(dead_code)]` claim true.

6. **Error paths exist and fail closed.** ✓ Missing `registry.toml` → `Io` (path kept out of
   `Display`, `source_path` channel); wrong-format registry file → `InvalidValue`; over-budget
   file → `DocumentTooLarge` before read; unsupported/absent declared version →
   `UnsupportedVersion` naming the majors (pre-existing gate, exercised by the new format);
   declared version ≠ directory major → `InvalidValue` one-liner; non-major directory →
   `InvalidValue`; malformed id, duplicate (within and across arrays), non-string element,
   missing axis array, unknown entry → `InvalidValue`/`MissingEntry` naming id/entry, echoed
   through `one_line` (escape + 80-char truncate), verified against `error.rs` rendering.
   Id grammar edge cases I traced by hand: empty string, `""` segments, leading/trailing dot,
   `move.steer.left`, 63 vs 64 octets, non-ASCII first byte, control char via `\uXXXX` —
   all refused, none canonicalized.

7. **Revision rounds.** Not applicable — first pass for this seat; no prior blocker/major
   from this checklist to re-check.

## Caveat

The cargo gate could not be run in this review session (shell access for `cargo` was
refused mid-review; git inspection worked). The formatter-shape and compile-shape claims
therefore rest on the build notes' round-3 rustfmt comparison plus my inspection of all
cited sites against the current text — consistent. A16's CI evidence leg still stands as
the gate's proof, as both proposal and notes say.

## Findings

- **nit** — `crates/core/tests/action.rs:26-29` and `crates/core/tests/action_ids.rs:32-33`:
  the suite headers still state they are "written from the proposal while
  `crates/core/src/action.rs` does not exist yet, so they fail until the design is
  implemented". That was true of the test-first commit; at HEAD the implementation has
  shipped and the sentence is stale. Fix: reword to past tense (or drop the sentence) in a
  docs-only test edit; no assertion changes.
- **nit** — `03-build-notes.md:84-90` (deviation 7) enumerates the files still carrying
  ADR-009 references but omits `crates/core/tests/conformance.rs:318-321`, whose doc comment
  still says the walk picks up "`patch`, `manifest` and `policy` when ADR-009 adds them".
  The file is declared unchanged by the delta inventory, so leaving it is right; the notes'
  list is just incomplete. Fix: add `conformance.rs`'s header comment to the
  retained-ADR-009 list (the same ADR-012/013-series retarget will clean it).

FYI (no severity): proposal Test strategy #3 says "a document with exactly the four defined
entries parses", but the entry table defines five; the code and `accepts_every_entry_the_action_ids_grammar_defines`
correctly use the five — the proposal line is the typo, not the code.

## Verdict

No blockers, no majors. The model matches the proposal's names, the closed sets refuse in
the right variants with the right echoes, the shipped registry is the proposal's
enumeration, and every gap is declared in the build notes. **approve**.
