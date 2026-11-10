# Code review — testing seat, round 1 dissent pass (c1)

Scope: `git diff 95b73560..HEAD -- ':(exclude)docs/designs'`, read against the
full 02-proposal.md and docs/prompts/audit-by-deeds.md. This pass checks first
whether the last revision answers my round-1 findings; other findings only
where the changed code makes them true.

## Verdict: approve

Round-1 findings: all four resolved (1 major, 2 minor, 1 nit). New findings:
one minor (untested third enumeration route added by the fix round), one nit
(untested deadline clamp), and an environment note recorded as a finding
(checklist item 6 could not be executed by this seat — see below). No open
blocker or major.

## Re-check of my round-1 findings

1. **major — fallback (session.list + parentID) route had no positive-path
   coverage. RESOLVED.** The list route is now pinned at both layers:
   - Unit (`tests/pipeline/child-session-deeds.test.ts:76-105,193-258`):
     new `listOnlyClient` fake carries **no `session.children` anywhere** and
     pages `session.list` behind an opaque `cursor.next`. `reaches descendants
     through the parentID index on a list-only client` (198-215) reaches the
     grandchild through the parentID index; `follows cursor.next across
     session-list pages` (236-258) places the violating grandchild on page 3
     and asserts all pages are crawled (`listCalls()==3`);
     `enumerates the session list once per walk` (221-231) pins the
     one-enumeration cost rule.
   - Integration (`tests/pipeline/child-deeds-retry.test.ts:287-314`):
     `deedsStub({ listOnly: true })` removes `session.children` entirely from
     an otherwise full `resumePipeline` stub; the child's transcripts read is
     found through `session.list` + `parentID`, the attempt fails naming the
     path and `(via child session`, and attempt 2 carries the failure reason.
     The `pays at most one descendant enumeration per attempt` test (321-330)
     runs on the list route too. This includes my "ideally" clause — the #6
     end-to-end walk now runs on the list route. Shape regressions in
     parentID indexing, BFS, or pagination now fail green-to-red.

2. **minor — nine fake clients not extended with session.list. RESOLVED.**
   All nine named fakes now carry `session.list` returning one empty page:
   reply-contract:143, pass-finalize:129, parallel-integration:144,
   parallel-fanout:157, budget:127 (with an explicit `// Test strategy #9`
   comment), parallel-escalation:162, stage-lifecycle:75, funnel:133,
   proposal-size:132 (both fakes in that file at 132/176).

3. **minor — missing `// proves: Test strategy #8` citations. RESOLVED.**
   `prompts.test.ts:207-213` (bright-line assertions in the design-review
   guidance block) and `prompts.test.ts:358-361` (run-7 stage guidance /
   formatter duty in both test templates) now carry the citation, and the
   pinned sentences exist in `prompts/design-review.md:9`,
   `test-authoring.md:9`, `test-update.md:6`.

4. **nit — budget pin asserted gate-entry existence, not count. RESOLVED,
   with a better count than I proposed.** `child-deeds-retry.test.ts:275-284`
   now asserts exactly one `stageFailure` **and** exactly one `escalate` gate
   entry. I verified against the runner: `runWithRetry` logs `stageFailure`
   once for the first failure (runner.ts:1911) and the exhausted retry goes
   through `finalizeEscalate` → one `escalate` entry (runner.ts:2439, 1939-
   1946) — there is no second `stageFailure`. My round-1 suggestion of "two
   stageFailure entries" was wrong about the runner's shape; the coded
   assertion matches actual behavior and the comment at 276-279 explains why.
   Count pinned. Closed.

## New findings (changed code makes them true)

- **minor** — `src/pipeline/runner.ts:609-624,789-830`, `src/pipeline/cli.ts:171-179`:
  the fix round added the compat matrix's third enumeration route
  (`directChildSessions` — direct `GET /api/session/:id/children` with basic
  auth, plus the per-node BFS and the no-route error path) and wired a
  `ServerConnection` through the CLI. It has **zero test coverage**: no test
  passes `connection` to `listChildSessionMessages` or stubs `fetch` for it
  (the suite's only fetch stub is `tests/opencode-runner.test.ts:56`, a
  different module). A shape regression (URL path, auth header, response
  parsing) or the `throw` at 827-830 would ship silently — checklist item 7
  treats untested branches as findings. Not a blocker/major: the pinned SDK
  exposes `session.list`, so production never reaches this branch, and the
  call site already degrades to a caught partial audit. *Fix:* one walk test
  with a client lacking both `session.children` and `session.list` plus
  `vi.stubGlobal('fetch', …)` (the existing pattern), passing
  `connection: { baseUrl: … }`; assert the request path
  `/api/session/<id>/children`, that a child found that way is audited, and
  that client-with-no-routes-and-no-connection lands in the caught
  partial-audit path rather than escaping the call site.

- **nit** — `src/pipeline/runner.ts:664-677`: `budgetMs()` clamps every walk
  request to the stage deadline and throws past it ("truncated at the stage
  deadline"); no test fires the clamp or the expired-budget path. The new
  deadline/crawl-bound tests exercise the repeated-cursor stop and the page
  budget shape, but not the deadline clamp itself. *Fix:* drive a walk with an
  already-past `deadline` option and assert the throw (or the caught
  partial-audit outcome at the call site).

- **minor (checklist item 6, un-evaluated)** — this seat's shell tool was
  permission-denied for the whole pass, so I could not run `npm run check`
  and cannot paste its tail. Per the skill rule, an un-evaluated checklist
  item is a finding, not a pass; the run's runner verifies the suite
  independently. My static read found nothing red: every new assertion I
  could cross-check against the implementation matches it (route dispatch
  order, `countDelegationParts` guard vs. `walkedDelegations`, the
  stageFailure/escalate gate shape, list-route paging semantics against
  `listOnlyClient`).

## Checklist walk

1. Strategy pins: #1-#4 cited in `independence.test.ts:245-315`; #5 cited
   five times in `child-session-deeds.test.ts` (grandchild, cycle, pre-since,
   zero-descendant warn, cost/paging); #6 in `child-deeds-retry.test.ts`
   (both enumeration routes, budget gate counts, seat lane excluded to #7);
   #7 at 439-480; #8 at `prompts.test.ts:207,358`; #9 is the CI run itself
   (see item 6 above) with the nine-fake extension visible at the cited
   lines. (The `// proves: Test strategy #6` at `prompts.test.ts:326,337`
   belongs to another design's strategy numbering — pre-existing, not this
   diff.)
2. Both layers exist: matcher units, walk units, full `resumePipeline`
   integration on temp git repos with mock clients.
3. My round-1 major now has tests that would have caught it (list-only unit +
   list-only e2e).
4. Determinism: fakes only, no network, no live server, mkdtemp repos;
   `Date.now` stamps are order-relative against the audit's own `since` —
   outcome-deterministic, matching the pre-existing pattern in this suite.
5. Names describe behavior ("fails the attempt on a child deed found through
   the session-list route", …). Good.
6. Not executed — see finding above.
7. Untestable additions: the direct-HTTP route finding above.

FYI (out of specialty, no severity): the third-route fetch path touches
URL/auth surfaces the security and compatibility seats own; their re-checks
should confirm `basicAuthHeader` reuse and the `/api` prefix note.
