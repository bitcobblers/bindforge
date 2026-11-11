# Code review — security seat, round 1, pass c0

Scope: `git diff f523ce7e273a434fb052fa4b146245b2155bd28d...HEAD -- ':(exclude)docs/designs'`.
Inputs: the diff, `02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`. No prior findings (first pass).

## Checklist walk

1. **External input validated at the boundary** — PASS. `MovePairSchema` bounds endpoints
   to 1–200 chars and the list to 64 pairs (`types.ts:32-45`); every schema-valid list still
   runs through `validateMoves` before acceptance (`runner.ts:675-696, 950-956`) and again
   through the rules 4–7 guard inside `applyMoves` before the first rename
   (`moves.ts:324-329`). No code evaluation anywhere.
2. **Path handling** — PASS. Rule 1 rejects `\`, `:`, control chars incl. NUL, absolute
   paths, `.`/`..`, empty segments, Windows reserved names, and trailing `.`/space
   (`moves.ts:88-122`); `..`-free relative paths make `join` safe; the only git call
   (`git ls-tree rev -- path`) passes argv arrays, never a shell, with `--` ending option
   parsing and rule 1's `:` denial closing git pathspec magic (`moves.ts:51-64`).
   Rule 7's realpath containment plus the `.git` segment check, case-folded, close symlink
   escapes into the repo-external gitdir (`moves.ts:156-224`).
3. **Injection** — PASS. No shell interpolation, no regex over untrusted text with
   backtracking risk (`WINDOWS_RESERVED` is anchored and linear over ≤200-char inputs).
   Model-authored failure text reaching gate log/ESCALATION.md passes `printable()`
   (`runner.ts:1241-1258`).
4. **Secrets/PII** — PASS. Nothing logs secrets; issue strings carry only path text.
5. **New dependencies** — PASS. None added. The only dependency-adjacent change raises the
   Node engines floor from 22.12.0 to 22.23.4 (`package.json:31`), a security-positive pin.
6. **Filesystem writes** — FAIL on one path, see finding 1. Per-pair `rename(2)` is atomic
   and the reverse rollback is exact for a disjoint matching (`moves.ts:330-349`), but the
   design's containment for the one rollback-failure state is not implemented, so partial
   state can reach a commit.
7. **Clean rejection** — PASS for malformed/hostile replies: a returned issue or a throw
   from any rule becomes `invalid-reply: moves — <issue>` on the existing nudge/retry/
   escalation path, pinned even for a rule that throws internally
   (`runner.ts:689-692`, `reply-contract.test.ts` second-rejection case). Nudge cycles are
   bounded by `MAX_NUDGES` and the stage deadline.

## Findings

### 1. major — `moves-rollback-failed` starts attempt 2 and can commit the half-applied list; the design's stop-at-operator containment is unimplemented

- **Location:** `tools/orchestration/src/pipeline/runner.ts:1550-1594` (`runWithRetry`),
  with `runner.ts:1133-1141` (`finalizeStage` failure outcome) and `runner.ts:1210-1216`
  (doc comment).
- **Problem:** The proposal (Apply paragraph and Risks row) is explicit: "A rollback rename
  that itself fails is the one state nothing in-process can undo: `moves-rollback-failed:
  <error>` escalates to the operator **instead of starting attempt 2 on a known-damaged
  worktree**"; test strategy #6 requires "escalates to the operator and starts no attempt
  2". `runWithRetry` has no branch for this failure class: after attempt 1 returns
  `moves-rollback-failed: …`, `retryInSameSession` is false (correctly), and the loop
  proceeds straight into attempt 2 with a fresh session **on the known-damaged worktree**.
  Attempt 2's finalize then sweeps only ownership violations — the half-applied pair's
  endpoints are ownership-satisfying by construction — so if attempt 2 succeeds,
  `commitStage` commits the half-moved state inside a normal-looking stage commit. That
  defeats the design goal "a failed apply never leaves a half-applied list", i.e. the
  checklist's atomicity requirement for writes whose partial state corrupts. Compounding
  it: the comment at `runner.ts:1216` states "`moves-rollback-failed:` starts nothing at
  all", which the code does not do, and the class is unpinned — the unit pin at
  `moves.test.ts:524` is `it.skip`, and `reply-contract.test.ts:183-196` only asserts
  `retryInSameSession(...) === false`, a value identical to `moves-apply-failed` and
  therefore unable to distinguish "fresh attempt 2" from "no attempt 2"; the test comment
  there claims the escalation the runner never performs.
- **Fix:** In `runWithRetry` (and any path that consumes a stage-failure outcome), special-
  case the failure prefix: when `out.failure` starts with `moves-rollback-failed`, call
  `finalizeEscalate(...)` immediately after the `stageFailure` gate log — no attempt 2 —
  and keep `moves-apply-failed` on the existing fresh-session retry. Pin the class with a
  fault-injected integration test (a stubbed/overridable rename hook, or the operator-run
  fault injection the skipped test anticipates) asserting zero attempt-2 sessions and an
  escalation reason naming `moves-rollback-failed`; or at minimum correct the comments and
  add a distinct unit pin that distinguishes the class from `moves-apply-failed`.

## FYI (no severity, outside specialty)

- `finalizeStage`'s apply-failure path adds a staging step (`git add -A` without commit,
  `runner.ts:1120-1132`) that the proposal's Apply paragraph does not mention ("returns
  without sweeping or committing"). I verified it does not weaken containment —
  `dirtyFiles`/`porcelainPaths` read both index and worktree columns, and
  `restore --staged --worktree` / `git rm -f` revert staged violations on attempt 2 — but it
  is a design deviation worth a correctness-seat glance.
- The diff also adds a browsing-discipline sentence to `prompts/design-review.md`, beyond
  the proposal's four-prompt Prompts section; it strengthens, not weakens, the review-seat
  boundary. The validator's `git ls-tree` is one more runner-side git invocation reading
  the A6 config surface the proposal already enumerates and explicitly defers to a
  follow-up ADR — recorded design debt, not new exposure.

## Verdict

request-changes — 1 major open (finding 1). All other checklist items pass.
