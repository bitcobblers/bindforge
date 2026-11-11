# performance review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (editorial confirmation pass, round 2). Inputs:
`02-proposal.md`, `docs/adrs/006-self-update-checkpoint-compatibility.md`, the Revision log,
and my own previous findings (none — the editorial phase is a single language pass per round).

## Findings

None.

## Checklist evaluation (against the diff)

1. **Input-path latency / per-event cost** — Pass. The diff touches only prose in Goals, §2
   (May-refuse message wording), §3 (loadState directory-repair bullet), §4 (acquireLock
   sentence grammar), and Test #12. No new per-event or per-keystroke work is introduced.
   §4 preserves the bound exactly: "one `'wx'` retry" survives the rewording unchanged — the
   editorial pass did not turn the single retry into a retry loop or add polling.
2. **Structure costs (O(n²) on unbounded n)** — Pass. The reworded §3 bullet ("catch
   `EISDIR`/`ENOTDIR`/`ENOTEMPTY` alongside today's `ENOENT` handling") describes the same
   single error-handling branch on one path; no nested iteration is added anywhere in the diff.
3. **Memory growth** — Pass. The diff adds no cache, table, or in-memory structure. §1's
   in-memory "bytes `saveState` last wrote" snapshot (unchanged by this diff) stays bounded to
   one file's worth of content per checkpoint.
4. **Startup cost** — Pass. The diff adds no file reads or schema parsing at boot; the §2
   change is to the refusal *message* text only ("gains that sentence" → "gains that
   wording"), not to the load path's work.
5. **Caching / invalidation** — Pass. Nothing in the diff touches the §1 authorship-test
   snapshot or its invalidation ("still holds the bytes `saveState` last wrote"); the reworded
   §3 and §4 text preserves the same behavior contract, so the invalidation rule cited earlier
   in the proposal remains true as written.
6. **Research baseline gap** — Not applicable to this diff and not evaluable in this pass:
   `01-research.md` is not among the inputs the caller permits for this seat. The diff itself
   introduces, removes, or changes no expected numbers, budgets, or cost claims, so it cannot
   have created a gap against the baseline. No numeric claim appears in any hunk.

## Consistency notes (no finding)

- Goals bullet "tmp shape" now matches the §1 term "matches the `saveState` tmp shape"
  (`state-store.ts:186`) — the rename from "tmp pattern" improves internal consistency with
  the mechanism text and changes no semantics.
- Test #12's voice change ("the checkpoint is never written outside the runner's own write
  path") states the same invariant the fence implements; it does not weaken or strengthen the
  guarantee any performance section depends on.
- The Revision log's five r1 entries record no performance-relevant decision, and none of the
  diff's rewordings contradicts them.

## FYI (no severity)

- Editorial only: all five hunks preserve meaning; no section cited by any performance
  reasoning in the proposal was made false by the diff.
