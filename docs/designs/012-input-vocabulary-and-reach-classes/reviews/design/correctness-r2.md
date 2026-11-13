# correctness review r2

Scope: `reviews/design/editorial-r2.diff` (the single editorial pass of r2).
Inputs used: `02-proposal.md` (as applied), `docs/adrs/012-input-vocabulary-and-reach-classes.md`, the Revision log, and the diff itself. Prior-round correctness findings for this seat: none.

verdict: approve

## Findings

None. Each of the five hunks is language-only and every claim the changed
sentences carry still holds:

- Context — "the existing version gate refusing a v2 document for a v1-only
  reader (A2)": consistent with ADR Decision One ("a v1 reader refuses a v2
  document by naming the supported majors") and A2/A1.
- Context — "an ANSI-US reference geometry (A4)": the A4 basis checks out.
  Spot-checked sources (anti-hallucination gate): https://www.w3.org/TR/uievents-code/
  is live and states code values "are based on the name for the US key in that
  position (i.e., they are based on the US keyboard layout), which follows the
  naming convention found in [USB-HID]"; https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
  is live and its `VK_OEM_*` rows read "It can vary by keyboard. For the US
  ANSI keyboard, the …", exactly the geometry assumption A4 says must be
  explicit; https://www.w3.org/TR/gamepad/ is live and defines the standard
  layout behind the 16 `gamepad:` member names (A7).
- Options considered — "Symmetric reach, pixel-distance grading, and physical-
  position naming are now rejected for the reasons the ADR's Alternatives
  section states": true — the ADR's "Alternatives considered" section carries
  all three rejections ("A symmetric reach function", "Reach as per-key pixel
  distance", "Naming keys by physical position (LayoutX) now"). The reordering
  of "now" changes no meaning, and the id-shape reservation sentence still
  matches ADR Decision Three.
- The model — "as action ids do (A11)" / "as an echoed id does (A11)": the
  A11 echo shape (single-line, truncating, path-free) is untouched; no
  semantics moved.
- Worked examples — "comes out of the data here, not from a special case":
  re-verified against the shipped tables and the predicate: WASD→IJKL
  left→right is a uniform (6,0) translation with all degradations 0 (W stretch
  →I stretch; A,S,D home →J,K,L home), so it is legal at `GradeBound(0)` from
  the table data alone. Nothing in the change makes this sentence false.

No hunk touches the assumption register, the interfaces (`InputId`,
`InputGrade`, `ReachTable`, `KeyboardGeometry`, `GradeBound`,
`PlacementCheck`), the tables, or the Deferred section, so checklist items 1,
4–7 raise nothing against the diff (item 7 has no prior correctness findings
to re-test; the Revision log's r2 entries for placement/model are outside the
editorial diff and their text is intact).

## FYI (no severity)

- Context now says "an ANSI-US reference geometry" while assumption row A4
  still reads "a US-ANSI reference geometry" — the terminology unification is
  half done in the register; the design's own identifier is `ansi-us`
  (`geometry = "ansi-us"`), so A4's word order is the odd one out. That is a
  wording alignment, not a correctness defect — for the editorial seat.
