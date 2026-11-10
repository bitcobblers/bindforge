# Performance review r1 (pass c0, whole document)

verdict: approve

Inputs read: `02-proposal.md` (incl. Revision log), `docs/adrs/004-mirror-src-hierarchy-in-tests.md`,
and `01-research.md` for the baseline numbers the checklist requires quoting.
Revision log shows no prior findings rejected (r1 initial), so nothing below is a re-report.

## Findings

- [minor][new] Test strategy #2 + Risks row 4 + Rollout — "with the same total test count as the
  pre-move baseline" and "the reported file/test counts are the acceptance evidence" — the design's
  acceptance evidence is a count comparison whose baseline exists nowhere in its inputs and is
  specified to be taken after the thing it baselines is gone. The research baseline it leans on
  records only *file* counts (26 = 23 `pipeline-*` + 3 root) and 65 import specifiers; no total
  test/assertion count appears in the research, the ADR, or this proposal. Test strategy #1's
  "assertion counts per file are identical before and after the move" is unexecutable for the same
  reason. Worse, the ordering makes the baseline unrecoverable at the point it is used: Rollout 2
  lands the commit (`git mv` ×23, "Land the single commit") and Rollout 3 then says "Verify Test
  strategy items 1–6 locally", so the flat tree is no longer on disk when the comparison is
  performed. Fix: insert a Rollout step 0 *before* the moves — run `npm run check` at the pre-move
  commit and record the reporter's file and test totals (plus per-file counts if item 1 is kept)
  as the named baseline, in the artifact the rollout already points at for evidence — and re-point
  Test strategy #1/#2 and the Risks "pre-move baseline" cell at that recorded run. One added line
  plus one pointer replaces the unanchored phrase; it also collapses the verification cost, since
  the single pre-move `npm run check` and a single post-move `npm run check` satisfy items 1, 2, 4
  and 5 together, instead of the four-plus suite invocations the strategy currently implies.

## Checklist coverage (evaluated, no finding)

1. Input-path latency: no keystroke/event path is touched — the change is `git mv` plus a uniform
   `../src/` → `../../src/` rewrite (Chosen design ¶2); no allocation, table rebuild, or
   synchronous IO is introduced. Nothing in the design adds per-event cost.
2. Structure costs: work is linear in a bounded n — 23 renames, 65 specifier edits (A8). The
   convention rule (¶4, "new subsystems follow the same shape") is the design's answer to the ADR's
   growth objection (each new subsystem adds a directory instead of another prefix to one flat
   list); no bindings × devices × platforms product appears.
3. Memory growth: no table, cache, or index is introduced or resized; the only structures named are
   `TEST_PATH_RE` and the ownership checks, which match per path and are asserted unchanged (A7, A11).
4. Startup cost: explicitly covered — vitest discovers through its existing default recursive
   `include` with no new config to parse (A5), and TypeScript never reads the test tree at all
   (`rootDir: "./src"`, `exclude: ["tests"]`, A4), so typecheck/boot cost cannot move. Same 26 files
   discovered either way.
5. Caching/invalidation: nothing in the diff is cached, and the one stale-reference window the
   change creates — an orchestration run in flight holding the old paths — is gated by Rollout 1 and
   Risk row 1 (A18), matching ADR Ordering ("land directly between runs"). The carve-out (¶2) closes
   the one *content* stale reference the move creates.
6. Research baseline vs. design numbers: all load-bearing numbers agree with 01-research.md — 26
   files / 23 + 3 (A1), one-for-one name match with no collisions (A2), 13 `src/pipeline/` modules
   rather than the ADR Context's 14 (A3, recorded in Deferred rather than copied forward), 65
   `../src/...` specifiers (A8), one real in-test comment reference vs. inert fixture strings (A14),
   two convention surfaces not one (A13). The only unexplained numeric gap is the test-count
   baseline, reported above.

## FYI (no severity)

- Compatibility: A16 (case-insensitive checkouts) is self-labelled `unverified` and Test strategy #7
  leaves macOS/Windows on inference only; the compatibility matrix marks those rows "Expected —
  assumption". Sound as a stated assumption, but it is the compatibility seat's call whether the
  ADR's platform silence is enough.
- Correctness/process: the Context says "ADR-004 … decides to move those 23 files" while the ADR's
  front-matter status is `proposed`; the deferral of the "14 modules" correction to a future edit is
  consistent with ADR-004 as written, so it is not re-opened here.
- Test strategy #3 depends on vitest's positional argument being a path-*substring* filter (A6); a
  scoped-run claim about exclusion is a correctness/compatibility question, not a performance one.
