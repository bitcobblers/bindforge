# Transcript: design-review:complexity r1c2

## Provenance
- stage: design-review:complexity (round r1c2)
- agent: review/design-complexity
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T22:38:41.081Z
- ended: 2026-11-10T22:41:30.043Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/007-audit-by-deeds/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff ce95f3b6b5756a5fb08196351bd60ee4ff2866f5 f4f64f9c7a62e1487a3ad86706965ae11544eb87` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/prompts/audit-by-deeds.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "minor",
    "location": "Chosen design §1 (Matcher: paths only, not payloads) — \"const PATH_ACCESS_KEYS = new Set([ 'path', 'filePath', 'file', 'directory', 'dir', 'workdir', 'command', 'include', 'pattern', ]);\"",
    "problem": "The new allowlist is a hand-typed second copy of access keys that already exist in ACCESS_FIELDS (independence.ts:69–82): six of its nine entries (path, filePath, workdir, command, include, pattern) are exactly the values already listed for the nine known tools, so independence.ts ends up with two lists that must be kept in sync by hand. The other three (file, directory, dir) are speculative — no tool in the repo's own mapping uses them — and the proposal's own Risks row 2 mitigation promises the allowlist \"covers the key names the platform's tools actually use\". Does not re-open the r0 Revision-log decision (allowlist-not-payload-scan stands); only the list's source changes.",
    "fix": "Derive it instead of duplicating it: `const PATH_ACCESS_KEYS = new Set(Object.values(ACCESS_FIELDS).flat());` — one list, known and unknown tools cannot drift, and the three speculative keys drop out unless a named platform tool justifies them. Semantics for the six shared keys are unchanged (pattern stays scanned, already accepted in Risks row 2)."
  },
  {
    "severity": "minor",
    "location": "Test strategy #9 — \"the nine fake clients in `tests/pipeline/` (reply-contract, pass-finalize, parallel-integration, parallel-fanout, budget, parallel-escalation, stage-lifecycle, funnel, proposal-size) extended with the session-list method their flows need\"",
    "problem": "Test scope the design does not need. Each of those fakes is an object literal cast `as unknown as OpenCodeClient` (funnel.test.ts:110–152, budget.test.ts:146), so TypeScript does not require the new method, and Chosen design §2 already specifies that a child-walk fetch error is caught after bounded()'s retry, logged, and audited parent-only — which is exactly what a fake without a children/list method produces. Nine file edits buy nothing beyond suppressing console.warn noise.",
    "fix": "Name the method only in the fakes used by Test strategy #5–#7 (the flows that must reach a child session) and add one sentence to #9 that the other flows exercise §2's fetch-error fallback on purpose — replaces nine edits with a line."
  },
  {
    "severity": "nit",
    "location": "Chosen design §2 (Deeds of children) — \"`client.session.children({ sessionID })` if the installed SDK exposes it, else `client.session.list()` filtered by `parentID === sessionID`\"",
    "problem": "The design carries two implementations of one call and picks between them only at implementation step 1, leaving the fake-client shape Test strategy #5 asserts and the \"session-list method\" of #9 ambiguous between the two paths (two paths to test versus one).",
    "fix": "Commit to `session.list()` + `parentID` filter as the implementation (no per-session endpoint needed, matches what #9's fakes are already given) and keep the children endpoint as a one-line note that it may replace the filter if the SDK exposes it."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/prompts/audit-by-deeds.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/007-audit-by-deeds/reviews/design/complexity-r1c2.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design §1 (Matcher) — \"instead of `stringValues(input)`, collect string values under an allowlist of access-naming keys, via a node-capped queue that drops the deepest nodes\"","problem":"new — The r1 revision replaced the text approved in c1 (\"collect string values ... recursively, in any nesting depth\") with a queue that silently drops the deepest nodes, and the cap's value appears nowhere in the document. Deferred hands off \"values beyond the §1 walk's node cap\" as an accepted residual, but a blind spot with no named bound cannot be reviewed, implemented, or pinned consistently; no Goal, cost bullet (§2's cost bullet budgets fetches, not walk nodes), or Risks row names the input-size concern the cap defends against, and the Revision log carries no r1 Matcher entry for this change. Fix: the smallest design that was already approved is a recursive collect over the existing `stringValues` traversal shape keeping only values under allowlist keys — tool inputs are finite JSON trees and the current scan is already uncapped. If the cap stays, name the constant and its value (e.g. `MAX_WALK_NODES`) so the §3 pin and the Deferred row have a number to check, and record the decision in the Revision log.","fix":"Reuse the recursive `stringValues` traversal shape filtered to allowlist keys; or keep the cap but name its value inline and add the missing r1 Revision-log entry."},{"severity":"minor","location":"Compatibility matrix, row \"OpenCode server/SDK without a children method\" — \"else SDK `session.list()` + `parentID` filter, else a direct `GET /session/:id/children` against the configured `baseUrl` with the client's auth headers\"","problem":"regression (escalates my c1 nit about two enumeration routes) — The r1 revision grows the enumeration routes from two to three, and the third is speculative machinery: it only fires if the pinned SDK exposes neither `session.children` nor `session.list`, yet A4's verified half already exercises the client on `session.create/prompt/remove` and A3 places the session listing server-side, so the row itself treats the no-`session.list` case as impossible (\"the walk always has a route\"). Hand-rolling raw HTTP with auth-header handling outside the SDK adds a second integration surface to build, fake, and pin for an unreachable branch; if the SDK assumption were that wrong, step 1 should escalate rather than route around it. It also multiplies the route ambiguity my c1 nit flagged, which §2's \"implementation step 1 confirms which\" and Test strategy #9's \"session-list method their flows need\" both inherit. Fix: drop the direct-GET clause from the row, keeping the two SDK routes decided at step 1.","fix":"Remove the direct-GET fallback clause from the Compatibility matrix row; let implementation step 1 choose between SDK `session.children` and `session.list` + `parentID` filter (my c1 nit preferred committing to the latter outright)."}]}
```
