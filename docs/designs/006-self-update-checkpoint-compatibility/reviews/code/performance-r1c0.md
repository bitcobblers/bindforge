# performance review r1c0

Seat: code performance (`references/performance.md`).
Input: `git diff 846ca090053b43ec99295a7cedafb47910043df9...HEAD -- ':(exclude)docs/designs'`,
`02-proposal.md`, `docs/adrs/006-self-update-checkpoint-compatibility.md`.
Round 1, pass c0. No previous findings from this seat.

**Verdict: approve** — no blocker and no major finding. Two minors, one nit.

## Evaluation note (tooling)

The shell tool was denied in this session, so I could not execute
`npm test` or a micro-benchmark. Every magnitude below is an analytical bound
from reading the code and the fixtures, not a measured number, and is labelled
as such. Checklist item 6 is otherwise inapplicable: the diff adds no
benchmark-like test and no timing note to run (the only `durationMs` values in
it are historical data inside the copied run-003 fixture, not assertions).

## Checklist

1. **Event/keystroke path** — pass, with a nit. Nothing here sits on a
   per-keystroke or per-event path: the new synchronous fs
   (`holdsRunnerBytes`'s `readFileSync`, `state-store.ts:253–264`) is reached
   only from `enforceOwnership` (runner.ts:284), which is already fully
   synchronous and dominated by its `git` subprocess calls per swept path.
   `stagePathAllowed` (gates.ts:261) is called only from the sweep filter
   (runner.ts:276) and from the moves validator (moves.ts:157), i.e. per swept
   path and per declared move pair — never per session node or per tool event.
   Finding 3 records the avoidable per-path work the new rule adds.
2. **Complexity** — pass. No new quadratic: the fence is one pass over the
   path's segments (`namesReservedSegment`, types.ts), the authorship test is
   a `Map` lookup, and the sweep remains one iteration over `touched`. Identity
   lookup is a map/set keyed by resolved path, which is the intended shape. The
   new cost is O(size of the checkpoint file) per sweep, folded into Finding 1
   rather than a complexity defect.
3. **Unbounded growth** — Finding 1 (minor). `gateLog` is the one persisted
   array the diff keeps growing with no bound and no documented bound, while
   the sibling `stageCommits` is explicitly windowed at 200 (runner.ts:379–384).
   The new in-memory registry (`runnerBytes`/`runnerDeleted`, state-store.ts:235–236)
   is bounded by distinct reserved paths and is documented as per-process, so it
   is the smaller half of the problem; the retained value is the whole
   serialization, which is what ties it to Finding 1.
4. **Startup** — pass. `pipeline status` gains exactly one `existsSync`
   (cli.ts:197) and `types.ts` compiles one anchored regex at module load;
   nothing on boot parses more than it did.
5. **Cache invalidation covered by a test** — pass. The authorship cache is
   invalidated by content comparison, and `ownership.test.ts:236–296` covers
   each branch: tampered checkpoint reverted (241–249), planted tmp removed
   (254–263), directory replacement reverted (268–281), runner's own save kept
   (286–296). What is *not* covered is the cost side — see Finding 2's fix.
6. **Benchmark-like test or timing note** — none in the diff to run (see the
   evaluation note). The new per-operation costs are stated nowhere in the
   design or the code as a number or an expectation; Finding 2 asks for the
   cheap form of that statement (a pre-check that makes the cost unnecessary)
   rather than a benchmark harness.

## Findings

### 1. minor — `gateLog` stays unbounded while the diff adds both a new writer to it and a new full-file reader of the array it inflates

- **Location:** `tools/orchestration/src/pipeline/runner.ts:1783–1800` (the
  unbounded push, untrimmed), with the new per-resume writer at
  `runner.ts:2078` (`terminalSaveFailed`); consumed as one serialized blob at
  `runner.ts:284` → `state-store.ts:253–264` and retained whole at
  `state-store.ts:235` via `recordRunnerWrite` (called at `state-store.ts:298`).
- **Problem:** `stageCommits` is bounded at 200 entries with an explicit
  comment (runner.ts:370–384), and the proposal names that bound as contract
  (§2: "`stageCommits` keeps its bounded-window meaning … as named contract,
  not comment"). `gateLog` gets no such sentence and no mechanism: every gate
  verdict, every escalation, and now one entry per split-record resume is
  appended forever, persisted into the checkpoint, re-parsed by `loadState` on
  every resume, re-serialized by every `saveState` (and by every seat
  `structuredClone` in `createStateWriter.enqueue`, state-store.ts:324). This
  change adds a consumer whose cost is linear in that blob on a new, repeated
  occasion: the ownership sweep runs at every stage and pass boundary
  (runner.ts:1673, 1724), and per the proposal's own §1 the checkpoint is dirty
  at every sweep, so the authorship test does one full `readFileSync` plus a
  full string compare of a monotonically growing file per boundary. That is an
  O(checkpoint size) × O(boundaries) cost with no bound on the multiplier and
  no documented expectation. It is not hot enough to break a run — the run-003
  fixture is 1,366 lines (roughly 55 KB analytical estimate, unmeasured), so
  one read+compare is sub-millisecond against a stage that runs for minutes —
  which is exactly why the growth should be named rather than assumed.
- **Fix:** do what the design already does for `stageCommits`: window
  `gateLog` the same way (a splice at a named cap in `gateLog()`), or state the
  bound in one sentence at runner.ts:1783 and in the §2 contract text ("bounded
  by the run's stage-and-attempt count, at the byte size the golden fixtures
  pin"), so the growth is a named contract instead of an accident. Do not make
  the escalation report's window (`report.ts:225` takes the last 8) the only
  thing that ever trims.

### 2. minor — the authorship test re-reads and re-compares bytes the runner itself just wrote, on every sweep

- **Location:** `tools/orchestration/src/pipeline/state-store.ts:253–264`
  (`readFileSync(abs, 'utf8') === bytes`), called at `runner.ts:284` for every
  swept violation; recorded at `state-store.ts:239–241` / `298`.
- **Problem:** the fence's cheap half (a map hit telling us the runner wrote
  this path) is paid for by the expensive half: a synchronous full-file read
  plus a full string comparison, per sweep, for a file that the same process
  wrote moments earlier in the same run. The re-read only has to catch a
  *different* writer, so a size or mtime pre-check decides almost every case
  without touching the content; the full compare then becomes the rare path
  (a same-size tamper), not the routine one. As written, the cost scales with
  Finding 1's unbounded array and buys nothing most of the time. There is also
  no cost note or test asserting the check stays cheap, so nothing keeps it
  from growing alongside the checkpoint.
- **Fix:** record `{ bytes, size }` (or a digest of the bytes) in
  `recordRunnerWrite` and, in `holdsRunnerBytes`, compare `statSync(abs).size`
  first and only then read and compare (or hash) the content; keep the content
  comparison as the final authority so the trust property is unchanged. One
  added `ownership.test.ts` case — same size, different bytes, still reverted —
  pins the path that still needs the full read.

### 3. nit — per-path work duplicated inside the sweep for every violation

- **Location:** `tools/orchestration/src/pipeline/runner.ts:276` and
  `runner.ts:284` / `290` / `291`.
- **Problem:** for each violation path the loop recomputes what it already
  knows: `resolve(repoDir, p)` is computed twice (284 and 290), and
  `namesReservedSegment(p)` runs a second time at 291 after
  `stagePathAllowed` already ran it in the filter at 276. `namesReservedSegment`
  also allocates a segment array plus a lowercased string per call. The cost is
  trivial against the `git` subprocess spawned per path, so this is worth
  tidying, not reworking — it is listed because checklist item 1 asks for
  avoidable allocations on a per-path loop, and here they are literally
  duplicated.
- **Fix:** hoist once per iteration — `const abs = resolve(repoDir, p)` and
  `const reserved = namesReservedSegment(p)` (or have the filter return
  `{ path, reserved }` pairs) — and reuse both in the authorship skip, the
  directory-clear, and the restore branch.

## FYI (no severity, out of specialty)

- `tests/pipeline/fixtures/checkpoints/run-3-live.json` copies the 1,366-line
  live run-003 checkpoint into the repo purely to pin parseability, and
  `checkpoint-compat.test.ts:26–35` parses each fixture twice (`JSON.parse` for
  the deep-equality oracle, then zod inside `loadState`). That is a
  repo-size/test-maintenance question for the testing seat, not a runtime one.
- `renderStatus` was split into `statusLines` + `renderStatus` (runner.ts:2781–2815);
  `statusLines` is consumed only by `runner.test.ts`. Harmless, but the export
  surface widened without a production caller.
