# Security review r4

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r4.diff`, confirmed against
`02-proposal.md`, `docs/adrs/009-canonical-profile-and-manifest-model.md`, and
the proposal's Revision log. Inputs kept to those four; no review report or
transcript was opened.

## Checklist evaluation (security)

1. **Trust boundaries** — intact. Shared/`data/` documents are treated as
   untrusted throughout (Adversarial posture, A18, Test strategy #9), and the
   diff did not move any classification. The Context rewrite of the five
   contracts keeps the `contract N` cross-references resolvable in the ADR's
   order, so the guard claims still attach to the sections they cite.
2. **Parsing untrusted data** — unchanged: 1 MiB pre-read cap, 16-level nesting
   cap, closed entry sets, bounded one-line rejections (A3, A6, A8). No new
   format, budget, or parser was introduced by this pass. No decompression and
   no code-evaluation surface exists in the accepted TOML subset.
3. **Path handling** — unchanged and, in one place, clearer. The Id → file
   rewrite ("a document declaring another is rejected with the shipped
   `InvalidValue` naming the tree") states the tree/format rejection as an
   explicit rejection; the rule it reports on (index lookup only, no path built
   from a document string, `..`/absolute/encoding tricks inert, symlinks not
   followed, size cap on bytes read) is untouched.
4. **Secrets and PII** — unchanged. `generated-by` stays a `<namespace>:<part>`
   id over the closed producer list, so the r2 privacy closure (no free-text
   attribution such as `author:alice`) survives the language pass; signatures
   and trust remain Deferred to ADR-014 (A22), consistent with the ADR's
   "recorded, never trusted".
5. **Dependencies** — evaluated: this design adds no dependency (`bindforge-core`
   depends on nothing, `bindforge-transforms` on the path crate only). The two
   crates named in prose are `serde`, staged in `[workspace.dependencies]` and
   used by no member, and `toml_edit`, named only as ADR-012's consumer route;
   neither is adopted here, and the RustSec checks run for this round surfaced
   no unfixed advisory for either. Nothing to block.
6. **Privileges** — unchanged: read-only over `schemas/` and `data/`, no game
   config I/O, no device access, no cwd/network/database (Compatibility matrix,
   Test strategy #14); no transform writes outside a base.
7. **Fail closed** — unchanged: per-condition rejections, a rejected `stack` is
   the `Err` channel and never reads as "applied with skips", an ungraded key is
   `UnknownInput` in placement validation, `DuplicateId` prevents an ambiguous
   pin, and a dangling `for` pin is `UnresolvedAncestor`. The r3/r4 closures
   cited in the Revision log are still stated in the text this diff touches.

No hunk in the diff altered a guard, a limit, a rejection path, a trust
classification, a privacy rule, or a dependency edge; the rewrites
(US-ANSI grid wording, section-name cross-references, clause reordering,
heading rename) stay true to the sections they report on. No finding to report.

## Findings

none.

## FYI (no severity)

- Chosen design → Encoding rules → Nested arrays for structured lists — the
  newly added "Editor notes" line flags "`stack`, `swap`, and the manifest's
  `action`/`default`/`exclusive` rows are each a leading id (action, group)
  followed by its values" as ambiguous. The parenthetical omits `stack`, whose
  leading element is a transform name, not an action or group id; the rule the
  reader enforces is stated in Chosen design → Transforms ("each a non-empty
  array of strings whose first element is the transform name"), so no
  parse-side gap opens — the author's clarification should just not contradict
  that sentence. Clarity for the language/correctness seats.
- Chosen design → Reach classes and placements → Authoring bar — "CI enforces
  what it can: … and every grading change carries a named reviewer (`git
  log`)". Committer metadata is unforged only as far as the forge's
  authentication goes, so this reads as a review requirement recorded in
  history, not a machine-checked gate; A22 and the ADR-014 deferral already
  frame provenance as not a trust grant, so the claim is consistent as it
  stands. Supply-chain/provenance seat's call if it wants the enforcement
  wording tightened.
