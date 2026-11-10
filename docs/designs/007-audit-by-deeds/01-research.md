# Research: Audit by deeds — independence audit on acts, retry feedback, and pinned prompt lines

## Summary

The source prompt names three harness defects in `tools/orchestration`, and all
three reproduce in the current code: the independence audit gives every tool not
listed in `ACCESS_FIELDS` — including `subagent` and `execute` — a full scan of
every string in the tool input (source:
tools/orchestration/src/pipeline/independence.ts:69-82,168-189); the
non-resumable retry in `runWithRetry` re-renders the identical template prompt
with nothing about the attempt-1 failure (source:
tools/orchestration/src/pipeline/runner.ts:1453-1497,693-710); and the two
prompt lines run 005 proved necessary are absent from `prompts/design-review.md`,
`prompts/test-authoring.md` and `prompts/test-update.md` (source:
tools/orchestration/prompts/design-review.md, test-authoring.md,
test-update.md). The server API the child-session fix needs already exists —
sessions carry `parentID` and `GET /session/:id/children` returns a session's
children — but the runner never calls any session-enumeration method, and none
of the nine fake clients in the test suite implements one (source:
https://opencode.ai/docs/server/, grep `session\.` over tools/orchestration/src,
grep `session: {` over tools/orchestration/tests/pipeline — 10 matches in 9
files). The biggest design surface is the definition of "judge where a tool
looks, never what text it carries" for unknown tools: one existing test pins
exactly the behavior the change removes, and the exemption is only safe if the
child-session deeds check lands in the same change.

## Findings

How the current matcher eats its own words (defect 1, verified in code)

- `ACCESS_FIELDS` enumerates access fields for `read`, `list`, `glob`, `grep`,
  `shell`, `bash`, `write`, `edit`, `patch` only; any other tool name —
  `subagent`, `execute`, `question`, … — falls through `accessValues` to
  `stringValues(input)`, which recurses through every array and object and
  returns every string in the input, so a delegation prompt or a program-text
  field is matched against `TRANSCRIPTS_SEGMENT`/`REVIEWS_SEGMENT` exactly like
  a path (source: tools/orchestration/src/pipeline/independence.ts:69-82,
  97-101, 132-134, 168-189).
- The three run-005 flavors in the source (subagent prohibition recital at
  `implement`, same flavor at `fix`, `execute` test-code string literals at
  `test-update`) are each explained by that one fallback; the numbers (977 s
  lost, the escalation) are recorded only on the 005 branch
  (source: docs/prompts/audit-by-deeds.md:13-25).
- The "judge paths, not content" discipline already exists twice in the same
  file: `write`/`edit`/`patch` are scoped to their path fields ("Writing is
  judged on the written-to path only (never the content)") and `grep`'s
  `pattern` is treated as query text while its target fields are scanned —
  each with a pinned regression test. Extending the precedent to
  content-carrying and delegating tools is consistent, not new doctrine
  (source: tools/orchestration/src/pipeline/independence.ts:74-82,
  tests/pipeline/independence.test.ts:134-153, 201-212).
- Deeds-vs-words is already the audit's stated posture for other cases: only
  COMPLETED tool parts count (a permission-denied read delivered nothing), and
  a standalone `shell` message is judged as an act (source:
  tools/orchestration/src/pipeline/independence.ts:22-28,116-131).
- One existing test pins the behavior the change removes:
  "scans nested input values, not just the first field" asserts that an
  `execute` part with a nested `{ path: '…/transcripts/y.md' }` value FAILS.
  A "paths only, not payloads" rule must deliberately re-shape or replace this
  pin (source: tools/orchestration/tests/pipeline/independence.test.ts:124-130).

Child sessions are real and enumerable (defect 1's new detection)

- OpenCode subagents "run with fresh context in foreground or background child
  sessions" through the `subagent` tool; `mode: subagent` agents run only in a
  child session (source: https://opencode.ai/docs/agents/).
- The server session schema carries `parentID` (create body `{ parentID?,
  title? }`) and exposes `GET /session/:id/children` returning `Session[]`;
  `GET /session` lists all sessions (source: https://opencode.ai/docs/server/).
- The runner touches exactly three session methods today — `session.create`,
  `session.prompt`, `session.remove` — plus paginated `message.list`; there is
  no session-enumeration call anywhere in `src/` (source: grep `session\.` over
  tools/orchestration/src — matches only at runner.ts:538,719,734,896,929 and
  opencode-runner.ts:189,250).
- Nine pipeline test files fake the client with only `session.create`/`prompt`/
  `remove` and `message.list`; a child-enumeration call in the audit path must
  be added to every fake that drives `executeInference` (source: grep
  `session: {` over tools/orchestration/tests/pipeline — 10 matches across
  reply-contract, pass-finalize, parallel-integration, parallel-fanout, budget,
  parallel-escalation, stage-lifecycle, funnel, proposal-size).
- The audit is pure and synchronous over a message list today
  (`auditSessionIndependence(messages, opts)`); a child walk needs the client
  and the parent sessionID, so either it moves up into `executeInference`
  (which already holds both, and runs the audit once per attempt after the
  reply) or the module gains an async wrapper (source:
  tools/orchestration/src/pipeline/independence.ts:104-107, runner.ts:832-849).
- The `since` filter already re-scopes a retried attempt: messages older than
  the attempt's prompt moment are skipped, so child message lists fetched
  through `listSessionMessages` and run through the same matcher inherit the
  "don't pay twice" rule without filtering children by session-creation time
  (source: tools/orchestration/src/pipeline/independence.ts:44-50,112-114;
  tests/pipeline/independence.test.ts:110-122; runner.ts:473-499).

The retry that never learns (defect 2, verified in code)

- `runWithRetry` calls `executeStage` for attempt 1; only an `invalid-reply`
  failure resumes the same session (`retryInSameSession` checks
  `failure.startsWith('invalid-reply')`). An independence violation or a
  stage-timeout is non-resumable, and the fresh attempt-2 handle spreads the
  same `vars` into the same template, producing a byte-identical prompt
  (source: tools/orchestration/src/pipeline/runner.ts:1121-1126,1453-1497,
  693-710).
- There is an in-repo precedent for feeding the failure into the retry:
  `repairPrompt` opens the same-session retry with "Your previous attempt at
  the ${def.name} stage ended without a usable reply (${failure})" — the
  fresh-session path simply lacks the analogous injection (source:
  tools/orchestration/src/pipeline/runner.ts:1133-1140).
- The failure text is already sanitized for reuse: independence violations are
  built through `brief()` → `printable()`, and every gate-log reason passes
  through `printable()`; injecting `out.failure` into the attempt-2 prompt
  therefore carries no unsanitized control bytes (source:
  tools/orchestration/src/pipeline/independence.ts:191-207,
  runner.ts:1144-1161).
- The same fresh-session attempt-2 gap exists in the review-pass fan-out
  (`runReviewPass`'s inline retry, non-resumable branch), and the source's own
  motivating example — run 005's editorial-security seat globbing
  `reviews/design/*` and failing twice — is a review seat, i.e. it lives on
  that lane, while the required change names only `runWithRetry`. Whether both
  lanes must carry the failure reason is a design decision (source:
  tools/orchestration/src/pipeline/runner.ts:1239-1284;
  docs/prompts/audit-by-deeds.md:57-60).

The two prompt lines and their pins (defect 3, verified in code)

- `prompts/design-review.md` carries the existing "Do not open review reports
  or transcripts, and do not ask for them" sentence but no bright line that
  listing/globbing/grepping those directories counts as opening them (source:
  tools/orchestration/prompts/design-review.md:3-8).
- `prompts/test-authoring.md` carries the transcripts prohibition but no
  formatter duty; `prompts/test-update.md` carries neither a transcripts line
  nor a formatter duty (source: tools/orchestration/prompts/test-authoring.md,
  tools/orchestration/prompts/test-update.md).
- The pin pattern the source asks for exists twice: `fix-trigger.test.ts`
  loads from `prompts/`, renders, and asserts on content
  (`expect(text).toContain('npm run check is red')`), and `prompts.test.ts`
  pins prompt sentences in a "run-2 stage guidance" block — including
  `expect(text).toContain('Do not open review reports or transcripts')`, so
  the design-review pin will extend a block that already guards that sentence
  (source: tools/orchestration/tests/pipeline/fix-trigger.test.ts:58-67,
  tests/pipeline/prompts.test.ts:179-210).
- The formatter instruction is consistent with the harness's ownership model:
  the runner deliberately never executes a formatter (a planted
  `node_modules/.bin` binary in a model-writable worktree is untrackable), the
  test-engineer agent is shell-allowed `npm run format*`, and `npm run check`
  chains `lint && format:check && test` — the prompt line hands the last step
  to the stage that owns the files (source:
  tools/orchestration/src/pipeline/runner.ts:267-285,
  .opencode/agents/code/test-engineer.md:29-31,
  tools/orchestration/package.json:23).
- Prettier drift is a plausible recurring hazard: the package formats with
  prettier plus the `@trivago/prettier-plugin-sort-imports` plugin, which
  rewrites import order — exactly the kind of churn a fresh test batch
  reintroduces (source: tools/orchestration/package.json:19-20,39;
  drift history in run 005 per docs/prompts/audit-by-deeds.md:85-89).

Security landscape lens

- The audit is defense-in-depth over the agent-permission fence, and its own
  header states the limit: it is text matching over recorded command lines and
  cannot prove anything about obfuscated commands; the fence (deny read of
  `**/transcripts/**`, `**/reviews/**` for review seats) is the primary
  control (source: tools/orchestration/src/pipeline/independence.ts:6-41,
  .opencode/agents/review/design-security.md:17-25,
  .opencode/agents/code/test-engineer.md:17-22).
- The perverse loop the source names — prompt hardening makes agents quote the
  rule more, raising the false-positive rate of a content scan — is the
  audit's input data being misread as the act it describes; the classic name
  for a program conflating input that describes an access with the access
  itself is the confused-deputy problem (source:
  docs/prompts/audit-by-deeds.md:24-25;
  https://en.wikipedia.org/wiki/Confused_deputy_problem).
- Exempting `subagent` prompt text WITHOUT the child-deeds check would create
  a real bypass: today the audit never sees child sessions, so the delegation
  exemption alone would route forbidden reads through children with zero
  detection — the source's detection pins (test (c): a child reading
  `transcripts/` or globbing `reviews/**` fails the parent) are load-bearing,
  not optional (source: docs/prompts/audit-by-deeds.md:29-51; absence of any
  child-session call in runner.ts per grep above).

## Prior art

- OpenCode's own permission fence is path/command-scoped, never content-scoped:
  permission rules match an `action` against a `resource` that is a "path,
  command, agent ID, or other value matched by the action" with wildcard
  globs. The audit's full-scan fallback is the outlier relative to the control
  it supplements; judging unknown tools on access fields aligns the audit with
  the fence (source: https://opencode.ai/docs/agents/, "Permissions").
- Child sessions through delegation are a first-class server concept: sessions
  carry `parentID`, `GET /session/:id/children` exists, and the `subagent`
  tool's children are exactly those child sessions — so "enumerate the
  attempt's child sessions and run their completed parts through the matcher"
  is supported by the platform without new server surface (source:
  https://opencode.ai/docs/server/, https://opencode.ai/docs/agents/).
- In-repo precedent for every part of this change:
  - content-exemption: the run-003 misfire fix exempted `write`/`edit` payload
    ("code ABOUT run records is not a read of run records") with pinned
    regressions — the source asks to generalize exactly this (source:
    tools/orchestration/src/pipeline/independence.ts:18-28,78-79;
    tests/pipeline/independence.test.ts:132-153);
  - query-vs-target separation for `grep` (source:
    tools/orchestration/src/pipeline/independence.ts:74-75;
    tests/pipeline/independence.test.ts:201-212);
  - failure-in-prompt on retry: `repairPrompt` (source:
    tools/orchestration/src/pipeline/runner.ts:1133-1140);
  - prompt-line test pins: the run-2 guidance block and `fix-tests.md` pin
    (source: tools/orchestration/tests/pipeline/prompts.test.ts:179-311,
    fix-trigger.test.ts:58-67);
  - the ownership answer to formatting (fix→`npm run format`→test-update
    routing) instead of runner-executed formatters, kept alive by a comment
    pin (source: tools/orchestration/src/pipeline/runner.ts:267-285;
    tests/pipeline/fix-trigger.test.ts:15-20).
- The prior audit-hardening round (run 003's security r1 finding) is recorded
  in this tree: segment matching replaced substring matching, and the residual
  bypass was documented instead of chased — the same "deterministic plain-text
  coverage, nothing more" posture the new child-session matcher should keep
  (source: docs/designs/003-parallel-inference-lane/reviews/code/security-r1c1.md:28-44,
  independence.ts:30-41).
- The source prompt records that run 005's own security seat filed the missing
  child-session enumeration as a design major with near-verbatim wording for
  the gap; that receipt lives in the 005 branch's review reports (source:
  docs/prompts/audit-by-deeds.md:36-38; branch
  `pipeline/005-runner-executed-file-moves` exists — git ref
  `refs/heads/pipeline/005-runner-executed-file-moves`).

## Platform notes

- OpenCode server session API relevant to the audit rewrite: `GET /session`
  (list all), `POST /session` body `{ parentID?, title? }`, `GET
  /session/:id/children` → `Session[]`, `GET /session/:id/message` with
  pagination (source: https://opencode.ai/docs/server/).
- The runner pins `@opencode/client ^2.1.4`, a v2 client whose request paths
  are prefixed `/api` (source: tools/orchestration/package.json:46,
  package-lock.json:881-883, src/opencode-runner.ts:17-19). The SDK method name
  for listing sessions/children could NOT be verified in this worktree:
  `node_modules/@opencode/client` is not installed here (source: glob
  `tools/orchestration/node_modules/@opencode/client/**` — no matches; only
  `session.create/prompt/remove` are exercised, so the rest of the client
  surface is unconstrained by current code).
- Node floor is `>=22.12.0`; nothing in the change needs more (source:
  tools/orchestration/package.json:31).
- Message fetching must keep the paging discipline: the server defaults to
  newest-first 50-message pages, and `listSessionMessages` exists precisely to
  fetch asc-ordered pages via `cursor.next` — child message lists should reuse
  it (source: tools/orchestration/src/pipeline/runner.ts:459-499).
- The audit runs once per attempt after the reply, so child enumeration costs
  one children call plus one paged fetch per child per attempt — bounded and
  off the poll loop (source: tools/orchestration/src/pipeline/runner.ts:832-849).
- The stage agents that delegate run with permission fences that already deny
  `transcripts/**` reads, and stage agents evidently can launch subagents (the
  005 coder did), so no agent-configuration change is implied by the source
  (source: .opencode/agents/code/test-engineer.md:17-22,
  docs/prompts/audit-by-deeds.md:15-16; the `subagent` action is a normal
  permission action per https://opencode.ai/docs/agents/).

## Risks & unknowns

- The run-005 receipts (977 s implement loss, the two globbing seat failures,
  the prettier-drift escalation at 5/5 fix rounds) are only on branch
  `pipeline/005-runner-executed-file-moves` at
  `docs/designs/005-runner-executed-file-moves/.pipeline.json`; this worktree
  does not contain them (source: glob `docs/designs/005*/**` — no matches; git
  ref exists), so they are cited from the source prompt, not re-verified.
- Removing the unknown-tool full scan could regress detection for hypothetical
  tools that hide paths in nested non-standard fields (the fallback's stated
  purpose); the "resolve by effect" replacement must decide what counts as an
  access field (source: tools/orchestration/src/pipeline/independence.ts:64-68).
- Unknown whether code-mode `execute` tool invocations appear in the invoking
  session's own message list as tool parts, or run under separate child
  sessions; this decides whether the child walk also closes the `execute`
  effect surface or whether `execute` needs its own path-field handling
  (unverifiable here; the `execute` part shape the tests use is a fixture, not
  ground truth — tests/pipeline/independence.test.ts:124-130).
- `SessionInfo` timestamp fields were not verified (client package absent in
  this worktree); if children cannot be filtered by creation time, fall back to
  the message-level `since` filter, which the matcher already applies
  (source: tools/orchestration/src/pipeline/independence.ts:112-114,
  src/opencode-runner.ts:4-10).
- Attempt-2 prompts with injected failure text will quote forbidden paths —
  safe under the current audit, which scans tool parts and shell messages, not
  prompt text; the transcript records the full prompt regardless (source:
  tools/orchestration/src/pipeline/independence.ts:112-155,
  runner.ts:955-967).
- This run executes under the current buggy matcher: stage agents of THIS run
  must name allowed paths/commands directly in any subagent delegation and
  must not copy prohibition sentences from stage prompts into delegation text
  (source: docs/prompts/audit-by-deeds.md:91-96).
- Widening the audit to child sessions grows the false-attribution surface:
  a sibling run's sessions or a resumed session's older children must not be
  charged to this attempt; the `since` stamp and the parent-scoped children
  endpoint are the boundary (source:
  tools/orchestration/src/pipeline/independence.ts:44-50;
  https://opencode.ai/docs/server/).

## Open questions

1. What exactly is an "access field" for an unknown tool — a key-name allowlist
   (`path`, `filePath`, `pattern`, `command`, `directory`…), a value-shape
   heuristic, or nothing at all (effect-only judgment)? What happens to the
   existing nested-`path` pin at tests/pipeline/independence.test.ts:124-130 —
   does a nested `{path: '…/transcripts/y.md'}` inside an `execute` input still
   fail (it names a path), while the `code` payload never does?
2. Which tools are "delegation-like"? Only `subagent`, or anything that can
   spawn sessions? Should the child walk recurse (grandchildren — a subagent
   is itself forbidden from launching subagents only by convention: OpenCode's
   built-in `general` subagent "cannot launch more subagents", but custom
   agents can, so depth-1 may not suffice) (source:
   https://opencode.ai/docs/agents/)?
3. Does the attempt-2 failure-reason injection cover the review-pass lane
   (`runReviewPass`), whose non-resumable retry is the lane where run 005's
   deterministic seat habit actually failed twice (source:
   docs/prompts/audit-by-deeds.md:57-60 vs the named function
   `runWithRetry` at runner.ts:1453)?
4. How is the failure reason framed in the retry prompt — the raw printable
   gate-failure text, or a structured "rule broken + paths named" line for
   independence violations (the violation strings already name the offending
   paths via `brief()`)?
5. Should the design-review bright line be pinned by exact phrase or by
   `toContain`-style semantics (the existing pins are substring assertions)?
6. Does the test-engineer formatter line need any gate-side companion, given
   the format-red route (fix runs `npm run format`; test-owned files route to
   `test-update`) already exists as the ownership answer?
7. Where does the child-session fetch live — `independence.ts` gains an async
   client-aware entry point, or `executeInference` assembles the message lists
   and the pure matcher stays synchronous for testability?
