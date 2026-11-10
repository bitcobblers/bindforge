# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T22:35:45.818Z
- ended: 2026-11-10T22:38:41.000Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/007-audit-by-deeds/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff ce95f3b6b5756a5fb08196351bd60ee4ff2866f5 f4f64f9c7a62e1487a3ad86706965ae11544eb87` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/prompts/audit-by-deeds.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Chosen design §2 Deeds of children — \"once per attempt after the reply — off the poll loop, bounded\"",
    "problem": "The child walk is a point-in-time snapshot and the design never states what happens to a descendant still running at that moment. A background subagent (or any child that outlives the parent's reply) contributes no completed parts, so its read of a transcripts/ file is never audited for that attempt and the subtree is never re-audited later. Because subagent: [] removes the delegation-text signal entirely, this is a net detection loss in exactly the class the mechanism exists to catch (source §1: \"The child's deeds are the violation\"). Neither §2, the Risks table (which covers misattribution and fetch failure only), nor Test strategy #5-#6 (all fixtures completed) acknowledge it.",
    "fix": "State the snapshot semantics in §2 (\"the audit does not wait for descendants still running at reply time\") and add the matching Risks row with the A10 mitigation, the same treatment already given to the fetch-error fallback; if a mechanism is preferred, reuse the existing bounded() single-retry pattern for one settle re-fetch of descendants that had in-flight parts at snapshot time, replacing the flat \"once per attempt\" sentence."
  },
  {
    "severity": "major",
    "location": "Chosen design §2 Deeds of children — \"else `client.session.list()` filtered by `parentID === sessionID`\"",
    "problem": "The fallback enumeration has an unguarded silent failure mode. The message fetch is explicitly paged (A6, cursor.next) but the session-list fallback is specified with no paging and no shape assumption, so children past page 1, or a SessionInfo without parentID (A8 concedes it is unverifiable here), yield an empty subtree: no error, no log, detection inert on every attempt while all stages pass. A missing session.children method at least lands in the §2 fetch-error catch; an empty-but-successful enumeration lands nowhere. Tests #5-#6 cannot catch it because their fake clients always return children. With subagent: [] the walk is the only signal for delegated deeds.",
    "fix": "In §2, page the fallback exactly like listSessionMessages (cursor.next until exhausted) and add one tripwire: when an audited message list contains a completed subagent part but the walk enumerated zero descendants, console.warn naming the enumeration gap — same log-only posture the design already chose for fetch errors, no new failure mode."
  },
  {
    "severity": "minor",
    "location": "Chosen design §2 Deeds of children — \"A child-walk fetch error after `bounded()`'s single retry is caught, logged (`console.warn`), and the audit proceeds on the parent list alone\"",
    "problem": "The r0 rejection of failing the stage on an infrastructure fault is sound and is not re-argued, but the degradation is recorded only on stdout, which is not the run's record. In the gate log an attempt audited on a partial subtree is indistinguishable from a fully audited clean attempt, and that verdict then feeds the attempt-2 prompt via §3.",
    "fix": "Keep parent-only proceeding and additionally write one gate-log line through the existing gateLog/printable() path (runner.ts:1144–1161) naming the partial audit and the session that could not be enumerated; log only, no stage failure, no schema change."
  },
  {
    "severity": "minor",
    "location": "Chosen design §1 Matcher — \"Unknown tools' `pattern` is treated as a glob-style access, matching the glob precedent; the grep-style reading of `pattern` as query text is a known residual (Risks row 2).\"",
    "problem": "That residual is a false positive (a search tool's regex query naming transcripts fails an attempt that looked nowhere), but Risks row 2 is the opposite direction: \"An unknown tool names its access under a key outside `PATH_ACCESS_KEYS` → missed violation\". The cited mitigation therefore does not exist, and the false-positive class this document exists to eliminate (run 005's 977 s loss, the quote-the-rule loop) is asserted then left untracked. The same applies to `command`, whose value can name transcripts/ inside a message string.",
    "fix": "Keep the r0 allowlist decision; correct the cross-reference by adding a Risks row for the false-positive direction, mitigated by the design's existing property that the violation string names the offending key and value (so a mis-scanned key is a one-line allowlist fix), and note the known search/glob tools are already in ACCESS_FIELDS with pinned exemptions, so the FP only reaches genuinely unknown tools."
  },
  {
    "severity": "minor",
    "location": "Chosen design §3 Retry feedback — \"each with `failureNote: printable(out.failure ?? 'no reply')`\" and the appended \"Do not repeat the behavior that caused that failure.\"",
    "problem": "This is the design's only new data flow into an agent and it is classified for one threat only: A9 proves quoting forbidden paths is safe for the audit, and printable() strips control bytes. Neither bounds content. For non-audit gates out.failure can carry verbatim tool or test output that an injected or misbehaving agent influenced, now injected into a fresh session's prompt with no length limit on top of the rendered template.",
    "fix": "Build failureNote through the existing brief() helper (already used for violation text in §2) so it is length-bounded, and label the appended block as a quoted log excerpt at the injection point (\"Previous attempt's gate failure (log excerpt): …\"). Does not require the structured \"rule broken\" field r0 rejected."
  },
  {
    "severity": "nit",
    "location": "Chosen design §1 Matcher — \"collect string values found under an allowlist of access-naming keys, recursively, in any nesting depth\"",
    "problem": "The traversal into server-recorded, agent-produced tool input is unbounded recursion, and the §2 error catch covers fetch/parse faults, not a RangeError thrown while matching the parent's own list. Since this branch is being rewritten anyway, the bound is free.",
    "fix": "Use an iterative queue with a node/depth cap that drops the deepest nodes instead of throwing, so a hostile or merely enormous tool input cannot take the audit call site down."
  },
  {
    "severity": "nit",
    "location": "Chosen design §2 Deeds of children — \"`client.session.list()`\"",
    "problem": "The fallback reads every session on the server to select descendants, a broader read than the walk needs, and the design never scopes what happens to the rest of that data.",
    "fix": "Add one clause stating the list result is used only to build the descendant set and is never logged or persisted, so unrelated sessions' metadata cannot reach the gate log or the attempt-2 note."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/007-audit-by-deeds/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design §2 Deeds of children — \"A child-walk fetch error after `bounded()`'s single retry is caught, logged … and the audit proceeds on the parent list alone\"","problem":"new: the sentence was accurate for the r0 array-returning helper (all-or-nothing fetch), but the revised helper is a lazy AsyncGenerator, so a late fetch error leaves the violations of every descendant already yielded sitting in hand; read literally the sentence instructs the implementer to discard them and audit the parent alone, silently shrinking the audited set while the gate log only says 'partial audit'. The new zero-descendant tripwire does not cover it — it fires only on zero enumerated descendants.","fix":"One clause in the same bullet, replacing the flat 'proceeds on the parent list alone': 'violations already collected from descendants fetched before the error are kept; the catch skips only the un-fetched remainder.'"},{"severity":"nit","location":"Compatibility matrix, row \"OpenCode server/SDK without a children method\" — \"else a direct `GET /session/:id/children` against the configured `baseUrl` with the client's auth headers\"","problem":"new: the row adds a hand-rolled network route outside the SDK client with three unspecified details — where the auth headers come from, that the session id taken from the walked SessionInfo records is not interpolated raw into the URL path, and that the headers are never logged. The BFS trusts those ids (the design itself guards against a malformed parentID graph), so an unencoded id is path-controlled data in a request to the local server; and A4 is still recorded unverified, so the row reads as if this route's reachability were verified.","fix":"One clause on the row: credentials come from the same config already used to construct OpenCodeClient (no second credential read), the id is percent-encoded (encodeURIComponent) into the path, headers are never logged (the §2 gateLog line names the session only), and this route lands in the same §2 fetch-error catch as the SDK routes."},{"severity":"minor","location":"Chosen design §2 Deeds of children — \"one `client.session.list()` paged with `cursor.next` … and indexed by `parentID` into a Map the BFS walks\"","problem":"re-raised: the revision widened the fallback from a filtered call per node to one full paged enumeration of every session on the server per walk, and the design still never scopes the remainder of that data. Unrelated sessions' metadata (titles, timestamps) is now resident in the runner at audit time with no statement that it cannot reach the gate log or the attempt-2 note.","fix":"Append to that bullet: 'the list result is used only to build the descendant set; it is never logged or persisted.' No behavior change."},{"severity":"nit","location":"Risks row 2 — \"A descendant still running at the snapshot contributes no completed parts, so its deeds go unseen for that attempt\"","problem":"new: contradicts §2, which says such a descendant 'contributes only its completed parts'. The real gap is only the deeds completed after the snapshot; the row overstates it, and the row is what a reader of the gate record leans on to judge whether the accepted gap is acceptable.","fix":"Align the row's wording with §2: 'its deeds after the snapshot go unseen for that attempt'."}]}
```
