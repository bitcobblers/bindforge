# complexity review r1 (ratification pass c4)

verdict: approve

Scope: the diff `1f3a83c → 1d1e68a` of `02-proposal.md`, then the full current
document. Inputs: the proposal (incl. its Revision log), ADR-004, and my own r1
findings. Checked only for damage done by the revisions to what r1 approved.

## Checklist walk

1. New package/module/interface: none added by the revisions; no reuse candidate
   missed (`tools/orchestration/src/pipeline/gates.ts` is reused, not forked).
2. Trace to a stated Goal: the new parts (A19, Test strategy #4, Rollout 1's
   assertion commit) trace to the acceptance evidence rather than to a Goal, but
   each is load-bearing: #4 closes a real gap the existing suite leaves (its
   gate cases run over inert fixture strings, never a real nested path), and the
   pre-move commit is what keeps the moved-file diff inside ¶2's allowed set, so
   folding it into the move commit would break Test strategy #1. Not YAGNI.
3. A 30–50% smaller design: not available after the revisions. The one thing that
   could be dropped — the extra ADR clause — is a wording fix, not a size cut
   (finding 1).
4. New dependencies: none.
5. Deferrable second half: already deferred correctly (fixture-string refresh,
   repackaging, historical artifacts); those deferrals do not contradict ADR-004's
   Decision or requirements.
6. Config knobs / one-implementation abstractions: none, before or after.
7. Value check: proportionate. The mechanism footprint is the same in kind as the
   one r1 approved — 23 `git mv`, ~79 path lines (62 imports + 16 `new URL`, both
   counts confirmed by grep over `tools/orchestration/tests`), 2 skill-file lines,
   one comment — plus one small test-only commit and one assumption row. The
   growth is verification, not new machinery, and nothing config-shaped appeared.

## Findings

- [minor][new] Chosen design ¶2 — "a relative path reference — code or comment —
  naming a location this move passes takes its new depth" — this clause is the
  new permanent rule the move commit writes into ADR-004 Decision bullet 2
  (Deferred bullet 3, Rollout 2), but it keys on the *target* being "a location
  this move passes". The 16 `new URL` sites it exists to sanction point at
  `../prompts/` and `../../../.opencode/…`, locations the move does not pass —
  their depth changes because the *containing file* moved, which is exactly ¶2's
  own reasoning ("resolve against the moved file's own location and would ENOENT
  one tier deeper"). As written the amendment leaves a second, vaguer copy of one
  rule in a permanent surface while the proposal's closed enumeration stays the
  only operative statement — the added mechanism buys no simplification. Fix:
  state the rule once, in the precise ¶2 phrasing, and put that clause into the
  ADR amendment instead of the current one: "any relative `../` path in a moved
  file — code or comment — resolves against that file's location and takes its
  new depth". No text is added; the existing sentence is replaced.
- [nit][re-raised] Chosen design ¶4 — "Convention rule (Decision, bullets 4–5),
  stated once here: … The commit amends both surfaces encoding the flat form … to
  `tests/**/*.test.ts` plus this rule (A13)" — unchanged in substance by this
  revision: the ¶4 rewrite sharpened the claim into a "stated once here" heading
  while still shipping a copy of the naming rule into two live instruction
  surfaces, so the drift r1 flagged remains. Not in the Revision log, so it was
  neither taken nor rejected. Fix (unchanged from r1): put the rule once in
  repo-practices' existing "## Tests" section and amend only the glob that
  test-strategy:27 already restates behind its "Follow `repo-practices`
  conventions:" pointer. Both flat forms still disappear, so Test strategy #6's
  grep keeps passing.

## FYI (no severity)

- Chosen design ¶1 / Rollout 1–2: the revision log says "¶1 drops the path
  qualifier" because "the commit's areas are named in Rollout 2", but Rollout 2
  names no paths ("Land the move commit") and Rollout 1 now says "those test
  files" with no antecedent; `tools/orchestration/` survives only in Context and
  Test strategy #2 — a dangling internal reference for the correctness seat.
- Rollout 1 + 2 now land two commits, one of them a test-content change, while
  ADR-004 Consequences describes the churn as "one rename commit"; the proposal
  re-scopes Decision bullet 2 to "the move commit's diff only", which is a reading
  of the ADR, not a quotation of it — compatibility/correctness territory.
- Test strategy #2's fail-closed check is `git ls-files`, which lists tracked
  paths; a half-applied move that left an untracked flat file would not show up.
