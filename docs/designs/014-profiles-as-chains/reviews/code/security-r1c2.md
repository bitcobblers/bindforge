# Code review — security, round 1, ratification pass c2

Scope: `git diff 4844669384fd046eebfd47960283e178990617a4..HEAD -- ':(exclude)docs/designs'`,
read against the current `schemas/profile/v2/spec.md`, `docs/designs/014-profiles-as-chains/02-proposal.md`,
and `docs/adrs/014-profiles-as-chains.md`. This pass checks only that the revisions
(fix stage `f299913`, test-update `aaf0796`) did not break what round 1 approved.

## Verdict: approve

No blocker or major findings. The revisions are, on the whole, security-positive:

- **r1 finding #2 (one-sided provenance inheritance) is fixed, not merely patched around.**
  `inherit_provenance` is deleted; `new_patch` leaves `seeded_from`/`generated_by` `None`
  (chain_ops.rs:660-661) and merge no longer copies labels. This is *stricter* than my r1
  fix suggestion and consistent with the proposal's merge contract (02-proposal.md:460-465
  lists only id/derives/for/deltas) and ADR-014 Decision Four. It is pinned by
  `merge_leaves_the_provenance_labels_absent_from_its_output`. Nothing in the revision
  reintroduces a label onto merge output.
- **The `load_chain` head/ancestor split reduces message echo.** An absent head is now
  `parse_file`'s `Io`, whose path rides the variant (`source_path`) and stays out of
  `Display` (error.rs:159-161); `ChainLinkMissing` no longer answers a caller-named path.
  The new test `refuses_an_absent_head_as_the_read_refusal_carrying_its_path` pins both
  halves (path present in `source_path`, absent from the message). Ancestor paths are still
  built only from parse-validated slug slugs (document.rs:811 `check_profile_slug`), so the
  one-directory rule and traversal posture (spec.md rule 1) are unchanged.
- **The directory-major refusal converging on `check_dir_version` (document.rs:1346-1361)
  is the A5 single-spelling the proposal itself names (02-proposal.md:338-339).** The new
  message names `{file_stem}.toml`, the declared major, and the `vN` segment only — the
  segment is regex-bounded (`major_dir_segment`), and no path enters the line; the updated
  test pins the no-path property. Walk semantics (cycle-before-descent, depth cap, id↔name)
  are behavior-identical under the restructure.
- **Test revisions did not weaken any security pin.** The Cycle case swapped a wording
  needle for strictly stronger log-invariant assertions (non-empty, one line, no control
  characters); the merge tests are additive. No control-char or path-disclosure assertion
  was removed.
- `serialize_profile_link`'s unconditional `kind`/`for` is fail-closed: a hand-built model
  without a kind writes `kind = ""`, which the reader refuses naming the entry — no silent
  shape change on round-trip.

Two r1 minors remain open; one of them gained a small new surface in this diff, so it is
restated against the changed code.

## Findings

### minor — `one_line` still passes U+2028/U+2029 raw, and the revision added one echo path through the gap
- location: crates/core/src/error.rs:172-186 (`one_line`), with the new echo at
  crates/core/src/document.rs:1346-1361 (`check_dir_version` message) reached from
  crates/core/src/chain_ops.rs:372,405 (`open_head`/`check_link`); label side unchanged at
  crates/core/src/document.rs:1110-1125 (`is_label_char`)
- problem: r1 finding #1 was not addressed: `is_label_char` still admits U+2028 LINE
  SEPARATOR / U+2029 PARAGRAPH SEPARATOR, and `one_line` escapes only `is_control()` (Cc),
  so Zl/Zp flow raw into any refusal message that echoes document- or name-controlled text.
  The revision widens the echo surface by one path: a directory-major mismatch on the head
  now answers `InvalidValue` naming `{file_stem}.toml`, where the base answered
  `UnsupportedVersion` without the stem — so a caller-named head file whose stem carries
  U+2028 (legal in a Linux filename) now has that code point echoed into the rejection
  line, against spec.md:198-203 ("a single bounded line: it escapes control characters and
  truncates"), which is the mitigation the contract states for exactly this class. No log
  line can be *forged* (`\n`/`\r` are still escaped and the 80-char cap holds), so severity
  stays minor; the residual harm is the same terminal/JS-adjacent log-splitting class already
  accepted as minor for 013's `native_name`.
- fix: as proposed in r1 — add `'\u{2028}' | '\u{2029}'` (and ideally U+0085) to
  `is_label_char` and mirror the sentence in spec.md's `generated-by` row and fixtures; if
  the log-line invariant is meant to cover echoed values generally, escape Zl/Zp/U+0085 in
  `one_line` — the new `check_dir_version` echo is now a consumer of that gap that is not
  document-content-guarded at all.

### minor — merge output is still unbudgeted against `MAX_PATCH_DELTA_ELEMENTS` (r1 #3, unchanged)
- location: crates/core/src/chain_ops.rs:340-359 (`merge`, post-`emit_composition`),
  emit_composition at 772-820 (touched by this diff)
- problem: the diff rewrote `emit_composition` (reciprocal-pair `continue`) but added no
  bound on what it writes. Parse caps each operand list at 256 (document.rs:921-930), yet
  touched actions per operand reach 1024 (256 set + 256 clear + 2×256 swap halves) and the
  union 2048, so a grammar-valid pair of operands can still yield a merged patch whose
  `set`/`clear` exceeds 256 elements — a Profile the same grammar refuses at the next
  `load_chain`, so the refusal surfaces downstream instead of at the operation, and the
  composite-cost paragraph (spec.md:192-196) still does not describe merge output. The
  revision's `continue` only shrinks the swap side; it does not close the gap.
- fix: unchanged from r1 — after `emit_composition`, refuse with the same `InvalidValue`
  shape `delta_elements` uses when any emitted list passes `MAX_PATCH_DELTA_ELEMENTS`, and
  pin the refusal with a test in tests/chain_ops.rs.

## FYI (no severity)

- spec.md:46 describes the `generated-by` character policy as "no ASCII control or
  bidi-control characters", but the implementation also refuses default-ignorables and soft
  hyphen (document.rs:1110-1125), and the proposal's own table (02-proposal.md:236) includes
  "default-ignorable" — a third-party reader conforming to the shipped spec alone would
  accept `U+200B` the reference refuses. Pre-existing, not touched by this diff; worth
  folding into the spec.md sentence if the label policy is widened per the finding above.
- `CoreError::Cycle`'s Display text lost the "`(Cycle)`" marker (error.rs:149). Not a
  security matter; flagging only so the compatibility seat checks whether any consumer
  pinned the old contract string.
