# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T22:38:41.080Z
- ended: 2026-11-10T22:42:14.348Z
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
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "Assumption register A8 and Chosen design → Deeds of children: \"`SessionInfo` timestamp fields unverifiable here\" / \"children need no creation-time filter because the message-level `since` filter already excludes messages from before this attempt (A7, A8)\"",
    "problem": "The A8 basis is contradicted by repo evidence: src/opencode-runner.ts:264–265 already reads session.time.created/session.time.updated off the typed v2 client.session.create result, so the source §1 requirement to enumerate only child sessions \"created at or after the attempt's `since` stamp\" is implementable at the session level. The omission is safe for fresh attempts (children of a newly created session are always newer than `since`) but not fully closed on the resumable lane: a background child spawned by attempt 1 in the same session can emit post-`since` messages during attempt 2, and Risks row 3's mitigation (\"the `since` filter skips pre-attempt messages\") covers only pre-attempt messages.",
    "fix": "Correct A8's basis to cite src/opencode-runner.ts:264–265, then either apply the creation-time filter in the BFS (the session objects the walk already fetches carry the stamp, replacing the current unconditional inclusion of all descendants) or state the still-active-older-child residual explicitly in Risks row 3 instead of implying `since` closes it."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Matcher §1: \"the grep-style reading of `pattern` as query text is a known residual (Risks row 2)\"",
    "problem": "Risks row 2 is \"An unknown tool names its access under a key outside `PATH_ACCESS_KEYS` → missed violation\" — a false-negative class. The pattern-as-query residual is a false positive (an unknown search tool whose query text contains `transcripts/` fails the attempt — the same words-not-acts defect the design removes, narrowed to unknown tools), and no Risks row covers it, so the pointer anchors the residual to a row that does not describe it.",
    "fix": "Add a Risks row — \"an unknown search-like tool names its query under `pattern` → false violation; Low–Medium/Low: named `grep` is exempt at independence.ts:74 and pinned at independence.test.ts:201–212, so only unknown search tools carry it\" — and point §1 at that row instead of the outside-allowlist row."
  },
  {
    "severity": "minor",
    "location": "Assumption register A3: \"the server exposes `GET /session/:id/children` and `GET /session` | verified | https://opencode.ai/docs/server/\"",
    "problem": "The cited server-docs page is the OpenCode V1 docs (top banner: \"These are the docs for OpenCode V1. The latest docs, for OpenCode V2, are at .../llms.txt\"), while the design pins @opencode/client ^2.1.4 (V2) and the repo records that the v2 client namespaces routes to /api/session (src/opencode-runner.ts:17–18). The V1 page supports the children endpoint and parentID for V1 only, so the Compatibility row \"OpenCode server with `GET /session/:id/children` | yes ... (A3)\" and Risks row 1's \"both exist server-side (A3)\" rest a V2 claim on a V1 source.",
    "fix": "Re-anchor the endpoint half of A3 to the V2 API reference (opencode.ai/docs/api/) or mark it \"verified on V1 server; V2 surface resolved at implementation step 1 (A4)\", and note that the V2 JavaScript client docs (opencode.ai/docs/build/client/) already show client.session.list(...) exists, which firms up the named fallback."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Matcher §1: \"Update the two doc comments that promise the full scan (independence.ts:64–68 and the header scope-discipline block)\"",
    "problem": "Only the 64–68 comment promises the full scan; the header scope-discipline block (independence.ts:18–28) already states the opposite rule (\"Only the fields that NAME where a tool looks are scanned\"), so characterizing both as promising the full scan misdescribes the code even though the intended edit is fine.",
    "fix": "Reword as: \"update the independence.ts:64–68 comment to state the path-key rule, and extend the scope-discipline block with one line adding the delegation rule\"."
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
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/007-audit-by-deeds/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Compatibility matrix → \"OpenCode server/SDK without a children method\": \"a direct `GET /session/:id/children` against the configured `baseUrl` with the client's auth headers — all three exist server-side (A3), so the walk always has a route (A4 decides which, not whether)\"","problem":"The revision strengthened a claim this seat flagged in round 1 instead of anchoring it: the row moved from \"unknown / resolved at implementation step 1\" to an unconditional \"always has a route\", still citing A3, whose endpoint basis is the V1 docs page (opencode.ai/docs/server/ carries the banner \"These are the docs for OpenCode V1\"). The V2 API reference (https://opencode.ai/docs/api/), the surface the pinned @opencode/client ^2.1.4 speaks (the repo records the /api namespace at opencode-runner.ts:17–18), documents GET /api/session with cursor pagination but lists no children route on any session path — GET /session/:id/children is V1-only, so the third named route does not \"exist server-side\" on the V2 server as written, and whether V2 Session.Info carries parentID is unverified here. \"A4 decides which, not whether\" asserts a certainty the cited basis cannot carry.","fix":"Re-anchor A3's endpoint half to https://opencode.ai/docs/api/ — \"V1 documents GET /session/:id/children (docs/server/); V2 documents GET /api/session with a cursor; a V2 children endpoint is not in the reference\" — and reword the row to \"step 1 picks the route from the installed SDK's types (A4); the SDK session-list + parentID index is the V2-documented route, the direct children call is the V1 path\", replacing \"all three exist server-side (A3), so the walk always has a route\"."},{"severity":"minor","location":"Risks, new snapshot row vs Chosen design §2 snapshot bullet: \"A descendant still running at the snapshot contributes no completed parts\" / \"a descendant still running at that moment contributes only its completed parts (Risks)\"","problem":"Internal contradiction introduced by the revision: a running child can have completed parts at snapshot time (the message list returns them with state.status === 'completed', which is exactly what the matcher judges at independence.ts:129–130), so §2 says it contributes them and the Risks row says it contributes none.","fix":"Reword the Risks row to \"parts of a descendant not yet completed at the snapshot go unseen for that attempt\", matching §2's stated snapshot semantics."},{"severity":"minor","location":"Assumption register A8 (\"`SessionInfo` timestamp fields unverifiable here\") and the unchanged Risks row \"Child actions misattributed to this attempt\" (mitigation: \"the `since` filter skips pre-attempt messages in any fetched list (A7)\")","problem":"A8's basis remains contradicted by repo evidence: opencode-runner.ts:264–265 already reads session.time.created/session.time.updated off the typed v2 client result. And the revision's new text makes the still-open misattribution sub-case concrete without closing it: §2's Cost bullet admits the walk fetches \"entirely pre-`since` lists on the resumable lane\", and the new snapshot row's likelihood names \"background delegations outliving the parent's reply\" — so a child spawned by attempt 1 that is still running on the resumable lane can emit post-`since` deeds in attempt 2, judged against attempt 2's since and misattributed (false failure), while the row's mitigation implies A7 closes misattribution.","fix":"Correct A8's basis to cite opencode-runner.ts:264–265, and either apply the creation-time filter in the BFS or add to the misattribution row's mitigation: \"an older child still running across the retry can emit post-`since` deeds attributed to the new attempt; accepted, fence-primary (A10)\"."},{"severity":"nit","location":"Chosen design §2: \"one enumeration per walk, not per node: per session `client.session.children({ sessionID })` when the SDK exposes it\"","problem":"The headline claim is only true of the fallback branch; the primary path is one children call per node, as the section's own Cost bullet says (\"one children call per session\").","fix":"Scope the phrase to the fallback: \"on the list-based fallback, one enumeration per walk, not per node\"."},{"severity":"nit","location":"Chosen design §1: \"query-text `pattern` on an unknown tool is a known false positive — the violation names key and value, so a mis-scan is a one-line allowlist fix\"","problem":"The violation strings as built (independence.ts:135) name the tool name and the value, not the key; and the \"one-line allowlist fix\" (dropping `pattern` from PATH_ACCESS_KEYS) would also forfeit the glob-style reading the design just chose for unknown glob tools.","fix":"Either state that the matcher will also name the matched key, or reword to \"diagnosable from the violation's tool name and value; the allowlist is the fix knob, at the cost of unknown-tool glob detection\"."}]}
```
