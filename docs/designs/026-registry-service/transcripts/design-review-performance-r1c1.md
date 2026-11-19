# Transcript: design-review:performance r1c1

## Provenance
- stage: design-review:performance (round r1c1)
- agent: review/design-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:06:03.070Z
- ended: 2026-11-19T19:10:37.297Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":48914,"output":17749,"reasoning":0,"cache.read":350142,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 4a75e3d76953a458683e6db8c0d42e27351f4acf 9ff2ec7839f444005fc7df06fbad447c26f078c6` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/performance-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
[
  {
    "severity": "major",
    "location": "Caching mode, lookup step 4 (\"Miss with reachable upstream fetches `GET <upstream>/v1/candidates?slug=`, validates the answer, and caches it with its `fetched_at`\") together with Chosen design → Budgets (\"after any cache write, the registry evicts the oldest `fetched_at` until **both** `count ≤ 026-B1` **and** `serialized bytes ≤ 026-B2` hold — so the file is provably inside both caps after every mutation\")",
    "problem": "The per-lookup cost on the request path is unbudgeted and unbounded in work. Each cold lookup must (a) serialize the whole cache document to test the byte invariant, (b) re-sort entries by `slug` byte order for the next write per `026-C7`, (c) find the oldest `fetched_at` among up to 5,000 entries, and (d) re-persist the entire document through the `026-C7`/A7 tmp-write + rename + re-chmod discipline — up to 1,048,576 octets rewritten per slug, synchronously, in the handler. The shipped client's scan is a serial per-slug loop (apps/discovery-cli/src/main.rs:191-192, `for slug in &unresolved { catalog.candidates(slug) … }`) and `026-R3` keeps prefetch deferred, so a first scan of N games pays N × (serialize + sort + evict scan + full-file rewrite) of a document growing toward `026-B2`: O(N × cache bytes) on the user-visible critical path on top of the N serial upstream round trips. No request-latency budget appears anywhere in the document; the Risks row for the cold-scan cliff mitigate… [elided by the checkpoint cap]",
    "fix": "Keep the dual invariant but make it incremental: hold each entry's serialized octet count in the in-memory cache model with a running total, so the post-write check is O(1) plus evictions rather than a whole-document re-serialization, replacing the currently implied per-mutation serialization; and state in one sentence when the cache file is rewritten (write-through per lookup, or coalesced at end-of-scan/idle plus clean shutdown with the crash-loss window named). Add the matching assertion to Test-strategy #5."
  },
  {
    "severity": "major",
    "location": "Chosen design → Catalog document and its spec (\"The registry's loader (`corpus.rs`) refuses by name: `version != 1`, `revision` negative, a `game_id` failing the grammar+bound (A4), a `name` over 128 octets (A5), a `source_key` failing `spells_lookup_key` or `025-B1` (A3, A16), a non-object/missing field\") and §Budgets",
    "problem": "The `026-C3` catalog document is the only document this design loads with no octet ceiling and no entry-count cap, while every other load is budgeted (cache at `026-B2` = 1,048,576; the newly elaborated `MAX_REQUEST_HEADER_OCTETS`/`MAX_REQUEST_BODY_OCTETS`). All listed refusals are field-level. The A7 load discipline the section cites as its model includes a size gate: matches.rs refuses \"past the 65536-octet read budget\" on `fs::metadata` before allocating (apps/discovery-cli/src/matches.rs:36,194) and core refuses over `MAX_DOCUMENT_BYTES` \"without being allocated\" (crates/core/src/toml.rs:56). So the root's boot read, its retained in-memory model, and the slug→candidates index — the design's main translation table — are unbounded against the ADR's own sizing (\"tens of thousands of games, each with several store keys\"), and in caching mode the admin face can grow the local-entries document without limit.",
    "fix": "Add one private constant in `constants.rs` (`MAX_CATALOG_DOCUMENT_OCTETS`, following the `STORE_READ_BUDGET_OCTETS` precedent the design already invokes: an `fs::metadata` gate before the read), refused by name at load for both the `--corpus` file and the local-entries document; list it beside the other private budgets in §Budgets and add a refusal case to Test-strategy #1. No new ADR unit is needed, since §Budgets already justifies private request-side budgets as `026-C2`/`026-C4` elaboration."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets (\"The research's \\\"budgets do not close\\\" finding (5,000 entries × up to 256 candidates can exceed 1 MiB) is resolved by making both caps **write-time** invariants\")",
    "problem": "The resolution makes `026-B1`'s 5,000 nominal but states no resulting effective capacity, which the numbers put far lower. Each entry's candidates arrive in a response capped at `MAX_RESPONSE_OCTETS = 65_536` with ≤ 256 `{game_id, name}` pairs (game_id ≤ 128, name ≤ 128: crates/core/src/placement.rs:83, crates/core/src/manifest.rs:58), so one saturated entry is ≈64 KiB and `026-B2` alone caps the cache at ≈16 slugs; at a realistic ~40-octet candidate it is ≈90. With oldest-first eviction, a few fat entries evict many thin ones, so the working set may be tens of slugs rather than 5,000 — deepening the cold-scan cliff the design itself flags (`026-R3`). The r0 revision entry made the caps jointly satisfiable, which is sound, but supplies no capacity number for T-007's eviction test or for reading `health.cached_slugs`.",
    "fix": "One sentence in §Budgets giving the per-entry worst case derived from `MAX_RESPONSE_OCTETS` and the resulting effective slug capacity, stated as accepted, so the eviction unit test and operator-facing counts are pinned to the right expectation."
  },
  {
    "severity": "minor",
    "location": "Chosen design → The admin face and auth, \"Concurrency (open Q10)\" (\"serving reads take the lock on the **in-memory** model (never the file)\" / \"mutations take the same lock, mutate the model, then write the atomic tmp-and-rename\")",
    "problem": "As written every `GET /v1/candidates` takes an exclusive `Mutex<Corpus>`, so candidate queries serialize against each other even with no writer, and the tmp-write + rename + re-chmod — synchronous filesystem IO on a document the ADR sizes in the tens of thousands of entries — happens inside that same lock, so one admin POST/DELETE stalls the whole serving face for the duration of the disk write. This bears on the design's flagship deployment (a public root on a routable bind per `026-C5`) with thread-per-connection serving. The mutex is only load-bearing for the two-writer refusal, which is already enforced by the fixed-name `create_new(true)` tmp (A7).",
    "fix": "Name the primitive and the persist's placement in one sentence: `Arc<RwLock<Corpus>>` with reads on the shared lock, and the tmp-and-rename either stated as held under the write lock with a stated cost expectation, or performed under a separate persist lock so the serving path never waits on the filesystem."
  },
  {
    "severity": "minor",
    "location": "Risks row \"Low-and-slow / one-inbound-per-outbound exposure on a routable bind\" (\"Thread-per-connection with a request-read timeout; `MAX_REQUEST_HEADER_OCTETS`/`MAX_REQUEST_BODY_OCTETS`; oldest-first cache eviction bounds the entry growth (`026-B1`)\")",
    "problem": "The mitigations bound bytes and time per connection but nothing bounds the number of connections, and the model allocates one OS thread per connection under the `std`-only posture with no runtime to absorb it. A peer holding many idle keep-alive sockets on a non-loopback bind drives unbounded thread/stack growth. The cited \"entry growth\" bound covers cache entries (`026-B1`), not connections, so the row claims coverage it does not provide.",
    "fix": "Add one more private constant to the §Budgets `constants.rs` list (`MAX_CONCURRENT_CONNECTIONS`, excess refused at accept, or a stated accept-backlog posture) and replace the cache-eviction clause in that mitigation with it; pin it in Test-strategy #11."
  },
  {
    "severity": "minor",
    "location": "Caching mode, \"Upstream reachability\" (\"that verdict is **cached and refreshed on the same TTL**\") read against Serving face (\"it does not perform a live upstream probe per call\")",
    "problem": "The design never names what performs the TTL refresh. Lookup steps 1-2 (local entry, warm cache) make zero upstream requests, so `health.upstream` can go arbitrarily long without an upstream interaction, and the module plan (`serve.rs` accept loop/router/mode selection, `upstream.rs` client) names no timer or refresher. If the refresh is lazy inside `GET /v1/health`, the route the design says never probes does issue one at every TTL boundary, and a monitoring poller pays the mirrored 2 s connect / 5 s read timeout inside a health check — with `--refresh-interval` free to make that boundary frequent.",
    "fix": "One sentence naming the driver: a background refresher on the TTL interval owned by `serve.rs` (which also makes \"last-known\" literally true), or an explicit statement that the health route may pay at most one bounded upstream probe per TTL, with that probe's timeout pinned."
  },
  {
    "severity": "minor",
    "location": "Chosen design → The serving face, join step 2 (\"Answer = the join: games whose `source_keys` contain the slug\")",
    "problem": "This is the design's only statement of the hot-path algorithm and it describes a linear scan over every game and its `source_keys` per request. The structure that avoids it appears only in the module list (`corpus.rs` — \"load + validate + the slug→candidates index\") and is never connected to the route or to the stable-ordering requirement, so as written a request scans and sorts before \"Ordered by `game_id` byte order; truncated to the first `MAX_CANDIDATES`\".",
    "fix": "State in step 2 that the slug→candidates index maps the decoded slug to postings already sorted by `game_id` byte order at load, so a request is a probe plus a clone of at most `MAX_CANDIDATES` projected records, and that sort and truncation cost is paid at load, not per request."
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
Write your report to docs/designs/026-registry-service/reviews/design/performance-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → The admin face and auth, \"Concurrency (open Q10)\" (\"mutations take the write lock, mutate the model, and perform the atomic tmp-and-rename under a separate persist-only lock, so serving reads never wait on filesystem IO\")","problem":"re-raised: the r0 stall is only removed if the model write lock is released before the persist, and the sentence does not say so — std::sync::RwLock blocks new shared readers while a writer holds it, so a mutation that keeps the write lock across the tmp-write/rename reproduces the r0 stall with a second lock added. Three further consequences of the same under-specification: (a) if the persist serializes a snapshot taken at mutation time rather than the model's current state under the persist lock, two mutations can complete their persists out of order and the file can end at an older revision than one already served; (b) the paragraph names Arc<RwLock<Cache>> only in passing, yet in caching mode the ordinary request path is a model mutation plus a write-through rewrite of up to 026-B2 octets, so whether that rewrite runs under the Cache write lock, and whether \"a separate persist-only lock\" is one lock shared with the corpus persist or one per document, decides whether a cold lookup's disk IO stalls warm candidate reads or an admin POST's persist stalls every cache write; (c) §The serving face step 2 now puts the slug→candidates postings in corpus.rs and nothing says whether a mutation updates that game's postings in place or rebuilds the index under the write lock.","fix":"One sentence fixing the protocol: take the model write lock, mutate the model and update that game's postings at their sorted position (no rebuild), release it, then under a per-document persist lock serialize the model's current state and rename; and state that the caching-mode cache write-through follows the same order with its own persist lock, so \"serving reads never wait on filesystem IO\" holds on the hot path, not only for admin mutations."},{"severity":"minor","location":"Chosen design → Budgets (request-budgets paragraph), with §apps/registry serve.rs and the Risks row \"Low-and-slow / one-inbound-per-outbound exposure on a routable bind\" (\"MAX_CONCURRENT_CONNECTIONS (over-cap closed at accept, one thread per admitted connection)\")","problem":"re-raised: this is the only budget in the constants.rs list with no value — the two request budgets carry 65,536, the corpus and cache carry 1,048,576, the TTL carries 3,600. Under thread-per-connection with no runtime or pool, that number is the peak-thread and peak-stack-reservation bound for the flagship 026-C5 routable public root, so the growth bound r0 asked for is nominal: Test-strategy #11 cannot pin an assertion to a value, T-005 will choose one arbitrarily, and an operator sizing a public root has no capacity expectation.","fix":"Give the constant a default in the same parenthetical (a stated value, optionally overridable by --max-connections) and name that default in #11's assertion."},{"severity":"minor","location":"Caching mode → \"Upstream reachability\" (\"probed at most once per TTL, fired lazily by the first lookup or health read after expiry on that request's own thread — there is no timer thread … A query pays no health round trip beyond that one TTL-gated probe\")","problem":"new: naming the lazy driver closes r0's question but relies on an amortization bound nothing enforces — with one thread per connection, several threads can pass the expiry test before the verdict stamp is written, so \"at most once per TTL\" is a claim about intent, not a property, and each extra probe costs the mirrored 2 s connect / 5 s read on a serving thread. Compounding it, --refresh-interval \"overrides 026-B3\" with no stated floor, so a small operator-set TTL degrades the bound toward a synchronous upstream probe per request on both /v1/candidates and /v1/health — the per-request synchronous IO this revision exists to avoid.","fix":"One clause making the probe single-flight (the thread that compare-and-sets the last_upstream_fetch stamp under the Cache write lock is the only one that probes for that TTL; the rest serve the previous verdict) and one clause stating the floor for --refresh-interval (at least the upstream read timeout)."},{"severity":"nit","location":"Chosen design → Budgets (\"after a write the registry evicts the oldest entry by insertion sequence (ties by fetched_at, …)\")","problem":"new: the switch from fetched_at order to insertion order fixes the clock-jump hazard, but the document never says whether a warm hit (Caching mode step 2, \"Cached entry within TTL answers as-is\") moves the entry to the newest sequence. If it does not, eviction is FIFO, and with the design's own accepted working set (≈16 fat / ≈90 realistic entries, or a few thousand thin ones under 026-B1) a burst of cold slugs evicts the hot ones and the next query pays another serial upstream round trip.","fix":"One clause naming the policy — either \"a warm hit re-stamps the entry's insertion sequence (LRU)\" or \"eviction is FIFO by insertion sequence; a hot slug may be evicted by cold churn\" — so T-007's eviction test pins the right behavior."},{"severity":"nit","location":"Chosen design → Budgets (\"026-B2 is re-checked at load\" read against \"MAX_CATALOG_DOCUMENT_OCTETS (1,048,576), fs::metadata-gated before the read of the --corpus or local-entries file\")","problem":"new: the revision applies the pre-allocation gate to the two catalog documents but not to the third load path, the cache file: as written an oversized cache-v1.json is read and parsed before 026-B2 refuses it. Both precedents the design invokes gate before allocating (apps/discovery-cli/src/matches.rs:194; crates/core/src/toml.rs:51-58, \"refused without being allocated\"), and the ADR's 026-B2 itself says \"refused by name like the match store's\".","fix":"Extend that parenthetical to \"… or local-entries file, and the cache read is gated at 026-B2 the same way\", with the refusal noted in Test-strategy #5."},{"severity":"nit","location":"Chosen design → Budgets (\"MAX_REQUEST_BODY_OCTETS (65,536, sized to one 026-C3 record inside the document cap)\")","problem":"new: the \"sized to\" justification does not close — 026-C3 bounds each source_key by 025-B1 and name by 013-B4 but nothing bounds the number of source_keys on one record (no shipped or proposed count cap exists), so a record with a few hundred keys is legal at load under the 1 MiB document cap while being refuseable as a POST body, leaving a hand-edited local-entries record the server loaded but its own admin face can never upsert.","fix":"Bound source_keys per record with a constants.rs count cap chosen so one record fits inside MAX_REQUEST_BODY_OCTETS, refused by name in the corpus.rs refusal list and Test-strategy #1; or re-word the parenthetical so the body cap is stated as the per-record cap that load also enforces."}]}
```
