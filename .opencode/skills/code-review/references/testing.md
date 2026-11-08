# Code testing checklist

1. Every numbered item in the proposal's Test strategy has at least one
   test citing it (`// proves: Test strategy #N`).
2. Both unit and integration layers exist where the proposal promises
   them.
3. Revision rounds: each previously rejected blocker/major now has a test
   that would have caught it.
4. Tests are deterministic: no wall-clock, no real network, no live
   server, no port collisions.
5. Test names describe behavior, not implementation steps.
6. Run `npm run check` in the affected package yourself and paste the
   tail of the output in your report. If it is not green, verdict is
   request-changes — the runner verifies independently.
7. Untestable additions (config knobs without consumers, dead branches)
   are findings, not passes.
