# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T22:22:31.142Z
- ended: 2026-11-10T22:29:53.331Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/007-audit-by-deeds/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/prompts/audit-by-deeds.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/prompts/audit-by-deeds.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/007-audit-by-deeds/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Assumption register A8 and Chosen design → Deeds of children: \"`SessionInfo` timestamp fields unverifiable here\" / \"children need no creation-time filter because the message-level `since` filter already excludes messages from before this attempt (A7, A8)\"","problem":"The A8 basis is contradicted by repo evidence: src/opencode-runner.ts:264–265 already reads session.time.created/session.time.updated off the typed v2 client.session.create result, so the source §1 requirement to enumerate only child sessions \"created at or after the attempt's `since` stamp\" is implementable at the session level. The omission is safe for fresh attempts (children of a newly created session are always newer than `since`) but not fully closed on the resumable lane: a background child spawned by attempt 1 in the same session can emit post-`since` messages during attempt 2, and Risks row 3's mitigation (\"the `since` filter skips pre-attempt messages\") covers only pre-attempt messages.","fix":"Correct A8's basis to cite src/opencode-runner.ts:264–265, then either apply the creation-time filter in the BFS (the session objects the walk already fetches carry the stamp, replacing the current unconditional inclusion of all descendants) or state the still-active-older-child residual explicitly in Risks row 3 instead of implying `since` closes it."},{"severity":"minor","location":"Chosen design → Matcher §1: \"the grep-style reading of `pattern` as query text is a known residual (Risks row 2)\"","problem":"Risks row 2 is \"An unknown tool names its access under a key outside `PATH_ACCESS_KEYS` → missed violation\" — a false-negative class. The pattern-as-query residual is a false positive (an unknown search tool whose query text contains `transcripts/` fails the attempt — the same words-not-acts defect the design removes, narrowed to unknown tools), and no Risks row covers it, so the pointer anchors the residual to a row that does not describe it.","fix":"Add a Risks row — \"an unknown search-like tool names its query under `pattern` → false violation; Low–Medium/Low: named `grep` is exempt at independence.ts:74 and pinned at independence.test.ts:201–212, so only unknown search tools carry it\" — and point §1 at that row instead of the outside-allowlist row."},{"severity":"minor","location":"Assumption register A3: \"the server exposes `GET /session/:id/children` and `GET /session` | verified | https://opencode.ai/docs/server/\"","problem":"The cited server-docs page is the OpenCode V1 docs (top banner: \"These are the docs for OpenCode V1. The latest docs, for OpenCode V2, are at .../llms.txt\"), while the design pins @opencode/client ^2.1.4 (V2) and the repo records that the v2 client namespaces routes to /api/session (src/opencode-runner.ts:17–18). The V1 page supports the children endpoint and parentID for V1 only, so the Compatibility row \"OpenCode server with `GET /session/:id/children` | yes ... (A3)\" and Risks row 1's \"both exist server-side (A3)\" rest a V2 claim on a V1 source.","fix":"Re-anchor the endpoint half of A3 to the V2 API reference (opencode.ai/docs/api/) or mark it \"verified on V1 server; V2 surface resolved at implementation step 1 (A4)\", and note that the V2 JavaScript client docs (opencode.ai/docs/build/client/) already show client.session.list(...) exists, which firms up the named fallback."},{"severity":"nit","location":"Chosen design → Matcher §1: \"Update the two doc comments that promise the full scan (independence.ts:64–68 and the header scope-discipline block)\"","problem":"Only the 64–68 comment promises the full scan; the header scope-discipline block (independence.ts:18–28) already states the opposite rule (\"Only the fields that NAME where a tool looks are scanned\"), so characterizing both as promising the full scan misdescribes the code even though the intended edit is fine.","fix":"Reword as: \"update the independence.ts:64–68 comment to state the path-key rule, and extend the scope-discipline block with one line adding the delegation rule\"."}]}
```
