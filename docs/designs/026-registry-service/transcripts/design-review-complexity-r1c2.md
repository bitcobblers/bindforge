# Transcript: design-review:complexity r1c2

## Provenance
- stage: design-review:complexity (round r1c2)
- agent: review/design-complexity
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:25:07.148Z
- ended: 2026-11-19T19:27:56.202Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":60367,"output":20588,"reasoning":0,"cache.read":802631,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 9ff2ec7839f444005fc7df06fbad447c26f078c6 07e8b3f8b64d6421c1f257e00aeeb876f54423ca` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/complexity-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "§Budgets — \"evicts the oldest entry by insertion sequence (ties by fetched_at, so a backwards clock jump cannot make fresh data the eviction target)\" + \"The in-memory cache holds each entry's serialized octet count with a running total … O(1) plus evictions, never a whole-document re-serialization\"",
    "problem": "The r1 rewrite changed the eviction key 026-B1 names (ADR-026: \"over the cap the oldest fetched_at is evicted\"), demoting fetched_at to a tie-break. Test strategy #4 keeps the cache document \"sorted by slug byte order\", so after any restart insertion sequence IS slug order and eviction silently becomes alphabetical — entries die because their slug sorts low, not because they are old, and #5 as written pins that wrong key for T-007. The machinery also buys nothing: the same section says the file is rewritten write-through after each lookup that adds an entry, so every write already serializes the whole ≤1 MiB document; the per-entry octet totals, the running total and the insertion counter (plus the tie-break existing only to defuse clock jumps) are three bookkeeping parts traced to no stated Goal, and \"provably inside both caps\" ends up resting on a sum of entry lengths, which is not the document's length.",
    "fix": "Replace the insertion-sequence clause with 026-B1's oldest-fetched_at eviction plus a single write-time serialize whose buffer length is the byte check (exact, no per-entry totals); delete the running total, the per-entry octet count and the insertion sequence, keeping the existing load bound and backwards-jump guard for clock jumps; change #5 to pin eviction by fetched_at."
  },
  {
    "severity": "major",
    "location": "§The serving face step 1 — \"the registry's one key validator tightens it: after the first `/`, exactly one non-empty `.`-free component, no embedded `/` — the grammar that keeps a `..` or a separator from ever asking the filesystem for a path\" with \"Lookup keys are opaque in-memory strings, never joined onto paths\"",
    "problem": "A second, registry-private spelling rule for keys the repo validates in exactly one place: the shipped validator is Source::spells_lookup_key plus len ≤ MAX_DISCOVERY_SLUG_OCTETS with no character or dot rule (apps/discovery-cli/src/matches.rs:222-234, 282-283; crates/discovery/src/game.rs:84-89 admits any non-empty external id after a closed-set token), so a dotted key such as `local/Steam.SE` is legal in the user's own store and refused by this fourth grammar site at three seams (candidate query, admin upsert via §Catalog document and its spec, cache load via §Budgets). Its sole stated purpose is path safety, which the very next sentence disclaims, and its cited precedent is_slug_name governs game_id, already bound by A4 — an added rule with no consumer and a contract-divergence cost.",
    "fix": "Keep the shipped pair (spells_lookup_key + 025-B1) as the registry's whole key rule; if the path-safety defense is wanted, keep only its real scope in one clause (a component equal to `.` or `..`, or any further `/` after the first, is the 404) and drop the flat `.`-ban from step 1, from the corpus-load refusal list, and from #3/#4's dot-bearing case."
  },
  {
    "severity": "minor",
    "location": "§Budgets — \"One saturated entry is MAX_RESPONSE_OCTETS (≈64 KiB), so 026-B2 alone caps the working set at ≈16 fat entries (≈90 at realistic ~40-octet candidates) — accepted; #5 and cached_slugs read expect that, not 5,000.\"",
    "problem": "The realistic band is wrong by an order of magnitude and no per-entry figure is shown. A1 pins MAX_RESPONSE_OCTETS = 65_536 (catalog.rs:23), so 16 entries is the at-the-cap worst case only; a realistic entry (slug + {game_id,name} + fetched_at + JSON framing) is on the order of 10^2 octets, so ~10^3–10^4 entries fit inside 1 MiB and 026-B1's 5,000 is the cap that binds for ordinary answers — which is what #5 states. The sentence is load-bearing twice: it justifies accepting the ≈16-entry working set and it tells the T-007 test author what cached_slugs should reach, contradicting #5's `count ≤ 5_000` in the same list.",
    "fix": "State the assumed per-entry octet figure and name which cap binds in which band (byte cap near MAX_CANDIDATES, 026-B1 for ordinary answers), and delete the \"#5 and cached_slugs read expect that, not 5,000\" instruction."
  },
  {
    "severity": "minor",
    "location": "§Budgets — \"MAX_CONCURRENT_CONNECTIONS (over-cap closed at accept, one thread per admitted connection)\" (also §serve.rs, §Risks, #11)",
    "problem": "The only one of the five new constants carried without a value, while #11 pins behavior against it (\"over-MAX_CONCURRENT_CONNECTIONS is closed at accept\"). It is also the sole bound on live thread count and, in caching mode, on simultaneous upstream hops each of which may hold the mirrored 2 s/5 s timeouts, a product the design never states. §Budgets' stated job is that budgets close against each other, and the task author must invent the number the proposal exists to decide.",
    "fix": "Give the value in the same sentence (e.g. 64) and cite it in #11 by value, as the other four constants now are."
  },
  {
    "severity": "minor",
    "location": "§Caching mode — \"probed at most once per TTL, fired lazily by the first lookup or health read after expiry on that request's own thread\" vs §The serving face — \"Answering it reads the last-known upstream verdict … it does not perform a live upstream probe per call\"",
    "problem": "The sentence that supplied the missing refresh trigger also made a health read a probe trigger, which is precisely what the health section promises the 026-C2 route never does; the two sections now contradict each other about the same route's behavior.",
    "fix": "Drop \"or health read\" so only lookups drive the TTL-gated probe and a health read reports the last-known verdict; alternatively move the health trigger into the health paragraph, qualified as at most one probe per TTL."
  },
  {
    "severity": "minor",
    "location": "§Catalog document and its spec — \"guard the fixture-tree loops — every_valid_fixture_…, both every_invalid_fixture_… message tests, and fixture_file_names_… (which keeps governing the .json fixtures: lowercase dash-separated names, case-unique) — behind the core-TOML check so catalog fixtures are not parsed as TOML\"",
    "problem": "The bullet both guards the fixture-name test for the catalog format and says that test keeps governing catalog's .json fixtures — opposite instructions. In the shipped harness, fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique walks format_dirs() and asserts is_fixture_name(name) per file, so guarded means catalog's fixture names go uncontrolled; and is_fixture_name (crates/core/tests/common/mod.rs:421) hard-codes the .toml suffix and takes no format argument, so leaving it unguarded without the new argument fails the walk.",
    "fix": "One sentence per loop: the parse and rejection loops are guarded for the catalog format, while fixture_file_names_… is not skipped and instead calls the directory-aware is_fixture_name(dir, name) — which is what actually keeps the name and case-uniqueness rule on the .json fixtures."
  },
  {
    "severity": "minor",
    "location": "§CLI surface — \"--refresh-interval overrides 026-B3 (default REGISTRY_UPSTREAM_TTL_SECONDS = 3_600)\"",
    "problem": "The pass retired the unconsumed --local-entries knob but left this one with no consumer anywhere in the Test strategy: #6 pins fresh/stale/cold against an injected fake, #12 pins health, #17 chains a caching instance, and none shortens the TTL. The knob itself must stay (026-B3 mandates \"overridable by --refresh-interval\"), and the same interval now also re-times the TTL-gated upstream probe, which nothing observes.",
    "fix": "Add one clause to #6: start a caching instance with --refresh-interval 0 (or 1 s) and pin that a previously fresh entry is stale on the next query and triggers the refresh; note there that the override also re-times the health probe."
  },
  {
    "severity": "minor",
    "location": "§Concurrency (open Q10) — \"mutations take the write lock, mutate the model, and perform the atomic tmp-and-rename under a separate persist-only lock, so serving reads never wait on filesystem IO\"",
    "problem": "r1 added a third lock domain for a benefit no Goal or ADR unit states (the cache is ≤1 MiB and the corpus is a local file; no Goal prices a reader waiting on one local write), and the split decouples persist order from model order: two accepted mutations can reach the tmp-and-rename in the opposite order, leaving the file at the older revision while memory holds the newer, so after a restart an accepted mutation is gone. Nothing in #10 or #11 pins persist order, so either outcome ships with the tests green.",
    "fix": "Do the tmp-and-rename while holding the model write lock (the r0 wording: one lock, ordered) and delete the persist-only lock; if read latency ever earns its keep, name it as a refinement beside the copy lift in §Deferred."
  },
  {
    "severity": "nit",
    "location": "§The admin face and auth — \"Enablement is judged from the address actually bound — the shipped probe() predicate local_addr().ip().is_loopback(), not a literal-spelling comparison\"",
    "problem": "The quoted expression is not the shipped one: probe() (apps/discovery-cli/src/catalog.rs:234-248) applies is_loopback() to each resolved address, and local_addr appears nowhere in that function, so a T-006 author going to the named precedent will not find it.",
    "fix": "Reword to \"the same std::net::IpAddr::is_loopback test probe() applies to each resolved address, applied here to the serving listener's local_addr()\"."
  },
  {
    "severity": "nit",
    "location": "§apps/registry — one crate, two modes — \"and on discovery for Source and spells_lookup_key\"",
    "problem": "The new dependency edge is taken for a two-line token predicate and pulls the whole engine crate into the serving binary, including its Windows-only winreg dependency (crates/discovery/Cargo.toml:38-39) that the registry never uses; the Compatibility matrix's Windows | Yes row is silent about it. Reusing the authoritative predicate is right, so this is record-keeping.",
    "fix": "Either state in that sentence that the dep is for the predicate alone and the transitive Windows cost is accepted, or name the alternative once beside the existing lift in §Deferred (give Source/spells_lookup_key the same shared home as the copied blocks so apps/registry depends on bindforge-core only)."
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
Write your report to docs/designs/026-registry-service/reviews/design/complexity-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"§Budgets — \"`MAX_REQUEST_BODY_OCTETS` (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally hold also fits an upsert and no per-record `source_keys` bound is needed)\" with \"`MAX_CATALOG_DOCUMENT_OCTETS` (16,777,216 — ≈120,000 records …), `fs::metadata`-gated before the read\"","problem":"This round's new equivalence closes only one direction: the corpus's 16 MiB cap exists at load only (§Catalog document's refusal list, §Budgets' metadata gate, #1), while §The admin face lets accepted mutations grow that same document with no counterpart check — so a root can be driven past the size its own loader refuses and then not restart until the operator hand-edits the JSON. The cap is sized at exactly the ADR's projected scale (\"tens of thousands of games, each with several store keys\"), so growth across it is foreseeable, not exotic; this is the same reasoning §Options used to reject refuse-the-file-alone for the cache (\"it does not stop a legal-but-large corpus … from exceeding 1 MiB between writes\"), yet the corpus/cache asymmetry is recorded nowhere and #10/#11 pin revision and persist order but never size. The bound declined (a per-record `source_keys` count) is precisely what the enlarged body cap is claimed to make unnecessary.","fix":"Reuse the mechanism §Budgets just defined instead of adding one: \"the corpus persist applies the same write-buffer check — a mutation whose serialized document exceeds `MAX_CATALOG_DOCUMENT_OCTETS` is refused by name, no rename\" (the persist is already under the one write lock per §Concurrency); add that refusal to §Catalog document's list and one case to #11. If the asymmetry is instead deliberate, say so in one clause and restore the r1 per-record `source_keys` bound with the 64 KiB body cap so the loadable-corpus claim is not load-only."},{"severity":"minor","location":"§`apps/registry` modules — \"`serve.rs` — accept loop (one thread per admitted connection, covering that connection's whole request loop, up to `MAX_CONCURRENT_CONNECTIONS`, over-cap closed at accept…)\" plus §Budgets' \"one thread per admitted connection for that connection's whole life, so ≤64 live threads and ≤64 in-flight upstream hops\"","problem":"r1 said one thread per connection; r2 adds a per-connection request loop — a mechanism with no consumer, since the only client this face must satisfy sends `Connection: close` with one request per connection (apps/discovery-cli/src/catalog.rs:438, A19) and §http.rs never states whether the connection is closed after the answer. The phrase also undercuts the bound it was added to state: with a keep-alive loop an idle client holds one of the 64 admitted slots for the connection's whole life (bounded only by the read timeout), which is exactly when the \"≤64 in-flight upstream hops\" claim matters.","fix":"Replace the phrase with \"one request-response per admitted connection, then close, echoing the shipped client's `Connection: close`\" and add that close to §http.rs; if keep-alive is intended, name it as such in the same sentence together with the idle-slot timeout that bounds it."},{"severity":"nit","location":"§Test strategy #6 — \"with the fresh/stale pair run under `--refresh-interval 1s`\" (with §CLI surface's \"`--refresh-interval` overrides `026-B3` (default `REGISTRY_UPSTREAM_TTL_SECONDS = 3_600`)\")","problem":"The flag's value spelling is never defined and the case added this round depends on it: the budget is a count of seconds, so a `u64`-seconds parser rejects `1s` and the new #6 case fails on a parse error rather than on the policy it means to pin.","fix":"State the unit once at the knob (\"`--refresh-interval <seconds>`\") and write the case as `--refresh-interval 1`."},{"severity":"nit","location":"§The serving face step 1 — \"Path safety is the only extra refusal, at its own scope: after the first `/`, one non-empty component with no further `/` or `\\`, none equal to `.` or `..`, and no control byte, space, or invalid-UTF-8 sequence\"","problem":"The `space` item is the only entry in the list that is not path safety: it cannot traverse or split a path, and it cannot break the one-line refusal discipline the control-byte ban already protects, while the same sentence keeps keys as \"opaque in-memory strings, never joined onto paths\". It is also the one clause that can still 404 a key the shipped pair admits — `spells_lookup_key` (crates/discovery/src/game.rs:84-89) and `Source::lookup_key` (game.rs:66-68) impose nothing on the external id, the shipped store checks only that pair plus `025-B1` (apps/discovery-cli/src/matches.rs:229,282), and the client sends any such key escaping a space as `%20` — with `local/<dir>`, the ADR's own example of what the admin face exists to curate, the natural home of a space-bearing id. Nothing shipped emits one today, hence nit.","fix":"Drop \"space\" from the refusal list, or keep it and name in one clause the `local/` key-derivation law it enforces."}]}
```
