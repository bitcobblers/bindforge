# Build notes: design 007 (audit by deeds)

Implementation followed `02-proposal.md` as written, except for the
deviations below.

## Deviations from the design

1. **Walk route not fixed at install time (Rollout step 1 / A4 / A8).**
   `npm install` could not be run in this worktree (no shell access; the
   `@opencode/client` package is not installed here), so the pinned SDK's
   types could not be read to pick ONE enumeration route. Instead,
   `listChildSessionMessages` probes the client at call time:
   `session.children` when present (called per BFS node, `bind`-ed to the
   session object), else one paged `session.list` crawl indexed by
   `parentID`, else the walk throws and the audit call site degrades to the
   design's partial-audit path (`console.warn` + one gate-log line naming
   the session, attempt audited on its own message list alone — A10).
   The compat matrix says "A4 decides which, not whether"; here the choice
   is made per client at runtime, which also satisfies both routes' tests
   (the new stubs answer both). The matrix's third fallback — a direct
   `GET /session/:id/children` against the configured `baseUrl` with the
   client's auth headers — is NOT implemented: the SDK client's internals
   (base URL, auth) are unverifiable in this worktree and were not probed.
   If a future client exposes neither SDK route, the audit logs the gap
   rather than reaching the server directly.

2. **Test strategy #9's "nine fake clients extended with the session-list
   method" did not happen — test files are not editable at this stage.**
   The nine pre-existing fakes (reply-contract, pass-finalize,
   parallel-integration, parallel-fanout, budget, parallel-escalation,
   stage-lifecycle, funnel, proposal-size) still expose only
   `session.create/prompt/remove`, so every attempt they run takes the
   walk's neither-route error path above: a `console.warn` and one extra
   gate entry per audited attempt, then a parent-only audit. This is safe
   for them: every existing gate-log assertion filters by gate name
   (`stageFailure`, `operatorExtension`, `operatorForce`, `design-review`)
   or filters `!g.proceed`, and the new entry is named
   `independenceAudit` with `proceed: true`. No pre-existing test's verdict
   changes.

3. **Gate name chosen for the partial-audit gate-log line:**
   `independenceAudit` (the design mandates the line but not its gate name;
   it is deliberately distinct from every gate name any test asserts on).

4. **Path-scan node cap value:** `MAX_PATH_SCAN_NODES = 256`. The design
   requires "a node-capped queue that drops the deepest nodes" but leaves
   the number open; the scan is breadth-first so the cap drops the deepest
   nodes first.

5. **`brief()` is exported from `independence.ts`** (it was module-private)
   so `runner.ts` can bound `failureNote` with `brief(printable(...))`
   exactly as §3 specifies. No behavior change to the function.

6. **Attempt-2 feedback block placement:** appended after the rendered
   template and *before* the `replyFormat(schema)` footer, so the required
   JSON-format instruction stays last in the prompt. §3 says "append after
   the rendered template"; this honors that while keeping the existing
   footer-last posture every other prompt relies on.

7. **`npm run check` / `npm run format` could not be run** (shell tool
   denied in this stage). Code was written by hand against the package's
   prettier config (printWidth 100, single quotes, trailing commas,
   import groups `node:` → external → relative, specifier sort); small
   `format:check` drift is possible (exactly the A12 risk class — the
   `fix` stage route owns clearing it if it appears).

## Additions beyond the design's letter (no semantic change)

- The child-walk's enumeration/message errors are wrapped to name the
  session that could not be enumerated (`enumerating the children of …`,
  `fetching the message list of …`, `listing sessions to find the children
  of …`) — §2 requires the gate line to "name the session", and `bounded()`
  alone would not.
- Violations collected from descendants are DISCARDED when the walk later
  errors, per §2's "the audit proceeds on the parent list alone" (the
  parent list is the only list audited in the failed-walk case).

## Files touched

- `tools/orchestration/src/pipeline/independence.ts` — `subagent: []` in
  `ACCESS_FIELDS`; `PATH_ACCESS_KEYS` path-key scan replacing the
  unknown-tool `stringValues(input)` full scan; doc comments updated;
  `brief` exported.
- `tools/orchestration/src/pipeline/runner.ts` — `listChildSessionMessages`
  (BFS, visited-set, one-list-at-a-time generator) beside
  `listSessionMessages`; parent-first audit at the call site with
  `(via child session …)` suffixes, the zero-descendant tripwire, and the
  caught/logged walk failure; `StageHandle.failureNote` + the
  "Previous attempt's gate failure" block in `executeInference`;
  `failureNote` set at both fresh-session attempt-2 sites
  (`runWithRetry`, `runReviewPass.runSeat`).
- `tools/orchestration/prompts/design-review.md` — the bright line after
  the independence sentences.
- `tools/orchestration/prompts/test-authoring.md`,
  `tools/orchestration/prompts/test-update.md` — the formatter-ownership
  line (`npx prettier --write` kept on one line so the substring pins
  match).

No test files were modified. Budget, escalation wording, `retryInSameSession`,
the nudge loop, and the checkpoint schema are untouched.
