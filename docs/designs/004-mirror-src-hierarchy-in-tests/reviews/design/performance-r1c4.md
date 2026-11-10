# performance review r1c4 (ratification pass)

verdict: approve

Scope: diff of 02-proposal.md between blobs 1f3a83c..1d1e68a, then the full
current document, checked only against what the performance seat approved in
r1 (checklist items, chiefly #6 — research baseline vs the design's expected
numbers). Inputs: 02-proposal.md, ADR-004, the Revision log, my r1 findings.

## Re-check of my r1 finding — closed

- [closed] Test strategy #1–#2 / Rollout / Risks — r1 minor: "the acceptance
  evidence is a count comparison with no baseline in any input and no step
  that captures one before it disappears."
  The revision closes it without breaking the surrounding text:
  - Rollout 1 now defines and anchors the baseline in time, before any move:
    "Then land the test-only commit with item 4's assertions ... That commit
    is the pre-move baseline for Test strategy #1–2 and Risks 4." The added
    `expect`s sit inside the existing `stagePathAllowed` block, so "test and
    file counts are unchanged" — the baseline count stays the ADR's
    "unchanged test count" gate (ADR Consequences), so the gate is not moved.
  - Risks 4 now compares "the vitest reporter's file/test counts to the
    Rollout-1 baseline" (was: an unstated "pre-move baseline").
  - Test strategy intro cites "the pre-move baseline Rollout 1 defines", and
    #1/#2 compare against it, so the flat tree no longer has to be on disk
    when the comparison is specified. Rollout 3's "reported file/test counts
    are the acceptance evidence" is now anchored to a named run on both sides
    of the comparison. The r1 ordering defect (verify-after-commit with no
    pre-move capture) is gone.

## Checklist walk (current text)

1. Input-path latency: N/A — the change moves test files; no keystroke/event
   path is touched.
2. Structure costs: N/A — no new data structures; the only growth is one
   directory tier.
3. Memory growth: N/A.
4. Startup cost: unchanged — no config change (¶3), vitest default recursive
   include (A5), tsconfig still excludes `tests/` (A4).
5. Caching/invalidation: N/A — A19's enumeration keeps the gate's single
   path classifier authoritative over the new nesting; no cache involved.
6. Research baseline: consistent. The design's counts now decompose against
   cited greps rather than against unstated numbers: 26 files = 23 moved + 3
   root (matches research and ADR); A8's 87 `../` occurrences = 65
   from-imports + 2 non-import + 16 `new URL` + 4 inert fixture strings,
   which explains the only former gap versus the research's 65 import
   specifiers; the previously missing total-test-count baseline is supplied
   by the Rollout-1 run instead of a research number, which the text now
   states.

## Findings

None. No r1-approved material was broken by the revision, and no section the
performance seat cited (Test strategy #1–#2, Rollout 3, Risks 4 baseline
cell) is false in the current text.

## FYI (no severity)

- A8's four-class decomposition may overlap by one line: the "2 non-import
  `../src/` sites at `cli.test.ts:29-30` (`vi.mock`, type-import)" include a
  type-import that would match the `from ['"]\.\./src/` grep reported as 65,
  yet the classes are listed additively to 87. Arithmetic/consistency, not
  performance — correctness seat's call.
- Rollout 2 amends ADR-004 Decision bullet 2 in the same commit that rewrites
  16 `new URL` sites, so the proposal's ¶2 diverges from the ADR text as it
  stands on disk until that clause lands. Disclosed in ¶2, Deferred, and the
  Revision log; governance for the compatibility seat.
