# complexity review r1

verdict: approve

Scope: whole document (c0, round 1). Checklist: references/complexity.md.
Inputs read: `02-proposal.md` (incl. its Revision log), `docs/prompts/audit-by-deeds.md`,
plus the repo surfaces the checklist names (check `tools/` before flagging reuse):
`tools/orchestration/src/pipeline/independence.ts`, `src/pipeline/runner.ts`,
`tests/pipeline/*` fake clients, `package.json`.

Checklist walk:
1. Reuse — the child walk reuses `listSessionMessages` and `bounded()`; the matcher
   stays the same pure function and is simply called per message list. No in-repo
   helper for session trees exists (grep `parentID|children|session.list` over
   `tools/` → no hits), so `listChildSessionMessages` is not duplicating anything.
2. Every moving part traces to a stated Goal (`subagent: []` → Goal 1; path-key rule
   → Goal 2; BFS walk + pins → Goal 3; `failureNote` → Goal 4; prompt lines + pins →
   Goal 5). Nothing gratuitous found.
3. Smaller design — see finding 1 (the allowlist) and finding 2 (test surface); the
   overall shape is already near-minimal for what the source demands.
4. New dependencies — none. `@opencode/client ^2.1.4` is already a
   `tools/orchestration` dependency (package.json:46).
5. Deferrable half — no: §1+§2 are coupled (a `subagent: []` exemption merged without
   the child walk removes the only thing that catches a delegated read today), and
   §3/§4 are source-required in this run. Merging all four together is the thin-slice
   answer here, and the proposal does that.
6. Knobs/abstractions — no config knobs added; `failureNote` and the helper each have
   exactly one consumer. See finding 3 for the one place carrying two implementations.
7. Value check: proportionate. The design is a matcher-branch rewrite, one ~30-line
   helper reusing two existing helpers, one optional in-memory field at two existing
   call sites, and two prompt lines with pins. That is about as small as the source's
   three required changes can be, and the Options section shows the smaller variants
   (depth-1, `runWithRetry`-only, matcher-side async) being rejected for stated
   reasons rather than invented for cover.

## Findings

- [minor][new] Chosen design §1 (Matcher: paths only, not payloads) — "`const PATH_ACCESS_KEYS = new Set([` / `'path', 'filePath', 'file', 'directory', 'dir', 'workdir', 'command',` / `'include', 'pattern',` / "`]);`" — the new allowlist is a hand-typed second copy of access keys that already exist in `ACCESS_FIELDS` (independence.ts:69–82): six of its nine entries (`path`, `filePath`, `workdir`, `command`, `include`, `pattern`) are exactly the values of the nine known tools, so the file ends up with two lists that must be kept in sync by hand — the same drift the module's scope-discipline comment is trying to prevent. The remaining three (`file`, `directory`, `dir`) are speculative: no tool in the repo's own mapping uses them, and the proposal's own Risks row 2 mitigation promises the allowlist "covers the key names the platform's tools actually use". Fix: derive it — `const PATH_ACCESS_KEYS = new Set(Object.values(ACCESS_FIELDS).flat());` — one list, one rule, known and unknown tools cannot drift, and the speculative keys drop out unless a named platform tool justifies them. (Semantics are unchanged for the six: `pattern` stays scanned for unknown tools, which the design already accepts as the residual in Risks row 2.) This does not re-open the r0 decision recorded in the Revision log — the allowlist-not-payload-scan choice stands; only the list's source changes.

- [minor][new] Test strategy #9 — "the nine fake clients in `tests/pipeline/` (reply-contract, pass-finalize, parallel-integration, parallel-fanout, budget, parallel-escalation, stage-lifecycle, funnel, proposal-size) extended with the session-list method their flows need" — this is test scope the design does not need. Every one of those fakes is an object literal cast `as unknown as OpenCodeClient` (e.g. funnel.test.ts:110–152, budget.test.ts:146), so TypeScript does not require the new method, and §2 already specifies that a child-walk fetch error is caught after `bounded()`'s retry, logged, and audited parent-only — which is exactly what a fake without a children/list method produces. Nine file edits buy nothing beyond suppressing `console.warn` noise. Fix: scope the extension to the fakes used by Test strategy #5–#7 (the flows that must reach a child session), and state in #9 that the other flows exercise §2's fetch-error fallback on purpose — one sentence instead of nine edits.

- [nit][new] Chosen design §2 (Deeds of children) — "`client.session.children({ sessionID })` if the installed SDK exposes it, else `client.session.list()` filtered by `parentID === sessionID`" — the design carries two implementations of one call and decides between them only at implementation step 1, so the fake-client shape Test strategy #5 asserts and the "session-list method" of Test strategy #9 are both left ambiguous between the two paths. Fix: commit to the `session.list()` + `parentID` filter as the implementation (it needs no per-session endpoint and matches what #9's fakes are already being given) and keep the children endpoint as a one-line note that it may replace it if the SDK exposes it.

## FYI (no severity)

- Chosen design §2 — "children need no creation-time filter because the message-level `since` filter already excludes messages from before this attempt (A7, A8)" is a deliberate simplification of the source's §1 wording ("child sessions ... created at or after the attempt's `since` stamp"). From this seat it is the right call — it removes a dependency on unverifiable `SessionInfo` timestamp fields — but it is a deviation from the source's literal requirement, so it belongs in a Risks row, not only in A8. Whether it opens a misattribution hole is the correctness/compatibility seats' question.
- Chosen design §2 runs each descendant's message list through the matcher with the parent's `reviewStage` and `ownReportPath`, so a child `write` anywhere under `reviews/**` other than the seat's own report fails the seat. That is a detection-semantics decision, not a complexity one.
- Chosen design §2 cost note ("one children call plus one paged message fetch per descendant per attempt") understates a subagent-heavy `implement` stage, where each descendant list is fully paged. The performance seat's call whether that needs a bound.
