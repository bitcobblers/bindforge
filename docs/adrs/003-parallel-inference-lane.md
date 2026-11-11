---
status: accepted
date: 2026-11-08
---

# ADR 003: Run review passes across up to N parallel inference slots

We will lift the single-session-per-run rule inside a review pass: the runner
launches the seats that pass selected - all five at `c0`, dissenters only at
later passes, the approvers at ratification, per ADR-002 - across a
bounded-concurrency lane of up to N in-flight inference requests. Every other
state-machine semantic from ADR-001 as amended by ADR-002 - panel selection,
pass-end aggregation, all-must-approve over revision stamps - stays as written.
This amends ADR-001 principle 5 ("Sequential inference lane") and its settled
default "deterministic review order within a round"; ADR-001 remains accepted
otherwise.

## Context

ADR-001 constrained the pipeline to one in-flight inference because the local
engine "holds enough context to run exactly one task at a time," and its
Alternatives section rejected review fan-out on that capacity ground alone.
The inference server on this machine has been updated to expose multiple
parallel inference slots, so the capacity constraint no longer holds.

The wall-clock cost of the constraint is the pipeline's dominant cost: ADR-002
budgets 40 design-side and 20 code-side reviewer sessions, and today every one
of them is serialized behind the previous one. Fan-out inside a review pass is
the only place in the pipeline where inference is genuinely independent:
reviewers read the same immutable inputs and write distinct per-specialty
files. Under ADR-002 a pass holds between 1 and 5 seats, so fan-out pays most at
`c0` and at the ratification pass and least at a single-dissenter pass - which
is why the two decisions are separable.

Because the pipeline is expected to implement this change itself (dogfooding),
the self-update run drives itself with the code loaded at its own start and
never executes its own concurrency code. Making the checkpoint a *resume-safe*
artifact across that transition is a larger, separable concern - it was the
single largest source of design surface when this ADR was last designed - so it
is carved out to ADR-006 and is **not** part of this decision.

## Decision

### What is amended, what is kept

ADR-001 principle 5 is replaced by: **a bounded-concurrency inference lane over
the panel each pass selects.** The seats of a pass launch in the fixed order
`correctness -> compatibility -> security -> performance -> complexity`
(deterministic launch order preserved), run up to N at a time, and the verdicts
are still computed only at pass end - no short-circuiting, no partial passes,
same retry/escalate budget. **Which seats a pass contains is ADR-002's
decision**; this ADR only schedules the list it is handed. Only the *scheduling*
of the fan-out changes; the transition table, gates, artifact contract,
transcripts, and per-stage session model do not.

The rejection of model-driven **subagent** fan-out in ADR-001 stands: the
fan-out stays runner code, never agent prompts. The capacity objection that
also rejected it is superseded by this ADR.

### Configuration

- `PipelineConfig.maxParallelReviews: number`, default **1** (today's
  behavior; raising it is opt-in). Applies to stages with
  `kind: 'review'` only.
- Pipeline CLI flag `--max-parallel <N>` with env fallback
  `$OPENCODE_MAX_PARALLEL`, validated as an integer >= 1 using the same
  strict fail-closed pattern as `--timeout` (bad env is a config error even
  when the flag overrides it).
- Guidance: N must not exceed the server's real slot count. Each stage's
  `stageTimeoutMs` deadline is measured from its own prompt, and server-side
  queue waiting counts against it, so N greater than slot count needs a
  raised timeout.

### Execution model in `runner.ts`

1. **Split `executeStage` into two phases.** An *inference phase* (render
   prompt, create session, poll/nudge loop, validate reply, write transcript,
   append to `state.results`) performs no git mutations and is safe to run
   concurrently. A *finalize phase* (`enforceOwnership` + `commitStage`) is
   the only code that mutates git.
2. **Review passes finalize once, at pass level.** `headBefore` is captured
   before the fan-out; after all seat tasks settle, the runner runs one
   `enforceOwnership` and one `commitStage` for the pass, then evaluates that
   pass's gate. This removes the interleaving races by
   construction - per-stage finalize under a mutex would let one reviewer's
   `git add -A` sweep a sibling's half-written report into the wrong commit,
   and `enforceOwnership`'s dirty-file scan would revert siblings' in-flight
   edits.
3. **Fan out with an in-process semaphore** (new `src/pipeline/concurrency.ts`,
   ~20-line counter + waiter queue, no new dependency). Retries stay inside
   the slot, so a retry consumes a slot and total in-flight inference never
   exceeds N. All settle before the pass gate evaluates; a seat that fails
   twice escalates after its siblings settle, per the existing
   one-retry-then-escalate rule.
4. **One code path.** N = 1 goes through the same fan-out with a
   semaphore of size 1; the sequential lane is the degenerate case, not a
   separate branch.
5. **Non-review stages never fan out.** `research -> proposal -> ... ->
   implement -> fix` is a strict data-dependency chain over one shared
   worktree; the runner guards fan-out behind `kind === 'review'`.
6. **`.pipeline.lock` is unchanged.** One pipeline *process* per repo;
   parallelism is strictly inside a single run.

### Self-update posture (bounded on purpose)

Because this change is expected to be implemented by the pipeline it changes:

- The self-update run never executes its own concurrency code: the driving
  process loaded the old code at startup, and module state never hot-swaps.
  The parallel lane is therefore validated by unit tests with a mock client
  (max-in-flight <= N; a full-panel pass launches all five seats concurrently; a
  dissent pass launches only its dissenters; N=1 result order identical to
  today; exactly one commit per review pass) plus the first post-merge parallel
  run.
- `.pipeline.json` schema changes introduced *by this change* are **additive
  with defaults**; `version` stays 1.
- Fan-out must not lose a settled reviewer's record: every checkpoint write
  made from inside a concurrent review pass goes through one in-process write
  path, and that path is drained before the pass finalize. This ADR requires
  that write path and its invariant, proven with a mock client - and nothing
  beyond it. Checkpoint trust, tamper recovery, and recovery from a corrupted
  or replaced checkpoint are out of scope here and belong to ADR-006.
- Default `N = 1` keeps ADR-002 behavior for gates, artifacts, and control
  flow. Commit shape and finalize timing do change at `N = 1` - one commit per
  review pass, one ownership sweep per pass - so a claim of bit-for-bit identity
  at `N = 1` is false and must not be made in the design.

## Alternatives considered

**Parallel mutating stages / concurrent pipeline runs:** rejected for now.
The mutating chain has real data dependencies and shares one worktree;
multi-run concurrency needs per-run git worktrees and a per-slug lock. A
possible future ADR; independent of this one.

**Per-reviewer commit under a git mutex:** rejected - ownership enforcement
cannot distinguish a sibling reviewer's in-flight edits from a permission
violation when stages overlap; pass-level finalize eliminates the race instead
of locking around it.

**Model-driven subagent fan-out:** rejected again for ADR-001's
still-valid reason: pipeline semantics must not depend on model compliance.

**Hardening the checkpoint as a cross-version, tamper-resistant artifact:**
deferred to ADR-006 rather than decided here. Designing it inside this ADR is
what turned a scheduling change into a storage-subsystem design; nothing about
fan-out depends on the checkpoint surviving a foreign or older writer, and the
one run that straddles two versions of the runner never executes its own new
code.

**Raising the default to N = 5 immediately:** rejected - the concurrency path
is unexercised until the first post-merge run, and N is a server-capacity
decision (KV-cache pressure), not a code constant.

## Consequences

Becomes possible: a full-panel or ratification pass shrinks from 5x to
ceil(|panel|/N)x the serial wall clock; the dominant cost of a full pipeline run
falls without touching the state machine, gates, or artifact contract. Dissent
passes are already 1-3 sessions, so ADR-002 narrows the panels this ADR
parallelizes - the two changes multiply rather than substitute.

Accepted costs: completion order inside a pass is nondeterministic when
N > 1, so transcripts record wall-clock ordering (gates are set-based over
`verdictsFor`, never order-sensitive, so this does not affect control flow);
`runner.ts` grows a two-phase stage model that every future stage kind must
implement correctly; the semaphore, fan-out, and pass-finalize are new review
surfaces.

Dogfooding posture: the self-update run is safe by construction (startup
code-freeze + `.pipeline.lock`). A `pipeline resume` that straddles this
change's merge is deliberately not covered here - ADR-006 owns it.
`docs/workflows.md` and the README gain the `--max-parallel` knob with the
"N <= server slots" guidance.

Deferred: per-stage model/endpoint routing already anticipated by ADR-001's
`stages.ts` mapping; multi-run worktree parallelism left to a future ADR;
cross-version checkpoint resume, checkpoint trust, and recovery from a corrupted
or replaced checkpoint to ADR-006.
