# Transcript: design-review:performance r1c5

## Provenance
- stage: design-review:performance (round r1c5)
- agent: review/design-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:45:58.538Z
- ended: 2026-11-19T19:49:39.621Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":49402,"output":19223,"reasoning":0,"cache.read":470871,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the ratification pass c5: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 9ff2ec7839f444005fc7df06fbad447c26f078c6 e1e63cb6acd1d9d855829f18df4d83607ae48aa1` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/performance-r1c5.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
[
  {
    "severity": "minor",
    "location": "Chosen design → The admin face and auth, \"Concurrency (open Q10)\" (\"mutations take the write lock, mutate the model, and perform the atomic tmp-and-rename under a separate persist-only lock, so serving reads never wait on filesystem IO\")",
    "problem": "re-raised: the r0 stall is only removed if the model write lock is released before the persist, and the sentence does not say so — std::sync::RwLock blocks new shared readers while a writer holds it, so a mutation that keeps the write lock across the tmp-write/rename reproduces the r0 stall with a second lock added. Three further consequences of the same under-specification: (a) if the persist serializes a snapshot taken at mutation time rather than the model's current state under the persist lock, two mutations can complete their persists out of order and the file can end at an older revision than one already served; (b) the paragraph names Arc<RwLock<Cache>> only in passing, yet in caching mode the ordinary request path is a model mutation plus a write-through rewrite of up to 026-B2 octets, so whether that rewrite runs under the Cache write lock, and whether \"a separate persist-only lock\" is one lock shared with the corpus persist or one per document, decides whether a cold lookup's… [elided by the checkpoint cap]",
    "fix": "One sentence fixing the protocol: take the model write lock, mutate the model and update that game's postings at their sorted position (no rebuild), release it, then under a per-document persist lock serialize the model's current state and rename; and state that the caching-mode cache write-through follows the same order with its own persist lock, so \"serving reads never wait on filesystem IO\" holds on the hot path, not only for admin mutations."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets (request-budgets paragraph), with §apps/registry serve.rs and the Risks row \"Low-and-slow / one-inbound-per-outbound exposure on a routable bind\" (\"MAX_CONCURRENT_CONNECTIONS (over-cap closed at accept, one thread per admitted connection)\")",
    "problem": "re-raised: this is the only budget in the constants.rs list with no value — the two request budgets carry 65,536, the corpus and cache carry 1,048,576, the TTL carries 3,600. Under thread-per-connection with no runtime or pool, that number is the peak-thread and peak-stack-reservation bound for the flagship 026-C5 routable public root, so the growth bound r0 asked for is nominal: Test-strategy #11 cannot pin an assertion to a value, T-005 will choose one arbitrarily, and an operator sizing a public root has no capacity expectation.",
    "fix": "Give the constant a default in the same parenthetical (a stated value, optionally overridable by --max-connections) and name that default in #11's assertion."
  },
  {
    "severity": "minor",
    "location": "Caching mode → \"Upstream reachability\" (\"probed at most once per TTL, fired lazily by the first lookup or health read after expiry on that request's own thread — there is no timer thread … A query pays no health round trip beyond that one TTL-gated probe\")",
    "problem": "new: naming the lazy driver closes r0's question but relies on an amortization bound nothing enforces — with one thread per connection, several threads can pass the expiry test before the verdict stamp is written, so \"at most once per TTL\" is a claim about intent, not a property, and each extra probe costs the mirrored 2 s connect / 5 s read on a serving thread. Compounding it, --refresh-interval \"overrides 026-B3\" with no stated floor, so a small operator-set TTL degrades the bound toward a synchronous upstream probe per request on both /v1/candidates and /v1/health — the per-request synchronous IO this revision exists to avoid.",
    "fix": "One clause making the probe single-flight (the thread that compare-and-sets the last_upstream_fetch stamp under the Cache write lock is the only one that probes for that TTL; the rest serve the previous verdict) and one clause stating the floor for --refresh-interval (at least the upstream read timeout)."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Budgets (\"after a write the registry evicts the oldest entry by insertion sequence (ties by fetched_at, …)\")",
    "problem": "new: the switch from fetched_at order to insertion order fixes the clock-jump hazard, but the document never says whether a warm hit (Caching mode step 2, \"Cached entry within TTL answers as-is\") moves the entry to the newest sequence. If it does not, eviction is FIFO, and with the design's own accepted working set (≈16 fat / ≈90 realistic entries, or a few thousand thin ones under 026-B1) a burst of cold slugs evicts the hot ones and the next query pays another serial upstream round trip.",
    "fix": "One clause naming the policy — either \"a warm hit re-stamps the entry's insertion sequence (LRU)\" or \"eviction is FIFO by insertion sequence; a hot slug may be evicted by cold churn\" — so T-007's eviction test pins the right behavior."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Budgets (\"026-B2 is re-checked at load\" read against \"MAX_CATALOG_DOCUMENT_OCTETS (1,048,576), fs::metadata-gated before the read of the --corpus or local-entries file\")",
    "problem": "new: the revision applies the pre-allocation gate to the two catalog documents but not to the third load path, the cache file: as written an oversized cache-v1.json is read and parsed before 026-B2 refuses it. Both precedents the design invokes gate before allocating (apps/discovery-cli/src/matches.rs:194; crates/core/src/toml.rs:51-58, \"refused without being allocated\"), and the ADR's 026-B2 itself says \"refused by name like the match store's\".",
    "fix": "Extend that parenthetical to \"… or local-entries file, and the cache read is gated at 026-B2 the same way\", with the refusal noted in Test-strategy #5."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Budgets (\"MAX_REQUEST_BODY_OCTETS (65,536, sized to one 026-C3 record inside the document cap)\")",
    "problem": "new: the \"sized to\" justification does not close — 026-C3 bounds each source_key by 025-B1 and name by 013-B4 but nothing bounds the number of source_keys on one record (no shipped or proposed count cap exists), so a record with a few hundred keys is legal at load under the 1 MiB document cap while being refuseable as a POST body, leaving a hand-edited local-entries record the server loaded but its own admin face can never upsert.",
    "fix": "Bound source_keys per record with a constants.rs count cap chosen so one record fits inside MAX_REQUEST_BODY_OCTETS, refused by name in the corpus.rs refusal list and Test-strategy #1; or re-word the parenthetical so the body cap is stated as the per-record cap that load also enforces."
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
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/026-registry-service/reviews/design/performance-r1c5.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "Chosen design → Budgets (request-budgets paragraph) — \"`MAX_REQUEST_BODY_OCTETS` (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally hold also fits an upsert and no per-record `source_keys` bound is needed)\"", "problem": "new/regression from this revision: the value reviewed in r1 was 65,536; the revision raises the admitted request body 256× to 16,777,216, and the paragraph's capacity accounting for MAX_CONCURRENT_CONNECTIONS names only \"≤64 live threads and ≤64 in-flight upstream hops\", never admitted octets. The reader this design mirrors allocates the whole promised body once it clears the cap (apps/discovery-cli/src/catalog.rs:364 `vec![0u8; length]`), and the design never orders the admin auth check ahead of the body read (\"an over-cap body is refused before any mutation\" bounds mutation, not reading), so peak inbound reservation for the flagship 026-C5 routable root is 64 × 16 MiB ≈ 1 GiB plus the parsed value, for records that need kilobytes. The stated justification buys only the r1 round-trip consistency (a loaded record the admin face could never re-upsert); it does not price what it costs.", "fix": "Either keep the equality and state what it implies — the admitted-octets bound 64 × `MAX_CATALOG_DOCUMENT_OCTETS`, and that the auth check precedes the body read — or re-split the constants with a named per-record `source_keys` count cap chosen so one record fits a small `MAX_REQUEST_BODY_OCTETS` (65,536), refused by name in the `corpus.rs` refusal list and Test-strategy #1 (the alternative r1 offered)."}, {"severity": "minor", "location": "Chosen design → The admin face and auth, \"Concurrency (open Q10)\" — \"a mutation serializes the new model, writes the fixed tmp name, renames it into place, and swaps the model in, **all under the one write lock** … A reader may wait on one local write, and that latency is accepted\" — read against §Budgets \"`MAX_CATALOG_DOCUMENT_OCTETS` (16,777,216 — ≈120,000 records …)\"", "problem": "regression in scale: the r0 stall is now accepted rather than removed (the author's call, and the false \"reads never wait on filesystem IO\" claim is gone), but the same revision raised the document cap 16× from the 1 MiB this seat approved against, so \"one local write\" is now a whole-document serialize plus a write of up to 16 MiB held in the Corpus write lock, during which std::sync::RwLock admits no new shared reader — up to 63 of the 64 admitted connections queue behind one operator upsert on the root sized for ≈120,000 records. Neither §Concurrency nor the Risks low-and-slow row bounds the accepted wait, and #11's new \"N concurrent upserts leave the persisted document equal to the model\" pins ordering, not the reader's worst-case wait.", "fix": "One clause naming the accepted wait as a number — \"the write lock is held for one serialize plus one local write of at most `MAX_CATALOG_DOCUMENT_OCTETS` octets; admin mutations are operator-rare on the root\" — mirrored in the Risks row; or write the tmp outside the model write lock from the buffer prepared under it and take the lock only for the rename and model swap, keeping the ordering guarantee #11 tests."}, {"severity": "minor", "location": "Chosen design → Budgets (cache eviction paragraph) — \"after a write the registry serializes the cache and re-serializes after each dropped entry — one extra pass per eviction at most — dropping the oldest-`fetched_at` entries … until the buffer is ≤ `026-B2` octets\"", "problem": "new in this revision: deleting the running totals moves the whole octet test onto the serialized buffer, and the parenthetical bounds passes-per-eviction but never bounds the number of evictions, which the loop condition leaves open. Reachable inside the design's own numbers: 026-B1's 5,000 entries at the paragraph's ~150–210-octet realistic size sit just under 1 MiB, so one saturated MAX_CANDIDATES answer (≈MAX_RESPONSE_OCTETS = 65,536, the figure this paragraph carried before the revision) forces dropping a few hundred thin entries, each drop triggering another full ~1 MiB re-serialization — O(evictions × document), hundreds of MiB serialized inside one cold-lookup request path. The text this diff replaces guaranteed the opposite (\"O(1) plus evictions, never a whole-document re-serialization\"), and #5 asserts only the end state, so nothing pins the cost.", "fix": "Keep the exact-length test but stop re-serializing per drop: subtract each dropped entry's own serialized length (from the pass just taken) from a buffer total, drop until that total clears 026-B2, then serialize once and check the exact length, looping only if that check still fails; name the lock the write-through runs under in the same sentence."}, {"severity": "minor", "location": "Caching mode → \"Upstream reachability\" — \"probed **at most once per TTL**, fired lazily by the first *lookup* after expiry on that request's own thread\"", "problem": "re-raised, not a regression: dropping health reads as a driver narrows the window but leaves the hot case — concurrent cold lookups after expiry. The design still names no stamp or compare-and-set, so with one thread per admitted connection several of the 64 threads can pass the expiry test before a verdict stamp exists, making \"at most once per TTL\" intent rather than a property; each extra probe costs the mirrored 2 s connect / 5 s read on a serving thread, the very cost the Risks row \"A cold caching registry's first scan is N serial upstream lookups\" claims to mitigate. Compounded by `--refresh-interval <seconds>`, now specified as \"a bare unsigned count of seconds\" with no floor and pinned live at `1` by #6, far below those timeouts, degrading the bound toward a synchronous upstream probe per lookup.", "fix": "One clause making the probe single-flight (the thread that compare-and-sets the `last_upstream_fetch` stamp is the only one that probes for that TTL; the rest serve the previous verdict) plus one clause stating the floor for `--refresh-interval` (at least the upstream read timeout)."}, {"severity": "nit", "location": "Chosen design → Budgets — \"`fs::metadata`-gated before the read of the `--corpus` or local-entries file and refused by name (A22)\"", "problem": "re-raised: the third load path is still outside the gate — the cache's 026-B2 is only \"re-checked at load\", so as written an oversized `cache-v1.json` is read and parsed before refusal, while the ADR's 026-B2 says the cache file is \"refused by name like the match store's\", and both precedents the design invokes gate before allocating (matches.rs `load`/`STORE_READ_BUDGET_OCTETS`, A22; catalog.rs:353–364 checks the promise before `vec![0u8; length]`).", "fix": "Extend the parenthetical to \"… or local-entries file, and the cache read is `026-B2`-gated at `fs::metadata` the same way\", and name that refusal in Test-strategy #5."}]}
```
