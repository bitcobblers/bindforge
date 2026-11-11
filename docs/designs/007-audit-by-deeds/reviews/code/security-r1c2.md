# Code review — security, round 1, ratification pass c2

Scope: `git diff 1ce23f4d..HEAD -- ':(exclude)docs/designs'`, read against the
full current `02-proposal.md` and `docs/prompts/audit-by-deeds.md`. This pass
checks only that the revisions after the security seat's c1 approval did not
break what c1 approved.

Method note (transparency): this environment rejected shell execution, so the
literal line-diff could not be produced. The security-relevant surface of the
diff range — `independence.ts`, the child walk and its three enumeration
routes in `runner.ts`, the `ServerConnection` wiring in `pipeline/cli.ts`, the
retry-feedback seam, and the walk/retry test files — was instead reviewed in
full at current HEAD and cross-checked against the c1 seat's recorded state
(`directChildSessions` still at runner.ts:609–624, byte-for-byte as reviewed in
c1; the tests' `ChildWalk` type still takes no options argument). Every
checklist item was evaluated against the current state; none was skipped.

## Checklist results

1. **External input validated at the boundary** — pass. The third route's
   unvalidated `response.json()` is shaped only through `sessionEntries`
   (runner.ts:563–590), which type-checks `id`/`parentID` and drops malformed
   entries; `pageCursorNext` type-checks the cursor. The matcher consumes
   message records defensively (`m.time?.created ?? 0`, `m.content ?? []`).
2. **Path handling** — pass. `directChildSessions` (runner.ts:609–624) builds
   its URL with `new URL('/api/session/<enc>/children', baseUrl)` and
   `encodeURIComponent(sessionID)` — a server-sourced session id cannot inject
   path segments. No shell string interpolation added; the pre-existing git
   helpers stay arg-array `spawnSync`.
3. **Injection** — pass. The segment regexes (`TRANSCRIPTS_SEGMENT`,
   `REVIEWS_SEGMENT`, `REVIEWS_KEY`) are linear single-pass patterns, no
   catastrophic backtracking. The unknown-tool scan is node-capped
   (`MAX_PATH_SCAN_NODES = 256`).
4. **Secrets/PII** — pass. The `ServerConnection` credentials
   (runner.ts:89–93) flow only CLI → in-memory run options →
   `basicAuthHeader`; nothing enters the checkpoint (verified: `opts.server`
   is read at exactly one site, the walk call at runner.ts:1256). The
   `basicAuthHeader` export (opencode-runner.ts:52–60) sends no header when no
   password is configured. Error/gate lines built from the direct route name
   only `url.pathname` and status — never baseUrl userinfo or credentials.
   No secrets in fixtures.
5. **New dependencies** — pass. `package.json` is unchanged in substance:
   `@opencode/client ^2.1.4`, `yargs`, `zod` + the pre-existing dev deps; no
   new runtime primitive, matching the compatibility matrix.
6. **Filesystem writes** — pass. No new writes; checkpoint writing untouched.
7. **Clean rejection** — pass. `directChildSessions` rejects a non-OK response
   with a plain bounded message; every walk throw (fetch failure, non-OK,
   repeated cursor, page-budget exhaustion, expired stage deadline) lands in
   the call site's caught partial-audit path (runner.ts:1272–1297), which
   names the truncation through `brief(printable(...))` on the warn channel,
   keeps violations already in hand, and never fails the stage on an
   infrastructure fault (A10 posture).

## What the post-c1 revisions changed, security read

The visible post-c1 additions are the walk's deadline clamp (`budgetMs()`,
runner.ts:668–677), the `MAX_SESSION_LIST_PAGES` crawl budget, the
repeated-cursor stop, and the delegation-count walk guard
(`walkedDelegations`, per-attempt, consistent with §2's "once per attempt"
cost rule). All are work-bounding additions on the credential-bearing fetch
path; none widens the attack surface, none touches credential handling, and
the caught partial-audit semantics (sec r1c0 #1: evidence in hand survives)
are intact and pinned (`a failed descendant fetch erases only the deeds it
never saw`). No regression of anything c1 approved.

## Findings

- **minor (carried open from c1, unaddressed by the revisions)** —
  `tools/orchestration/src/pipeline/runner.ts:609-624, 789-825`: the third
  enumeration route (`directChildSessions` + `ServerConnection` wiring) is
  still executed by no test. At current HEAD no pipeline test stubs
  `globalThis.fetch`, no fake client lacks both `session.children` and
  `session.list`, the tests' `ChildWalk` type (child-session-deeds.test.ts:107–111)
  still takes no options argument, and nothing passes `{ connection }`. The
  compat matrix's "the walk always has a route" claim over a
  credential-bearing fetch path remains verified by inspection only. The c1
  fix stands: stub fetch, use a client with neither SDK method, call
  `listChildSessionMessages(client, id, ms, { connection: { baseUrl, username,
  password } })`, assert the request hits `/api/session/<id>/children` with
  the expected `Authorization` header, that its records feed the audit (a
  child transcripts read fails the attempt), and that a non-OK status lands
  in the caught partial-audit path.

## Verdict

approve — zero open blocker/major findings; the post-c1 revisions did not
break the c1 approval. One minor from c1 remains open and non-blocking.
