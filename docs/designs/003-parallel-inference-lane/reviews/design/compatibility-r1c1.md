# Compatibility review r1 (dissent pass c1)

verdict: approve

Scope: `git diff 5dc494e7 9716bf4c` of `02-proposal.md` (finalize signature +
commit message, fan-out diagram, N = 1 caveat, escalation paragraph, config
bullet, escalation risk row, macOS/Windows matrix rows, Test strategy #5/#6,
Revision log), then the full current document. Checklist: `compatibility.md`.

## Round-1 findings — status

- [closed] Compatibility matrix: macOS/Windows now carry definite statuses
  (`no` — "Untested; no OS-specific code added (Linux-only CI, Test strategy
  #10)"). Rule 1 satisfied: no `unknown` remains anywhere in the matrix, and
  every row has a status. Rule 7 also holds: the only platform row claimed
  `yes` is Linux, and Test strategy #10 names Linux CI (`npm run check` in
  `tools/orchestration`). The `no` statuses therefore require no platform test.
- [closed] Configuration and CLI: the bullet now states the deviation from
  ADR-003 §Configuration in the open — "optional, deliberately relaxing the
  ADR's required field so existing configs compile" — and names
  `DEFAULT_CONFIG.maxParallelReviews = 1` as the default authority. The
  unrecorded ADR mismatch is gone. Residual gap in that same sentence is
  finding 2 below.

## Findings

- [minor][new] Pass-level fan-out — "no seat today's `StopDriving`
  (runner.ts:1158–1166) would skip ever runs — at N = 1 control flow is exactly
  today's." The scope of that sentence is just which seats run, but as written
  it is a blanket backward-compatibility claim for the whole N = 1 path, and
  the paragraph it sits in contradicts it two sentences later: the revised
  escalation path now runs "the pass `enforceOwnership` sweep, then
  `finalizeEscalate`", a git mutation today's escalation path does not perform
  (the r1 text itself said "a failed stage skips ownership enforcement today
  too"). ADR-003 §Self-update draws exactly this line — N = 1 "keeps ADR-002
  behavior for gates, artifacts, and control flow", commit shape and finalize
  timing excepted, and "a claim of bit-for-bit identity at `N = 1` is false and
  must not be made in the design". Fix: scope the clause to what the flag
  actually preserves, e.g. "at N = 1 the set of seats that run is exactly
  today's; the escalation path additionally runs the pass ownership sweep, so
  its commit differs from today's (see the N = 1 caveat)". The sweep decision
  itself is not contested — only the identity claim over it.

- [minor][new] Configuration and CLI — "`maxParallelReviews?: number` …
  optional … and `DEFAULT_CONFIG.maxParallelReviews = 1`, the sole default
  authority". The sentence asserts a guarantee the document gives no mechanism
  for: the only merge with `DEFAULT_CONFIG` shown anywhere is the CLI's
  `cfg: { ...DEFAULT_CONFIG, maxParallelReviews: n }`, while the very reason
  the field is optional is that config objects exist which omit it ("so
  existing configs compile"). Such a caller reaches `createSemaphore(cfg.maxParallelReviews)`
  (fan-out diagram) with `undefined`, which is not the documented default. No
  Test strategy item covers the omitted-field path either. Fix: name the
  normalization point in the same bullet — one clause stating that every
  `runPipeline`/`resumePipeline` entry merges `DEFAULT_CONFIG` before the
  config reaches the lane (or that the semaphore reads
  `cfg.maxParallelReviews ?? DEFAULT_CONFIG.maxParallelReviews`) — and let Test
  strategy #8 or #9 assert the omitted-field run serializes.

## Checklist coverage (no findings)

- Rule 1 (matrix statuses): pass — see closure above.
- Rule 2 (OS input APIs): n/a — no OS-specific input surface; the design adds
  only an in-process counter and waiter queue.
- Rule 3 (Node floor): pass — A10 plus the revised text adds no API above the
  `node >= 22.12.0` floor (`escalationPending` flag, `allSettled` barrier).
- Rule 4 (native modules): pass — no new dependency; the `p-limit`/`p-queue`
  option is explicitly rejected.
- Rule 5 (keyboard reality): n/a.
- Rule 6 (file paths): pass — no OS path construction introduced; the changed
  text only names `.pipeline.json.*.tmp` inside the design dir, and the drained
  writer plus pass-level sweep keep it out of `git add -A`.
- Rule 7 (platform test per supported platform): pass — Linux only, covered by
  Test strategy #10.

## FYI (no severity)

- Correctness seat: the escalation paragraph lost "and stop driving" during the
  revision; after the barrier-and-sweep branch, whether the `drive` loop halts
  is now unstated in the changed text.
- Correctness/security seats: "its commit carries only in-policy complete
  sibling reports" — `enforceOwnership` guarantees the in-policy half, not the
  complete half; a sibling cut off mid-write leaves a partial report in the
  escalation commit.
- Test strategy: the matrix claims "Checkpoints written before this change |
  yes", but no item loads a pre-change checkpoint fixture; Test #9 loads a
  checkpoint written by the new code.
- ADR conformance: the new commit-message sentence ("`docs: <nnn-slug>
  design-review stage`") hard-codes one review stage's name as the example while
  Test #6 also covers the editorial pass, whose stage key differs.
