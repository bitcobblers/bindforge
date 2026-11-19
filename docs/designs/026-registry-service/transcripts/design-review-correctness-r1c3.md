# Transcript: design-review:correctness r1c3

## Provenance
- stage: design-review:correctness (round r1c3)
- agent: review/design-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:33:26.431Z
- ended: 2026-11-19T19:37:05.407Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":38451,"output":19402,"reasoning":0,"cache.read":360391,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 07e8b3f8b64d6421c1f257e00aeeb876f54423ca 1e8f240c665b6ebaf83b0b7c8f239714c9c0a740` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/correctness-r1c3.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Budgets / The admin face and auth — \"`MAX_REQUEST_BODY_OCTETS` (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally hold also fits an upsert and no per-record `source_keys` bound is needed)\" and, in §Catalog document and its spec, the cap placed only in the loader's list: \"a document over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets)\"",
    "problem": "The corpus document cap has no write-time rule anywhere in the design, while the cache cap now has one (\"after a write the registry serializes the cache … until the buffer is ≤ `026-B2` octets\"). An accepted `POST /v1/admin/games` can therefore persist a corpus whose serialized length passes 16,777,216, and the next start refuses the whole corpus by name — every accepted mutation on it is lost, which falsifies the sentence this revision just added to §Concurrency (\"a restart cannot lose an accepted mutation (#11)\"). Both reachable paths are newly opened by this revision: (a) the body cap was raised from 65,536 to the document cap, so a single legal upsert (026-C3 bounds each source_key by 025-B1 and the design now names no per-record bound) can push an empty corpus past the cap in one accepted request; (b) the document cap was raised to the ADR's \"tens of thousands of games\" scale, so a root corpus living at the cap is now a supported configuration (under r1's 1 MiB it was already an e… [elided by the checkpoint cap]",
    "fix": "Replace \"validated against `026-C3` and refused by name; bumps `revision`; persists atomically through the tmp-and-rename discipline\" with \"validated against `026-C3` and refused by name; the write is then completed by serializing the whole document and refusing, by name, `MAX_CATALOG_DOCUMENT_OCTETS` as a `422` that leaves `revision`, the model and the file untouched; only then does `revision` bump and the tmp-and-rename run (the loader's over-cap refusal stays as the same rule's read side)\", add that refusal to `schemas/catalog/v1/spec.md`'s roster next to the record-level ones, and pin in #10: \"an upsert whose serialized corpus would pass `MAX_CATALOG_DOCUMENT_OCTETS` is the named `422`, `revision` unchanged, no tmp written\"."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Budgets — \"after a write the registry serializes the cache once and drops the oldest-`fetched_at` entries … until the buffer is ≤ `026-B2` octets and the entry count ≤ `026-B1`, then renames that buffer into place, so the octet test is the document's exact length and not a sum of per-entry totals\"",
    "problem": "The two halves pull apart: measuring \"the document's exact length\" after each drop requires a re-serialization per eviction, while subtracting each dropped entry's length from one buffer is exactly the \"sum of per-entry totals\" the clause disclaims.",
    "fix": "Replace \"serializes the cache once\" with \"serializes the cache, and re-serializes after each dropped entry (at most one extra pass per eviction)\", leaving the rest of the sentence."
  },
  {
    "severity": "nit",
    "location": "Chosen design → The admin face and auth, Concurrency — \"a persist-only lock to cut that latency joins §Deferred's lift if it ever earns its keep\"",
    "problem": "§Deferred's lift entry is \"Lifting the five copied invariant blocks … into a shared crate\" and carries no persist lock; a latency mechanism is deferred to a list that does not contain it, and that lift is a code-sharing refactor rather than a latency item.",
    "fix": "Add \"and, if #11's N-upsert latency ever matters, the persist-only lock §Concurrency deletes\" to that §Deferred bullet, or drop the pointer and state the \"reader may wait on one local write\" latency as accepted."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Catalog document and its spec, harness list — \"`fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique` is **not** guarded … which is the only coverage behind the Windows `Yes` row's case-insensitive-filesystem claim (#19)\"",
    "problem": "The Compatibility matrix's Windows row makes no case-insensitivity claim in either its r1 or r2 form (\"`<data>` = `%APPDATA%` … owner-only modes are a documented no-op … cache and local-entries paths …\"); the case-collision claim is #19's (\"**Windows** and **macOS** behavior is proven by the env-closure seam (#13) and case-unique fixture names\"), so the rationale cites a matrix claim that is not there.",
    "fix": "Say \"the only coverage behind #19's case-unique-fixture-name claim for a case-insensitive filesystem\", or add the case-collision note to the Windows row so the pointer lands."
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
Write your report to docs/designs/026-registry-service/reviews/design/correctness-r1c3.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → The serving face step 1 / Caching mode (upstream client) — \"Miss with reachable upstream fetches `GET <upstream>/v1/candidates?slug=`, validates the answer, and caches it\", read against §`apps/registry`'s closed copy list (\"five invariants are copied … the response reader (#7), the endpoint gate (#14, #15), the `game_id_is_valid` predicate (#8), the `<data>` rule (#13), and the tmp-and-rename persist (#10, #11)\") and the Revision log's r3 line \"deletes the space refusal\"","problem":"[new; introduced by this revision's closure] Widening the admitted key alphabet proves the keys safe against the filesystem (\"keys are opaque in-memory strings, never joined onto paths\") but never against the wire, and nothing in the design says how the caching registry spells an admitted key in the request line it sends upstream. The shipped `percent_encode`/`candidates_url` request builder (A19) is not among the five copied invariants, and #7 pins only response refusals, so the outbound request target is unspecified. The ADR's own `local/<dir>` case makes this concrete: a directory name with a space is now a legal slug (`local/Steam Big Game`), and fetching `GET /v1/candidates?slug=steam/Steam Big Game` sends a request target with a raw space, which the root's own request reader refuses as a framing fault — the user gets the stale/`503` path for a key their corpus can legally hold. For a `%`-bearing key (also admitted, since `%` is no control byte and no separator) it is worse: forwarding the decoded `steam/foo%2Fbar` raw makes the upstream decode twice, refuse it as a further separator, or answer for a different key, which this registry then caches under its own key — a wrong answer that outlives the request. T-007 must invent this rule, and no §Test strategy item would catch the choice.","fix":"Name the shipped `percent_encode`/`candidates_url` discipline (A19) as the sixth copied invariant in §`apps/registry` (and change \"five\" to \"six\" in §`apps/registry` and §Deferred, since the lift now carries it too), and replace step 4's \"fetches `GET <upstream>/v1/candidates?slug=`, validates the answer, and caches it\" with \"fetches `GET <upstream>/v1/candidates?slug=`, the request target built from the *decoded* slug through the copied `percent_encode` (A19), so the bytes the upstream decodes are exactly the key this instance asked about, validates the answer, and caches it\"; pin it in #7: \"a key bearing a space or `%` reaches the upstream only as its encoded spelling\"."},{"severity":"minor","location":"Chosen design → The admin face and auth — \"the model with the upsert is then serialized whole, and one over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets) is the named `422` … leaving model, `revision` and file untouched with no tmp written. Only then does `revision` bump and the tmp-and-rename persist run (A7), under §Concurrency's one lock\"","problem":"[new; residue of the r1 major's closure] The buffer tested is the pre-bump document, but the bytes renamed into place are the post-bump model (§Concurrency: \"a mutation serializes the new model, writes the fixed tmp name, renames it into place\"), so the persisted document can be one octet longer than the checked one whenever `revision` grows a digit (9→10, 99→100, …). A corpus serializing to exactly 16,777,216 with `revision` 9 passes the check and persists 16,777,217, and the next start refuses the whole corpus by name — falsifying the two absolutes this revision just wrote: \"no accepted mutation can persist a corpus its own loader refuses\" (§Budgets) and \"a restart cannot lose an accepted mutation (#11)\" (§Concurrency).","fix":"Order the bump before the check on the candidate model so the buffer tested is the buffer renamed: \"the model with the upsert and the next `revision` value is then serialized whole, and one over `MAX_CATALOG_DOCUMENT_OCTETS` … is the named `422` … leaving model, `revision` and file untouched with no tmp written; only then is that same buffer written to the fixed tmp, renamed, and the model swapped in\"; and in #10, \"an upsert whose serialized corpus would pass\" → \"an upsert whose serialized corpus with its bumped `revision` would pass\"."}]}
```
