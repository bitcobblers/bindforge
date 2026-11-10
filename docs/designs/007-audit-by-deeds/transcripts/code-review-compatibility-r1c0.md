# Transcript: code-review:compatibility r1c0

## Provenance
- stage: code-review:compatibility (round r1c0)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-10T23:14:53.852Z
- ended: 2026-11-10T23:21:54.338Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/007-audit-by-deeds/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 7844c17171465dd779f8db6671801eb058bd2eb3...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/prompts/audit-by-deeds.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/007-audit-by-deeds/reviews/code/compatibility-r1c0.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:590-634 (compatibility matrix row, 02-proposal.md:257)", "problem": "The matrix's 'server/SDK without a children method' row promises three enumeration routes and says the walk 'always has a route'; the code implements two and then throws, and it picks the route by client-object feature detection (`typeof api.children === 'function'`) rather than by outcome — so an SDK that exposes `session.children` against a server lacking that route 404s on the first parent, never falls back to `session.list`, and the whole §2 child-deeds detection is silently disabled (parent-only audit plus warn/gate line) for every attempt.", "fix": "Order the fallback by outcome: on a `children` enumeration error, re-run the walk once through `session.list` + `parentID` before giving up; and either implement the direct `GET /session/:id/children` against `baseUrl` the matrix names, or restate that row as the two-route-then-logged-parent-only behaviour the code actually has."}, {"severity": "minor", "location": "tools/orchestration/tests/pipeline/child-session-deeds.test.ts:47-68 and the nine fake clients in tests/pipeline/ (reply-contract:122-141, stage-lifecycle:61-73, proposal-size:108-129/167-171, funnel:110-130, budget:107-124, pass-finalize:107-126, parallel-integration:111-141, parallel-fanout:131-154, parallel-escalation:141-159)", "problem": "Checklist item 7: the plan's walk pins (Test strategy #5) only ever exercise the `session.children` route — `treeClient` supplies both `children` and `list` and the probe prefers `children` — so the `session.list` + `parentID` BFS at runner.ts:637-686, which is the code path for the matrix's claimed-supported 'SDK without a children method' environment, is never shown to enumerate a single descendant (the only fake with `list` alone returns entries without `parentID`). Test strategy #9's instruction to extend the nine fake clients with the session-list method was not done: all nine session stubs still expose only create/prompt/remove, so every audited attempt in those flows takes the neither-route throw path (warn plus an `independenceAudit` gate entry) instead of walking.", "fix": "Add a walk test whose fake client exposes `list` only, with a ses_root → ses_child → ses_grandchild parentID chain and a forbidden deed at depth 2 (and re-run the cycle/pre-`since` pins on that client), and give the nine named fakes the `session.list` handler the plan calls for."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/independence.ts:284-287, used at runner.ts:923-930", "problem": "`brief()` truncates at 140 UTF-16 code units, and §3 is the first place a brief()ed string becomes a request body: `failureNote` is `brief(printable(...))` of attempt 1's failure, which quotes the offending path and now also carries `(via child session …)`. A non-BMP character landing on the cut is split, sending a lone surrogate into `session.prompt({ text })` and into the checkpoint JSON as a `\\udXXX` escape that strict JSON parsers reject.", "fix": "Truncate by code point (`[...safe].slice(0, 140).join('')`) or strip a trailing lone surrogate before returning from `brief()`."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:1084-1092 (with 637-668)", "problem": "The zero-descendant tripwire only fires for a completely empty enumeration. On the `list` fallback the walk assumes a `{ data, cursor: { next } }` paged response; a server/SDK that answers `session.list` with a plain array, or caps the page below the session count, returns a partial descendant set that reads as a clean audit next to a completed `subagent` part — the same masquerade the design rules out, one session short.", "fix": "Warn when the fallback enumeration returns an un-paginated array or a full page without `cursor.next`, and compare the walk's descendant count against the number of completed delegation parts instead of against zero."}]}
```
