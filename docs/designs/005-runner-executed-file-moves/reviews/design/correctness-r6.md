# correctness review r6

verdict: approve

Scope: `docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r6.diff`
(the r6 editorial pass's language-only diff against `02-proposal.md`). Inputs:
the proposal, `docs/adrs/005-runner-executed-file-moves.md`, the Revision log
inside the proposal, and my own prior correctness findings (none — editorial is
a single language pass per round). Per the pass rule, a finding is reported only
if the revisions broke something previously sound or made a cited section false.

## Findings

None. Every hunk was checked against the rest of the proposal and the ADR:

- **Context** — "budget limit` because its 23 test-file moves": comma removal
  only; the reason string still matches the ADR's Context verbatim
  (`tests not green at the code-review budget limit`) and A12.
- **Chosen design / Reply field** — the reflowed
  "`MovePairSchema = z.object({ from: z.string().min(1).max(200), to: z.string().min(1).max(200) })`
  and `moves: z.array(MovePairSchema).default([]).max(64)`" joins a
  mid-expression line break without touching a bound; the length cap and pair
  cap that Test strategy #1 asserts ("an endpoint over the length cap, and a
  list over the pair cap") and the "bounds cap the validation and apply work"
  sentence still describe the same schema.
- **Wiring into retry** — "a failure — a returned issue or a throw from a rule —
  takes the existing schema-failure branch": em-dash appositive; the mapping
  validator failure → `invalid-reply: moves — <issue>` →
  `retryInSameSession` → second-failure escalation still matches #6 and #9.
- **Apply** — "`finalizeStage` between reply acceptance and the
  `enforceOwnership` call, i.e.," and "a guard refusal takes the same
  `moves-apply-failed` outcome and preserved worktree as a rename error":
  punctuation plus ellipsis of "that of". Both readings of the compressed
  comparative are the same classification that #5 ("that refusal takes the
  `moves-apply-failed` class of #6"), #6 ("`'moves-apply-failed: …'` is false")
  and the Revision log entry "a failed apply or guard refusal is
  `moves-apply-failed`" already pin, so no failure class was blurred: guard
  refusal and rename error share `moves-apply-failed` (fresh attempt 2),
  rollback failure stays `moves-rollback-failed` (operator).
- **Sweep interaction / Prompts headings** — full stop moved outside the bold;
  heading text and the "No new code in `enforceOwnership`" claim, and the four
  prompt files named, are unchanged and still agree with ADR item 4.
- **Risks** — "breaks Test strategy #4 rather than silently breaking
  production": same invariant-pin claim, still consistent with the Options
  entry "A new sweep branch keyed on reply moves (literal ADR-005 item 4)" and
  with #4.
- **Test strategy #11** — "a pair that can no longer be applied between
  validation and apply produces no commit, a byte-identical worktree": replaces
  the non-standard "unappliable" with a plain clause of identical extension.
  Both paths the design allows still satisfy "byte-identical": a rules 4–7
  guard refusal performs no rename, and a mid-list rename error rolls back in
  reverse, "which is exact because the pairs are disjoint". Consistent with #5
  and with the TOCTOU row of Risks.
- **Rollout #2 / #4** — "`moves` is optional, defaulting to `[]`" and
  "no `operatorForce`/`operatorExtension` gate entries and no `stageFailure`
  reason containing `moves`": reworded only; still matches A13, the Reply field
  default, and the ADR's "Dogfood acceptance: a later run of any file-moving
  change, green without a single escalation".
- Diff faithfulness: every post-image line in the diff is present in
  `02-proposal.md` as written, and no substantive statement changed anywhere in
  the hunks — the edit stays inside language.
- Anti-hallucination spot-check (citation rows were untouched by the diff, so
  this is a re-confirmation, not a finding): A5's "on by default, at a 50%
  similarity threshold" is supported by https://git-scm.com/docs/git-diff
  ("diff.renames … Defaults to true"; "The default similarity index is 50%.");
  A8's "`fs.renameSync` is a thin wrapper over `rename(2)`: atomic, fails
  `EISDIR`/`EXDEV`/`EINVAL`" is supported by
  https://man7.org/linux/man-pages/man2/rename.2.html, which also backs rule
  5's dangling-symlink rationale ("if newpath refers to a symbolic link, the
  link will be overwritten") and the Compatibility matrix's `EXDEV` row
  ("not on the same mounted filesystem").

## Deferred check

- "Directory moves as single pairs — this ADR-005 if a real change needs them;
  file pairs cover the motivating case; no follow-up yet" and the other four
  deferred items: none contradicts the ADR's Decision or Consequences. The ADR
  never asks for directory moves, item 2's "Every `from` must be a tracked path
  at `headBefore`" is a necessary condition the proposal tightens rather than
  loosens, and the read-only-git deferral repeats the ADR's own clause deferring
  item 5 to a follow-up ADR. Not re-opened.

## FYI (no severity)

- The diff's only substantive addition is the trailing "Editor notes: Deferred /
  out of scope — "Directory moves as single pairs — this ADR-005 if a real
  change needs them" is ambiguous; author to clarify." The flagged bullet is
  grammatically missing its verb, so the sentence leaves the owner of a future
  directory-move follow-up unclear ("this ADR-005" vs. the stated "no follow-up
  yet"). It contradicts no ADR clause, so it is not a correctness finding here,
  but it is an unresolved author action item left standing at the end of the
  document — an editor/author matter, and the editor's own note already hands it
  over.
