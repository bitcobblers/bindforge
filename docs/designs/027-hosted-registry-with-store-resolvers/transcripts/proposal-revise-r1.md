# Transcript: proposal-revise r1

## Provenance
- stage: proposal-revise (round r1)
- agent: design/designer
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/proposal-revise.md @ 23b4fda203cb
- started: 2026-11-22T04:16:40.345Z
- ended: 2026-11-22T04:20:18.524Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":26822,"output":21416,"reasoning":0,"cache.read":1122498,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Revise docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md (round 1).
Address every open blocker and major finding in these review reports:
docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c3.md
Consuming a finding means CHANGING the design, and changing it is not the same
as adding to it. For each finding use the smallest move that makes it stop
being true, preferring them in this order: delete the passage or sub-system
that produced it; rewrite or narrow the passage so the conflict cannot arise;
hand the concern to the Deferred / out of scope section when another document
owns it; add text only when none of those applies. Deleting content is a
complete answer to a finding. A round whose diff only adds text has not
consumed the feedback.
Aim for a net-zero or net-negative line count. If your diff is net positive,
spend one sentence of `summary` saying why.
Read in slices: the sections the findings quote, the Revision log, and the
reports above. Re-read the whole document only when a finding spans
sections; never re-read a passage you are not going to change.
Keep the change targeted. Maintain the Revision log in place: one current
entry per finding that is still material, anchored by section name (never a
line number), superseding the entry it replaces instead of adding to it.
If you conclude the proposal ALREADY addresses every finding, make no
edits and reply with {"changed": false, "summary": "<why each finding
is already resolved>"} — that is a valid outcome; inventing edits is not.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"changed":{"default":true,"type":"boolean"},"summary":{"type":"string","minLength":1},"commitMessage":{"type":"string","minLength":1}},"required":["changed"],"additionalProperties":false}
```

## Raw reply
```text
{"changed": true, "summary": "Major (control-byte strip miscredited to the shipped clamp): deleted the false clause 'with control bytes dropped in the same pass' and the restatement of the rule in §Risks; §The store pull now states the one line the registry owns — drop `char::is_control()` characters, then the shipped `friendly_name` clamp — with A7 rewritten to the verified shipped rule (whitespace collapse + char-boundary clamp only, `split_whitespace` leaves NUL/ESC/DEL/C1 in place, basis now also `crates/discovery/src/lib.rs`), mirrored in Test strategy #6 and Rollout 3, and the loader's control-byte refusal left as the contract-document discipline; also deleted §The steam snapshot resolver's duplicate 'every appid the pull was handed is in it' claim, consolidated into §The hosted lookup policy step 5. Nit (Budgets arithmetic): `MAX_STEAM_PAGE_OCTETS` now reads as 12 MiB, above the 50k-at-~240-octets estimate, so the constant stops looking derived. Nit (empty-normalized name): one clause — it stores empty, renders blank, identity is mint step 3's `steam-<appid>` — plus its Test #6 pin. Nit (path): `discovery::game::friendly_name` spelled correctly in A7, §The store pull and Rollout 3. Revision log maintained in place: the over-bound-names entry is superseded in place (r0→r4) and the page-budget entry amended, not appended. Net +1 line (three deletions/consolidations against the two dispositions that needed new wording).", "commitMessage": "docs: own the control-byte strip at the pull seam in the 027 proposal"}
```
