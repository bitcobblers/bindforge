# Code review — security, round 1 dissent pass c1

Scope: `git diff 95b73560fab5571ad65610ffe4f790d080d715c1..HEAD -- ':(exclude)docs/designs'`
(16 files: runner.ts, independence.ts, opencode-runner.ts, cli.ts, 12 test files),
read against `02-proposal.md` and `docs/prompts/audit-by-deeds.md`.
Checklist: `references/security.md`. This pass checks the r1 findings first;
new findings only where the changed code makes them true.

## Re-check of round-1 findings

### 1. major — child-walk catch erased collected violations (`violations.length = 0`) — RESOLVED

The reset is gone (runner.ts:1272–1297). The catch keeps the `violations`
array from the generator passes that succeeded, names the kept count in both
the `console.warn` and the `gateLog('independenceAudit')` entry, and the
flow falls through to the `violations.length > 0` failure branch at
runner.ts:1299–1303. The generator's throw point is `descendantMessages()`
before a child is yielded, so every already-yielded child's audit survives.
Pinned as requested: `child-session-deeds.test.ts` —
`fails the attempt on a deed seen before a later child fetch throws`
(first child reads `docs/designs/002-x/transcripts/proposal-r0.md`, second
child's `message.list` throws): asserts `outcome.ok === false`, the failure
names `independence violation` / `transcripts` / `(via child session`, and
the warn names `ses_bad_child`. Matches the design's failure-mode posture
(Risks: gap for deeds UNSEEN, never erasure of evidence in hand).

### 2. minor — compat-matrix third enumeration route (direct `GET /session/:id/children`) not implemented — RESOLVED IN CODE (pin missing, see new finding)

`directChildSessions()` (runner.ts:609–624) implements the matrix's third
route; the walk now probes `session.children` → paged `session.list` →
direct HTTP when a `ServerConnection` is configured, and throws only when no
route exists at all (runner.ts:824–828). Wiring is real: `ServerConnection`
added to `RunOptions`/`ResumeOptions` (the resume lane casts through and
carries the field), built in `cli.ts:171–180` from the same baseUrl and
credentials the client got, and used at the audit call site
(runner.ts:1256). Security review of the new path:
- sessionID is `encodeURIComponent`-ed into the URL path — no path injection;
- `new URL(..., connection.baseUrl)` — baseUrl is operator CLI config, not
  untrusted data; an invalid baseUrl throws and lands in the caught
  partial-audit path (clean rejection, checklist #7);
- the auth header is the extracted `basicAuthHeader()` — byte-identical to
  what `createClient` sends; sent only to the configured origin (fetch spec
  strips `Authorization` on cross-origin redirects);
- credentials never reach a log: the error strings interpolate `url.pathname`
  and status only; `opts.server` appears in exactly one code path
  (grep: runner.ts:1256) and nothing serializes it toward the checkpoint;
- the response is unvalidated but `sessionEntries()` shapes it strictly
  (string non-empty `id` required, runner.ts:573–588) before any value is
  fed back into a client call;
- the request is bounded by `AbortSignal.timeout(budgetMs())`, the deadline
  clamp included.
The matrix row (02-proposal.md:257) matches the shipped envelope again.
However, no test exercises this route (see finding N1).

### 3. nit — walk-failure warn interpolated the server error raw — RESOLVED

The warn now wraps the error in `brief(printable(...))` (runner.ts:1283),
consistent with the module's posture. The `gateLog` reason stays raw at the
call site but that is fine: `gateLog()` sanitizes internally
(`reason: printable(result.reason)`, runner.ts:1612), so the checkpoint
line carries no control bytes either.

### 4. nit — `session.list` crawl had no anti-loop bound — RESOLVED

The rebuilt crawl (runner.ts:718–770) adds both requested bounds: a
repeated `cursor.next` throws (walk stops instead of re-accumulating
duplicates), and `MAX_SESSION_LIST_PAGES = 50` caps page count and map size
independent of the server's total session store. Both throws land in the
call site's caught partial-audit path, naming the truncation. Pinned:
`degrades to the session-list cursor never advances` asserts
`listCalls() < 100`, a clean pass on the parent list, and a warn. The
pre-existing `listSessionMessages` loop is untouched by this diff — my nit
was about not cloning the flaw in new code, which no longer applies.

## Security checklist over the changed code (c1 delta)

1. External input: session-list/children payloads shaped by
   `sessionEntries`/`pageCursorNext`; `budgetedStringValues` puts the
   access-key value subtrees under the same 256-node cap (recursion depth
   bounded by the budget — no unbounded descent on a hostile deep value). OK.
2. Path handling: encoded URL path; no shell interpolation. OK.
3. Injection: no new regex or path sinks. OK.
4. Secrets: no credential in source, fixtures, or log lines; `server` is
   in-memory only. OK.
5. New dependencies: none in the diff. OK.
6. Filesystem writes: none new. OK.
7. Clean rejection: every walk throw (deadline, cursor repeat, page cap,
   HTTP status, missing route) lands in the caught partial-audit path with
   collected violations intact. OK.

The new delegation-keyed guard (`countDelegationParts` + `walkedDelegations`,
runner.ts:1244–1249) keys on ANY `subagent` part regardless of status or age
— deliberately wider than the tripwire's completed-in-window test, and
pinned by the stale-delegation test, so the exemption cannot hide a deed
behind an out-of-window part. Not weakening the detection posture; accepted.

## New findings

- **[minor] N1 — the third enumeration route ships unpinned.**
  `directChildSessions`/`ServerConnection` (runner.ts:609–624, cli.ts:171–180)
  is a credential-bearing network path that no test executes: every fake
  client in the suite carries `session.list` (or `children`), so the walk
  never reaches route three, and the tests' `ChildWalk` type
  (child-session-deeds.test.ts:107–111) doesn't even accept the options
  argument. The compat matrix's "the walk always has a route" claim is again
  verified by inspection only — exactly the shape my r1 finding #2 was about,
  one level down. Fix: pin the route — stub `globalThis.fetch`, a fake client
  with neither `session.children` nor `session.list`, and walk with
  `{ connection: { baseUrl: 'http://127.0.0.1:1', username, password } }`:
  assert the recorded request URL is `/api/session/<id>/children` with the
  expected `Authorization` header, and that its entries feed the same audit
  (a child transcript read fails the attempt).

## FYI (no severity, out of specialty)

- The direct route builds its URL with an absolute path
  (`new URL('/api/...', baseUrl)`), which drops any path segment a
  `--baseUrl` like `http://host/gateway` might carry; the DEFAULT_BASE_URL
  note documents root-pointing base URLs, so this is consistent today —
  worth a correctness seat's eye only if sub-path deployments are in scope.

## Verdict

All four round-1 findings resolved in code; the majors are closed with pins.
One new minor (unpinned credential-bearing fallback route) — non-blocking.
