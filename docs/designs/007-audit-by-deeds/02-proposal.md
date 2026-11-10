# Design: Judge the independence audit on acts — child-session deeds, retry feedback, pinned prompt lines

## Context

`tools/orchestration`'s review-independence audit (`src/pipeline/independence.ts`)
inspects what each stage attempt's session actually did — completed tool parts
and shell messages — and fails the attempt on a run-records access. Today every
tool not in `ACCESS_FIELDS`, including `subagent` and `execute`, falls back to a
full scan of every string in the tool input (A1), so a delegation prompt that
recites the prohibition, or test code whose string literals name `transcripts/`,
fails an attempt that accessed nothing. The source document
([audit-by-deeds.md](../../prompts/audit-by-deeds.md), [research](./01-research.md))
attributes three run-005 losses to this (A2), and adds two sibling defects: the
non-resumable retry in `runner.ts` gives attempt 2 a byte-identical prompt with
nothing about attempt 1's failure (`runWithRetry`, runner.ts:1453–1497), and two
prompt lines the run proved necessary — a design-review bright line and a
test-engineer formatter duty — have no test pins.

The one design principle across all three changes: **the audit judges where a
tool looks, never what text it carries.**

## Goals / Non-goals

- Goals:
  - `subagent` (delegation) parts are never judged on their input text;
    the child sessions' own completed tool parts become the evidence, so the
    audit sees deeds it is blind to today (source §1).
  - `execute` and every unknown tool are judged on path-naming access fields
    only — never payload text (source §1).
  - The child-session detection is pinned, not just the new exemptions: a
    child that reads a `transcripts/` file or globs `reviews/**` fails the
    parent attempt (source §1, test pins (a)–(c)).
  - A fresh-session attempt 2 carries attempt 1's failure reason in its prompt
    (source §2); budget and escalation flow stay identical.
  - The two prompt lines re-land as reviewed artifacts, each pinned by a
    rendered-content test (source §3).
  - Every change stays inside `tools/orchestration`; existing tests stay
    green.
- Non-goals (each prevents scope creep at code review):
  - No new agent-permission rules, agent config changes, or server-side effect
    recording — the permission fence stays the primary control (A10).
  - No change to the retry budget, nudge loop, escalation wording, or
    resumability rule (`retryInSameSession`, runner.ts:1121–1126) — only
    attempt-2 prompt content changes.
  - No runner-executed formatter — rejection re-affirmed in run 003, pinned at
    `tests/pipeline/fix-trigger.test.ts:15–20` (A11).
  - No claim of detecting obfuscated or unrecorded accesses — the audit's
    stated coverage posture (A10) is unchanged.
  - No checkpoint (`PipelineState`) schema change.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | Tools absent from `ACCESS_FIELDS` (incl. `subagent`, `execute`) get `stringValues(input)` — a scan of every string in the tool input | verified | tools/orchestration/src/pipeline/independence.ts:69–82,168–189 |
| A2 | Run-005's three audit flavors (977 s `implement` loss, two seat failures, the prettier escalation at 5/5 fix rounds) happened as the source describes; the receipts live only on branch `pipeline/005-runner-executed-file-moves`, not in this worktree | unverified | docs/prompts/audit-by-deeds.md:13–25,85–89; research Risks |
| A3 | `subagent` delegation runs as child sessions; the session schema carries `parentID`; the server exposes `GET /session/:id/children` and `GET /session` | verified | https://opencode.ai/docs/agents/, https://opencode.ai/docs/server/ |
| A4 | The pinned `@opencode/client ^2.1.4` exposes a typed method for listing a session's children (or at least all sessions) | unverified | research Platform notes — the package is not installed in this worktree; only `session.create/prompt/remove` and `message.list` are exercised (grep `session\.` over src) |
| A5 | Code-mode `execute` calls appear in the invoking session's own message list as tool parts, rather than under separate child sessions | unverified | research Risks; the `execute` part shape in tests/pipeline/independence.test.ts:124–130 is a fixture. This design does not depend on the answer: the path-key rule judges the part where it is recorded, and the child walk catches effects recorded in any descendant session |
| A6 | Child sessions' messages are fetchable through the existing paged `listSessionMessages` (asc order, `cursor.next`) | verified | tools/orchestration/src/pipeline/runner.ts:473–499 |
| A7 | The message-level `since` filter already prevents a retried attempt paying twice, and child message lists run through it the same way | verified | independence.ts:44–50,112–114; tests/pipeline/independence.test.ts:110–122 |
| A8 | Child sessions can be filtered by session-creation timestamp in the client | unverified | research Risks — `SessionInfo` timestamp fields unverifiable here; the design therefore does not rely on it (Chosen design → Deeds of children) |
| A9 | Prompt text is not audit input, so an attempt-2 prompt that quotes forbidden paths is safe under the audit | verified | independence.ts:112–155 scans tool parts and shell messages only |
| A10 | The agent-permission fence (deny reads of `**/transcripts/**`, review-seat denies of `**/reviews/**`) is the primary control; the audit is defense-in-depth over it, with "deterministic plain-text coverage, nothing more" posture | verified | independence.ts:6–41; .opencode/agents/review/design-security.md:17–25 |
| A11 | The runner deliberately never executes a formatter; the test engineer has shell-allowed `npm run format*`; `npm run check` chains `lint && format:check && test` | verified | runner.ts:267–285; .opencode/agents/code/test-engineer.md:29–31; tools/orchestration/package.json:19–23 |
| A12 | The package's prettier setup rewrites import order (`@trivago/prettier-plugin-sort-imports`), so each fresh test batch can reintroduce `format:check` drift | verified | tools/orchestration/package.json:19–20,35 |

## Options considered

**Word-based exemption (negative scan: don't flag prohibition sentences).**
Rejected: it is the same words-not-acts error seen from the other side, and it
perpetuates the perverse loop the source names — hardened prompts quote the rule
more, raising the false-positive rate of a content scan. The fix must remove the
content judgment, not curate its vocabulary.

**Full-scan removal with no replacement ("judge nothing for unknown tools").**
Rejected: it would flip the existing pin "scans nested input values, not just
the first field" (`tests/pipeline/independence.test.ts:124–130`) red, and the
source requires existing tests to stay green. A nested value under key `path`
names where a tool looks; it must still fail.

**Value-shape heuristic (flag any string that looks path-like).** Rejected: it
reintroduces content judgment — a string literal in code can look path-like —
and makes the audit's verdict unpredictable, which the "deterministic
plain-text coverage" posture (A10) forbids.

**Async audit entry point inside `independence.ts`.** Rejected: the matcher
stays pure and synchronous for testability, and the child fetch needs
`client` + `listSessionMessages`, which live in `runner.ts` — importing them
into `independence.ts` would create a module cycle. The runner assembles the
message lists; the matcher stays a function over message lists.

**Depth-1 child walk (direct children only).** Rejected: two levels of
delegation (a subagent delegating to its own subagent — custom agents can, per
https://opencode.ai/docs/agents/) would restore the blind spot, and closing it
costs no extra machinery: a BFS over the attempt's session subtree costs one
children call per session in that subtree.

**Failure-reason injection in `runWithRetry` only.** Rejected: the source names
`runWithRetry`, but its own motivating failure — the editorial-security seat
globbing `reviews/design/*` and failing twice — happened on the review-pass
lane (`runReviewPass`, runner.ts:1239–1284), and both lanes build the same
attempt-2 `StageHandle`. One seam covers both; scoping it to `runWithRetry`
would leave the documented failure unfixed.

## Chosen design

### 1. Matcher: paths only, not payloads (`src/pipeline/independence.ts`)

- `ACCESS_FIELDS` keeps its per-tool mapping for the nine known tools.
- Add `subagent: []` to the map: a delegation part's input carries no access
  fields at all — delegation text is not an access (source §1). Its evidence is
  the child walk (§2).
- Replace the unknown-tool branch of `accessValues` (independence.ts:168–177):
  instead of `stringValues(input)`, collect string values under an allowlist
  of access-naming keys, via a node-capped queue that drops the deepest nodes:

  ```ts
  const PATH_ACCESS_KEYS = new Set([
    'path', 'filePath', 'file', 'directory', 'dir', 'workdir', 'command',
    'include', 'pattern',
  ]);
  ```

  Payload keys (`code`, `prompt`, `content`, `text`, `question`, …) are never
  scanned — extending the `write`-content and `grep`-query exemptions (already
  pinned at independence.test.ts:134–153,201–212) from named tools to the
  general rule stated in Context. Unknown tools' `pattern` is read glob-style;
  query-text `pattern` on an unknown tool is a known false positive — the
  violation names key and value, so a mis-scan is a one-line allowlist fix.
- The existing nested-path pin survives untouched: its fixture value sits
  under key `path`, which the allowlist keeps. The research expectation that
  the pin be "re-shaped or replaced" resolves as "kept as-is."
- Update the two doc comments promising the full scan (independence.ts:64–68
  and the header scope-discipline block) to state the path-key and delegation
  rules.

### 2. Deeds of children (`src/pipeline/runner.ts`)

New async-generator helper beside `listSessionMessages`, so peak memory is one message list:

```ts
async function* listChildSessionMessages(
  client: OpenCodeClient, sessionID: string, requestTimeoutMs: number,
): AsyncGenerator<{ sessionID: string; messages: SessionMessageInfo[] }>
```

- BFS over the attempt session's descendant sessions (visited set, so a
  malformed parentID graph cannot loop), one enumeration per walk, not per
  node: per session `client.session.children({ sessionID })` when the SDK
  exposes it, else one `client.session.list()` paged with `cursor.next` like
  `listSessionMessages` (A6) and indexed by `parentID` into a Map the BFS
  walks (A3, A4 — implementation step 1 confirms which).
- The walk is a point-in-time snapshot taken after the reply: a descendant
  still running at that moment contributes only its completed parts (Risks).
- At the audit call site (runner.ts:839–849), run the pure
  `auditSessionIndependence` on the parent list first; when it already
  returns violations, skip the descendant walk and fail on what is in hand.
  Otherwise audit each descendant list with the same `IndependenceAuditOptions`
  (`since`, `reviewStage`, `ownReportPath`), appending child violations as
  `` `${violation} (via child session ${sessionID})` ``. `since` bounds what
  the matcher judges, not what the walk fetches (A7, A8).
- A child-walk fetch error after `bounded()`'s single retry is caught and
  logged (`console.warn` plus one `gateLog` line naming the session it could not
  enumerate), and the audit proceeds on the parent list alone — a stage
  failure on an infrastructure fault is not what the source asks for (A10).
  Tripwire: `console.warn` also fires when an audited message list holds a
  completed `subagent` part yet the walk enumerated zero descendants, so an
  empty-but-successful enumeration never masquerades as a clean audit.
- Cost per attempt: one children call per session (or one session-list call
  and a `parentID` index on the fallback) plus one full paged message fetch
  per descendant — including entirely pre-`since` lists on the resumable
  lane — each inside `requestTimeoutMs` with `bounded()`'s retry, once per
  attempt after the reply, off the poll loop.

The violation text keeps naming the rule and the offending path (via
`brief()`/`printable()`), so the gate log and — via §3 — attempt 2's prompt
carry it.

### 3. Retry feedback (`src/pipeline/runner.ts`)

- Add to `StageHandle` (runner.ts:629–647):

  ```ts
  /** Set on a fresh-session attempt 2: why attempt 1 failed. */
  failureNote?: string;
  ```

- In `executeInference`, when building `fullPrompt` (runner.ts:693–710), if
  `handle.resume` is absent and `failureNote` is set, append after the rendered
  template:

  ```
  Previous attempt's gate failure (log excerpt): <failureNote>.
  Do not repeat the behavior that caused that failure.
  ```

  mirroring the `repairPrompt` precedent (runner.ts:1133–1140) for the
  resumable lane. The block is labeled a log excerpt and bounded through
  `brief()` after `printable()` (same posture as `gateLog`,
  runner.ts:1144–1161); quoting forbidden paths in it is safe (A9).
- Both non-resumable attempt-2 sites set it — `runWithRetry`
  (runner.ts:1468–1484) and `runReviewPass`'s `runSeat` (runner.ts:1259–1276) —
  each with `failureNote: brief(printable(out.failure ?? 'no reply'))`.
- Untouched: `retryInSameSession`, the nudge loop, `state.stageFailures`, gate
  entries, and the one-retry-then-escalate flow — attempt 2 stays one attempt.

### 4. Prompt lines and their pins (`prompts/`, `tests/pipeline/prompts.test.ts`)

- `prompts/design-review.md`: after the existing independence sentences
  (design-review.md:3–8), add the bright line:

  ```
  Listing, globbing, or grepping those directories counts as opening them.
  Every path you need is named in this prompt — never browse to verify one,
  even when a named path lives under a directory you must not open.
  ```

- `prompts/test-authoring.md` and `prompts/test-update.md`: append the
  ownership line — you own the test files you write; run the package formatter
  (`npx prettier --write` on the files you touched) BEFORE replying. This hands
  the last `format:check` step to the stage that owns the files (A11) and
  targets the plugin-driven drift (A12).
- Pins follow the `fix-trigger.test.ts:58–67` pattern (load from `prompts/`,
  render, assert on rendered content): extend the existing design-review
  guidance block in `prompts.test.ts:179–210` (which already pins the base
  sentence at line 206) with a `toContain` on the bright line, and add
  rendered-content pins for the formatter line in both test templates.
  Substring pins, matching every existing prompt pin.

### Validation posture (dogfooding)

This run executes under the CURRENT matcher, so the design's fixes are
exercised live only by the first post-merge run. Per the source's note, this run's
stage agents name allowed paths/commands directly in any subagent delegation and
do not copy prohibition sentences into delegation text.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| An unknown tool names its access under a key outside `PATH_ACCESS_KEYS` → missed violation | Medium | Low–Medium (audit is defense-in-depth; the fence is the primary control — A10) | Allowlist covers the key names the platform's tools actually use; the child walk catches delegated effects; the miss class is the same accepted class as obfuscated shells (independence.ts:30–41) |
| A descendant still running at the snapshot contributes no completed parts, so its deeds go unseen for that attempt (a net loss vs today's delegation-text scan) | Low (background delegations outliving the parent's reply) | Medium (detection gap for one attempt) | Stated snapshot semantics (§2); the fence still blocks the access (A10); same accepted-gap posture as the fetch-error row |
| Child actions misattributed to this attempt (a resumed or sibling session's older children) | Low | Medium (false failure) | Walk is parent-scoped from this attempt's own sessionID; the `since` filter skips pre-attempt messages in any fetched list (A7); pinned by Test strategy #5 |
| Child-walk fetch fails and the attempt is audited parent-only | Low | Low (detection gap for one attempt, fence still holds) | `console.warn` plus a `gateLog` line name the partial audit in the run record; A10 posture; not treated as a stage failure |
| Failure note quotes forbidden paths into the attempt-2 prompt/transcript | Low | Low | Prompt text is not audit input (A9); the note enters as a labeled log excerpt bounded by `brief(printable(...))` |
| The formatter prompt line is guidance, not a gate — an engineer can still skip it | Medium | Low (today's failure mode unchanged, and the fix→format ownership route still catches it) | Pinned prompt (Test strategy #8) keeps the line from being silently reverted, which is what the source asks for |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux, Node ≥ 22.12.0 | yes | The dev/CI environment (package.json:31); no new runtime primitive |
| macOS | no | Untested; no OS-specific code touched (Linux-only CI, Test strategy #9) |
| Windows | no | Untested; the audit is pure string matching over server records, so no OS dependency is added |
| OpenCode server with `GET /session/:id/children` | yes | Child walk's primary path (A3) |
| OpenCode server/SDK without a children method | yes | Determinate at step 1: SDK `session.children`, else SDK `session.list` + `parentID` filter, else a direct `GET /session/:id/children` against the configured `baseUrl` with the client's auth headers — all three exist server-side (A3), so the walk always has a route (A4 decides which, not whether) |
| Existing checkpoints and configs | yes | No `PipelineState`/`PipelineConfig` schema change; `failureNote` is in-memory handle state |
| Runs already in flight across the merge | yes | No wire or state format changes; a straddling `pipeline resume` just gets the new audit from its next attempt |

## Test strategy

1. Unit — pin (a): a completed `subagent` tool part whose delegation prompt
   recites the transcripts prohibition yields no violation (`reviewStage` both
   ways).
2. Unit — pin (b): a completed `execute` part whose `code` payload contains
   `transcripts/` and `reviews/` string literals yields no violation.
3. Unit — path-key rule kept for unknown tools: the existing pin at
   `tests/pipeline/independence.test.ts:124–130` (nested `{ path:
   '…/transcripts/y.md' }` inside an `execute` input) stays green; the same
   fixture with the value under a non-access key (`code`) yields none.
4. Unit — pin (c) at matcher level: a message list containing a child's
   completed `read` of a `transcripts/` path fails with the violation naming
   the path; with `reviewStage: true`, a child `glob` over `reviews/**` fails;
   the child's `write` of a report other than `ownReportPath` fails.
5. Unit — `listChildSessionMessages` against a fake client: grandchildren are
   reached (depth 2), a parentID cycle terminates, pre-`since` child messages
   yield no violation through the matcher, and at the call site a completed
   `subagent` part with zero enumerated descendants warns.
6. Integration — full `runPipeline` attempt on a temp git repo with the mock
   client: a stage session whose child session reads a `transcripts/` file
   fails the attempt with `independence violation` naming the path (and
   `(via child session …)`), and the attempt-2 prompt recorded by the mock
   client contains the failure reason; budget is unchanged (attempt 2 fails →
   escalate, one gate entry per failure).
7. Integration — review-pass seat (`runReviewPass`) whose attempt 1 violates
   independence: the fresh attempt-2 prompt contains the failure reason; seat
   retry/permit semantics otherwise unchanged.
8. Unit — prompt pins: rendered `design-review.md` contains the bright-line
   sentence; rendered `test-authoring.md` and `test-update.md` contain the
   formatter duty — load from `prompts/`, render, `toContain` (the
   `fix-trigger.test.ts` pattern).
9. Platform — Linux CI: `npm run check` in `tools/orchestration` green, with
   every pre-existing test unchanged in verdict — including #3's pin — and
   the nine fake clients in `tests/pipeline/` (reply-contract, pass-finalize,
   parallel-integration, parallel-fanout, budget, parallel-escalation,
   stage-lifecycle, funnel, proposal-size) extended with the session-list
   method their flows need.

## Rollout

1. Implementation step 1: `npm install` in `tools/orchestration`, read
   `@opencode/client`'s types, and fix the walk's enumeration route (A4).
2. Merge; nothing to flip — the matcher change and the retry note take effect
   on the next run's attempts. No migrations, no config, no persisted field.
3. The two prompt lines land with their pins (source §3: re-landed as reviewed
   artifacts); their first live effect is the next test-authoring/test-update
   and design-review stages.
4. First post-merge dogfooded run is the live check of the child walk and the
   attempt-2 note (Validation posture).

## Deferred / out of scope

- Detecting accesses that never name the run-records segment in a recorded
  plain-text field (obfuscated shells, a script file written first) — the
  documented residual in `independence.ts:30–41`, extended to off-allowlist
  keys and to values beyond the §1 walk's node cap; no follow-up document.
- Server-side effect records ("what a tool actually touched") as audit input —
  an OpenCode platform surface, not this repo; no follow-up yet.
- Identity-level misattribution guards beyond `since` + parent-scoped
  enumeration (e.g., a run-id stamped on sessions) — no follow-up yet.
- A structured "rule broken" field beyond the bounded failure string —
  deferred; the violation strings already name the rule (Chosen design §2).
- Subagent-prompt hardening guidance for in-flight runs (source "Notes for the
  stages of this very run") — process note owned by the source document for
  this run only; nothing ships.

## Revision log

- r0: Chosen design → Matcher — decided: unknown tools judged on a path-key
  allowlist, not the payload scan; the nested-path pin is kept, not re-shaped
  (resolves research Q1).
- r1: Chosen design → Deeds of children (+ Compatibility matrix row) — one
  paged enumeration per walk, determinate endpoint route, snapshot semantics,
  gate-logged partial audits, zero-descendant tripwire (resolves Q2, Q7).
- r1: Chosen design → Retry feedback — one `failureNote` seam on `StageHandle`
  covering both lanes; labeled log excerpt bounded by `brief(printable(...))`,
  no structured schema (resolves research Q3, Q4).
- r0: Test strategy — decided: prompt pins are `toContain` on rendered
  content, extending the existing design-review block (resolves research Q5);
  no gate-side formatter companion, the ownership line stands alone
  (resolves research Q6).
