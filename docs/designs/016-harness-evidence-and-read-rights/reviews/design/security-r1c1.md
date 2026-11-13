# security review r1c1

verdict: request-changes

Scope: `git diff 21602159 9b6b5edf` on `02-proposal.md`, then the full current
document. Dissent pass against the security seat's r1 findings. Checked the
Revision log first: all four r1 findings are indexed as accepted, and none is
listed as rejected.

Closures confirmed (no finding): the §1 design-directory containment plus the
Test-strategy #3 cross-design-directory fixture close the intent of r1 #1; the
`gateEvidenceText` provenance header and "cited with its leg command and exit"
close r1 #2 (the data source is now classified as captured output of the code
under review, citable, not instructional); `engines.node >= 22.23.5` in the
compatibility row and Rollout #2 closes r1 #3 — re-verified against nodejs.org:
the 2026-11-11 security release fixes the HIGH items affecting 22.x < 22.23.5
(CVE-2026-123864/-123865/-123866/-123872) in exactly v22.23.5. A17's grammar
claims (`*` spans `/`, whole-value match, slash normalization, case-insensitive
on Windows, last match wins, `grep` matches the regex not the path, `read` is
Location-relative or canonical absolute, shell is scanner command text) match the
V2 permissions page as cited. `@opencode/client ^2.1.4` carries no unfixed
advisory: the only High still in the range (GHSA-632h-h47v-g4x4, `/global/upgrade`
arbitrary package install) is patched in 1.18.22, well below 2.1.4.

## Findings

- [major][re-raised] Chosen design §1 Own-report identity, shell branch —
  "collect every `REVIEWS_KEY` match in the command; if the set is non-empty and
  every match canonicalizes, under the rule above, to `own`, exempt" — the new
  containment test has no defined operand on the shell route, so the cross-run
  read r1 asked to close is not closed by determinate text. The rule above tests
  a "canonical spelling … resolved against the repo root" for lying under the
  attempt's design directory, but the operand this bullet hands it is the
  `REVIEWS_KEY` **match**, which by the design's own definition (A2, §1's
  "everything from the `reviews` segment on") has the design-directory prefix
  stripped. A prefix-less fragment (`reviews/design/security-r1c1.md`) resolved
  against the repo root can never lie under `docs/designs/<run>/`, so either
  (a) the exemption never fires on the shell route — the only route a seat can
  reach its own report by (A21, and §1's new prompt sentence says re-reading is
  allowed "through your `git diff*` allow") — so Decision 1 does not take effect
  and Test strategy #11 fails; or (b) the implementer restores it by resolving
  the fragment against the attempt's own design directory, which makes
  containment vacuously true and reopens the exact hole: `git diff
  docs/designs/014-<…>/reviews/design/security-r1.md` yields the same fragment as
  the seat's own key and passes the exemption — and prior runs' `reviews/` trees
  are committed in this repo, the per-seat wildcard already opens those paths at
  the permission layer, and the Risks seam row now concedes that reach, so the
  audit's identity rule is the only backstop left and it is the text that is
  ambiguous. Fix: name the compared operand once in §1's shell bullet — take the
  whole path token in the command leg that contains the `reviews` segment,
  canonicalize that against the repo root, require it to lie under the attempt's
  design directory, and only then compare `reportKey`; state that a match with no
  design-directory prefix is not an identity match (fails closed). Add to Test
  strategy #3 the two spellings the §3 allow actually opens: `git diff
  docs/designs/<own>/reviews/design/<specialty>-r1c1.md` clean, `git diff
  docs/designs/<other-run>/reviews/design/<specialty>-r1c1.md` fails.

- [minor][re-raised] Risks, tail row vs Chosen design §2 build-note sentence —
  "no redaction pass ships and no whole tail is published" against "a failing-leg
  tail belongs in the `03-build-notes.md` 'Gate posture' section cited with its
  leg command and exit" — the rebuttal's premise contradicts the §2 sentence it
  is meant to answer: `03-build-notes.md` is a committed artifact of the run in
  this open repo, and §2 mandates tail content land there, so tail content *is*
  published, just selectively. The same row concedes a tail "can carry absolute
  paths or registry text", which is precisely checklist item 4's boundary — a
  failing `npm run check` line can embed a registry URL with credential-bearing
  text and a test can echo an environment variable — yet nothing states what must
  be dropped before the citation is committed, and the design never says where
  the record persists (checkpoint and transcripts stay local; the build note
  leaves the machine). The author's decision to ship no redaction pass is sound
  and not re-argued here; only the stated coverage is. Fix: one clause on §2's
  build-note sentence — the citation is line-selective and drops any line
  carrying a credential-bearing token or an absolute local path, the same posture
  the Risks row claims — and name in that row which artifacts are committed.

## FYI (no severity)

- The V2 permissions page the design now leans on (A17, compatibility row) also
  states that `external_directory` is enforced for a shell command's working
  directory and only best-effort for paths embedded in the command, and that
  relative-path confinement is a *mutation* rule. So §3's `cat`/`head`/`tail`/
  `find` widening for the five document families is not path-confined by the
  permission layer the way their `read` tool is; it rests on prompt plus deed
  audit. Consistent with ADR-016 Decision 3 ("stop the accidental access, not the
  deliberate one") and with the coder/code seats' existing `cargo *`/`npm run *`
  posture, so not held against this design — flagging for the least-privilege and
  correctness seats.
