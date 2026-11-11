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
   client's auth headers — was NOT implemented at first (the SDK client's
   internals were unverifiable in this worktree); code review r1c0
   (security finding 2) required closing that gap, and the fix stage
   implemented it: see "Code-review round 1 addendum" below.

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
   changes. *(Superseded: the test-update stages later extended all nine
   fakes — see the Test-update addendum at the end.)*

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
- Violations collected from descendants were originally DISCARDED when the
  walk later errored, per §2's "the audit proceeds on the parent list
  alone". Code review r1c0 (security finding 1) showed that erases deeds
  already seen — worse than the design's stated failure mode, a gap for
  deeds UNSEEN. Superseded: collected child violations are now KEPT; only
  the unseen remainder of the walk is a gap. See the round-1 addendum.

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

No test files were modified at the implement stage *(superseded — the test
stages later modified 14 test paths; see the Test-update addendum at the
end)*. Budget, escalation wording, `retryInSameSession`,
the nudge loop, and the checkpoint schema are untouched.

## Fix-stage addendum (clearing the red `npm run check`)

The shell stayed denied at the fix stage, so the gate was audited
statically against the package's configs instead of by running it. Two
hand-written slips in the design-007 regions of `runner.ts` were fixed
there, consistent with deviation 7's forecast:

- `SessionChildApi.children?` was split across four lines although its
  one-line form is 98 chars — `format:check` drift (printWidth 100),
  collapsed to one line.
- The audit call site's `let violations` was never reassigned
  (`.length = 0`/`.push` mutate); rewritten `const`. Hygiene only — the
  package's ESLint config (`js.configs.recommended` +
  `tseslint.configs.recommended`) does not enable `prefer-const`, so
  this was not the gate failure.

The rest of the touched regions (`independence.ts`, the child-walk and
audit call sites, both `failureNote` retry sites, `report.ts`'s
`printable` import, and the three prompt files) were re-read against the
prettier config and the enabled lint rules and found clean; every line
over 100 chars in them is an unbreakable string literal, and the
pre-existing over-100 lines elsewhere are unchanged code that was green
before design 007. No test file needs changing: `testsNeedUpdate` stays
false.

## Fix-stage addendum 2 (gate actually run)

Shell was available this round. `npm run check` in `tools/orchestration`
was red on one rule: `@typescript-eslint/preserve-caught-error` flagged
the three child-walk error wrappers in `runner.ts` (`descendantMessages`
and `listChildSessionMessages`' two fetch sites) for rethrowing a new
`Error` inside `catch` without the original error. Fixed by passing
`{ cause: error }` — the house pattern already used at
`opencode-runner.ts` and `state-store.ts` — with no message or control
flow changes (a small addition beyond the design's letter, listed with
the other error-wrapping additions above). `npm run format` found no
drift; `npm run check` then exited green: lint clean, prettier clean,
241/241 tests passing. No test file was touched.

## Code-review round 1 addendum (fix stage)

All open findings from the three code-review seats (security, performance,
testing, all `request-changes`) were cleared on the source side:

- **Sec #1 (major):** the walk's catch no longer runs `violations.length =
  0`. Violations the walk collected before a mid-walk fetch failure survive
  into the normal `violations.length > 0` branch — a deed seen is a deed
  failed for — and the `console.warn`/gate line now names the kept count
  instead of claiming a parent-only audit when the attempt in fact fails.
- **Sec #2 (minor):** the compat matrix's third enumeration route is now
  implemented: with neither `session.children` nor `session.list` on the
  client, the walk issues a direct `GET /api/session/:id/children` against
  the configured `baseUrl` with the same basic-auth header `createClient`
  sends (`basicAuthHeader` extracted from `opencode-runner.ts` so the two
  routes share one header construction). The connection is threaded as a
  new OPTIONAL `server: { baseUrl, username?, password? }` field on
  `RunOptions`/`ResumeOptions` and set by `pipeline run|resume` in
  `pipeline/cli.ts` from the same `--base-url/--username/--password` the
  client is created with. (The path carries the v2 client's `/api` prefix,
  matching the SDK's own surface — a deliberate reading of the matrix's
  `GET /session/:id/children` wording.) `server` is in-memory run-options
  state, not a checkpoint or config schema change.
- **Sec #3 (nit):** the walk-failure `console.warn` wraps the
  server-supplied error string in `brief(printable(...))`.
- **Sec #4 + Perf #3:** the paged `session.list` crawl throws when
  `cursor.next` repeats, and a 50-page budget (`MAX_SESSION_LIST_PAGES`)
  bounds its request count and `byParent` size; either throw degrades to
  the caught partial-audit path instead of spinning.
- **Perf #1 (major):** the call site counts `subagent` parts over the whole
  audited list (`countDelegationParts`, any status, any age) and skips the
  walk entirely when the attempt delegated nothing; the zero-descendant
  tripwire is untouched for the case where a delegation part exists but
  enumeration returns nothing. Consequence for deviation 2: the nine
  pre-existing fakes never delegate, so their attempts now skip the walk
  entirely — the per-attempt warn + `independenceAudit` gate line
  deviation 2 described no longer appears in their run records.
- **Perf #2 (major):** a per-attempt `walkedDelegations` counter bounds the
  walk to at most one network pass per growth of the delegation count, so
  the nudge/`wait` re-entries of the stage loop re-audit the parent list
  (pure, cheap) without re-walking the tree.
- **Perf #4 (minor):** the walk takes the stage `deadline` (new optional
  `deadline` on `listChildSessionMessages`' options bag) and clamps every
  request's timeout to what the deadline leaves, throwing a named
  "truncated at the stage deadline" error into the same caught
  partial-audit path when it expires.
- **Perf #5 (nit):** in `independence.ts`, the `MAX_PATH_SCAN_NODES` budget
  is now ONE shared `ScanBudget` drawn on by both the breadth-first payload
  traversal and the value subtrees under access keys
  (`budgetedStringValues`), so the cap bounds the work under an access key
  as its comment always promised.

**Test-side findings not done here (test files are not editable at this
stage):** testing #1 (positive list-route pins with descendants present —
e.g. `treeClient` without `children`, a multi-page `cursor.next` crawl,
the #6 walk on the list route), testing #2 (the nine fakes extended with
`session.list`, Test strategy #9), testing #3 (`// proves: Test strategy
#8` citations on the two prompt pins), and testing #4 (assert exactly two
`stageFailure` gate entries in the budget pin). Security #1's pin (first
child violates, second fetch throws → the attempt still fails) and perf
#3's "one enumeration per walk" stub assertion are also new pins. All of
them are test-file changes → `testsNeedUpdate: true`; the source edits
above change no existing test's verdict.

## Fix-stage addendum 3 (this round: the red gate's cause is a test pin)

The shell stayed denied at this stage, so the gate was audited statically
again, as in addendum 1 — lint rules (`typescript-eslint` recommended:
no unused symbols in the new regions, no `any`, every `catch`-rethrow in
the walk carries `{ cause }`), the prettier config (printWidth 100, the
`node:` → external → relative import groups, one-line-vs-broken call
shapes), and then the r1 test batch against the implementation.

No source-side red was found:

- No `format:check` drift: every line over 100 chars in the touched
  regions is an unbreakable string literal or a `it(...)` title, which
  prettier leaves alone; no trailing whitespace, no tabs; the new test
  files' import blocks already match the plugin's grouping.
- Every other pin of the r1 test batch matches the implementation:
  the `list`-route pins (`session.list` + `parentID` index, the
  `cursor.next` crawl — 3 calls for a 5-session 2-page store, 1 call for
  a single-page store: perf #3's "one enumeration per walk"), the
  security #1 pin (a deed collected before a mid-walk fetch failure still
  fails the attempt, and the warn names `ses_bad_child`), the perf #1/#2
  cost pins (an attempt with no `subagent` part makes zero `session.list`
  calls; at most one enumeration per attempt), the tripwire warn naming
  the attempt's own session, and both attempt-2 failure-note pins on the
  single-stage and review-pass lanes.

**The one red assertion:**
`tests/pipeline/child-deeds-retry.test.ts:277` —
`expect(saved.gateLog.filter((g) => g.gate === 'stageFailure')).toHaveLength(2)`.
That scenario runs `implement` once through `drive`'s `runWithRetry`,
which logs `stageFailure` after attempt 1 (runner.ts:1911) and, when
attempt 2 fails, calls `finalizeEscalate` — whose entry is named
`escalate` (runner.ts:2439), reason
`stage implement failed twice (independence violation: …)`. So the run
record holds exactly one gate entry per failure — `stageFailure` for
attempt 1, `escalate` for attempt 2 — which is what Test strategy #6
("budget is unchanged (attempt 2 fails → escalate, one gate entry per
failure)") asks for, and §3 lists gate entries among what stays
untouched. Giving attempt 2 a second `stageFailure` entry would be the
behavior change §3 forbids, and it would rewrite the gate log of every
two-attempt escalation in every lane; so the fix belongs to the pin, not
to the runner. Test-side fix: assert the pair, e.g. one `stageFailure`
plus one `escalate` naming the same stage — or count
`['stageFailure', 'escalate']` together and expect 2.

No deviation from the proposal: no source file changed this round, and no
test file was touched (the runner reverts those paths anyway).
`testsNeedUpdate: true`.

## Test-update addendum (2026-11-10: the test side lands; final change set)

The test-authoring and test-update stages completed everything the fix-stage
addendums listed as "not done here (test files are not editable at this
stage)". Deviation 2 and this file's "No test files were modified" line are
superseded by this round and describe the implement-stage state only — they
are kept as the stage record, not the final record.

Test files in the final change set:

- `tests/pipeline/child-session-deeds.test.ts` (new) — Test strategy #5 at
  walk and call-site level: BFS reaches grandchildren, a parentID cycle
  terminates, pre-`since` child messages are fetched but not judged, the
  `list` route via the `parentID` index, one enumeration per walk (perf #3's
  pin), `cursor.next` paging across pages, the zero-descendant tripwire warn,
  zero walk cost for an attempt that delegated nothing (perf #1's pin), the
  repeated-cursor and page-budget crawls degrading to the partial audit, a
  child deed behind an out-of-window delegation still failing, and security
  #1's pin (a deed collected before a later child fetch throws still fails
  the attempt).
- `tests/pipeline/child-deeds-retry.test.ts` (new) — Test strategy #6 and #7
  end to end on a temp git repo with a mock client: a child's `transcripts/`
  read fails the attempt with `(via child session …)`, the fresh attempt-2
  prompts on both the single-stage and review-seat lanes carry attempt 1's
  failure reason, and the budget is unchanged. The red pin addendum 3
  diagnosed is fixed exactly as it prescribed — one `stageFailure` (attempt
  1) plus one `escalate` (attempt 2) gate entry — with no runner change.
- `tests/pipeline/independence.test.ts` — matcher pins for Test strategy
  #1–#4 (subagent delegation text, execute payload literals, the path-key
  rule under a non-access key, and child-level read/glob/other-report
  violations); the pre-existing nested-path pin at :124 survives untouched.
- `tests/pipeline/prompts.test.ts` — the bright-line `toContain` in the
  design-review block and the formatter-duty pins for `test-authoring` and
  `test-update`, each carrying its `// proves: Test strategy #8` citation
  (testing #3 closed).
- The nine pre-existing fakes extended per Test strategy #9 — budget,
  funnel, parallel-escalation, parallel-fanout, parallel-integration,
  pass-finalize, proposal-size (two fixtures), reply-contract,
  stage-lifecycle: each session stub gains
  `list: async () => ({ data: [], cursor: { previous: null, next: null } })`
  (the truth for fixtures that create no child sessions). With the perf #1
  guard they still never walk, so no verdict changes.

Code-review round 1 ended with all five seats (security, performance,
testing, compatibility, correctness) approving; the c2 ratification passes
found no regression. Left open and non-blocking at HEAD:

- security-r1c2 minor: the direct-HTTP third enumeration route
  (`directChildSessions` + `ServerConnection`) is executed by no test — no
  fetch stub exists; the compat matrix's "always has a route" claim is
  verified by inspection only.
- correctness-r1c2 minors/nits: the walk guard is keyed on the recorded part
  name `subagent` (fixture-only evidence); `directChildSessions` replaces a
  path component of a `--base-url` that carries one (fails closed); the
  zero-descendant tripwire inspects the parent list only;
  `npm run typecheck` (not part of `check`) has never been run — shell was
  denied in every seat that asked.
- testing-r1c2 nit: the walk's deadline clamp (`budgetMs`) and its
  "truncated at the stage deadline" throw are live in production but carry
  no test pin — no test passes a `deadline` option.

Addition to the additions list (correctness r1c2 nit 5): `listChildSessionMessages`
is exported from `runner.ts` (the design's snippet kept it module-private) so
the unit tests can drive the walk directly.
