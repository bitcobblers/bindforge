# complexity review r1c3

verdict: approve

Ratification pass over the diff 2160215→4e18f4f (02-proposal.md blobs), checked
against the round-1 complexity findings. Both prior findings' sections verified:

- §3 doc-family bullet: my r1 minor (duplicated deny lists) is closed exactly as
  proposed — the inline trailing-deny clause is gone, the bullet ends "Then the
  run-record block below: it is the single authority for these families'
  run-record denies", and Test strategy #4 now pins the star-form inspection set
  and the shared glob/grep denies. No residue, no new duplication.
- §2 canonical shape: `GateEvidence = { at: number; ... }` is byte-unchanged, so
  the r1 nit on `at` is neither closed nor broken by the revisions (see FYI).

The §1 identity rewrite (directory-scoped, canonicalized per A17) and the §2
char-based tail are revisions answering other seats; both trace to Goals/Risks
already in the approved document and add no unowned mechanism. Value check:
complexity remains proportionate to the ADR-016 goals.

## Findings

- [minor][new] Rollout, item 2 — "`tools/orchestration/package.json`
  `engines.node` rises to `>=22.23.5` for the 2026-11-11 Node security release —
  the only runtime prerequisite added" — this is the one change the revisions
  landed that traces to no Goal or ADR-016 Decision clause: the ADR never
  mentions Node, and nothing this design ships (byte capture, UTF-8
  replacement decode, `printable` bounding) requires 22.23.5. It sits in tension
  with the approved Context claim "implements the ADR's Decision clauses 1–4 and
  nothing else", and since Rollout #1 lands as one branch, hosts on 22.23.4 now
  fail an engines gate for a reason external to the design (checklist #2 —
  moving part without a stated Goal trace). Fix: one clause marking it as an
  environment chore riding the branch ("no Decision 1–4 code requires it; it
  rides this branch as a chore bump"), or drop it from Rollout and land it as a
  separate `chore:` change — either keeps "nothing else" true without adding a
  mechanism.

## FYI (no severity)

- The r1 nit on `GateEvidence.at` (no stated consumer) was not addressed by the
  r2 revisions and is not indexed in the Revision log; it remains open and
  non-blocking, unchanged from what round 1 approved. If a later round states
  `gateEvidenceText`'s header prints `at` as the gate-run timestamp, the nit
  closes for free.
