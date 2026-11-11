# complexity review r1 c2 (ratification)

verdict: approve

Scope: `git diff da725a4 305aa36` over `02-proposal.md`, then the full current
document. Inputs: proposal, ADR-006, the in-document Revision log, and my r1
findings. Pass rule applied: report only what the revisions broke from what
was approved, or a false section among those I cited.

## Checklist pass over the changed text

1. **New moving part (r1 closure in §1):** the authorship test — "a match is
   skipped while it still holds the bytes `saveState` last wrote for it, kept
   in memory (or is absent because the runner deleted it)" — is the only new
   mechanism in the diff. It traces directly to Goal 1 and A7: without it,
   fencing the name reverts the runner's own post-commit saves and boundary
   commits stop advancing the committed checkpoint. It reuses the existing
   sweep machinery (runner.ts:263–276) and the existing `types.ts` vocabulary
   home rather than adding a module or interface; no new package anywhere in
   the diff.
2. **Smaller-design check (item 3):** the r1→c2 delta is net-negative in text
   — the auto-reconcile option paragraph and the schema-unchanged paragraph
   were removed, §2 and §3 wording trimmed — while the authorship test is the
   smallest distinction available between runner bytes and planted bytes
   (content equality, no timestamps, no new state beyond a string the runner
   already serialized). Nothing suggests a 30–50% smaller variant.
3. **Dependencies (item 4):** none added; `'wx'` is stdlib (A6, A11).
4. **Deferral (item 5):** digest reconciliation moved out of Options into
   Deferred with its reason; the deferral does not contradict ADR-006's
   Decision (the fence, not digest compare, answers Q2).
5. **Knobs / one-implementation abstractions (item 6):** `CheckpointWriteFailure`
   has three consumers (drive catch, `finalizeEscalate` guard, entry-save
   guard); the case-insensitive segment comparison follows the existing
   moves.ts:134–147 precedent and is pinned by Test #1 rather than being a
   new knob.
6. **Value check (item 7):** complexity remains proportionate to the feature;
   the new Test #2 clause ("a boundary whose only checkpoint change is the
   runner's own save commits that save") pins the new mechanism, so it is
   tested rather than free-floating.

## Regression check against my r1 findings

- **r1 minor (tmp pattern re-derived in gates.ts vs minted at
  `state-store.ts:186`):** the cited rule text was rewritten — matcher
  narrowed from a bare prefix to the exact `.pipeline.json.<8 hex>.tmp` shape
  and pinned by Test #1 — which mitigates but does not adopt the single-home
  fix. The revision did not break anything I approved and did not make the
  cited section false, so it is not re-reported on this pass.
- **r1 nit (contract verbatim in `StateSchema` header and `docs/workflows.md`):**
  the cited sentence is unchanged in current text; the "May refuse" revision
  only swaps the paraphrase for the ADR's own two postures ("restore the
  committed copy … or start a fresh run", ADR-006:55–57), which is verbatim
  the ADR's interim posture — consistent, not broken.
- A12 → verified, Windows `no (unverified)`, macOS `partial`, Platform wording:
  honesty/consistency edits that reduce claims rather than add mechanism; the
  removed "Auto-reconcile" Non-goal bullet leaves no dangling reference
  (the Deferred entry carries the reasoning).

## Findings

None. No open blocker or major from this seat; no regression introduced by the
revisions.

## FYI (no severity)

- Deferred: "the §1 fence makes 'dirty ⇒ runner' true" is stated absolutely;
  a process killed mid-stage after a stage write but before the boundary
  sweep can leave a dirty, non-runner worktree copy at resume — whether the
  resume path tolerates that is a correctness/compatibility question, not
  complexity.
- The tmp shape is still specified in gates.ts terms while minted at
  `state-store.ts:186` (my r1 minor); Test #1 pins gates' matcher with a
  literal fixture, not its coupling to the mint site — visible to the
  compatibility/correctness seats if a future change touches the tmp scheme.
