# Code review — performance (round 1, ratification pass c2)

Verdict: **approve** (zero open blocker/major; one new nit)

Scope as given: `git diff 48c80bde34da123b8dd0306343c361617d5cb227..HEAD --
':(exclude)docs/designs'` — `docs/workflows.md`, `src/pipeline/cli.ts`,
`src/pipeline/runner.ts`, `tests/pipeline/{cli,restart-boundary-integration,runner}.test.ts` —
read against `02-proposal.md` (Decisions 1–4, Test strategy) and
`docs/adrs/020-restart-boundary-for-harness-self-changes.md`. This pass asks only
whether the post-r1 fixes broke what this seat approved; the checks below are
therefore pointed at the diff's own additions.

## What the later fixes touched that this seat owns, and what it costs

1. **`exitCodeFor` escalated arm; `main`/`dispatch` split** (`cli.ts:53-63`,
   `149-166`): one extra string-compare branch on a terminal result, and a
   `try`/`catch` around the dispatch half. No cost on any path a run repeats.
2. **Writer-side `printable` re-applied at render** (`runner.ts:2539-2546`,
   `3648-3653`): two array `map`s over `record.paths`, which `boundedSurfacePaths`
   already caps at `MAX_GATE_LEGS + 1` = 33 entries — and each site runs once per
   pause / once per crossing. Negligible, and it is what the design's §2
   writer-side posture asks for.
3. **`surfacePathCount` + `SURFACE_ELISION_RE`** (`runner.ts:1894-1917`): O(1)
   per render — a length guard, then one anchored `exec` on a single element, no
   backtracking risk in `\d{1,9}`. The regex is a **module constant**, so it does
   not recompile per call, and the count is computed once in
   `finalizeRestartBoundary` and reused at its three render sites instead of
   re-deriving per line. Net: no worse than r1's `record.paths.length` reads.
4. **`templateText` pinned on `StageHandle`** (`runner.ts:1248-1258`,
   `1352-1360`, `2661-2676`, `2708-2716`): this is a **performance gain** over
   what r1 approved. Attempt 2 no longer falls through to
   `loadTemplate(opts.promptsDir, template)`, so a retried stage drops one fs
   read, and the stage-entry read at `2646-2650` stays the only one per stage.
   The pinned text is not persisted: `StageHandle` is a drive-loop local and
   `StageResult`/`state.results` (`types.ts:81`) carries no `templateText`, so the
   checkpoint — which every later save clones — does not grow by a template body.
5. **`statusLines` recent-gates filter** (`runner.ts:3951-3961`): new; see
   finding 1.

## Performance checklist

1. **Event/keystroke path** — nothing in this diff sits on a per-event or
   per-keystroke path. Every touched function is terminal-or-operator facing:
   the pause finalize, the resume crossing, the escalate finalize, one CLI exit
   mapping, one status render. No new synchronous fs or network is added; the
   only fs read the diff affects is the template read, and it *removes* a repeat
   of it. **Pass.**
2. **Complexity** — no new nested loop over an unbounded collection.
   `surfacePathCount` O(1); `boundedSurfacePaths` O(min(n, 32));
   `finalizeRestartBoundary` walks its ≤33-entry list a constant number of times.
   The one new linear-over-the-whole-collection step is the `statusLines`
   `filter` (finding 1, nit). **Pass with one nit.**
3. **Unbounded growth** — the boundary record stays capped
   (`boundedSurfacePaths`), which is what this diff touches. `state.gateLog` is
   still uncapped in code (`state-store.ts:180-188` puts no max on the array;
   only the 8 MiB checkpoint-read ceiling, `state-store.ts:201`, bounds it from
   the read side) — pre-existing and deliberately so: the design counts
   boundaries only in the gate log (Deferred row 3). This diff adds at most one
   entry per boundary. **Pass.**
4. **Startup** — `cli.ts`'s new `import { printable } from './independence.js'`
   (`cli.ts:10`) adds no boot work: `cli.ts` already pulls `./runner.js` for
   `runPipeline`/`resumePipeline`/`renderStatus`, and `runner.js` imports
   `independence.js`. The module is in the graph before this change; the added
   binding costs nothing at boot, and `independence.ts` does no top-level fs.
   **Pass.**
5. **Cache invalidation covered by a test** — the stage-entry template pin is
   the diff's only cache, and its invalidation rule ("never re-read mid-stage")
   is pinned non-vacuously: `runner.test.ts:1341-1373` rewrites `implement.md`
   on disk *between* the attempts, asserts both attempts hash to the entry sha,
   and asserts `templateHash(rewritten) !== expected` so the pin, not two reads
   of one file, is what the test proves. **Pass.**
6. **Benchmark-like test or timing note** — the design's only cost claim is A12
   (a boundary costs one process start "of seconds" against minutes-long stage
   windows), and it is marked `unverified`: no benchmark test ships with it and
   this diff adds none, so there is nothing new to run for A12. The
   `while (code === 3)` driver-loop pin (`restart-boundary-integration.test.ts`)
   counts processes, not time. I ran the suite myself for the numbers below.
   **Evaluated; numbers quoted.**

## Measured on this box (quiet, `tools/orchestration`)

- `npx vitest run --maxWorkers=2` → **54 files, 607 passed | 2 skipped (609),
  Duration 21.63s.** Green, matching the runner's gate leg
  (`npm run check`, exit 0).
- The three files this diff touches, run together → **49 passed, 2.61s**;
  `runner.test.ts` alone, including the new 34-file cap fixture → **34 passed,
  2.55s**. The new fixtures (34 probe templates, a sabotaged `check` script, a
  planted record) cost no measurable suite time.
- Contention note, so the record is honest: my first unconstrained
  `npm run check` reported 16 tests timing out at the 5000 ms default, spread
  over files this diff never touches (`light-flow`, `funnel`, `pass-finalize`,
  `own-report-seat`, `checkpoint-integration`, …). Those same files pass in
  isolation (19 passed, 2.85s) and pass in the `--maxWorkers=2` run, and vitest
  reported 54 isolated workers at ~1.51s startup each. That is worker contention
  on this box, not this change.

## Findings

### nit 1 — `tools/orchestration/src/pipeline/runner.ts:3959-3961`

**Problem.** The new recent-gates filter re-tests the loop-invariant
`state.restartRequired !== undefined` for every entry and copies the *whole*
`gateLog` before slicing its last six lines. `statusLines` is a one-shot CLI
render (`cli.ts:259` is its only caller), so this is invisible in practice — but
it is an avoidable O(len(gateLog)) allocation on a collection the code never
caps (item 3), and `gateLog` is exactly the array that grows across a run's
rounds and resumes. It also costs the common case (no record standing) a full
copy that filters nothing.

**Fix.** Hoist the predicate out of the callback and skip the copy when no
record stands:

```ts
const standing = state.restartRequired !== undefined;
const recent = standing
  ? state.gateLog.filter((g) => g.gate !== RESTART_BOUNDARY_GATE_NAME)
  : state.gateLog;
// … ...recent.slice(-6).map(…)
```

Same output, one comparison per render instead of per entry, and no array copy at
all on the no-record path.

## Carried from r1, not re-filed

`gates.ts:566-569` (`prefix.slice(0, -1)` re-evaluated inside
`touchesExecutableSurface`'s callback, where both spellings are compile-time
constants) is untouched by this range — `gates.ts` does not appear in the diff.
It stands exactly as I recorded it at r1: still open, still a nit, and nothing
in these fixes made it hotter or colder. Not counted against this change.

## FYI (out of specialty, no severity)

`tests/pipeline/runner.test.ts:649-651` restates `MAX_GATE_LEGS` as the literal
`MAX_RECORD_PATHS = 32`, with a comment naming the coupling. The cap test's
shape (and the `SURFACE_ELISION_RE` digit width that has to hold its count) now
needs two coordinated edits if the cap ever moves.
