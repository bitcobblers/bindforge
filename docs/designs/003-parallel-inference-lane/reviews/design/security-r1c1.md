# security review r1c1 (dissent pass on the r1 revision)

verdict: approve

Scope: `git diff 5dc494e 9716bf4` of `02-proposal.md`, then the full current
document, `docs/adrs/003-parallel-inference-lane.md`, and the Revision log.

## r1 closure check

- **[closed] Pass-level fan-out — escalation commit carried unenforced sibling output.**
  The revision does what my r1 finding asked: the escalation branch now runs
  "drain the writer, run the pass `enforceOwnership` sweep, then `finalizeEscalate`
  once", the posture on trip is stated ("The sweep reverts out-of-policy sibling
  files and escalation still lands"), the Risks row names the sweep in its
  mitigation, and Test #5 names the sweep. `enforceOwnership` reverts dirty
  violations in the worktree (runner.ts:222–238) *before* `finalizeEscalate`'s
  `commitStage` does its `git add -A` (runner.ts:1615–1620), so an out-of-policy
  sibling file can no longer reach history through the escalation commit. The
  placement — after the barrier, so the dirty-file scan cannot see in-flight
  sibling edits — is the only placement that is sound under fan-out. Closed.
  Three residual nits and one minor around that closure are below.

## Findings

- [minor][new] Chosen design → Pass-level fan-out in `drive`, bullet "Escalate
  after siblings settle" — "The sweep reverts out-of-policy sibling files and
  escalation still lands" — the promise has no stated mechanism, and the closure
  introduces a new throw point in front of the run's only terminal landing.
  `enforceOwnership` is not error-tolerant: its `git()` helper throws on any
  non-zero exit (runner.ts:97–103) and it uses `git()` for the revert itself
  (`restore`, `add -A`, `commit`; runner.ts:227, 243–251). Today it is reached
  only inside `if (outcome.ok)` (runner.ts:917), so the escalation path has no
  git-mutating step ahead of `finalizeEscalate`; this design adds one. If it
  throws — an index.lock left by a seat that shelled out to git before failing
  validation, a `restore --source=<headBefore>` that cannot be resolved — the
  error escapes `drive` (only `StopDriving` is caught, runner.ts:1401–1405), so
  `finalizeEscalate` never runs: `state.status` stays `running`, no `escalate`
  gateLog entry, no `ESCALATION.md`, and the revert is left half-applied with the
  out-of-policy file still dirty in the worktree. The run's only continuation is
  `pipeline resume`, whose escalation lands at drive's `next.kind === 'escalate'`
  (runner.ts:1121–1123) and runs **no** sweep, so `commitStage`'s `git add -A`
  lands exactly the out-of-policy file the closure was meant to keep out of
  history. A seat steered by injected content has a motive to break this step.
  Fix: replace "The sweep reverts out-of-policy sibling files and escalation
  still lands" with the mechanism plus the degraded posture — the escalation
  sweep is best-effort: a sweep error is logged and swallowed the way runner.ts
  already swallows a failed ownership delete ("Deleting an out-of-ownership path
  must never take the run down", runner.ts:230–235), `finalizeEscalate` still
  runs, and when the sweep did not complete the escalate gate entry's reason
  records that the escalation commit is un-enforced. Mirror that wording in the
  Risks row "Escalation mid-pass loses sibling work or races its writes".

- [minor][new] Test strategy #5 — "escalation is one `enforceOwnership` sweep
  then one `finalizeEscalate` after siblings settle" — the assertion pins call
  count and order, not the security property the r1 closure rests on. Nothing in
  #1–#10 proves that a file the review ownership rule forbids is absent from the
  escalation commit. An implementation that sweeps after `finalizeEscalate`'s
  `commitStage` has already run, or sweeps with a `headBefore` that is not the
  pass-entry value, satisfies the sentence as written (one sweep, one finalize,
  siblings settled) and still lands the file — the r1 hole returns with the
  design text intact. Fix: name the fixture inside #5 — a *succeeded* sibling
  leaves a path outside `docs/**` (e.g. `src/injected.ts`) while another seat
  fails twice — and assert three effects, not two calls: the path is reverted or
  deleted after the pass, the escalation commit's `--name-only` listing contains
  no path outside `docs/**`, and `state.status === 'escalated'` with
  `ESCALATION.md` present. This is the executable form of the Risks-row
  mitigation.

- [nit][new] Chosen design → Pass-level fan-out in `drive`, bullet "Escalate
  after siblings settle" — "its commit carries only in-policy complete sibling
  reports" — the sweep contains *paths*, not roles: `stagePathAllowed` allows
  every review stage anything under `docs/**` (gates.ts:260–273), so a seat's
  edit to the document under review or to another seat's report passes the sweep
  and lands in the escalation commit. The sentence reads as a per-seat report
  guarantee, which is not what runs. Behaviourally this is strictly narrower than
  today's unenforced commit and it is the same premise A3/A6 use to make the pass
  sweep sound, so it is wording only — but `ESCALATION.md` plus this commit is
  the record the operator reconstructs the run from. Fix: replace the phrase with
  "only paths the review ownership rule allows (`docs/**`, not per-seat report
  paths)".

- [nit][new] Configuration and CLI — "`maxParallelReviews?: number` to
  `PipelineConfig` — optional, deliberately relaxing the ADR's required field so
  existing configs compile — and `DEFAULT_CONFIG.maxParallelReviews = 1`, the
  sole default authority" — the design names who owns the default but not what
  the read site resolves when the field is absent. `RunOptions.cfg` is
  caller-supplied, so with the field optional `cfg.maxParallelReviews` is
  `number | undefined` at `createSemaphore(cfg.maxParallelReviews)`; a
  `PipelineConfig` literal built without it (embedded caller, test harness) gets
  a semaphore whose limit is `undefined`, i.e. a counter that never blocks, and
  ADR-003 §Execution 3's hard invariant ("total in-flight inference never exceeds
  N") fails open rather than closed. Fix: state the resolution in the same
  bullet — `createSemaphore(cfg.maxParallelReviews ?? 1)` — replacing "the sole
  default authority" with the expression that enforces it, or keep
  `maxParallelReviews: number` required on `PipelineConfig` (what the ADR says)
  and confine the optionality to the CLI/config-file boundary that parses
  existing configs.

- [nit][new] Configuration and CLI — "a `^\d{1,5}$` format gate on the env string
  that runs even when the flag overrides it (bad env is a config error), range
  check (integer ≥ 1) as a separate error naming the source" — the resolver it
  says it mirrors (`resolveReplyTimeoutSeconds`, src/cli.ts:86–137) has two
  fail-closed properties this summary drops: a bound on *both* sources
  (`flagValue > maxSeconds` is rejected — "unbounded waits are not supported",
  src/cli.ts:116–121) and escaping of the reflected value, so "control and ANSI
  bytes can never reach the terminal raw" (src/cli.ts:98–99). As specified,
  `resolveMaxParallel` bounds only the env string (5 digits); the flag path takes
  yargs `type: 'number'`, where `1e30` passes `Number.isInteger` and `≥ 1`. And
  if the error line interpolates `$OPENCODE_MAX_PARALLEL` raw, an
  operator/planted-environment value writes unescaped bytes to the terminal.
  Impact is low here — the panel caps real fan-out at 5 seats, and nothing is
  interpolated into a shell string — but "mirrors the `--timeout` pattern" is the
  design's contract for the fail-closed behaviour, and this bullet is what an
  implementer copies. Fix: in the same bullet, add an explicit upper bound with
  the constant named (e.g. reject `N > 16`, or `N > |panel|`), or state in one
  clause why the panel size makes an upper bound unnecessary; and name
  `JSON.stringify` on the reflected env value as part of the mirrored pattern.

## FYI (no severity)

- The flow diagram runs `writer.drain()` → `finalizeReviewPass(headBefore)` →
  `recordSeatVerdicts → cycle bump → pass gate` unconditionally, while the
  escalation bullet describes drain → sweep → `finalizeEscalate`. Whether
  `finalizeReviewPass`'s commit also runs when `escalationPending` is set (two
  commits per escalated pass) or is replaced by it is unstated; Test #5 implies
  one. Correctness seat.
- Same bullet: "at N = 1 control flow is exactly today's" over-claims in the one
  respect the bullet itself changes — at N = 1 the escalation path now runs an
  ownership sweep it never ran, and the commit message loses the per-seat stage
  key. The caveat the bullet above it states ("only commit shape and finalize
  timing change") is the ADR §Self-update wording; this sentence contradicts it.
  Correctness / compatibility seats.
- `maxParallelReviews?: number` deviates from ADR-003 §Configuration's required
  "`PipelineConfig.maxParallelReviews: number`, default **1**". The design labels
  the deviation deliberately and keeps the default at 1, so it is not a
  contradiction that re-opens anything on my seat; it is an ADR-conformance call
  for the compatibility seat.
- Checklist item 5 is clean by construction: the third-party-limiter option is
  rejected, so the design adds no dependency to advisories-check; git stays
  `spawnSync` with an argument array (no path interpolated into a shell string),
  which keeps item 3 satisfied for the new sweep and commit.
- Deferred items match ADR-003's own carve-outs (checkpoint trust/resume to
  ADR-004, multi-run worktrees to a future ADR, slot discovery to operator
  guidance); none contradicts the ADR's Decision, so none is re-opened here.
