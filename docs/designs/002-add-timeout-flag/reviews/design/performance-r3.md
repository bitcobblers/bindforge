# performance review r3

verdict: approve

Scope: ONLY the change set in `reviews/design/editorial-r3.diff` (7 hunks: two
`honoring→honouring` spellings, `unit-suffix syntax, which we are not adopting`,
`avoid a second set of hard-coded bounds`, `pins the behaviour that research
line 43`, the comma in `without the stub result, main()`, and the dedup of the
doubled "Deliberately not changed this round" paragraph in the r3 revision
log). Cross-checked against `02-proposal.md` and `01-research.md`.

## Checklist evaluation

1. Input-path latency / per-event cost — every −/+ pair is orthographic; no
   wording on the wait path changed. The loop shape (immediate check, 250 ms
   `sleep()`, deadline tested after each reply check) is byte-identical on both
   sides of every hunk, as is the ≤ one-poll-interval overshoot language. Pass.
2. Structure costs — no algorithmic claim touched; the poll loop stays linear
   (`timeoutMs / 250` polls), no n² shape introduced. Pass.
3. Memory growth — the growth ceiling language (`pending-promise depth`, the
   ~14 400 cap) survives unchanged; the diff's paragraph dedup was verified
   line-by-line: the two copies removed on diff line 57 are textually identical
   and the retained line 58 keeps the register-statuses sentence, so no
   memory-relevant claim was dropped. Pass.
4. Startup cost — the only edit near boot-related prose (`"avoid second
   hardcoded bounds"` → `"avoid a second set of hard-coded bounds"`) is about
   constant duplication in code, not file reads or schema parsing; none changed.
   Pass.
5. Caching — no cache, memoization, or invalidation exists in the design or is
   introduced by the change set; item is not applicable rather than unevaluated.
6. Research baseline — all numbers inside touched paragraphs (~121 polls at
   30 s, ~14 400 at 3 600 s, ~345 600 at 86 400 s, 250 ms interval, ~119×) are
   byte-identical between the − and + sides and re-checked against the research
   baseline (`POLL_INTERVAL_MS = 250`, 30 s default): 30/0.25 + 1 = 121;
   3 600/0.25 + 1 = 14 401 ≈ 14 400; 14 400/121 ≈ 119. No new gap between the
   design's numbers and the baseline. Pass.

## Findings

None. The change set is language-only and moves no performance-relevant claim;
the performance story closed in performance-r1 (risks row pricing ~14 400
polls/pending promises vs ~121 baseline, cap justified, adaptive-interval
follow-up named) stays intact.

## FYI (no severity)

- Anchor drift in unchanged context: the r4 audit line says `captureFetch()` at
  `:53–61` while the A17 register row and the r3 log say the anchor was
  tightened to `:53–66`. Not performance; for the verification/editorial pass.
- The diff's dedup of the doubled revision-log paragraph is a formatting fix
  with identical retained content — no semantic delta (editorial outcome).
