# Research: Run review passes across up to N parallel inference slots (ADR-003)

## Summary

ADR-003 replaces ADR-001's sequential inference lane with a bounded fan-out:
the seats a pass selects under ADR-002 launch in fixed order, run up to N at a
time, and finalize once per pass, while every other state-machine semantic
stays as written (source: docs/adrs/003-parallel-inference-lane.md). Today's
runner has no seam for this: `executeStage` mixes the inference phase and the
git finalize in one function, and seats run in a strictly awaited loop
(source: tools/orchestration/src/pipeline/runner.ts). External tooling —
eval harnesses, test runners, build tools — converges on the same shape: an
integer concurrency cap enforced by runner code, never by model prompts,
sized against server capacity (source: https://inspect.aisi.org.uk/reference/inspect_eval.html,
https://www.promptfoo.dev/docs/configuration/rate-limits/). The server-side
capacity that caps N is well documented: KV-cache memory and slot count
bound batch concurrency in llama.cpp and vLLM alike (source:
https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md,
https://docs.vllm.ai/en/stable/configuration/optimization/). The main design
risks the proposal must resolve are the checkpoint last-writer-wins race and
the timeout/queueing interaction.

## Findings

### What the current code forces the design to touch

- `executeStage` is one function that renders the prompt, creates the session,
  polls/nudges, writes the transcript, appends to `state.results`, *and* runs
  the git finalize (`enforceOwnership` + `commitStage`) when the outcome is
  ok; `headBefore` is captured per stage inside it. The ADR's two-phase split
  is therefore a split of this single function, not a new layer
  (source: tools/orchestration/src/pipeline/runner.ts, lines 603–930).
- Review seats run strictly one at a time in the drive loop —
  `for (const specialty of plan.seats) { await runWithRetry(...); await
  saveState(...) }` — and that loop is the exact fan-out seam; the pass-end
  aggregation (record `plan.seats` verdicts, then evaluate the gate) already
  happens only after all seats settle
  (source: tools/orchestration/src/pipeline/runner.ts, lines 1282–1340).
- A pass holds 1–5 seats chosen by `planPass` (full at `c0`, dissenters only
  at later passes, stale approvers at ratification) — this ADR schedules that
  list without changing it (source: tools/orchestration/src/pipeline/panel.ts).
- Gates are already order-insensitive exactly as ADR-003 claims:
  `verdictsFor` takes the *last* valid reply per `stage:round` (a retry
  supersedes its attempt), and `allReviewsApproved` denies on missing seats or
  rejections without inspecting arrival order — completion-order
  nondeterminism at N > 1 cannot change a verdict
  (source: tools/orchestration/src/pipeline/gates.ts, lines 81–168).
- All review seats share one ownership rule (`stagePathAllowed` →
  document/review stages may change only `docs/**`), so a single pass-level
  ownership sweep is sound for review passes; the per-stage sweeps today run
  inside `executeStage` per seat (source: tools/orchestration/src/pipeline/gates.ts,
  lines 252–273; tools/orchestration/src/pipeline/runner.ts, lines 895–928).
- The config surface exists but the knob does not: `PipelineConfig` has
  `stageTimeoutMs` (default 30 min) and no `maxParallelReviews`
  (source: tools/orchestration/src/pipeline/stages.ts, lines 26–72); the
  pipeline CLI has no `--max-parallel` (source: tools/orchestration/src/pipeline/cli.ts).
  The "strict fail-closed" env pattern the ADR points at is
  `resolveReplyTimeoutSeconds` in the sample CLI: a `^\d{1,5}$` format gate on
  the env string that runs whether or not the flag overrides it, range check
  as a separate error, and validation before any side effects
  (source: tools/orchestration/src/cli.ts, lines 86–137).
- Checkpoint writes are already the shared mutable surface: `saveState`
  validates, serializes, writes a unique temp file, and renames over
  `.pipeline.json`; today every call is awaited in sequence, so ordering is
  implicit. Concurrent calls from inside a pass would each succeed and the
  last rename to land wins — the ADR's single drained write path is what turns
  that from luck into an invariant (source: tools/orchestration/src/pipeline/state-store.ts).
- Retry semantics that must survive fan-out are per seat: one retry (same
  session for `invalid-reply`, fresh session after a zombie-cancelled timeout),
  then escalate via `StopDriving` (source: tools/orchestration/src/pipeline/runner.ts,
  lines 1104–1148).
- `editorial-review` is registered with `kind: 'review'` and runs a
  sequential five-seat loop in the drive loop with its own branch — the ADR's
  "applies to stages with `kind: 'review'`" wording covers it, so its commit
  shape and finalize timing change too (source: tools/orchestration/src/pipeline/stages.ts,
  lines 107–111; tools/orchestration/src/pipeline/runner.ts, lines 1151–1191).
- `.pipeline.lock` is a repo-level, PID-checked single-process lock; ADR-003
  leaves it unchanged, so parallelism is strictly intra-run
  (source: tools/orchestration/src/pipeline/runner.ts, lines 313–353).

### Performance baselines (lens)

- The ADR's own budget: a full-panel or ratification pass shrinks from 5× to
  ceil(|panel|/N)× the serial wall clock; dissent passes are already 1–3
  sessions, so ADR-002's funnel narrows exactly the passes this ADR
  parallelizes — the two multiply (source: docs/adrs/003-parallel-inference-lane.md).
- The serial cost being attacked: ADR-002 budgets 40 design + 20 code reviewer
  sessions per run and every session is serialized; run `002-run2` spent 40
  design sessions across 8 rounds
  (source: docs/adrs/002-funneled-review.md).
- Server capacity is the hard bound on N: llama.cpp exposes `-np, --parallel
  N` = "number of server slots" (default -1 = auto) with continuous batching
  enabled by default, and its unified-KV option sizes the shared KV pool to
  `n_parallel * N` — memory per slot is what makes slots a capacity decision
  (source: https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md).
- vLLM frames the same trade: throughput "requires batching sufficiently many
  requests," KV-cache memory "is huge and grows and shrinks dynamically," and
  a conservative KV-cache sizing "caps batch concurrency (and therefore
  throughput)" (source: https://arxiv.org/abs/2309.06180,
  https://docs.vllm.ai/en/stable/configuration/optimization/).
- Comparable harness knobs set small defaults: Inspect AI caps concurrent
  model-API connections at 10 by default and bounds parallel samples with
  `--max-samples` (source: https://inspect.aisi.org.uk/reference/inspect_eval.html);
  promptfoo's `evaluateOptions.maxConcurrency` defaults to 4 (source:
  https://www.promptfoo.dev/docs/configuration/reference/). A default of N = 1
  (ADR-003) is therefore more conservative than peers, which matches the
  ADR's "unexercised path" posture (source: docs/adrs/003-parallel-inference-lane.md).
- No published latency/throughput number exists for the OpenCode server itself
  under concurrent sessions — see Open questions.

## Prior art (lens)

- Bounded-concurrency primitives in JS are mature: p-limit is "run multiple
  promise-returning & async functions with limited concurrency" and p-queue is
  a "promise queue with concurrency control" with zero dependencies (source:
  https://github.com/sindresorhus/p-limit,
  https://github.com/sindresorhus/p-queue). ADR-003 instead specifies a
  ~20-line in-process semaphore with no new dependency — consistent with
  orchestration.ts's existing dependency-free style for lane machinery
  (source: docs/adrs/003-parallel-inference-lane.md).
- LLM fan-out runners all enforce the cap in harness code, never in prompts:
  promptfoo's `-j, --max-concurrency` and `--delay` are CLI/config options
  (source: https://www.promptfoo.dev/docs/usage/command-line/), Inspect AI's
  `--max-connections` / `--max-samples` are eval options (source:
  https://inspect.aisi.org.uk/reference/inspect_eval.html). This supports
  ADR-001/003's rule that fan-out stays runner code, not agent prompts
  (source: docs/adrs/001-code-driven-agent-pipeline.md,
  docs/adrs/003-parallel-inference-lane.md).
- The jobs-knob naming convention is `-j/--jobs` (cargo's `-j N --jobs N`,
  "number of parallel jobs to run"; promptfoo also uses `-j` aliased to
  `--max-concurrency`); jest's equivalent is `--maxWorkers` (alias `-w`)
  (source: https://doc.rust-lang.org/cargo/commands/cargo-build.html,
  https://www.promptfoo.dev/docs/usage/command-line/,
  https://jestjs.io/docs/cli). `--max-parallel <N>` fits this family; none of
  these tools has won a single shared name, but all default conservatively
  and require opt-in to raise (cargo defaults to logical CPU count; promptfoo
  4; Inspect 10) (source: as cited above).
- promptfoo additionally *adapts* concurrency when the server signals rate
  limits (halve on hit, +1 after sustained success); ADR-003 deliberately has
  no adaptive path — N is a static operator decision bounded by slot count.
  Worth noting as the obvious future knob, not part of this decision (source:
  https://www.promptfoo.dev/docs/configuration/rate-limits/).
- Failure handling under the cap is the same shape as the ADR's: promptfoo
  retries errored calls under the same scheduler and resumes evals by skipping
  completed test/prompt pairs — retries stay inside the concurrency budget,
  matching "a retry consumes a slot" (source: https://www.promptfoo.dev/docs/usage/command-line/,
  docs/adrs/003-parallel-inference-lane.md).

## Platform notes (lens)

- Package engines are `node >= 22.12.0`, so nothing in the fan-out (async
  iteration, `AbortSignal.timeout`, promise combinators) needs a version
  floor above the existing one (source: tools/orchestration/package.json;
  AbortSignal.timeout is a documented Node global:
  https://nodejs.org/api/globals.html).
- JavaScript's event loop is single-threaded (source:
  https://developer.mozilla.org/en-US/docs/Web/JavaScript/Event_loop).
  Consequences for this design: (1) `state.results.push` and the semaphore
  counter can never tear — the only races are at `await` boundaries, which is
  exactly the interleaving the ADR removes by pass-level finalize and the
  drained write path (source: docs/adrs/003-parallel-inference-lane.md);
  (2) N concurrent poll/nudge loops (`pollIntervalMs: 2 s`,
  `minNudgeIntervalMs: 20 s`) share one timer queue and cannot starve each
  other except through long synchronous blocks
  (source: tools/orchestration/src/pipeline/stages.ts).
- Every git call in the runner uses `spawnSync`, which "blocks the Node.js
  event loop until the spawned process either exits or is terminated" (source:
  https://nodejs.org/api/child_process.html,
  tools/orchestration/src/pipeline/runner.ts lines 96–135). So git
  mutations are serialized by construction even at N > 1 — the interleaving
  risk is not two gits racing, it is a git sweep landing while a sibling
  session is mid-write, which pass-level finalize eliminates
  (source: docs/adrs/003-parallel-inference-lane.md).
- The checkpoint's atomicity rests on `rename(2)`: "If newpath already exists,
  it will be atomically replaced" — crash-safe, but the syscall says nothing
  about ordering between two concurrent writers; last landed rename wins
  (source: https://man7.org/linux/man-pages/man2/rename.2.html,
  tools/orchestration/src/pipeline/state-store.ts).
- Per-seat artifacts are naturally disjoint at pass level: reports are
  `reviews/<phase>/<specialty>-rNcM.md` and transcripts `<stage>-rNcM.md`,
  keyed by specialty, so parallel seats write distinct files
  (source: tools/orchestration/src/pipeline/panel.ts lines 162–178,
  docs/workflows.md).

## Risks & unknowns

- **Checkpoint write ordering (the top one).** Concurrent `saveState` calls
  each validate–serialize–write-temp–rename; two in flight means an older
  snapshot can land last. The ADR mandates one in-process write path drained
  before pass finalize, proven with a mock client; anything beyond that
  (trust, corruption recovery) is ADR-004's (source:
  tools/orchestration/src/pipeline/state-store.ts,
  docs/adrs/003-parallel-inference-lane.md,
  docs/adrs/004-self-update-checkpoint-compatibility.md).
- **`git add -A` sweep races.** Per-stage finalize under overlap would sweep
  a sibling's half-written report into the wrong commit and the dirty-file
  scan in `enforceOwnership` would revert siblings' in-flight edits; the
  pass-level finalize is the fix, and it must cover the editorial branch too
  (source: docs/adrs/003-parallel-inference-lane.md,
  tools/orchestration/src/pipeline/runner.ts lines 168–279, 1151–1191).
- **Timeout vs queueing.** Each seat's `stageTimeoutMs` deadline starts at its
  own prompt, and the ADR states server-side queue waiting counts against it,
  so N > slots needs raised timeouts; whether the OpenCode server queues,
  rejects, or errors when all slots are busy is not documented anywhere in
  this repo (source: docs/adrs/003-parallel-inference-lane.md,
  https://opencode.ai/docs/server/ — the API lists session endpoints with no
  concurrency guarantees).
- **Slot count is machine state, not repo state.** The "server updated to
  expose multiple parallel slots" claim lives in the ADR's context; nothing in
  the repo records how many slots exist, so "N <= server slots" is guidance
  the runner cannot verify (source: docs/adrs/003-parallel-inference-lane.md).
- **No bit-for-bit identity at N = 1.** Commit shape and finalize timing
  change (one commit per review pass, one ownership sweep per pass); the
  design must not claim otherwise (source: docs/adrs/003-parallel-inference-lane.md).
- **Dogfooding boundary.** The self-update run never executes its own
  concurrency code (startup code-freeze + `.pipeline.lock`); validation is
  mock-client unit tests plus the first post-merge parallel run
  (source: docs/adrs/003-parallel-inference-lane.md).

## Open questions (for the designer)

1. Where does `--max-parallel` / `$OPENCODE_MAX_PARALLEL` parsing live? The
   strict fail-closed pattern exists only in `src/cli.ts`'s
   `resolveReplyTimeoutSeconds`; should it be extracted and shared, or
   duplicated in `src/pipeline/cli.ts`? (source:
   tools/orchestration/src/cli.ts, tools/orchestration/src/pipeline/cli.ts)
2. Does `editorial-review` fan out too? It has `kind: 'review'` and a
   sequential five-seat loop; the ADR's kind-based rule says yes, but its
   per-seat saveState/commit timing and the `-rN` (no pass number) report
   naming deserve an explicit decision
   (source: tools/orchestration/src/pipeline/stages.ts, panel.ts).
3. How is launch order observable when N < |panel|? A FIFO waiter queue
   preserves `correctness -> compatibility -> security -> performance ->
   complexity` launch order; which mock-client test pins that, and does the
   transcript's wall-clock ordering need a recorded launch timestamp to stay
   debuggable? (source: docs/adrs/003-parallel-inference-lane.md)
4. Should the runner ever raise or relate `stageTimeoutMs` to N, or purely
   document the interaction? The ADR only gives guidance ("N > slot count
   needs a raised timeout")
   (source: docs/adrs/003-parallel-inference-lane.md).
5. What does the OpenCode server do with more concurrent session prompts than
   slots — queue, reject, or error — and is there an endpoint to read slot
   count (llama.cpp has `/slots` with `total_slots`; the OpenCode server docs
   list no equivalent)? This decides how loudly an over-large N fails at
   runtime (source: https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md,
   https://opencode.ai/docs/server/).
6. Should `maxParallelReviews` be one integer per run (ADR wording) or
   per-stage later? ADR-001 already defers per-stage model/endpoint routing —
   confirm a single knob is intended now (source: docs/adrs/001-code-driven-agent-pipeline.md,
   docs/adrs/003-parallel-inference-lane.md).
7. Does the drained write path subsume the per-seat `await saveState(...)`
   calls that today run between seats (failure records, gate entries), and in
   what order do concurrent seats' `gateLog` entries land
   (source: tools/orchestration/src/pipeline/runner.ts)?
