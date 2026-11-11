# security review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` only (editorial confirmation pass, round 2),
checked against `02-proposal.md`, `docs/adrs/006-self-update-checkpoint-compatibility.md`,
and the Revision log. My previous security findings (r1): none, so this pass confirms
that the editorial revisions did not break approved text or make any cited section false.

## Findings

None. Each security-relevant passage touched by the diff was verified against the
checklist:

1. **Trust boundaries** — unchanged. The diff's Goals edit ("or its tmp shape")
   preserves §1's fence as the source of trust for the worktree copy; the committed
   copy stays the recovery anchor. No data-source classification moved.
2. **Parsing untrusted data** — not touched by the diff.
3. **Path handling** — the reserved-name matcher (segment-wise, case-insensitive,
   `.pipeline.json` / `.pipeline.json.<8 hex>.tmp` / `ESCALATION.md`) is unchanged;
   "pattern" → "shape" now matches the single defined term in §1 ("the `saveState`
   tmp shape"), removing a synonym that could have widened a matcher reading. Net
   security-neutral to positive.
4. **Secrets and PII** — the diff adds no storage or logging surface.
5. **Dependencies** — the diff names no new dependency; the Node `>= 22.23.4` floor
   row is untouched, so no new advisory exposure.
6. **Least privilege** — the §4 articles-only rewrite keeps the `'wx'` atomic
   exclusive create, live-holder throw, dead-holder clear with one retry, and
   racing loser re-read-then-throw; single-winner guarantee intact.
7. **Fail closed** — the reordered §3 bullet ("catch `EISDIR`/`ENOTDIR`/`ENOTEMPTY`
   alongside today's `ENOENT` handling") remains additive error handling that
   produces the named repair instead of a raw errno; rejection stays clean, no
   partial apply. Test #12's rewrite to "the checkpoint is never written outside
   the runner's own write path (fence untouched)" states the fence invariant at
   least as strongly as the active-voice original.

Deferred items (lock-ID nonce, pre-fence checkpoints runs 001–007) were re-checked
against the ADR: both track the ADR's own interim posture (ADR-006:55–57) and its
Decision scope; no contradiction. The §2 refusal-message wording matches the ADR's
two named postures, per the r1 revision-log entry.

## FYI (no severity)
- Test strategy #12 — the passive rewrite shifts the asserted subject from the run
  to the checkpoint; semantics preserved, but if the test seat wants the actor
  explicit, that is an editorial/test-strategy call, not a security one.
