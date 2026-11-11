# correctness review r3

verdict: approve

Scope: `reviews/design/editorial-r3.diff` (the round-3 editorial pass's diff
into `02-proposal.md`). Inputs per the caller: the proposal, the ADR, the
Revision log. No previous correctness findings exist to re-check (editorial
phase is a single language pass per round), so this pass confirms that the
revised text breaks nothing.

## Findings

None. Every hunk in the diff is language-only; each revision was checked
against the passages and test strategies it leans on:

- "a list over either cap fails `safeParse` like any invalid reply" (Reply
  field) — "either cap" matches the two numeric caps in the schema quoted in
  the same paragraph (`max(200)` per endpoint, `max(64)` pairs) and matches
  Test strategy #1's two cases; no falsehood introduced.
- "a guard refusal takes the same `moves-apply-failed` outcome and preserved
  worktree as that of a rename error" (Apply) — still agrees with Test
  strategy #5 ("that refusal takes the `moves-apply-failed` class of #6"),
  the Risks TOCTOU row ("re-runs rules 4–7 as one guard"), and the Revision
  log's r0/r1/r3 failure-classes entry.
- "roll back in reverse, which is exact because the pairs are disjoint"
  (Apply) — agrees with rule 6's disjointness (including the no-prefix
  clause, which is what makes reverse rollback exact under `rename(2)`).
- "a failure, a returned issue or a throw from a rule, takes the existing
  schema-failure branch" (Wiring into retry) — dash-to-comma only; still
  agrees with Test #2 ("asserting a rule that throws still lands as
  `invalid-reply: moves`, not a crash") and Test #6's
  `retryInSameSession` classification.
- Rule 5/6 rewordings ("does not exist anywhere", "What remains") and the
  Context/Non-goals comma changes leave their A10/A12 citations and the
  Options-1 reference intact. No interface gains or loses a name or shape
  anywhere in the diff (`validateMoves`/`applyMoves` signatures, reply field,
  failure-class strings all unchanged).

## Checklist coverage

1. Assumption register: the diff touches no `verified` row's claim or basis;
   A8/A10/A12/A13 passages it cites are unaltered in substance.
2. URL spot-check (anti-hallucination gate): three cited sources fetched —
   man7.org `rename(2)` (atomicity, `EISDIR`/`EXDEV`/`EINVAL` — supports A8
   and the Apply/Options-4 text), git-scm `git-diff` (`diff.renames`
   "Defaults to true"; "The default similarity index is 50%" — supports A5
   and the `git show --name-status -M` claim), git-scm `git-cat-file`
   (`-t` reports the object type — supports rule 4 and the Compatibility
   matrix's git-version row). All live and supporting.
3. Internal contradictions: none introduced; revised Apply/Wiring sentences
   agree with Sweep interaction, Risks, and Test strategies #2/#5/#6/#11.
4. Edge cases: rule 5's parent-exists and rule 6's no-prefix clauses
   untouched in substance.
5. Interfaces: no name/shape drift across sections.
6. ADR decision coverage: items 1–4 all still decided as written in the ADR
   (item 3's "git plumbing" vs `fs.renameSync` is a recorded r0/OQ1 decision
   with a sound basis — rename detection is diff-time per A5, index update
   stays with the existing `git add -A` — not re-opened).
7. Revision rounds: no prior correctness blocker/major on record for this
   seat; nothing regressed by this diff.

Deferred section: the read-only-git deferral matches the ADR's own Deferred
clause ("deferred to a follow-up ADR. Items 1-4 stand without it") — no
contradiction, not re-opened.

## FYI (no severity)

- The hunk at "are added to the `implement`, `fix`, and `test-update` entries
  of\n`StageReplySchemas`\n(types.ts:44-46)" leaves the identifier stranded on
  its own line, slightly awkward prose flow — language-seat formatting call,
  not a correctness issue.
