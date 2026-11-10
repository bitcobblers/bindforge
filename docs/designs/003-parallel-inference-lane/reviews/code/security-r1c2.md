# Code review — security — ratification pass c2 (round 1)

**Verdict: approve** (zero open blocker/major; one carried minor, one nit)

## Scope and method

Ratification pass c2: check only that the revisions landed after my r1
approval broke nothing I approved. Inputs: `git diff
3745f3c6d0d0855bf2b77ea2f400a9d16c0b6a6a..HEAD -- ':(exclude)docs/designs'`
read against the full `02-proposal.md` and ADR-003, plus my own prior
findings.

Scope note (honesty about method): the `shell` tool was permission-denied in
this seat, so I could not run the git diff command mechanically. I reviewed
the current state of every source and test path in the branch's scope
(`tools/orchestration/src/pipeline/*`, `src/cli.ts`, `src/opencode-runner.ts`,
`tests/*`, prompts, package.json/lockfile) and re-verified the exact lines
my r1 finding cited — `independence.ts:143-145` and `runner.ts:842-843` are
byte-identical at HEAD to what I reviewed, so the area I conditioned my
approval on is unchanged. Every security-checklist item was evaluated
against that current state.

## Checklist results

1. **External input validated at the boundary** — pass. Verdict replies stay
   schema-validated (`VerdictSchema`); checkpoint loads revalidate through
   `StateSchema` with the same grammar as the identity generator; model-
   authored findings are size-capped where recorded (`boundedFindings`,
   runner.ts:1821-1851) and re-bounded before prompt re-injection
   (`reviewVars`, runner.ts:1906). Fan-out adds no new untrusted-input
   surface; `cfg.maxParallelReviews` falls back to `DEFAULT_CONFIG`/1 so a
   config object that omits the field can never reach
   `createSemaphore(undefined)` (runner.ts:1203), and the semaphore itself
   rejects a non-integer/`< 1` limit (concurrency.ts:17-19).
2. **Path handling** — pass. `reportPathFor` interpolates only fixed
   specialty lists and validated `round`/`cycle` numbers; the checkpoint
   grammar (`NNN_RE`/`SLUG_RE`, types.ts:151-152) keeps `nnn-slug` a single
   safe path segment; `writeEditorialDiff` builds its tmp snapshot from the
   slug-validated name and calls git via `spawnSync` argument arrays — no
   string-interpolated shell.
3. **Injection** — pass. Model/server-authored strings reaching the operator
   terminal, gate log, or ESCALATION.md go through `printable`
   (independence.ts:200-202; gateLog runner.ts:1158; finalizeEscalate
   runner.ts:1983; renderStatus runner.ts:2035). `resolveMaxParallel`
   reflects env/flag values through `JSON.stringify`/`String` and its
   `^\d{1,5}$` gate runs before any `Number()` conversion.
4. **Secrets/PII** — pass. No secrets in source, fixtures, or new log lines.
5. **New dependencies** — pass. None: direct deps remain
   `@opencode/client`, `yargs`, `zod`; the ADR's semaphore is the in-process
   ~20-line `concurrency.ts` as designed; `p-limit` in the lockfile is only
   a transitive dev-chain entry, not a direct dependency.
6. **Filesystem writes** — pass. Checkpoint writes stay tmp-file + rename
   (state-store.ts:185-189); the `createStateWriter` chain serializes them
   in enqueue order and the pass barrier awaits `drain()` before the pass's
   `git add -A` — the stray-tmp hazard A7 named is closed; ESCALATION.md
   lands after a checkpoint save on the escalation path.
7. **Clean rejection** — pass. `--max-parallel`/`$OPENCODE_MAX_PARALLEL`
   validation runs before any client/lock/checkpoint side effect
   (cli.ts:161-166); malformed checkpoints produce actionable messages, not
   stack traces; an escalation-path ownership sweep that throws is caught
   and logged so the terminal escalation record still lands
   (runner.ts:1309-1320).

## Regression check of my r1-approved area

The sibling-report write guard (`independence.ts:143-150`, runner wiring
runner.ts:842-843, `reportKey` identity) is unchanged from what I reviewed:
same lines, same behavior. The later fixes did not touch it — and did not
fix it either (see finding 1; it remains minor, so approval stands).
`reviewVars` still emits `reportPath` for every review phase
(runner.ts:1869 editorial, runner.ts:1918 design/code), so the runner
wiring is sound as reviewed.

## Findings

1. **minor — write guard still untested (carried, unchanged from my r1
   finding).** Location: `tools/orchestration/tests/pipeline-independence.test.ts:97-108,222-233`
   with `tools/orchestration/src/pipeline/independence.ts:143-150` and
   `runner.ts:842-843`. No test anywhere passes `ownReportPath` or asserts
   the `wrote a review report` violation; the two tests that read like
   write coverage still omit `ownReportPath` and so exercise only the
   vacuous "cannot judge writes" branch — the guard can break or the
   `reportPath` var can rot with no test going red. Unchanged since my r1
   report (I verified the cited lines are byte-identical at HEAD); not a
   regression introduced by the later revisions, and the guard itself works
   as designed when handed a path, so it stays minor and non-blocking.
   Fix: as previously stated — unit-test a completed write to a sibling's
   path under `reviews/**` while handed a different `reportPath`, an
   absolute-path write of the own path treated as own (reportKey identity),
   and an assertion that review-stage vars carry `reportPath` so the wiring
   cannot rot.
2. **nit — `$OPENCODE_MAX_PARALLEL` control-byte reflection is untested.**
   Location: `tools/orchestration/tests/pipeline-cli-max-parallel.test.ts`
   (whole file) against `src/pipeline/cli.ts:72-77`. The behavior is
   correct — the error reflects the raw env string through
   `JSON.stringify`, so ANSI/control bytes cannot reach stderr raw — but
   the resolver's test file never exercises it, while the timeout mirror it
   copies has exactly that case (`tests/cli.test.ts:108-115`). Fix: port
   the `'\x1b[2J'` reflection case to the max-parallel suite.

## FYI (no severity)

The seat prompt template (`prompts/code-review.md`) and the
`prevFindings` injection path are now the primary defense against
cross-seat anchoring, with the file-level audit as defense-in-depth; the
module header states that boundary honestly (obfuscated shells are out of
reach by construction).
