# Transcript: design-review:security r1c0

## Provenance
- stage: design-review:security (round r1c0)
- agent: review/design-security
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-12T01:32:46.451Z
- ended: 2026-11-12T01:34:08.903Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/010-lightweight-plan-flow/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/010-lightweight-plan-flow.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/010-lightweight-plan-flow.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/010-lightweight-plan-flow/reviews/design/security-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"The `plan` stage — oversized-plan gate: \"`planOutgrewFlow(lines, files, cfg)` denies when … the `## Files to touch` section's bullet count exceeds `cfg.planMaxFiles`; a missing section counts as 0 items (a faithfulness concern for the seats, not a gate)\"","problem":"The design's only deterministic control against its headline risk (Risks row 2: an operator marks a wide-consequence change light; Deferred: \"only `planMaxFiles` is designed here\") fails open on its own input. The plan is model-authored prose, so the section header is not runner-controlled: an absent, renamed, re-leveled, or case-varied header yields a count of 0 and the file gate silently becomes a no-op for exactly the wide change it exists to catch. Test strategy #5 pins that fail-open (\"a missing section counting as 0\"), so implementation builds it. The parenthetical rebuts a different question (should absence itself be a scope failure) and waiving that is reasonable; it does not address that the waiver also zeroes the measured count and disables the gate. The Revision log's OQ4 entry only records adding the gate, so this is not a rejected finding.","fix":"Fail closed on extraction, not on content: match the header case- and whitespace-insensitively against the template's spelling, and have `planOutgrewFlow` deny an unextractable section with `plan has no \"## Files to touch\" section: the light flow requires it, cap 10 files`, replacing the \"counts as 0\" clause; flip Test strategy #5's missing-section case to \"missing/renamed section denies\"."},{"severity":"minor","location":"Ownership — the security spine, widened in one flow: \"then light `implement`/`fix` may write test paths, full keeps `!inTests`\"","problem":"The widening is stated only as a write permission, so the light branch of the security spine has no rule about removal of test paths — the sharpest form of the ADR's separation-of-duties concern, since a coder that hits a failing test can delete it and no deterministic gate sees it. Removal is reachable by construction: the threaded `ownershipIssue(stage, pair, flow)` lets a light `implement` \"declare a test→test move\", which is a delete plus an add, so pure deletes ride in on the same rule. Test strategy #7, labeled \"the security contract\", asserts only the positive allow case, so the removal direction is untested in both flows.","fix":"Bound the widening in the same sentence: light `implement`/`fix` may add and edit paths under `tests/**`, while a stage commit that removes a `tests/**` path present at the run's base commit without a paired addition stays denied and is reverted; add that case (removal reverted in light, test→test move allowed) to Test strategy #7."},{"severity":"minor","location":"Compatibility matrix / Assumption register A20: \"Node ≥ 22.23.4 (package `engines`), yes — No new platform surface\" and \"Node older than the `engines` floor — no — Unchanged policy\"","problem":"The design names a runtime floor and re-affirms it as supported. Node.js shipped 22.23.5 on 2026-11-11 fixing 16 CVEs, 7 rated high, with llhttp updated on the 22.x line; the floor release 22.23.4 carries the Brotli decoder RCE (CVE-2026-111497) now on CISA's KEV list. Exposure here is client-side and modest (the runner is an HTTPS client to the inference provider; the 2026-11-11 high issues are HTTP/2/TLS-server side), and no dependency the design introduces is unfixed, so this is not a blocker — but the supported-platform row pins a version with published high advisories.","fix":"Raise the `engines` floor and the matrix row to the line's current security release (>=22.23.5), or state that the floor tracks the latest security release of the supported line, so the row does not pin an advisory-bearing version."},{"severity":"nit","location":"Selecting the flow: \"when a `flow:` key is present with an unreadable value, throws `invalid flow \"<value>\" for <source>: expected full or light`\"","problem":"The rejection path echoes raw, unvalidated document text into operator-visible CLI output. `parseSource` is a regex scan over document bytes, so a source document under `docs/prompts/**` (user-supplied per ADR-001) can carry ESC/OSC sequences inside the value; printed verbatim in a terminal next to genuine gate lines, it can spoof gate output, set the terminal title, or load the clipboard. The accepted path is already safe (the value is enum-matched before it reaches the `sourceAdmissible` gate line), which is the right posture.","fix":"Echo a sanitized rendering in the same throw: strip C0/C1 control characters and cap the value (e.g. first 32 characters)."}]}
```
