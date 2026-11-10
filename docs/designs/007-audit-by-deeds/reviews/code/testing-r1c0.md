# Code review — testing seat, round 1 pass c0

Scope: `git diff 7844c17171465dd779f8db6671801eb058bd2eb3...HEAD -- ':(exclude)docs/designs'`.
Inputs: that diff, `02-proposal.md`, `docs/prompts/audit-by-deeds.md`. No prior
findings (first pass). Checklist: `.opencode/skills/code-review/references/testing.md`.

## Checklist walk

1. **Test-strategy pins.** #1, #2, #3, #4 (independence.test.ts:245,264,281,293,305,315),
   #5 (child-session-deeds.test.ts:95,116,134,289), #6 and #7
   (child-deeds-retry.test.ts:234,380) all carry `// proves: Test strategy #N`
   citations and assert what the proposal says they must. #8's pins exist
   (bright line + formatter duty in rendered prompts) but cite no strategy
   number — finding 3. #9's first two clauses are verified by my own
   `npm run check` run (green, below); its third clause — extending the nine
   existing fake clients with the session-list method — was not done: the diff
   touches none of those nine files — finding 2.
2. **Layers.** Unit (independence.test.ts, child-session-deeds.test.ts) and
   integration (child-deeds-retry.test.ts over a temp git repo + mock client)
   both exist, as the proposal promises.
3. **Revision rounds.** N/A — no prior blocker/major findings to re-pin.
4. **Determinism.** No network, no live server, no ports. `git` runs in
   throwaway `mkdtempSync` repos. The `realClock()` stamps (Date.now,
   monotonic +1) are deliberate and justified in a comment: the runner takes
   `auditStart = Date.now()` (runner.ts:953) before the prompt, so fake
   integer clocks would hide every deed from the `since` filter; assertions
   never depend on elapsed time. Acceptable.
5. **Names.** Behavior-phrasing throughout ("fails an attempt on a child deed
   and feeds attempt 2 the failure reason"). Pass.
6. **`npm run check`** — run by me, green; tail below.
7. **Untestable additions.** Every new source branch has a consumer except the
   `session.list` + `parentID`-index fallback route — finding 1 (major).

## Findings

### 1. major — the enumeration route that production actually takes has no positive test

`tools/orchestration/src/pipeline/runner.ts:595` probes `session.children`
first and falls back to paged `session.list` + `parentID` index
(runner.ts:629–681). The pinned `@opencode/client ^2.1.4` exposes **no
`session.children` anywhere** on the promise client — a grep over
`node_modules/@opencode/client/dist/**/*.d.ts` finds the only `children`
token as an option of the solid store's `sync()` — so in every production run
`api.children` is undefined and the walk takes the fallback route. But every
fake client that holds a non-empty descendant tree also provides
`session.children`, so the probe always takes the primary route in tests:

- `tests/pipeline/child-session-deeds.test.ts:47–70` (`treeClient`) provides
  both → grandchild, cycle, and pre-`since` tests all exercise the children
  route.
- `tests/pipeline/child-deeds-retry.test.ts:160–166` (`deedsStub`) provides
  both → the end-to-end #6 pin also takes the children route.
- The only list-route exercise, `child-session-deeds.test.ts:241–245`
  (`delegatingClient`), returns sessions with **no descendants at all**, so
  `byParent` (runner.ts:637–681) indexes an empty map and the BFS yields zero.

The fallback's parentID indexing, BFS, and `cursor.next` pagination
(`pageCursorNext`, runner.ts:663) — the code that carries the design's core
promise ("a child that reads a transcripts/ file fails the parent attempt")
in every real run — are never exercised with descendants present. A shape
regression there (e.g. the cursor param or the `{data}` response) would
silently degrade every production attempt to a parent-only audit with one
gateLog line, and the suite would stay green. Fix: make at least one
fake with a non-empty tree list-only (no `session.children`) — e.g. drop
`children` from `treeClient` or add a sibling fake — and pin (a) the grandchild
reached through the `parentID` index, (b) multi-page enumeration following
`cursor.next`, (c) the #6 integration walk on the list route. This is also
what proposal Test strategy #9's "nine fake clients extended with the
session-list method" anticipated; the SDK read (Rollout step 1, A4) settles
the route, and the tests must pin the settled route.

### 2. minor — the nine existing fake clients were not extended (Test strategy #9)

Proposal: "the nine fake clients in `tests/pipeline/` (reply-contract,
pass-finalize, parallel-integration, parallel-fanout, budget,
parallel-escalation, stage-lifecycle, funnel, proposal-size) extended with the
session-list method their flows need." None of those files appear in the diff
(e.g. budget.test.ts:108–125: `session` has only create/prompt/remove). Their
flows all reach the audit call site with a clean parent audit, so every
attempt now enters the walk's fetch-failure catch: a `console.warn` plus a new
`independenceAudit` (proceed: true) entry in every scenario's saved `gateLog`.
The suite stays green only because those tests filter gateLog by gate name.
Consequences: the nine integration scenarios now verify the degraded
parent-only path instead of the designed walk, and their run records now
contain a partial-audit warning a healthy server would never emit. Fix: give
those fakes a minimal `session.list` returning one empty page where their
flows hit the audit site.

### 3. minor — Test strategy #8 pins carry no `// proves:` citation

`tests/pipeline/prompts.test.ts:207–212` (bright line) and the `run-7 stage
guidance` block at `prompts.test.ts:355–375` (formatter duty) assert exactly
what #8 requires but are commented only "Design 007 §4" — no
`// proves: Test strategy #8`, unlike every other new pin in this diff.
Checklist item 1 requires the citation. Fix: add the two comment lines.

### 4. nit — "one gate entry per failure" asserted as `some`

Proposal #6 requires the budget pin to show "one gate entry per failure";
`child-deeds-retry.test.ts:269` asserts `saved.gateLog.some(g => g.gate ===
'stageFailure')` — existence, not count. Fix: assert exactly two
`stageFailure` entries (attempt 1 and retry) for the implement stage.

## Out-of-specialty FYI (no severity)

`violations.length = 0` in the walk's catch (runner.ts post-loop error path)
discards child violations collected before a mid-walk fetch failure; that
matches the design's "proceeds on the parent list alone," but the collected
child violations are dropped without a word in the gateLog reason.

## `npm run check` (tools/orchestration) — tail

```
npm notice run @bindforge/orchestration@1.0.0 lint
npm notice run eslint .
npm notice run @bindforge/orchestration@1.0.0 format:check
Checking formatting...
All matched files use Prettier code style!
npm notice run @bindforge/orchestration@1.0.0 test
 RUN  v5.0.5 .../tools/orchestration
 Test Files  28 passed (28)
      Tests  241 passed (241)
   Duration  3.16s (tests 73%, import 22%, transform 3%, worker 1%)
```

## Verdict

request-changes — 1 major (finding 1), 2 minor, 1 nit.
