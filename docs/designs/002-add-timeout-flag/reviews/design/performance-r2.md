# Performance review r2

verdict: approve

## Findings

No open findings. Every performance checklist item evaluates clean against the
whole document as it stands after r2–r4 revisions, and the single r1 finding is
confirmed closed in the body (not just in the revision log).

Checklist evaluation:

1. **Event-path latency cost.** No per-event cost is added to the polling path.
   `waitForReply(client, sessionID, timeoutMs)` only swaps a module constant for
   a parameter (Chosen design, `src/opencode-runner.ts` bullets); the resolver
   regex, range check, and `× 1000` each run once per CLI invocation before
   `createClient()`. Verified against the real loop at
   `opencode-runner.ts:259–271` — the design adds no allocation, table rebuild,
   or synchronous IO inside it.
2. **Structure costs / O(n²).** Poll volume and the chained-pending-promise
   depth grow linearly (`timeoutMs / 250`), not quadratically, in `n`, and `n`
   is now bounded by `MAX_REPLY_TIMEOUT_MS` (3,600 s) and enforced at both
   boundaries (CLI resolver and the fail-closed `TypeError` in `runPrompt`).
   No unbounded-n structure introduced.
3. **Memory growth.** Bounded and priced: the Risks row "A valid but long
   timeout lets a hung run hammer the server" states ~14,400 polls and ~14,400
   nested pending promises at the cap vs ~121 at baseline (~119×). I re-verified
   the arithmetic against the code: 30 s / 250 ms + immediate check ≈ 121 polls;
   3,600 s / 250 ms ≈ 14,401. The recursion at `opencode-runner.ts:259–271`
   (`await sleep(); return poll()`) does keep the chain of unresolved
   `poll()` promises alive, so the promise-memory model in the Risks row is
   accurate.
4. **Startup cost.** The design adds only O(1) work at boot (one regex test,
   `Number.isInteger`, one `JSON.stringify` on the error path) and the
   advisory-only `engines` line; no new file reads or schema parsing.
5. **Caching / invalidation.** No cache or translation table is introduced
   anywhere in the change set — item is vacuously satisfied, and the design
   claims none.
6. **Baseline alignment (research quote).** The design quotes and honors the
   research baseline (01-research.md line 10 / README 34–35: 250 ms / 30 s,
   ~121 polls) and explicitly prices the divergence at the new cap with a
   justification (single-client, opt-in smoke test; A16 labels the cap a
   product decision) — the gap is explained, not unexplained.

r1 closure check:

- **[minor] performance-r1 (poll-volume divergence from baseline unpriced)** —
  CLOSED. The Risks row now quantifies ceiling IO and promise growth (~14,400
  vs ~121), the cap was lowered 86,400 s → 3,600 s on this abuse model, the row
  states why the ceiling is acceptable, and Options considered names the
  adaptive-poll-interval follow-up.
- **r1 FYI (poll interval is the load-relevant knob)** — addressed in Options
  considered ("Poll interval configurability"), which records the load
  relevance and points to the Risks table. Deferral itself remains a legitimate
  non-goal decision, outside this review to reopen.
- **Test suite runtime** — Tests #1–#6, #10–#11 use fake timers or reject
  synchronously before any session/HTTP work (Test #5: real timers but a
  synchronous `TypeError`, zero requests), so the new tests add no real-time
  cost to the suite.

## FYI (no severity)
- The chained-promise depth at the cap (~14,400 unresolved `poll()` promises)
  is linear and bounded, so it is not a perf blocker; if long timeouts ever
  become normal practice, converting `poll()` to a loop (removing the chain) is
  a cheaper follow-up than making the interval adaptive and belongs to whoever
  revisits the poll-interval non-goal.
