# security review r1 (dissent pass c1)

verdict: approve

Scope: the diff `17aee59..b08752d` of `02-proposal.md` (the sections rewritten
to answer round-1 findings), then the full current document. Inputs: the
proposal (incl. its Revision log), `docs/adrs/012-input-vocabulary-and-reach-classes.md`,
and this seat's round-1 findings. Checklist: `references/security.md`.

## Findings

None open. All three r1 findings are closed by the changed text, and the
changed text introduces no new security defect.

## Closure check of this seat's r1 findings

- [major][closed] **Placement legality — `load_geometry`** — "`name` matched
  against the same `[a-z][a-z0-9-]*` grammar *before* the join, a non-match
  refused as `InvalidValue` naming no path (A11), then `parse_file` on
  `dir.join(format!("{name}.toml"))`". This is exactly the requested fix:
  the grammar excludes `..`, separators, and any absolute path before the
  path is built, the refusal reuses the existing `InvalidValue` shape and
  leaks no path, and `geometry_matches_file_name` remains the post-load
  rule. Test strategy #4 now pins it ("a `name` outside the grammar — `..`,
  a separator — is refused before any file is read"). Checklist 3 satisfied
  — the grammar check is stronger than reject-`..`-and-absolute, so no
  canonicalization gap remains.
- [minor][closed] **The model — echoed version in `reject_unknown_entry`** —
  "the echoed version taking the same printable-escape-and-truncate path as
  an echoed id (A11)". The document-controlled version string now passes
  through the same escape-and-truncate path as echoed ids; multi-line or
  unbounded refusals are no longer constructible. Checklist 4 satisfied.
- [minor][closed] **Placement legality predicate — arithmetic regime** —
  coordinates are now "coordinates `0..=999` through checked integer
  conversion so an oversized literal is a grammar refusal", shape deltas are
  "taken in signed integers over the bounded coordinates", the `classes`
  list is a closed five-element canonical array (ranks 0..4), and Test
  strategy #4 refuses "a coordinate outside `0..=999`". Deltas are bounded
  (±999), degradations bounded (−4..=4) against `GradeBound(u32)`; neither
  subtraction can overflow, and a 300-digit coordinate fails closed as a
  grammar refusal. Checklist 7 satisfied as requested.

## Re-scan of changed text for new security issues

- The new version-segment equality check ("for **every** reach table at
  load, v1 directories included") is fail-closed hardening, not a widening;
  its refusal inherits A11's single-line, truncating, path-free message
  shape, which the design already asserts covers every refusal it needs.
- Coordinate bounds (`0..=999`) and the closed class list keep the rank
  arithmetic bounded even for hostile table data.
- No new dependency ("std-only, no new dependency, `Cargo.lock`
  byte-unchanged"), so checklist 5 is evaluated as a pass with nothing to
  check; filesystem access stays read-only under the inherited parse budget
  (checklist 2, 6); the design handles no secrets or PII (checklist 4).
- Revision log reviewed: the four r1 entries record exactly the closures
  verified above; no rejected finding of mine is being re-raised.

## FYI (no severity)

- The v2 contract prose no longer names `class-order-mismatch.toml` and
  `unknown-class-id.toml` as the corpus cases for the canonical-`classes`
  rule, though both remain in the directory listing and in Test strategy
  #2's explicit list (wrong order, duplicated id, unknown id) — the rule
  stays pinned end-to-end; a correctness seat may want the in-prose pointer
  back for readability.
