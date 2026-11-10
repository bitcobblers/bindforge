# Transcript: code-review:performance r1c1

## Provenance
- stage: code-review:performance (round r1c1)
- agent: codereview/code-performance
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-10T23:52:08.485Z
- ended: 2026-11-10T23:57:37.468Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/007-audit-by-deeds/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 95b73560fab5571ad65610ffe4f790d080d715c1..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/prompts/audit-by-deeds.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (performance seat):
[
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:1070-1083",
    "problem": "The descendant walk runs unconditionally on every clean attempt, with no key to delegation. Attempts that contain no `subagent` part at all still pay a `session.children` call per BFS node, or on the fallback route a full paged crawl of the server's whole session list (runner.ts:639-668). Per 03-build-notes deviation #2, every attempt under the nine pre-existing fake clients takes the neither-route throw, so each clean attempt burns a throw, a console.warn, and a persisted `gateLog('independenceAudit')` entry for no possible benefit.",
    "fix": "Pre-guard the walk with delegation presence computed over the whole `lastSeen` list — any `subagent` part regardless of status or `since` (a resumed attempt's part sits pre-`since` while its child's deeds sit in-window, and §2's still-running-child risk has a non-completed part, so restricting the guard to completed in-window parts would open a detection gap). Skip the walk when the list holds none; keep the zero-descendant tripwire at 1086-1092 for the delegation-exists-but-enumeration-empty case."
  },
  {
    "severity": "major",
    "location": "tools/orchestration/src/pipeline/runner.ts:1052-1116 (inside the outer stage loop at 983; re-entry via `continue` at 1148, 1174, 1190, 1212)",
    "problem": "The audit block — and with it the child walk — runs once per completed turn, not once per attempt. The `if (gate === 'wait') continue` paths at 1148/1190 re-enter it with no new prompt and no new reply. Worst case per attempt is 1 + MAX_NUDGES = 9 audit passes, each re-running the whole descendant walk, i.e. up to nine whole-store session crawls plus nine × (descendants) full message fetches, and up to nine extra `gateLog` entries appended to the persisted checkpoint. Design 007 §2's cost paragraph states the walk happens \"once per attempt after the reply, off the poll loop\".",
    "fix": "Bound the walk to at most once per attempt: keep a per-attempt count of the `subagent` parts walked, re-audit the parent list (pure, cheap) on later passes, and re-walk only when that count has grown — so a nudge that triggers a new delegation stays in view. If child lists are memoized across passes instead, keep the memo per-attempt, discard it at attempt end, and document that it is a point-in-time snapshot."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:637-668",
    "problem": "The fallback enumeration is one paged `session.list` crawl (limit 200) looping until `cursor.next` is absent, indexing every parentID-bearing session the server has ever recorded into `byParent`. Request count and map size scale with the server's lifetime session store, not with this attempt's subtree; nothing bounds it, and there is no guard that the cursor advances, so an echoed `cursor.next` spins forever while `byParent` grows (the `visited` set guards the BFS, not this crawl).",
    "fix": "Add a page budget (e.g. 50 pages) and break when `next === cursor`; on either limit, throw so the existing catch at 1093 degrades to the design's logged parent-only partial audit. Also pin §2's \"one enumeration per walk\" claim with a stub assertion that `session.list` is called once for a two-level tree."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:1074-1110",
    "problem": "The walk's awaits sit outside the stage deadline: `deadline` is consulted only in the inner poll loop (996, 1129). Each request site costs up to 2 × DEFAULT_REQUEST_TIMEOUT_MS = 60s through `bounded()`'s retry (runner.ts:712), and the walk makes one site per page plus one per descendant, so a half-open server pushes a clean attempt minutes past `stageTimeoutMs` (30 min default) — multiplied again by the per-pass repetition above. `bounded()`'s own comment says the caller owns the stage deadline; the walk never exercises that.",
    "fix": "Give the walk a whole-walk budget derived from the remaining deadline (e.g. `AbortSignal.timeout(Math.max(1, deadline - Date.now()))`, or a deadline check before each `descendantMessages`) and route expiry into the same caught path at 1093, naming the truncation in the `independenceAudit` gate line."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/independence.ts:249-251",
    "problem": "Once a key is in PATH_ACCESS_KEYS, its entire value subtree is handed to the uncapped recursive `stringValues` (259-268), so the MAX_PATH_SCAN_NODES budget documented at 118-123 does not apply to that subtree. Harmless for real tool inputs, but the comment promises a bound the code does not deliver.",
    "fix": "Apply the same node budget inside `stringValues` for the path-key case, or narrow the comment to \"caps traversal of payload nodes, not the values sitting under access keys\"."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/007-audit-by-deeds/reviews/code/performance-r1c1.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": []}
```
