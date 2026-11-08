# Performance review r1

verdict: approve

## Findings
- [minor] Chosen design — `src/opencode-runner.ts` (`waitForReply` change, proposal lines 63–66) and Risks table (proposal lines 83–89) — the design's expected numbers diverge from the research baseline without explaining the IO cost. Research baseline (01-research.md line 10): "`waitForReply` computes `deadline = Date.now() + POLL_TIMEOUT_MS`, polls `message.list` each 250 ms after the first immediate check" — i.e. the documented 250 ms / 30 s pair (README lines 34–35) bounds a hung run at ~121 `message.list` HTTP requests. The design keeps the 250 ms interval fixed (explicit non-goal, proposal line 16) while raising the wait cap to 86,400 s, a 2,880× amplification: a dead server under `--timeout 86400` absorbs ~345,600 polls, and the existing recursive `poll()`/`await sleep()` structure (opencode-runner.ts:259–271) keeps ~345k nested pending promises alive for the run's duration — bounded, but a design-induced 2,880× growth in both request volume and in-flight promise memory that no Risks row prices. The Risks table only treats the hang as elapsed wall-clock time ("lets a CI job hang for hours"), and A9/A16 justify the bound as a wall-clock/typo guard, never as an IO-volume decision. Fix: add one Risks row (or one sentence in Options considered) quantifying poll volume at the ceiling (~timeoutMs/250 `message.list` calls; ~345k at 86,400 s vs ~121 at the 30 s baseline), state why it is acceptable (single-client smoke test, opt-in, bounded cap), and note interval backoff as the obvious follow-up if long timeouts ever become normal.

## FYI (no severity)
- The poll-interval configurability non-goal (proposal line 16) is exactly the knob that becomes load-relevant at large `--timeout` values; worth a line in Options considered even though the decision to defer stands.
- Rejecting coreutils-style exit code 124 (Options considered, "Timeout reporting") is a consumer-experience/compatibility judgement, not a performance one; no perf objection from this review.
- Test strategy #8/#9 correctly use fake timers for the end-to-end timeout tests, so the new tests add no real-time cost to the suite.
