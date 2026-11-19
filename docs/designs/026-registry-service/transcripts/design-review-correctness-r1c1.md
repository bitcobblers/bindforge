# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T18:56:42.137Z
- ended: 2026-11-19T19:06:37.322Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":67591,"output":40820,"reasoning":0,"cache.read":2210888,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/correctness-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → The serving face — \"`upstream` is `none` (root) or `ok`/`stale`/`down` (caching)\"",
    "problem": "026-C2 closes the health body's `upstream` value to four tokens, and the design makes the verdict a cached gauge that health never probes live (\"the verdict is cached and refreshed on the same TTL\"), so a caching instance that has not yet contacted its upstream has no legal value to answer: `none` is claimed by root mode, `ok`/`stale` assert a fetch that never happened, and `down` asserts an unreachable upstream never tried. `last_upstream_fetch` also has no defined value before the first fetch, while the design's own `fetched_at` rules refuse non-integers. Test strategy #12 cannot pin the startup body and T-005/T-007 will decide differently; the never-checked state is created by this design's no-live-probe rule.",
    "fix": "Name the value for the never-contacted state in the serving-face paragraph (e.g. `upstream: down` with `last_upstream_fetch: 0`) and extend test #12 to pin a freshly started caching instance."
  },
  {
    "severity": "major",
    "location": "Chosen design → The admin face and auth — \"`DELETE /v1/admin/games/<game_id>` percent-decodes the path segment, validates the grammar (A4), removes the record.\"",
    "problem": "026-C4 requires that \"every accepted mutation bumps `revision` and persists atomically through the tmp-and-rename discipline\", but the design states the bump only for POST and test strategy #10 pins it only for upsert (\"upsert validates + bumps `revision` + atomic rename; delete decodes+validates the id\"). The Concurrency paragraph's generic \"mutations … then write the atomic tmp-and-rename\" covers DELETE's persist but not its bump. T-006 can ship a DELETE that leaves `revision` unchanged with every pinned test green, and the Goals promise that mutations are \"stated to the level a test engineer can pin\" is then false.",
    "fix": "Add \"bumps `revision` and persists through the same tmp-and-rename\" to the DELETE bullet, and \"delete bumps `revision` and persists\" to test #10."
  },
  {
    "severity": "major",
    "location": "Chosen design → Catalog document and its spec — \"The registry's loader (`corpus.rs`) refuses by name: `version != 1`, `revision` negative, a `game_id` failing the grammar+bound (A4) …\"",
    "problem": "The refusal list is presented as complete and never makes a duplicate `game_id` a refusal, although POST /v1/admin/games is an upsert keyed by game_id and DELETE names one record: a corpus with two records sharing a game_id loads, makes delete ambiguous, makes upsert's whole-record replacement non-deterministic about which row survives the next atomic write, and makes stats' `games` count disagree between loaded model and saved file. The same hole exists in the cache document: 026-C7 sorts entries by slug byte order (one entry per slug implied) while the named cache refusals (test #4: \"bad version/shape/`game_id`/`name`\") never name a duplicate slug, so a hand-edited cache with two entries for one slug has an undefined TTL winner.",
    "fix": "State that corpus.rs refuses a repeated `game_id` (and a repeated `source_key` within one record) and cache.rs refuses a repeated `slug`, by name; add both to the named refusal lists in tests #1 and #4 and to `schemas/catalog/v1/fixtures/invalid/`."
  },
  {
    "severity": "major",
    "location": "Chosen design → CLI amendment — \"The two pinned suites (unit + subprocess, A11) re-pin: remote hosts are now admitted\" vs Test strategy #15 \"**no test dials non-loopback**\"",
    "problem": "The subprocess suite being re-pinned is endpoint_policy_refuses_everything_beyond_the_loopback_spelling (cli_catalog_gate_tests.rs:112), whose remote cases are http://catalog.example and http://catalog.example:8080/v1 and which asserts today that each fails before enumeration with `catalog transport unavailable`. Once parse_endpoint admits any host those inputs are admitted, so the same subprocess run proceeds into probe() — a DNS lookup of a public domain plus a connect_timeout — producing network-dependent `catalog unreachable` output and violating both #15 and the suite's own invariant (tests/common/mod.rs:12). probe() has no resolver or connector seam, and the amendment names none, so the amendment is either unpinnable or network-dialing.",
    "fix": "Pin the split: remote-host admission pinned at parse_endpoint (pure, unit suite; the 026-L2 disclosure line moves there), the subprocess suite keeps only still-refused spellings plus the re-pinned refusal text, and probe()'s admission is pinned only against the in-process loopback fake — or sanction a 192.0.2.1 literal explicitly and correct #15."
  },
  {
    "severity": "major",
    "location": "Chosen design → Budgets — \"The three registry budgets are **private to `apps/registry`** (`constants.rs`)\"",
    "problem": "Every budget the design names bounds the cache (026-B1/026-B2) or inbound requests; nothing bounds the catalog document — the one document the ADR expects to grow to \"tens of thousands of games, each with several store keys\". 026-C3 makes the catalog loader's refusals \"the schema's, by name, as every contract document's are (`schemas/README.md` discipline)\", and that discipline is a size cap that \"binds *before* the read\" with `schemas/README.md` as \"this bound's one stated home\" (restated for disk-arriving contract documents in data/manifests/README.md:14-18). corpus.rs therefore reads an arbitrarily large --corpus or local-entries file with no named refusal, no `games` count cap and no cap on `source_keys` per record, and test #1's refusal list has no over-cap case where the cache's does.",
    "fix": "Add one private `MAX_CATALOG_DOCUMENT_OCTETS` (1_048_576, mirroring 026-B2) checked on file length before the read in corpus.rs, name it in constants.rs and in test #1's refusal list, and state it in schemas/catalog/v1/spec.md as that format's bound."
  },
  {
    "severity": "major",
    "location": "Chosen design → `apps/registry` — one crate, two modes — \"It depends on `bindforge-core` for the grammar and name bounds (`is_slug_name`, `MAX_SLUG_OCTETS`, `MAX_NATIVE_NAME_OCTETS`, `escape_and_truncate`)\"",
    "problem": "The predicate the design leans on hardest is not in core: spells_lookup_key and the closed source-token set are Source's in crates/discovery (A16's own basis; apps/discovery-cli/src/matches.rs:229,282 reaches them via its discovery dependency). The dependency set given (std + serde_json + bindforge-core) cannot serve \"a `source_key` failing `spells_lookup_key`\" in corpus.rs or \"`spells_lookup_key` to it\" at /v1/candidates, so T-004/T-005 must either add a dependency the design never lists — pulling the store scanners into the server binary — or re-spell {steam, epic, local} locally, giving the closed set two definitions.",
    "fix": "Decide it in the module plan: either \"apps/registry depends on `discovery` for `Source`, as discovery-cli already does\" in the crate's dependency sentence, or \"the closed source-token set and spells_lookup_key are lifted into bindforge-core and Source re-exports them\", naming the shipped text each option replaces."
  },
  {
    "severity": "minor",
    "location": "Chosen design → The serving face — \"The `026-L4` cold case is the named `503` with a legible reason phrase\"",
    "problem": "That sentence sits in the paragraph both modes share (\"The two modes share every handler\") and the design never states the root's answer for a well-formed, spellable slug that joins to nothing. 026-L4 confines the 503 to a caching registry with nothing to answer from, and its own reasoning makes [] the true statement for a root that has never heard of the key, so 200 [] is correct — but T-005 reading this section alone can reach 503 or 404, and neither test #2 nor #16 pins it.",
    "fix": "Add to serving-face step 2: \"an empty join in root mode is a `200` with `[]`; the `503` is reachable only from caching mode\", and one case in #16."
  },
  {
    "severity": "minor",
    "location": "Chosen design → The admin face and auth — \"`GET /v1/admin/stats` answers `{games, source_keys, revision}` from the instance's own catalog document\"",
    "problem": "The field types are never stated, and 026-L3's no-bulk rule turns on them: an implementer who answers `source_keys` as the list of keys answers with the catalog's whole key set over an admin route — what 026-L3 forbids and what the rejected GET /v1/admin/catalog was rejected for. Test #18 pins only that /v1/admin/catalog is 404.",
    "fix": "State that `games` and `source_keys` are integer counts (with `revision`) and pin the three-field count shape in test #10."
  },
  {
    "severity": "minor",
    "location": "Chosen design → The admin face and auth — \"Enablement is judged from the bind address literal and the presence of a configured token.\"",
    "problem": "The design never says what makes a literal loopback. --host takes the same spelling space the endpoint policy admits (parse_endpoint normalizes hostnames and the amendment widens hosts further), and the shipped precedent judges loopbackness twice and differently: the LOOPBACK_HOSTS literal set for configuration, and addr.ip().is_loopback() on the resolved address in probe(). Under a literal-only rule, `--host localhost` with no token is a coin flip that decides whether the admin face exists — the ADR's own no-token-friction use case.",
    "fix": "Judge enablement from the address actually bound (local_addr()'s ip().is_loopback(), the shipped probe() predicate), or name the closed loopback-literal set beside the quoted sentence."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets — \"the registry evicts the oldest `fetched_at` until **both** … hold\"",
    "problem": "Eviction order is the raw wall-clock stamp, while the same paragraph puts TTL on the wall clock \"with a guard that treats a backwards-jumped stamp as expired rather than immortal\". After a backwards clock jump the entries just fetched are the \"oldest\", so the dual-cap eviction discards the freshest data first and repeatedly, and re-fetched answers are evicted again while stale entries remain serviceable.",
    "fix": "Order eviction by a monotonic insert sequence held beside `fetched_at` (insertion order, ties broken by `fetched_at`) and say so in test #5 instead of \"oldest `fetched_at` evicted first\"."
  },
  {
    "severity": "minor",
    "location": "Risks — \"Two `registry serve` on one corpus fail loudly\" row — \"State it as a refused, unsupported configuration in `spec.md` and the CLI's startup check.\"",
    "problem": "The mechanism relied on is the fixed-name create_new(true) tmp, which fails only at save time: a second instance starts, loads the corpus and serves reads indefinitely, and a tmp left by a crash is indistinguishable from a live writer's. No module in the nine-file plan owns an instance marker, so the promised startup check has no mechanism and no test (#11 pins only \"a second save over a stale tmp fails loudly\").",
    "fix": "Drop \"and the CLI's startup check\", leaving the loud write-time failure as the contract answer, or name the marker (an exclusive-create registry.lock beside the corpus, taken at startup) as an apps/registry behavior with its own case in #11."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets — \"`MAX_REQUEST_BODY_OCTETS` (a POST body cap)\"",
    "problem": "Of the two new private request budgets only one is given a value (MAX_REQUEST_HEADER_OCTETS, 65,536), so the refusal test #11 pins has no boundary to sit on, and it interacts with the uncapped `source_keys` array: a record legal under 026-C3 can outgrow any number the implementer picks. Inbound framing is unstated with it: the response writer's Transfer-Encoding ban is pinned, but nothing says how a chunked or Content-Length-less admin POST is answered.",
    "fix": "State the number with the record bound it is sized against, and add to http.rs: a request body without a decimal Content-Length, or over the cap, is refused before any mutation."
  },
  {
    "severity": "minor",
    "location": "Options considered — \"the two shipped JSON documents (`matches.json`, the candidate body) already live *outside* the TOML formats contract (A5, A6)\"; Chosen design → Budgets — \"its private `STORE_READ_BUDGET_OCTETS` (A7)\"",
    "problem": "The anchors do not carry the facts. A5 is the MAX_NATIVE_NAME_OCTETS bound and A6 the data-directory rule; neither states that those documents sit outside the TOML subset (that is conformance.rs's .toml fixture walk plus matches.rs's serde_json store). A7 is the atomic-persist row; the over-budget file refusal is STORE_READ_BUDGET_OCTETS at matches.rs:36,194, which no register row covers — and it is also the load-bearing precedent for the cache and catalog load refusals.",
    "fix": "Add one register row for matches.rs's private read-budget refusal (`load`, STORE_READ_BUDGET_OCTETS) and re-anchor both passages to it."
  },
  {
    "severity": "minor",
    "location": "Deferred / out of scope — \"The seed corpus adopting the real shipped manifest ids once the first manifest PR lands (`data/manifests/` is empty, A14)\"",
    "problem": "This deferral contradicts its source clause rather than merely deferring it: 026-S1 requires a corpus \"opened with a handful of real games (game ids drawn from the shipped manifest ids…)\". The empty manifest tree makes the clause unsatisfiable today (A14 verified: data/manifests/README.md:10), so minting provisional ids is the only possible act — but Rollout step 6's ledger annotations name only 025-C4, 025-L1 and 025-R1, so the ledger will record 026-S1 as discharged by a corpus that does not satisfy it.",
    "fix": "Add 026-S1 to Rollout step 6's annotations (an amended-by / partially-satisfied-by line per 021-L4 recording that the seed mints grammar-valid ids while data/manifests/ is empty), so the divergence is visible on the unit itself."
  },
  {
    "severity": "nit",
    "location": "Chosen design → The serving face — \"mode is an enum field on the server state, so a client cannot tell which answered (`026-C1`)\"",
    "problem": "The same face's GET /v1/health answers `mode` and `upstream` by name, and a caching instance's cold 503 where root answers 200 [] is a mode-revealing difference the ADR itself mandates, so the claim as written is false.",
    "fix": "Scope it: \"GET /v1/candidates is byte-identical between modes; health reports `mode` by design.\""
  },
  {
    "severity": "nit",
    "location": "Chosen design → CLI amendment — \"**drops the `LOOPBACK_HOSTS` host check**\"",
    "problem": "Retiring the check leaves const LOOPBACK_HOSTS (catalog.rs:34) unused, and the repo gate is `cargo clippy --workspace --all-targets -- -D warnings`. The Goals line (\"retirement named at all three shipped sites\") covers it in spirit but the amendment's bullet list does not.",
    "fix": "Name the constant's deletion in the first bullet."
  },
  {
    "severity": "nit",
    "location": "Revision log — \"(§\\\"Where the format contract and the ADR disagree\\\", open Q1/Q2)\" and \"(§Platform notes, open Q7)\"",
    "problem": "Both entries anchor to sections this document does not have; the decisions now live in \"Catalog document and its spec\" and \"The serving face\", so the log's pointers cannot be followed by a reviser.",
    "fix": "Re-point both entries at the sections that now hold the decision."
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
Write your report to docs/designs/026-registry-service/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "Chosen design → The admin face and auth, Concurrency — \"mutations take the write lock, mutate the model, and perform the atomic tmp-and-rename under a separate persist-only lock, so serving reads never wait on filesystem IO\"", "problem": "The r1 revision made this sentence either self-refuting or silently unsound, with no pinned case on either horn. If the tmp-and-rename happens while the model write lock is still held (the literal reading of the clause chain), serving reads wait on filesystem IO, which is exactly what the clause promises they never do. If the write lock is released before the persist, the text states neither when the snapshot is taken nor how persists are ordered with mutations, so two accepted admin mutations can persist newest-first and leave the file holding the older revision's record set while memory holds the newer one: after a restart an accepted mutation is gone and `revision` runs backwards, which 026-C4 forbids (\"every accepted mutation bumps `revision` and persists atomically\"). Test #11 pins only \"a reader never sees a torn document (served from the locked in-memory model)\" and #10 pins one upsert and one delete, so T-006 ships either horn with every pinned test green.", "fix": "State the lock order and the snapshot point in one clause replacing \"mutate the model, and perform the atomic tmp-and-rename under a separate persist-only lock\": \"the persist lock is taken before the model write lock; the mutation serializes the model into bytes while still holding both, releases the model lock, then does the tmp-and-rename under the persist lock alone\"; and add to #11: N concurrent mutations ⇒ after all return, the file equals the model and `revision` equals the final value."}, {"severity": "minor", "location": "Compatibility matrix — \"Linux & other Unix | Yes\" vs \"Other / exotic Unix (BSD et al.) | No\"", "problem": "The r1 edit flipped the fourth row from `unknown` to `No` without touching row 1's label, so the matrix now says other Unix is both supported and not supported; the closing sentence (\"the three `Yes` rows (Linux & other-Unix CI, macOS, Windows)\") compounds the contradiction.", "fix": "Relabel row 1 \"Linux (primary CI target)\" and let row 4 carry the `No`, or state that row 1's \"other Unix\" names only the shared Unix data-dir branch, not a support promise."}, {"severity": "minor", "location": "Chosen design → Caching mode, Upstream reachability — \"probed **at most once per TTL**, fired lazily by the first lookup or health read after expiry on that request's own thread\"", "problem": "This contradicts the serving face's \"Answering it reads the last-known upstream verdict … it does not perform a live upstream probe per call\": a health read after TTL expiry now runs a synchronous upstream round trip (up to the 2 s/5 s timeouts) on the health request's own thread. And because `last_upstream_fetch: 0` is expired from the first instant, the new #12 pin (\"a freshly started caching instance answers `upstream: down`, `last_upstream_fetch: 0`\") is order-dependent — an implementer who applies the probe's verdict before writing the response fails exactly that case whenever the upstream is reachable.", "fix": "\"A health read never fires the probe; only lookups do\" (or \"health answers from the verdict as it stood at entry and fires the probe after the response is written\"), and say so in #12."}, {"severity": "minor", "location": "Chosen design → Budgets — \"evicts the oldest entry by **insertion sequence** (ties by `fetched_at` …)\"", "problem": "The r1 eviction-order fix changed the eviction key away from 026-B1's own words (\"over the cap the oldest `fetched_at` is evicted\") and records the divergence nowhere, while the design carefully records far smaller ones (`026-S1` in Rollout step 6, the 026-L1/T-008 window in step 5). Test #5 now pins \"oldest insertion sequence evicted first\", and #5 is the text handed to T-007.", "fix": "Add one clause — \"which equals `026-B1`'s oldest-`fetched_at` order except across a backwards clock jump, where the ADR's rule would evict fresh data\" — and add `026-B1` to Rollout step 6's annotations per `021-L4`; also state whether a refreshed entry keeps or takes a new insertion sequence."}, {"severity": "minor", "location": "Chosen design → Budgets — \"`MAX_CATALOG_DOCUMENT_OCTETS` (1,048,576), `fs::metadata`-gated before the read of the `--corpus` or local-entries file, refused by name\"", "problem": "At a realistic ~120-140 octets per `{game_id, name, source_keys}` record, 1 MiB holds roughly 7,500 records — an order of magnitude below the ADR's stated corpus (\"tens of thousands of games, each with several store keys\", Context) — so the newly named refusal fires on the root's own corpus long before the scale the ADR names as the reason for the design. My r1 fix proposed mirroring 026-B2; the design's adoption makes that number load-bearing for the corpus, where in the ADR it bounded a cache.", "fix": "Size the cap from the ADR's stated scale and show the arithmetic in `schemas/catalog/v1/spec.md` (e.g. 16 MiB ≈ 120k records), or state explicitly that a corpus past the cap is out of contract for v1 and what its operator does instead."}, {"severity": "minor", "location": "Chosen design → Budgets — \"`MAX_REQUEST_BODY_OCTETS` (65,536, sized to one `026-C3` record inside the document cap)\"", "problem": "There is no per-record size to size against: 026-C3 bounds each `source_key` by 025-B1 and never bounds how many a record carries, so the largest legal record is unbounded and the sizing claim cannot be satisfied. The two faces of one document then disagree — `corpus.rs` loads a record carrying 600 keys from disk while `POST /v1/admin/games` refuses the same record as over-cap — and no pinned test sits on that seam.", "fix": "Either set the body cap to the document cap (\"any record that could legally live in a loadable corpus fits, so no per-record key-count bound is needed\"), or name `MAX_SOURCE_KEYS_PER_RECORD` in `constants.rs` with a refusal in `corpus.rs`, `admin.rs` and #1, and state the record arithmetic behind the 65,536 figure."}, {"severity": "minor", "location": "Chosen design → Budgets — \"`MAX_CONCURRENT_CONNECTIONS` (over-cap closed at accept, one thread per admitted connection)\"", "problem": "This is the only budget in the section without a value, so #11's \"over-`MAX_CONCURRENT_CONNECTIONS` is closed at accept\" has no boundary to sit on; and a bare close is the one inbound refusal in the design that is not named, where every other refusal is (the client's own outcome is `transport unavailable`/unreachable rather than a stated answer).", "fix": "Give the number with the thread story it prices, and state the over-cap answer (\"closed without a status line; the CLI reports its named unreachable\") or a `503` with a reason phrase, which A21 already treats as temporary."}, {"severity": "minor", "location": "Chosen design → The serving face step 1 — \"the registry's one key validator tightens it: after the first `/`, exactly one non-empty `.`-free component, no embedded `/`\"", "problem": "The validator is stated only by two exclusions, so everything else is admitted: percent-decoded control bytes (`%0a%0d`, `%00`), spaces and non-UTF-8 become legal keys in the cache document, in the corpus via admin upsert, and in any refusal that names the key (the cold answer is written as `HTTP/1.1 503 No catalog answer for slug`; the design's single-line escaping discipline is stated only for the failed upstream id). Conversely the flat `.`-ban refuses the very keys the ADR names as the point of the admin face — Context: \"a `local/<dir>` install has a lookup key\" — and directory names carry dots routinely, while the stated purpose (never let a `..` or separator reach a path) needs no more than banning a `.`/`..` component and further separators.", "fix": "State the accepted key grammar positively: either name the byte set (`[a-z0-9-]` per component, `is_slug_name` as the cited precedent, plus \"a decoded key that is invalid UTF-8 or carries a control byte is the `404`\"), or keep the path-safety purpose honest (\"a component equal to `.` or `..`, or any further `/`, is the `404`\") and say whether a `.`-bearing external id is in contract for v1; extend #3 to pin the byte-set answer."}, {"severity": "minor", "location": "Chosen design → Catalog document and its spec — \"a repeated `game_id` (records are keyed by it, so a duplicate loads ambiguously)\"", "problem": "The r1 fix was taken halfway: a repeated `source_key` within one record is still not a refusal, so a record may carry `steam/730` twice, which the `corpus.rs` slug→candidates index turns into duplicate `{game_id, name}` postings (the same game twice inside `MAX_CANDIDATES`, crowding out other answers), and `stats`' `source_keys` count becomes ambiguous between total and distinct — the count shape #10 now pins.", "fix": "Add \"a repeated `source_key` within one record\" to the `corpus.rs` refusal list and to #1's `fixtures/invalid/` roster."}, {"severity": "minor", "location": "Chosen design → Catalog document and its spec, harness list — \"guard the fixture-tree loops — … and `fixture_file_names_…` (which keeps governing the `.json` fixtures: lowercase dash-separated names, case-unique) — behind the core-TOML check so catalog fixtures are not parsed as TOML\"", "problem": "The two halves cancel: a loop guarded behind the core-TOML check by definition stops governing catalog's fixtures, and #19's Windows/macOS case-collision argument rests on that very test covering them. Verified also that shipped `is_fixture_name` takes a bare name (`crates/core/tests/common/mod.rs:421`, one call site at `conformance.rs:348`), so \"accepts `.json` for that directory only\" is a signature change, not a predicate tweak.", "fix": "Split the bullet: the re-serialization and both `every_invalid_fixture_…` message loops skip `catalog`; `fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique` keeps running over it, with the dir-aware `is_fixture_name` named at its one call site."}, {"severity": "minor", "location": "Chosen design → Catalog document and its spec, harness list — \"re-scope the sweep to `data/manifests` + `data/profiles`\" (with the Risks row \"Single `const` names `catalog` at the skip sites; the re-scoped `boundary_tests` sweep … keeps the hygiene gate where it belongs\")", "problem": "Verified at `crates/discovery/tests/boundary_tests.rs:116-130`: the sweep walks every `schemas/*/v*/fixtures/valid/` today, so the re-scope deletes the gate's hold on every other format's valid fixtures — a future `steam/`-bearing `schemas/profiles/…/valid/*.toml` passes green. And the mitigation's single `const` cannot span the two harnesses it names: `crates/core/tests/common/mod.rs` is not reachable from `crates/discovery/tests/boundary_tests.rs` (separate crates, separate test binaries).", "fix": "Keep the sweep whole and exclude only the catalog format's own directories — a small local exclusion list in `boundary_tests.rs`, the `const` staying at core's skip sites — instead of narrowing the sweep to two `data/` subdirs."}, {"severity": "nit", "location": "Options considered — \"the rejected location dodges neither harness consumer — the conformance walk nor the `data/`-walking source-key sweep\"", "problem": "The reasoning is inverted for the first half: moving the format spec out of `schemas/` dodges the conformance walk entirely, which is A9's whole point; only the `data/`-walking sweep survives, because 026-S1 fixes the seed path whatever the spec's location is.", "fix": "\"The rejected location dodges only the conformance walk — the seed at `data/catalog/v1/` still trips the `data/`-walking sweep, and `026-C3` fixes the spec path anyway.\""}, {"severity": "nit", "location": "Chosen design → The admin face and auth — \"Enablement is judged from the address actually bound — the shipped `probe()` predicate `local_addr().ip().is_loopback()`, not a literal-spelling comparison\"", "problem": "That call is not shipped: `probe()`'s predicate is `addr.ip().is_loopback()` on a resolved outbound address (apps/discovery-cli/src/catalog.rs:248) and `local_addr` appears nowhere in the codebase; the same design deletes that shipped refusal at T-008, so \"the shipped `probe()` predicate\" points an implementer at a block the amendment removes. The loopback judgment is also absent from the five-block copy list in §`apps/registry`.", "fix": "\"The predicate is `IpAddr::is_loopback()`, the one `probe()` uses today at `catalog.rs:248`, applied here to the listener's `local_addr()`\" — and either add it to the copy list or mark it as the registry's own."}]}
```
