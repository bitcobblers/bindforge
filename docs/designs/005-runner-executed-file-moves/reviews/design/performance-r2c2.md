# Performance review r2 (ratification c2)

verdict: approve

Scope as given by the caller: the diff `44c52fa..82b346f` of `02-proposal.md`
(Read-only git paragraph, Risks row 1, Test strategy #13, Deferred item 3, and
the Revision log's read-only-git entry), plus the full current document and the
ADR. Ratification rule applied: report only where the revisions broke something
this seat approved or made a cited section false.

## Checklist walked against the changed text

1. **Input-path latency** — the changed text sits entirely on the audit and
   finalize/escalate paths (`auditSessionIndependence`, `finalizeEscalate`),
   never on a keystroke or event handler. The revision *removes* work on the
   plant path: "for this class `finalizeEscalate` (runner.ts:2008-2013) records
   the escalation in the run state instead of committing `ESCALATION.md`, so the
   escalation path runs no git over the plant" — one fewer git spawn and no
   index refresh over a hostile config. No new per-event cost.
2. **Structure costs** — no new loops. The audit matcher's input stays "one
   attempt's own session" (bounded by session size, unchanged wording). The
   O(n²) pairwise work of rule 6 is untouched and stays capped by `.max(64)`.
3. **Memory growth** — the escalation now lands in the run state instead of a
   file; one bounded record per halted stage, no new unbounded cache or table.
   Checkpoint growth stays under A13; no new field is named in the changed text.
4. **Startup cost** — unchanged; the shared `git()` env adds two `-c` keys and
   two env vars per spawn, which costs no extra process and no extra read.
5. **Caching/invalidation** — the new sentence "a planted repo-local
   `.git/config` survives every pin and is read by every runner git call —
   hence the halt" is accurate about the env's reach and does not create or
   invalidate a cache. The `headBefore` immutability my r2 fix leans on is
   untouched.
6. **Baseline gap** — no new expected numbers are introduced by the diff, so
   nothing to reconcile against the A12 baseline the proposal itself quotes
   ("Run 004 livelocked 6 fix↔test-update rounds over 23 inexpressible
   test-file moves"). Evaluated from the proposal and ADR only, per the input
   restriction on this pass.

## Findings

None. No changed section breaks or falsifies anything this seat approved in r2.

## Carried, not re-reported

- r2 minor (Apply — "Immediately before the first rename it re-runs rules 4–7
  as one guard across the inference→finalize window"): the guard's second
  validation pass and its per-pair `git cat-file -t` / per-`to` `headBefore`
  lookups are **unchanged** by this diff, so the finding is neither closed nor
  worsened. The Apply section and the Risks TOCTOU row ("`applyMoves` re-runs
  rules 4–7 as one guard immediately before the first rename") read exactly as
  approved. The proposed fix stands for implementation day: one snapshot read of
  the immutable `headBefore` tree (`git ls-tree -r --name-only <headBefore>`, or
  `git cat-file --batch` fed every `<headBefore>:<path>` endpoint query),
  consumed by both `validateMoves` and the apply guard; the disk-dependent
  halves of rules 4–7 still re-run at apply.

## FYI (no severity)

- Proposal → Read-only git, "This proposal also lands item 5, read-only git in
  the coder and test-engineer shells" — the ADR's Decision now says the opposite
  ("**Deferred — read-only git in stage shells** … It is deferred to a follow-up
  ADR. Items 1-4 stand without it"), and the Deferred section additionally hands
  the review seats' child-session `.git` route to the sandbox. Scope/ADR-
  alignment, not performance: for another seat.
