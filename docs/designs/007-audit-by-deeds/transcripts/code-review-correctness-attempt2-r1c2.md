# Transcript: code-review:correctness:attempt2 r1c2

## Provenance
- stage: code-review:correctness:attempt2 (round r1c2)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T00:04:53.880Z
- ended: 2026-11-11T00:10:10.576Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/007-audit-by-deeds/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 95b73560fab5571ad65610ffe4f790d080d715c1..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/prompts/audit-by-deeds.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:1070-1111",
    "problem": "The child-walk audit sits inside executeInference's outer poll/parse loop, and every nudge path (continue at 1148/1174/1190/1212) re-enters it, so the descendant walk runs up to MAX_NUDGES+1 = 9 times per attempt. The proposal's cost paragraph fixes it as 'once per attempt after the reply, off the poll loop'. Consequences are new side effects: up to 9 near-identical 'independenceAudit' gate entries per attempt when the walk route is missing (which is every audited attempt of the nine pre-existing fakes, and of any client exposing neither SDK route), crowding the six-entry 'recent gates' view (runner.ts:2300-2302) and the persisted checkpoint, plus a repeat of the whole session-list crawl and one full paged message fetch per descendant on each nudge cycle.",
    "fix": "Run the descendant walk at most once per attempt: hoist a walkAudited flag (or compute the descendant violations before the parse/nudge paths and reuse them), leaving the idempotent parent-list audit where it is."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:637-668",
    "problem": "The session.list fallback pagination breaks only when pageCursorNext(page) is absent. If the endpoint ignores the cursor parameter (A4/A8 leave it unverified) but answers with a cursor.next, the for(;;) crawl never terminates: the BFS visited set cannot stop it because the pagination loop is separate, and byParent accumulates the same page's ids forever, hanging the attempt in the post-reply audit with only per-request timeouts.",
    "fix": "Break when the returned cursor equals the cursor just sent (or has been seen before), and add a page/node cap on the enumeration."
  },
  {
    "severity": "minor",
    "location": "docs/designs/007-audit-by-deeds/03-build-notes.md:28,99-100",
    "problem": "The notes state 'test files are not editable at this stage' and 'No test files were modified', and 'Files touched' lists no test files, while the change under review carries tests/pipeline/child-session-deeds.test.ts, tests/pipeline/child-deeds-retry.test.ts and extended pins at tests/pipeline/prompts.test.ts:207-212,359-372. A reviewer handed that diff and these notes is misled about the change set (and about the Test strategy #1-#8 pins existing).",
    "fix": "List the test files in the change set, noting the test-authoring stage authored them, so the notes describe the diff a reviewer receives."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:585",
    "problem": "listChildSessionMessages is exported, while the proposal's snippet keeps it module-private; the export exists only so child-session-deeds.test.ts:80-86 can drive the walk against fake clients. The widened module surface is undeclared.",
    "fix": "Record the export in 03-build-notes.md's additions list (or drive the walk solely through the audit call site)."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/independence.ts:97-98",
    "problem": "The delegation exemption keys on the tool-part name 'subagent', established only by fixtures (independence.test.ts:249-262). If a real server records the delegation part under another name, 'subagent: []' is inert: the false positive is still cured because the unknown-tool path never scans payload keys, but pin (a) would pin nothing production exercises.",
    "fix": "Confirm the recorded delegation part name from one real session message list and map that name in ACCESS_FIELDS as well."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:1084-1092",
    "problem": "The zero-descendant tripwire inspects only the parent list (hasCompletedDelegation(lastSeen, ...)). Proposal 2 says 'an audited message list'; a child list that itself holds a completed subagent part whose own descendants went unenumerated triggers no warning.",
    "fix": "Apply hasCompletedDelegation to each audited child list too, or note the narrower parent-only reading in the build notes."
  },
  {
    "severity": "nit",
    "location": "docs/designs/007-audit-by-deeds/03-build-notes.md:127-139",
    "problem": "Claims this seat could not evaluate: shell was denied, so 'npm run check green / 241/241 tests' and the exact diff boundary were not executed or reproduced here; note also that 'check' does not run typecheck (package.json:23), so a type slip in the new code would not fail the gate the notes cite as evidence.",
    "fix": "No change required for the code: re-run 'npm run check' and 'npm run typecheck' in a seat that has shell and record the output."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/007-audit-by-deeds/reviews/code/correctness-r1c2.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "tools/orchestration/src/pipeline/runner.ts:859-871,1247-1248", "problem": "The new perf guard gates the entire child walk on countDelegationParts matching the recorded part name 'subagent' (the tripwire keys the same name). If a real server records the delegation part under another name — the question left open by r1 nit 5, established only by fixtures — the walk is now skipped silently (no enumeration, no warn, no gate line), whereas the proposal ran the walk after every passing parent audit and caught the child's deeds regardless of the part's name. The declared cost rule closes a cost hole but removes the design's name-agnostic safety net; the build notes justify it for cost and the stale-window case, not this one.", "fix": "Verify the recorded delegation part name against one real session message list; if it differs from 'subagent', map the real name in countDelegationParts (and ACCESS_FIELDS) so the guard and the exemption key on what production records."}, {"severity": "minor", "location": "docs/designs/007-audit-by-deeds/03-build-notes.md:27-39,102", "problem": "Carried r1 finding 3, unfixed and now sharper: deviation 2 still states the nine pre-existing fakes expose only session.create/prompt/remove, and line 102 still states 'No test files were modified', while this diff adds session.list to all nine fakes (Test strategy #9, landed by the test-update stages) and modifies 14 test paths including the child-deeds-retry gate-pair pin rework prescribed by addendum 3. No addendum records the test-update stages' outcome, so the notes contradict the change set a reviewer receives.", "fix": "Append one short addendum listing the test files in the final change set (nine fakes' session.list stubs, the two child-deeds suites' new pins, the prompts.test.ts citations, the corrected child-deeds-retry gate pin), noting the test-update stage authored them."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:614", "problem": "directChildSessions builds the URL with an absolute path in new URL('/api/session/…', connection.baseUrl), which drops any path component of a configured baseUrl, while createClient hands the whole base URL to the SDK. A --base-url with a path prefix makes the third enumeration route 404 into the caught partial-audit path (fails closed; the default URL has no path).", "fix": "Join the SDK's path prefix instead of using an absolute path, or document the no-path-prefix constraint on ServerConnection."}, {"severity": "nit", "location": "tools/orchestration/tests/pipeline/child-deeds-retry.test.ts:120-124, tests/pipeline/child-session-deeds.test.ts (list-route pin comments)", "problem": "The new pin comments state as fact that the pinned SDK 'exposes no session.children', but the package is not installed in this worktree and Rollout step 1 never ran — A4 remains register-unverified and the walk's real route stays a runtime probe. The comments pin an unverified platform claim.", "fix": "Reword the comments to name this as one probed/assumed shape rather than a verified SDK fact."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:655", "problem": "Carried r1 nit: listChildSessionMessages remains exported (the proposal's snippet keeps it module-private; the export exists for the unit tests), and the widened module surface is still not recorded in the build notes' additions list.", "fix": "Record the export in 03-build-notes.md's additions list, or drive the walk solely through the audit call site."}, {"severity": "nit", "location": "tools/orchestration/src/pipeline/runner.ts:1265", "problem": "Carried r1 nit: the zero-descendant tripwire inspects only the parent list. A child list that itself holds a completed subagent part whose own descendants went unenumerated triggers no warning; the narrower parent-only reading remains undeclared.", "fix": "Apply hasCompletedDelegation to each audited child list too, or note the parent-only reading in the build notes."}, {"severity": "nit", "location": "tools/orchestration/package.json:23", "problem": "Carried r1 nit: npm run check was re-verified green in this seat (250/241-consistent tests), but check still excludes typecheck and npm run typecheck was denied shell in every review seat, so no seat has recorded tsc --noEmit over the new code.", "fix": "Have one seat with shell run 'npm run typecheck' in tools/orchestration and record the output in the build notes."}]}
```
