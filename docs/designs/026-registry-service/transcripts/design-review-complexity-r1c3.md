# Transcript: design-review:complexity r1c3

## Provenance
- stage: design-review:complexity (round r1c3)
- agent: review/design-complexity
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:33:26.433Z
- ended: 2026-11-19T19:36:22.739Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":43682,"output":13593,"reasoning":0,"cache.read":237559,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/complexity-r1c3.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "§Budgets — \"`MAX_REQUEST_BODY_OCTETS` (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally hold also fits an upsert and no per-record `source_keys` bound is needed)\" with \"`MAX_CATALOG_DOCUMENT_OCTETS` (16,777,216 — ≈120,000 records …), `fs::metadata`-gated before the read\"",
    "problem": "This round's new equivalence closes only one direction: the corpus's 16 MiB cap exists at load only (§Catalog document's refusal list, §Budgets' metadata gate, #1), while §The admin face lets accepted mutations grow that same document with no counterpart check — so a root can be driven past the size its own loader refuses and then not restart until the operator hand-edits the JSON. The cap is sized at exactly the ADR's projected scale (\"tens of thousands of games, each with several store keys\"), so growth across it is foreseeable, not exotic; this is the same reasoning §Options used to reject refuse-the-file-alone for the cache (\"it does not stop a legal-but-large corpus … from exceeding 1 MiB between writes\"), yet the corpus/cache asymmetry is recorded nowhere and #10/#11 pin revision and persist order but never size. The bound declined (a per-record `source_keys` count) is precisely what the enlarged body cap is claimed to make unnecessary.",
    "fix": "Reuse the mechanism §Budgets just defined instead of adding one: \"the corpus persist applies the same write-buffer check — a mutation whose serialized document exceeds `MAX_CATALOG_DOCUMENT_OCTETS` is refused by name, no rename\" (the persist is already under the one write lock per §Concurrency); add that refusal to §Catalog document's list and one case to #11. If the asymmetry is instead deliberate, say so in one clause and restore the r1 per-record `source_keys` bound with the 64 KiB body cap so the loadable-corpus claim is not load-only."
  },
  {
    "severity": "minor",
    "location": "§`apps/registry` modules — \"`serve.rs` — accept loop (one thread per admitted connection, covering that connection's whole request loop, up to `MAX_CONCURRENT_CONNECTIONS`, over-cap closed at accept…)\" plus §Budgets' \"one thread per admitted connection for that connection's whole life, so ≤64 live threads and ≤64 in-flight upstream hops\"",
    "problem": "r1 said one thread per connection; r2 adds a per-connection request loop — a mechanism with no consumer, since the only client this face must satisfy sends `Connection: close` with one request per connection (apps/discovery-cli/src/catalog.rs:438, A19) and §http.rs never states whether the connection is closed after the answer. The phrase also undercuts the bound it was added to state: with a keep-alive loop an idle client holds one of the 64 admitted slots for the connection's whole life (bounded only by the read timeout), which is exactly when the \"≤64 in-flight upstream hops\" claim matters.",
    "fix": "Replace the phrase with \"one request-response per admitted connection, then close, echoing the shipped client's `Connection: close`\" and add that close to §http.rs; if keep-alive is intended, name it as such in the same sentence together with the idle-slot timeout that bounds it."
  },
  {
    "severity": "nit",
    "location": "§Test strategy #6 — \"with the fresh/stale pair run under `--refresh-interval 1s`\" (with §CLI surface's \"`--refresh-interval` overrides `026-B3` (default `REGISTRY_UPSTREAM_TTL_SECONDS = 3_600`)\")",
    "problem": "The flag's value spelling is never defined and the case added this round depends on it: the budget is a count of seconds, so a `u64`-seconds parser rejects `1s` and the new #6 case fails on a parse error rather than on the policy it means to pin.",
    "fix": "State the unit once at the knob (\"`--refresh-interval <seconds>`\") and write the case as `--refresh-interval 1`."
  },
  {
    "severity": "nit",
    "location": "§The serving face step 1 — \"Path safety is the only extra refusal, at its own scope: after the first `/`, one non-empty component with no further `/` or `\\`, none equal to `.` or `..`, and no control byte, space, or invalid-UTF-8 sequence\"",
    "problem": "The `space` item is the only entry in the list that is not path safety: it cannot traverse or split a path, and it cannot break the one-line refusal discipline the control-byte ban already protects, while the same sentence keeps keys as \"opaque in-memory strings, never joined onto paths\". It is also the one clause that can still 404 a key the shipped pair admits — `spells_lookup_key` (crates/discovery/src/game.rs:84-89) and `Source::lookup_key` (game.rs:66-68) impose nothing on the external id, the shipped store checks only that pair plus `025-B1` (apps/discovery-cli/src/matches.rs:229,282), and the client sends any such key escaping a space as `%20` — with `local/<dir>`, the ADR's own example of what the admin face exists to curate, the natural home of a space-bearing id. Nothing shipped emits one today, hence nit.",
    "fix": "Drop \"space\" from the refusal list, or keep it and name in one clause the `local/` key-derivation law it enforces."
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
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/026-registry-service/reviews/design/complexity-r1c3.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "§The admin face and auth — \"the model with the upsert is then serialized whole, and one over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets) is the named `422` … leaving model, `revision` and file untouched … Only then does `revision` bump and the tmp-and-rename persist run\"", "problem": "new: the buffer that is checked is not the buffer that is written. `revision` is a serialized field of the document (§Catalog document's `{version, revision, games}`; ADR `026-C3`'s \"bumped by every mutation\"), so the persisted document is the checked buffer plus the octets the bump adds, and at the boundary (≈16,777,215 octets, or a revision crossing a power of ten) an accepted upsert persists a corpus a few octets over the cap that its own loader then refuses — the exact state §Budgets' new sentence claims cannot happen (\"so no accepted mutation can persist a corpus its own loader refuses\"). The two clauses also disagree with each other about whether the bump is inside the checked serialization: \"the model with the upsert\" reads as though it includes it, \"Only then does `revision` bump\" says it does not. My r1 major is otherwise closed by the right mechanism; this is the ordering residue of the new text.", "fix": "One clause: serialize the candidate copy with the bump already applied, check that buffer, and have the persist write exactly that buffer, swapping the model (new record + new `revision`) in only after the rename succeeds; then #10 pins the boundary inclusive at the cap with the revision already incremented. No new mechanism — the cache document's §Budgets check has no such hole because `{version, entries}` carries no `revision`, which is precisely why the corpus is the one place the ordering must be stated."}, {"severity": "nit", "location": "§Test strategy #10 — \"an upsert whose serialized corpus would pass `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422` with model, `revision` and file untouched\"", "problem": "new: \"would pass\" reads as \"satisfies the cap\", which inverts the case being pinned. The document's own refusal voice is \"over\" (#1 \"document over `MAX_CATALOG_DOCUMENT_OCTETS`\", #5 \"load over `026-B2` is refused\"), and the r3 Revision-log line repeats the same ambiguous word (\"an upsert whose serialized corpus passes it being the named `422`\"). A test engineer pinning #10 literally writes the acceptance boundary backwards.", "fix": "\"would exceed `MAX_CATALOG_DOCUMENT_OCTETS`\" here and in the Revision-log clause."}, {"severity": "nit", "location": "§Deferred — \"Lifting the five copied invariant blocks … into a shared crate so both binaries call one copy, which also gives `Source`/`spells_lookup_key` that shared home\"", "problem": "new: this round's trim deleted the entry's tail (\"and retires `apps/registry`'s `discovery` edge with its Windows-only `winreg` transitive\"), while §`apps/registry` (unchanged text) still promises that edge is \"taken for that predicate and retired by the deferred lift below\" — a pointer into text that no longer makes the claim. The claim is true (`crates/discovery/Cargo.toml:39` carries `winreg = { workspace = true }`), so the casualty is the cross-reference, and with it the lift's only stated payoff beyond deduplication.", "fix": "Restore the retired-transitive clause in the same §Deferred sentence (preferred), or delete \"and retired by the deferred lift below\" from §`apps/registry`."}]}
```
