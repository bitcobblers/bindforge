# Transcript: design-review:correctness r1c4

## Provenance
- stage: design-review:correctness (round r1c4)
- agent: review/design-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:40:21.498Z
- ended: 2026-11-19T19:41:56.902Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":35923,"output":12091,"reasoning":0,"cache.read":661678,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the dissent pass c4: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 1e8f240c665b6ebaf83b0b7c8f239714c9c0a740 e1e63cb6acd1d9d855829f18df4d83607ae48aa1` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/026-registry-service.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/correctness-r1c4.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → The serving face step 1 / Caching mode (upstream client) — \"Miss with reachable upstream fetches `GET <upstream>/v1/candidates?slug=`, validates the answer, and caches it\", read against §`apps/registry`'s closed copy list (\"five invariants are copied … the response reader (#7), the endpoint gate (#14, #15), the `game_id_is_valid` predicate (#8), the `<data>` rule (#13), and the tmp-and-rename persist (#10, #11)\") and the Revision log's r3 line \"deletes the space refusal\"",
    "problem": "[new; introduced by this revision's closure] Widening the admitted key alphabet proves the keys safe against the filesystem (\"keys are opaque in-memory strings, never joined onto paths\") but never against the wire, and nothing in the design says how the caching registry spells an admitted key in the request line it sends upstream. The shipped `percent_encode`/`candidates_url` request builder (A19) is not among the five copied invariants, and #7 pins only response refusals, so the outbound request target is unspecified. The ADR's own `local/<dir>` case makes this concrete: a directory name with a space is now a legal slug (`local/Steam Big Game`), and fetching `GET /v1/candidates?slug=steam/Steam Big Game` sends a request target with a raw space, which the root's own request reader refuses as a framing fault — the user gets the stale/`503` path for a key their corpus can legally hold. For a `%`-bearing key (also admitted, since `%` is no control byte and no separator) it is worse: forwa… [elided by the checkpoint cap]",
    "fix": "Name the shipped `percent_encode`/`candidates_url` discipline (A19) as the sixth copied invariant in §`apps/registry` (and change \"five\" to \"six\" in §`apps/registry` and §Deferred, since the lift now carries it too), and replace step 4's \"fetches `GET <upstream>/v1/candidates?slug=`, validates the answer, and caches it\" with \"fetches `GET <upstream>/v1/candidates?slug=`, the request target built from the *decoded* slug through the copied `percent_encode` (A19), so the bytes the upstream decodes are exactly the key this instance asked about, validates the answer, and caches it\"; pin it in #7: \"a key bearing a space or `%` reaches the upstream only as its encoded spelling\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design → The admin face and auth — \"the model with the upsert is then serialized whole, and one over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets) is the named `422` … leaving model, `revision` and file untouched with no tmp written. Only then does `revision` bump and the tmp-and-rename persist run (A7), under §Concurrency's one lock\"",
    "problem": "[new; residue of the r1 major's closure] The buffer tested is the pre-bump document, but the bytes renamed into place are the post-bump model (§Concurrency: \"a mutation serializes the new model, writes the fixed tmp name, renames it into place\"), so the persisted document can be one octet longer than the checked one whenever `revision` grows a digit (9→10, 99→100, …). A corpus serializing to exactly 16,777,216 with `revision` 9 passes the check and persists 16,777,217, and the next start refuses the whole corpus by name — falsifying the two absolutes this revision just wrote: \"no accepted mutation can persist a corpus its own loader refuses\" (§Budgets) and \"a restart cannot lose an accepted mutation (#11)\" (§Concurrency).",
    "fix": "Order the bump before the check on the candidate model so the buffer tested is the buffer renamed: \"the model with the upsert and the next `revision` value is then serialized whole, and one over `MAX_CATALOG_DOCUMENT_OCTETS` … is the named `422` … leaving model, `revision` and file untouched with no tmp written; only then is that same buffer written to the fixed tmp, renamed, and the model swapped in\"; and in #10, \"an upsert whose serialized corpus would pass\" → \"an upsert whose serialized corpus with its bumped `revision` would pass\"."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/026-registry-service.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/026-registry-service/reviews/design/correctness-r1c4.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "nit", "location": "§Test strategy #10 (**Admin mutations**) — \"an upsert whose serialized corpus with its bumped `revision` would pass `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422`\"", "problem": "new; in this round's rewritten line. \"would pass <a budget>\" reads two ways: \"exceed it\" (the intent, matching §The admin face and auth's \"one over `MAX_CATALOG_DOCUMENT_OCTETS` … is the named `422`\" and §Budgets' \"checked against the exact post-`revision`-bump buffer an admin write renames into place\") or \"satisfy it\", which inverts the rule. Every other refusal site in the document uses \"over\", so this is the only place a test engineer authoring #10 from the item alone could pin the opposite of the contract. Not blocking: the two unambiguous statements elsewhere fix the meaning.", "fix": "One word: \"an upsert whose serialized corpus with its bumped `revision` would exceed `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422`\"."}]}
```
