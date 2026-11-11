# Code review — compatibility — 007-audit-by-deeds, ratification pass r1c2

Scope: `git diff 95b73560..HEAD -- ':(exclude)docs/designs'`, read against the
full current `02-proposal.md`. Ratification: check only that the post-r1
fixes did not break what the compatibility seat approved in round 1, and that
they hold against the checklist.

## Round-1 findings: status

1. **minor (r1) — third enumeration route promised but not implemented (A4
   unverified): RESOLVED.**
   - `node_modules` is now installed in the worktree, so A4 is finally
     verifiable, and the pinned `@opencode/client` settles it:
     `session` exposes `list: (input?: SessionListInput) => Promise<SessionsResponse>`
     (`dist/promise/generated/client.d.ts:33`), with `SessionsResponse = { data,
     cursor: { previous?, next? } }` and `SessionsCursor = string`
     (`types.d.ts:3342–3348, 85`), `SessionListInput` carries `limit`/`cursor`,
     `SessionInfo` carries `parentID` — exactly the shapes the walk's
     `sessionEntries`/`pageCursorNext` consume. The SDK exposes **no**
     `session.children` method; internally it derives children via
     `session.list` + `parentID` + cursor pages (`dist/chunks/contract-1x65x4hv.js:353–357`),
     confirming route 2 is the route real runs take with this pin.
   - The promised third route is now implemented: `directChildSessions`
     (runner.ts:609–624) GETs `/api/session/:id/children` against the
     configured `baseUrl`, using `basicAuthHeader` — the very header
     `createClient` sends, extracted in opencode-runner.ts:52–60 — and
     `encodeURIComponent` on the session id, mirroring the SDK's own path
     convention verified in the contract chunk (`path: /api/session/${encodeURIComponent(input.sessionID)}/…`).
     The `ServerConnection` is wired from `pipeline/cli.ts` (167–179) into
     both `runPipeline` and `resumePipeline`, and the walk's probe chain
     (children → list → connection → throw) matches the matrix's
     "children, else list+parentID, else direct GET — the walk always has a
     route" (runner.ts:680–830).
2. **minor (r1) — nine named fake clients not extended with the session-list
   method (Test strategy #9): RESOLVED.** All nine stubs now carry
   `list: async () => ({ data: [], cursor: { previous: null, next: null } })`
   on their session objects (reply-contract:143, pass-finalize:129,
   parallel-integration:144, parallel-fanout:157, budget:127,
   parallel-escalation:162, stage-lifecycle:75, funnel:133,
   proposal-size:132+176). The later call-site guard
   (`countDelegationParts`, runner.ts:1247–1249) additionally means
   non-delegating attempts never enumerate at all, so the round-1
   throw-path/gate-log noise is gone on both routes.

## Checklist re-pass on the changed code

1. **Matrix match:** all three compatibility-matrix enumeration routes now
   have code paths, probed in the matrix's stated order; the pinned-SDK case
   (no `children`) is exercised end-to-end by the new list-route tests
   (child-session-deeds.test.ts:198–258, child-deeds-retry.test.ts:292). The
   `subagent: []` matcher row and doc-comment updates match §1. No OS-specific
   code added; macOS/Windows rows stay "no" honestly.
2. **Node floor (≥22.12):** new APIs are `AbortSignal.timeout`, global
   `fetch`, `new URL` — all well under the floor. No engines/package.json
   drift.
3. **Paths:** the direct route builds its URL with `new URL(abs, baseUrl)`
   and `encodeURIComponent`, not string concatenation.
4. **Guarded platform imports:** none introduced.
5. **Locale/layout:** new prompt lines are ASCII; matcher segment logic
   untouched by the fixes.
6. **Native deps:** none added (the diff carries no dependency changes; the
   installed `node_modules` is just Rollout step 1 being done).
7. **Platform tests:** all new walk/deeds tests run on fakes (no real
   network), so the Linux-CI claim in Test strategy #9 holds; the nine stub
   extensions keep every pre-existing test running through the walk's
   success path.

## What the fixes must not have broken (r1-approved behavior)

Spot-checked intact: the parent-first audit order, the `(via child session …)`
violation tag, the `console.warn` + `gateLog` partial-audit path, the
zero-descendant tripwire, and the §3 `failureNote` seam
(runner.ts:1021, 1090–1097, set at 1726 and 1935). The `walkedDelegations`
per-attempt counter (runner.ts:1152) resets per attempt, so a fresh attempt 2
still walks.

## Verdict

Both round-1 minors are resolved with verified evidence; no new
compatibility breakage found. Approve.

FYI (no severity, out of specialty): the third route (direct HTTP) has no
test pin — Test strategy #5 did not name one, so it is not a planned-test
gap, but a fetch-mocked `ServerConnection` test would close the loop for the
matrix's "always has a route" claim; and the matrix row's endpoint text
(`GET /session/:id/children`) omits the `/api` prefix the code (correctly,
per the SDK's own paths) adds — worth a one-word proposal amendment if the
proposal is ever touched.
