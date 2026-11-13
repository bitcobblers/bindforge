# security review r1 (ratification pass c4)

verdict: approve

Scope: the diff `b08752d → 92beeaa` of `02-proposal.md` — the "The model"
paragraph on the load-time version check, predicate clause 4, and the
revision-log relabeling — then the full current document against the
security checklist. Prior security findings: none.

## Findings

None. The revised passages keep the security posture of the text approved in
round 1:

- The model paragraph — "every v1 document declares `v1`, so the check adds
  no corpus refusal" — narrows only the compatibility promise (what shipped
  behavior is preserved), not the check itself. The version-segment equality
  rule stays fail-closed exactly as approved: a declared version that differs
  from the loading directory's segment is a refusal, v1 directories included
  (Test strategy #3), a missing `version` is a `MissingEntry` refusal under
  the closed v1 entry set (A1), and the echoed version "tak[es] the same
  printable-escape-and-truncate path as an echoed id (A11)", so a hostile
  shared-file version string stays single-line and injection-free. This
  matches ADR Decision One's byte-exact v1 refusals and is pinned by the
  new `keys-entry.toml` fixture.
- Rewritten clause 4 — "the `PlacementCheck` result carries the per-move
  coordinate deltas — one common `(dx, dy)` when the moves agree — and each
  move's degradation" — discloses only bounded data: coordinates are
  `0..=999` via checked conversion, so deltas are small signed integers, and
  degradation is a rank difference over five classes. The recipient is a
  local caller (ADR-015's precondition/policy layer); no new path, secret,
  or egress surface is introduced, and "explain a refusal instead of seeing
  a bool" strengthens the fail-closed reporting of clause 1.
- Path-handling load-bearers are untouched by this diff and still present:
  `name` matched against `[a-z][a-z0-9-]*` **before** the join, a non-match
  refused as `InvalidValue` naming no path, and Test strategy #4 explicitly
  refusing `..` and separators before any file is read; `hand_matches_file_name`
  and the hand-file rule stay untouched.
- Parsing untrusted documents: the closed TOML subset and size budget are
  kept by v2 (ADR Decision One), pair encoding and coordinate bounds are
  refusal-checked, malformed ids and undefined inputs are structured
  refusals rather than verdicts (checklist item 7).
- Dependencies (checklist item 5): the design names none — "std-only, no new
  dependency, `Cargo.lock` byte-unchanged"; `Cargo.toml`/`Cargo.lock` are in
  the Unchanged list, so there are no advisories to check.
- The Revision-log relabeling (r1 → r2 for the placement-grammar and
  version-equality decisions) records only when the decisions landed; it
  does not alter any of them.

## FYI (no severity)

- Clause 2 says a shape mismatch is "illegal whatever the grades say" while
  clause 4 now describes one common `(dx, dy)` "when the moves agree" and
  per-move deltas generally; Test strategy #5 already pins that a
  non-translation set reports per-move deltas, but whether shape failure
  surfaces as an illegal verdict versus a structured refusal is an API-shape
  nuance for the correctness seat.
