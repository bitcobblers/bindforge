# Code review — security — ratification pass c2 (round 1 ratification)

Seat: security. Scope: `git diff 2682fb1f..HEAD -- ':(exclude)docs/designs'`, read
against `02-proposal.md` and `docs/adrs/012-input-vocabulary-and-reach-classes.md`.
Purpose: confirm the post-approval revisions did not break what the security seat
approved in round 1. Previous security findings: none.

## What the revision delta is, security-wise

Three source changes and their test/fixture counterparts:

1. **Per-format version gate** (`document.rs`: `format_majors`,
   `check_format_major`): a document declaring a major its own format does not
   ship is refused. This *tightens* boundary validation — it closes the
   fail-open where a `profile`/`action-ids`/`keyboard-geometry` document
   declaring `v2` would previously pass the crate-wide union check and be read
   with that format's v1 grammar (the exact misreading `schemas/README.md` and
   the proposal's A2 forbid). All three untrusted entry points (`parse`,
   `parse_file`, `parse_reader`) funnel through `parse`, where the gate runs
   before grammar dispatch; I found no bypass path.
2. **Shared `major_dir_segment`** (`document.rs`, consumed by `action.rs` and
   `check_dir_version`): deduplication only. I checked the semantics are
   byte-identical to the deleted `is_major_dir` helpers (last path component,
   `v` + ASCII digits; a path ending in `..` yields `None` because
   `Path::file_name` rejects it, which each loader already handled the safe
   way — registry refuses, stand-in pass is pre-approved behavior). No new
   path-handling surface, no traversal introduced.
3. **ASCII closure of the input grammar** (`input.rs` doc/rules, new
   `keyboard:é` cases): the implementation was already ASCII-exact
   (`is_ascii_lowercase`/`is_ascii_digit`/explicit glyph list at
   `crates/core/src/input.rs:197`); the revision makes the contract text and
   corpus state it, closing the normalization-fold vector (two spellings on one
   physical key) at the language-neutral contract level. Strictly a hardening.

Removal of the `version != MAJOR_V1` guard inside `parse_keyboard_geometry`
(`document.rs`) is safe *only because* `check_format_major` runs before the
dispatch — I verified the call order at `document.rs:241-243` and the new
`version_gate` case pins the keyboard-geometry `v2` refusal surviving the
subsumption.

## Checklist

1. **Boundary validation of untrusted shared files** — pass, strengthened: the
   per-format gate closes a fail-open; `é` fixtures/tests pin the closed set.
2. **Path handling** — pass: `major_dir_segment` takes a single validated path
   component; no `..`/absolute handling change; geometry-name-before-join
   validation untouched.
3. **Injection** — pass: no new regex or shell surface; echoed members go
   through `error.rs` `one_line` (bounded, control-char-escaped, single line).
   The `contains("é")` assertion in `tests/reach_v2.rs` tests a printable,
   in-budget fragment — consistent with the single-line bounded-message rule.
4. **Secrets/PII** — pass: the two new fixtures carry only inert vocabulary
   data; no new log lines echo anything beyond the already-escaped values.
5. **New dependencies** — pass: none; the diff touches no `Cargo.toml`/
   `Cargo.lock`.
6. **Filesystem writes** — pass: no new or changed writes; `serialize` is a
   pure string builder.
7. **Clean rejection of malformed/hostile input** — pass: every new invalid
   fixture is refused at `parse` with a structured `CoreError` before any model
   is built; no partial application path. `cargo test --workspace --locked` is
   green across all targets (0 failures) on this checkout.

## Revisions vs. round-1 approvals

Nothing the security seat approved regressed: the union gate still binds first
(no format check can accept a major the crate never ships), v1 refusals stay
byte-exact, error-shape rules are unchanged, and the test suite that pinned
those refusals is extended, not weakened.

## Findings

None (no blocker/major/minor/nit security findings on this delta).

## FYI (out of specialty, no severity)

- `document.rs::format_majors` is a hard-coded per-format list, while
  `version_gate.rs`'s majors-equality case pins only the crate-wide union to
  the `schemas/` tree. If a future PR adds e.g. `schemas/profile/v2/` without
  updating the hard-coded list, the parse gate stays fail-closed (refuses
  profile v2 documents) — the safe direction, but a per-format equality pin
  would catch the drift early.
- `document.rs:~232`: a union-gate refusal (e.g. a profile declaring `v99`)
  still names the crate-wide union, which today includes `v2`; the newly
  tightened README wording ("refused naming that format's majors") is satisfied
  literally only by the per-format step. Message-shape nit, not a rejection
  integrity issue.
- `document.rs` serializer: a hand-built v1 model carrying grades now silently
  drops `keys` on serialize instead of emitting a document the v1 reader
  refuses; neither output is constructible from parsed data, and neither leaks
  anything — noted for the correctness seat.

## Verdict

approve — zero open security findings; the post-approval revisions tighten
rather than loosen the boundary posture approved in round 1.
