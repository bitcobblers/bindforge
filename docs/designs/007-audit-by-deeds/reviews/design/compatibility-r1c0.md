# compatibility review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`), round 1 pass c0. Inputs: the proposal,
`docs/prompts/audit-by-deeds.md`, the proposal's Revision log. No previous
compatibility findings (first pass).

## Checklist walk

1. **Compatibility matrix** — seven rows, every row carries a status; no empty
   cells. One row is `unknown` → major finding below.
2. **OS input APIs** — satisfied by justified absence: the design touches no OS
   surface; the matrix justifies it — "the audit is pure string matching over
   server records, so no OS dependency is added" (Windows row), and
   "no new runtime primitive" (Linux row). Verified against the touched code:
   `independence.ts` is regex-over-recorded-strings, `runner.ts`'s new helper is
   HTTP calls through the already-pinned SDK.
3. **Node version floor** — pass. `tools/orchestration/package.json:31` pins
   `node >=22.12.0`; the matrix row agrees. Everything the design adds (BFS with
   a visited `Set`, `Set` literal for `PATH_ACCESS_KEYS`, string concat in the
   `failureNote` template, reuse of `listSessionMessages` with its existing
   `AbortSignal.timeout`) is far below the floor.
4. **Native modules** — pass. No new dependency of any kind; the design rides
   the already-pinned `@opencode/client ^2.1.4` (package.json:46), a pure-JS
   SDK; no prebuilds/node-gyp surface introduced.
5. **Keyboard reality** — N/A: no input-device or key-name surface anywhere in
   the change.
6. **File paths** — pass. The design constructs no OS paths; it pattern-matches
   recorded strings. The one path-identity comparison (`ownReportPath` →
   `reportKey`, independence.ts:163–166) already normalizes `\` → `/`, and the
   child audits pass the same `ownReportPath` through unchanged. No config-dir
   or separator handling is added.
7. **Test strategy vs matrix** — pass for OS platforms: the matrix claims only
   Linux supported, and Test strategy #9 names Linux CI (`npm run check` green,
   pre-existing tests unchanged). Fake-client extension for the new session-
   enumeration method is explicitly enumerated (#9).

## Findings

- [major][new] Compatibility matrix — "| OpenCode server/SDK without a children
  method | unknown | Method name unverifiable here (A4); `session.list` +
  `parentID` filter is the named fallback; resolved at implementation step 1 |" —
  a support-status cell of `unknown` is a major under checklist item 1: the
  matrix leaves the environment combination "SDK exposes neither a children
  method nor a session-list method" unclassified, and every remedy the design
  names is conditional on resolving A4 during implementation rather than a
  decision the matrix records. Fix: make the row determinate — e.g. re-status
  `yes` and state the step-1 decision rule in the Notes cell ("if the installed
  client exposes neither `session.children` nor `session.list`, the walk uses a
  direct `GET /session/:id/children` against the configured baseUrl with the
  client's auth headers — endpoint existence verified in A3"), or otherwise
  record the terminal outcome for the neither-method case, so no row reads
  `unknown` when implementation starts.

## FYI (no severity)

- Matrix "OpenCode server with `GET /session/:id/children` | yes": the
  child-walk tests (#4–#7) run against fake clients; the live-server check is
  deferred to the first post-merge dogfooded run (Validation posture / Rollout
  #4) — consistent with the whole suite's mock-client posture, but the "yes"
  has no pre-merge live test behind it.
- If the `session.list` + `parentID` fallback is chosen at step 1, it should
  inherit the paging discipline the module comments insist on for
  `message.list` (runner.ts:462–472); the proposal doesn't say whether the SDK
  list paginates. Correctness/complexity seat's call whether to pin it.
- The matcher's segment regex treats `\` as a non-boundary, so a Windows-style
  `...\transcripts\...` value would not match — pre-existing behavior on a
  platform the matrix marks unsupported, unchanged by this design; noted only
  because the Windows row's "no OS dependency" framing could be misread as
  "Windows-safe matching".
- Non-QWERTY/IME/layout concerns and native-module packaging are N/A for this
  change (no input surface, no new dependencies).
