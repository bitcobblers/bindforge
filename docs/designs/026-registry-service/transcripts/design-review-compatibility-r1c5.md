# Transcript: design-review:compatibility r1c5

## Provenance
- stage: design-review:compatibility (round r1c5)
- agent: review/design-compatibility
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:41:57.019Z
- ended: 2026-11-19T19:45:58.485Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":50509,"output":19907,"reasoning":0,"cache.read":417305,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the ratification pass c5: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 07e8b3f8b64d6421c1f257e00aeeb876f54423ca e1e63cb6acd1d9d855829f18df4d83607ae48aa1` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/compatibility-r1c5.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "§The serving face step 1 — \"Path safety is the only extra refusal, at its own scope: after the first `/`, one non-empty component with no further `/` or `\\`, none equal to `.` or `..`, and no control byte, space, or invalid-UTF-8 sequence\" (carried into §Catalog document and its spec, §Budgets' cache-load clause, and #1/#3/#4)",
    "problem": "re-raised (narrowed from r1's major; the dot case is closed). The space ban is not a path-safety refusal, and the same paragraph's parenthetical says lookup keys are \"opaque in-memory strings, never joined onto paths\" — yet the sentence above it claims \"that shipped pair is the registry's whole key rule\". Shipped evidence: Source::spells_lookup_key (crates/discovery/src/game.rs:84-89) admits any non-empty external id after a closed-set token; Source::lookup_key (game.rs:66-68), the single composer, imposes nothing further; the shipped store's only slug checks are the 025-B1 cap plus spells_lookup_key (apps/discovery-cli/src/matches.rs:222-234); and the shipped client puts such a key on the wire, escaping a space as %20 (catalog.rs:174-185, A19). So a space-bearing 025-C1 key — the local/<dir> external id, whose only stated derivation law is \"the kebab'd install directory name\" with no grammar — is refused at all four seams this rule governs: candidate query (404), admin upsert (refused… [elided by the checkpoint cap]",
    "fix": "Delete `space` from the banned set, leaving \"no control byte or invalid-UTF-8 sequence\", so the extra refusals are exactly the named path-safety and line-integrity classes; nothing else changes because every dependent seam already references this one clause. If a space ban is wanted on some other ground, keep it but state that ground here and name it as a narrowing of the 025-C1/026-C3 key vocabulary that no shipped spelling rule shares."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/026-registry-service/reviews/design/compatibility-r1c5.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "nit", "location": "§Deferred / out of scope, lift bullet — \"Lifting the six copied invariant blocks (§`apps/registry`) from `apps/discovery-cli` into a shared crate so both binaries call one copy, which also gives `Source`/`spells_lookup_key` that shared home\"", "problem": "The r4 rewrite of this bullet dropped the explicit retirement of `apps/registry`'s `discovery` edge \"with its Windows-only `winreg` transitive\" that r1 stated here, while §`apps/registry` still promises \"retired by the deferred lift below\" and the revision-log dependency line still asserts the retirement. The deferral now carries only half of the promise the body points at: its named scope is the six copied client blocks plus a home for `Source`/`spells_lookup_key`, so the OS-specific-dependency consequence must be inferred. No ADR or contract impact; the lift itself still exists and my r1 key-rule finding is fully closed.", "fix": "Restore the deleted clause into the same bullet: \"…, which also gives `Source`/`spells_lookup_key` that shared home and so retires `apps/registry`'s `discovery` edge with its Windows-only `winreg` transitive\" — no new scope, one clause."}, {"severity": "nit", "location": "§The serving face step 1 — \"Percent-decode `slug` **first**, then apply `025-B1`'s 128-octet cap and `spells_lookup_key` to the *decoded* bytes\", read with the r3 space admission (\"no control byte or invalid-UTF-8 sequence\")", "problem": "The widened alphabet makes an unspecified part of the decode seam load-bearing. The copied `percent_encode` (A19, verified at apps/discovery-cli/src/catalog.rs:174-185) never emits a bare `+` and escapes `+` as `%2B`, so a percent-decode that also folds `+` to a space (the form-urlencoding habit) turns a shipped-legitimate `local/foo+bar` into `local/foo bar`. Before the space ban was deleted that mistranslation died loudly at the seam; now it yields a legal key, which caching mode then forwards `%20`-encoded (step 4) and caches under, answering a different question than it was asked. The design never authorizes `+`-folding, so this is underspecification introduced by the closure of my own finding, not a contradiction.", "fix": "In step 1, pin the decode to percent-escapes only: \"percent-decode (a literal `+` stays `+`; the only space spelling is `%20`, as the copied `percent_encode` emits)\". One parenthetical; #3's `404` list and the outbound hop are unchanged by it."}]}
```
