# Code review — compatibility — round 1, pass c0

- Diff: `7844c17171465dd779f8db6671801eb058bd2eb3...HEAD -- ':(exclude)docs/designs'`
  (9 files, all under `tools/orchestration/`)
- Inputs: that diff, `02-proposal.md`, `docs/prompts/audit-by-deeds.md`, and this
  seat's own prior findings (none — first pass).
- Checklist: `references/compatibility.md` only.

## Summary

The change is platform-neutral: no OS-specific code, no platform-only imports,
no native dependencies, no new filesystem path construction in `src/`, and no
wire/checkpoint format change (`failureNote` is in-memory `StageHandle` state;
`PipelineState` untouched — the matrix rows for checkpoints, configs, and
straddling runs hold). Node-floor audit: everything new
(`AbortSignal.timeout`, async generators, `for await`, ES2022 syntax) is well
inside the package's `engines: node >=22.12.0` and the tsconfig `target:
ES2022` (`package.json:30–32`). New integration tests are Linux-CI-compatible:
they use `join(tmpdir(), ...)` joins and the same `git init -q -b main`
convention as the pre-existing suites, need no network, and fall inside
vitest's default include (no config restricts it). Two compatibility gaps
against the proposal's stated support story — both degrade visibly, not
catastrophically; both minor.

## Item-by-item

1. **Matrix vs code paths** — see finding C1. Routes 1 (`session.children`)
   and 2 (`session.list` + `parentID` index, one paged crawl, `cursor.next` —
   `runner.ts:585–660`) are implemented; route 3 (direct
   `GET /session/:id/children` against `baseUrl`) is not.
2. **Node floor** — pass. `AbortSignal.timeout` (≥17.3), async generator
   (`runner.ts:585`), BFS via index queue, `??`/`?.` — all inside ≥22.12.0.
   `pathKeyValues`' BFS node cap (`independence.ts:118–123`) bounds the scan
   work; no new runtime primitive.
3. **Paths** — pass. No path construction added in `src/`. Test temp dirs use
   `join(tmpdir(), prefix)` (`child-session-deeds.test.ts:200`,
   `child-deeds-retry.test.ts:59`). Violation strings embed session IDs and
   recorded paths as text, not constructed paths.
4. **Guarded platform imports** — pass. Nothing platform-only imported; the
   `session.children`/`session.list` probe (`runner.ts:591,629`) is a
   capability guard, not a platform guard, and degrades with a logged
   parent-only audit instead of a crash.
5. **Locale/layout reality** — pass. No file formats or IDs are defined or
   changed; matching stays on the pre-existing segment regexes. `failureNote`
   and violation text pass through `printable()` (control chars only) —
   non-ASCII survives; the 140-code-unit `brief()` truncation behavior is
   pre-existing, not introduced here.
6. **Native dependencies** — pass. No dependency added or version changed;
   `@opencode/client ^2.1.4` was already the dep (see FYI: `node_modules` is
   not installed in the worktree, so the A4 SDK-surface check that Rollout
   step 1 mandates cannot be confirmed from the diff).
7. **Plan-named tests on their platforms** — see finding C2. The new tests
   run on Linux CI; but the nine fake-client extensions named in Test
   strategy #9 were not made, and every existing integration flow now takes
   the walk's error path on every clean attempt.

## Findings

### C1 (minor) — third enumeration route promised by the Compatibility matrix is not implemented

- **Location:** `tools/orchestration/src/pipeline/runner.ts:629–634`
  (`listChildSessionMessages` fallback chain)
- **Problem:** The matrix row "OpenCode server/SDK without a children method"
  is marked supported (`yes`) with a three-route chain — SDK `session.children`,
  else SDK `session.list` + `parentID` filter, else a direct
  `GET /session/:id/children` against the configured `baseUrl` — and asserts
  "the walk always has a route (A4 decides which, not whether)". The code
  implements only the two SDK-method probes; when the client object exposes
  neither, it throws (`runner.ts:632`) and the call site degrades to a
  parent-only audit (`runner.ts:1095–1110`). A4 is explicitly *unverified*
  (the SDK is not installed in this worktree, and the diff carries no evidence
  that Rollout step 1 — install, read the `@opencode/client` types, fix the
  route — was performed: no lockfile/package.json change, no note recording
  the chosen route). If the pinned SDK ever exposes neither method, the
  promised route does not exist and every attempt's child walk silently
  degrades — mitigated (warn + `independenceAudit` gate entry, fence holds),
  but the matrix overstates what the code guarantees.
- **Fix:** implement the direct-`GET /session/:id/children` fallback against
  `baseUrl` with the client's auth headers, or verify the pinned
  `@opencode/client ^2.1.4` exposes `session.list` and amend the matrix row to
  the two routes actually implemented (recording the Rollout step 1 result in
  the build notes).

### C2 (minor) — Test strategy #9's nine fake-client extensions are absent; every existing integration flow now exercises the walk's failure path on every attempt

- **Location:** `tools/orchestration/tests/pipeline/budget.test.ts:109–125`
  (representative — all nine named stubs: reply-contract, pass-finalize,
  parallel-integration, parallel-fanout, budget, parallel-escalation,
  stage-lifecycle, funnel, proposal-size keep a `session:` stub with only
  `create/prompt/remove`); consequence at
  `tools/orchestration/src/pipeline/runner.ts:1103` (`gateLog(state,
  'independenceAudit', …)`)
- **Problem:** The plan required the nine fake clients be "extended with the
  session-list method their flows need"; the diff touches none of them. Their
  flows therefore deterministically hit the `listChildSessionMessages` throw
  (`runner.ts:632`) on every clean attempt: each attempt now adds an
  `independenceAudit proceed: true` entry to the persisted checkpoint's gate
  log and a `console.warn` per attempt. The suites still pass — no existing
  test asserts gate-log length or exact contents (verified by grep) — but CI
  no longer runs those nine flows through the walk's success path, the new
  gate-entry shape is exercised end-to-end only by the two new test files,
  and CI output/checkpoints accumulate degradation noise the design did not
  ask for.
- **Fix:** give the nine stubs' `session:` objects a minimal
  `list: async () => ({ data: [...], cursor: { previous: null, next: null } })`
  (as `child-session-deeds.test.ts`'s `delegatingClient` already does), so the
  existing flows walk successfully.

## FYI (no severity)

- The route is chosen by client-method presence, not by server support: if the
  SDK exposes `session.children` but the server lacks the endpoint, the walk
  errors to the parent-only fallback rather than retrying via `session.list`.
  The design's letter ("when the SDK exposes it, else") matches the code; a
  runtime-error fallback between routes is a design-level question, not an
  implementation deviation.
- `npm run check` chains `lint && format:check && test` without `typecheck`;
  since tests are outside `tsconfig.json`'s include, test-file type errors
  are not a CI gate. Pre-existing posture, not introduced by this diff.

## Verdict

approve — zero open blocker/major findings; two minors (C1, C2) as above.
