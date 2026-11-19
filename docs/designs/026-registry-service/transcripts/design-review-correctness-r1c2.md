# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:18:28.032Z
- ended: 2026-11-19T19:25:41.711Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":64404,"output":29229,"reasoning":0,"cache.read":1416649,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/correctness-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → The admin face and auth, Concurrency — \"mutations take the write lock, mutate the model, and perform the atomic tmp-and-rename under a separate persist-only lock, so serving reads never wait on filesystem IO\"",
    "problem": "The r1 revision made this sentence either self-refuting or silently unsound, with no pinned case on either horn. If the tmp-and-rename happens while the model write lock is still held (the literal reading of the clause chain), serving reads wait on filesystem IO, which is exactly what the clause promises they never do. If the write lock is released before the persist, the text states neither when the snapshot is taken nor how persists are ordered with mutations, so two accepted admin mutations can persist newest-first and leave the file holding the older revision's record set while memory holds the newer one: after a restart an accepted mutation is gone and `revision` runs backwards, which 026-C4 forbids (\"every accepted mutation bumps `revision` and persists atomically\"). Test #11 pins only \"a reader never sees a torn document (served from the locked in-memory model)\" and #10 pins one upsert and one delete, so T-006 ships either horn with every pinned test green.",
    "fix": "State the lock order and the snapshot point in one clause replacing \"mutate the model, and perform the atomic tmp-and-rename under a separate persist-only lock\": \"the persist lock is taken before the model write lock; the mutation serializes the model into bytes while still holding both, releases the model lock, then does the tmp-and-rename under the persist lock alone\"; and add to #11: N concurrent mutations ⇒ after all return, the file equals the model and `revision` equals the final value."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix — \"Linux & other Unix | Yes\" vs \"Other / exotic Unix (BSD et al.) | No\"",
    "problem": "The r1 edit flipped the fourth row from `unknown` to `No` without touching row 1's label, so the matrix now says other Unix is both supported and not supported; the closing sentence (\"the three `Yes` rows (Linux & other-Unix CI, macOS, Windows)\") compounds the contradiction.",
    "fix": "Relabel row 1 \"Linux (primary CI target)\" and let row 4 carry the `No`, or state that row 1's \"other Unix\" names only the shared Unix data-dir branch, not a support promise."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Caching mode, Upstream reachability — \"probed **at most once per TTL**, fired lazily by the first lookup or health read after expiry on that request's own thread\"",
    "problem": "This contradicts the serving face's \"Answering it reads the last-known upstream verdict … it does not perform a live upstream probe per call\": a health read after TTL expiry now runs a synchronous upstream round trip (up to the 2 s/5 s timeouts) on the health request's own thread. And because `last_upstream_fetch: 0` is expired from the first instant, the new #12 pin (\"a freshly started caching instance answers `upstream: down`, `last_upstream_fetch: 0`\") is order-dependent — an implementer who applies the probe's verdict before writing the response fails exactly that case whenever the upstream is reachable.",
    "fix": "\"A health read never fires the probe; only lookups do\" (or \"health answers from the verdict as it stood at entry and fires the probe after the response is written\"), and say so in #12."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets — \"evicts the oldest entry by **insertion sequence** (ties by `fetched_at` …)\"",
    "problem": "The r1 eviction-order fix changed the eviction key away from 026-B1's own words (\"over the cap the oldest `fetched_at` is evicted\") and records the divergence nowhere, while the design carefully records far smaller ones (`026-S1` in Rollout step 6, the 026-L1/T-008 window in step 5). Test #5 now pins \"oldest insertion sequence evicted first\", and #5 is the text handed to T-007.",
    "fix": "Add one clause — \"which equals `026-B1`'s oldest-`fetched_at` order except across a backwards clock jump, where the ADR's rule would evict fresh data\" — and add `026-B1` to Rollout step 6's annotations per `021-L4`; also state whether a refreshed entry keeps or takes a new insertion sequence."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets — \"`MAX_CATALOG_DOCUMENT_OCTETS` (1,048,576), `fs::metadata`-gated before the read of the `--corpus` or local-entries file, refused by name\"",
    "problem": "At a realistic ~120-140 octets per `{game_id, name, source_keys}` record, 1 MiB holds roughly 7,500 records — an order of magnitude below the ADR's stated corpus (\"tens of thousands of games, each with several store keys\", Context) — so the newly named refusal fires on the root's own corpus long before the scale the ADR names as the reason for the design. My r1 fix proposed mirroring 026-B2; the design's adoption makes that number load-bearing for the corpus, where in the ADR it bounded a cache.",
    "fix": "Size the cap from the ADR's stated scale and show the arithmetic in `schemas/catalog/v1/spec.md` (e.g. 16 MiB ≈ 120k records), or state explicitly that a corpus past the cap is out of contract for v1 and what its operator does instead."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets — \"`MAX_REQUEST_BODY_OCTETS` (65,536, sized to one `026-C3` record inside the document cap)\"",
    "problem": "There is no per-record size to size against: 026-C3 bounds each `source_key` by 025-B1 and never bounds how many a record carries, so the largest legal record is unbounded and the sizing claim cannot be satisfied. The two faces of one document then disagree — `corpus.rs` loads a record carrying 600 keys from disk while `POST /v1/admin/games` refuses the same record as over-cap — and no pinned test sits on that seam.",
    "fix": "Either set the body cap to the document cap (\"any record that could legally live in a loadable corpus fits, so no per-record key-count bound is needed\"), or name `MAX_SOURCE_KEYS_PER_RECORD` in `constants.rs` with a refusal in `corpus.rs`, `admin.rs` and #1, and state the record arithmetic behind the 65,536 figure."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Budgets — \"`MAX_CONCURRENT_CONNECTIONS` (over-cap closed at accept, one thread per admitted connection)\"",
    "problem": "This is the only budget in the section without a value, so #11's \"over-`MAX_CONCURRENT_CONNECTIONS` is closed at accept\" has no boundary to sit on; and a bare close is the one inbound refusal in the design that is not named, where every other refusal is (the client's own outcome is `transport unavailable`/unreachable rather than a stated answer).",
    "fix": "Give the number with the thread story it prices, and state the over-cap answer (\"closed without a status line; the CLI reports its named unreachable\") or a `503` with a reason phrase, which A21 already treats as temporary."
  },
  {
    "severity": "minor",
    "location": "Chosen design → The serving face step 1 — \"the registry's one key validator tightens it: after the first `/`, exactly one non-empty `.`-free component, no embedded `/`\"",
    "problem": "The validator is stated only by two exclusions, so everything else is admitted: percent-decoded control bytes (`%0a%0d`, `%00`), spaces and non-UTF-8 become legal keys in the cache document, in the corpus via admin upsert, and in any refusal that names the key (the cold answer is written as `HTTP/1.1 503 No catalog answer for slug`; the design's single-line escaping discipline is stated only for the failed upstream id). Conversely the flat `.`-ban refuses the very keys the ADR names as the point of the admin face — Context: \"a `local/<dir>` install has a lookup key\" — and directory names carry dots routinely, while the stated purpose (never let a `..` or separator reach a path) needs no more than banning a `.`/`..` component and further separators.",
    "fix": "State the accepted key grammar positively: either name the byte set (`[a-z0-9-]` per component, `is_slug_name` as the cited precedent, plus \"a decoded key that is invalid UTF-8 or carries a control byte is the `404`\"), or keep the path-safety purpose honest (\"a component equal to `.` or `..`, or any further `/`, is the `404`\") and say whether a `.`-bearing external id is in contract for v1; extend #3 to pin the byte-set answer."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Catalog document and its spec — \"a repeated `game_id` (records are keyed by it, so a duplicate loads ambiguously)\"",
    "problem": "The r1 fix was taken halfway: a repeated `source_key` within one record is still not a refusal, so a record may carry `steam/730` twice, which the `corpus.rs` slug→candidates index turns into duplicate `{game_id, name}` postings (the same game twice inside `MAX_CANDIDATES`, crowding out other answers), and `stats`' `source_keys` count becomes ambiguous between total and distinct — the count shape #10 now pins.",
    "fix": "Add \"a repeated `source_key` within one record\" to the `corpus.rs` refusal list and to #1's `fixtures/invalid/` roster."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Catalog document and its spec, harness list — \"guard the fixture-tree loops — … and `fixture_file_names_…` (which keeps governing the `.json` fixtures: lowercase dash-separated names, case-unique) — behind the core-TOML check so catalog fixtures are not parsed as TOML\"",
    "problem": "The two halves cancel: a loop guarded behind the core-TOML check by definition stops governing catalog's fixtures, and #19's Windows/macOS case-collision argument rests on that very test covering them. Verified also that shipped `is_fixture_name` takes a bare name (`crates/core/tests/common/mod.rs:421`, one call site at `conformance.rs:348`), so \"accepts `.json` for that directory only\" is a signature change, not a predicate tweak.",
    "fix": "Split the bullet: the re-serialization and both `every_invalid_fixture_…` message loops skip `catalog`; `fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique` keeps running over it, with the dir-aware `is_fixture_name` named at its one call site."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Catalog document and its spec, harness list — \"re-scope the sweep to `data/manifests` + `data/profiles`\" (with the Risks row \"Single `const` names `catalog` at the skip sites; the re-scoped `boundary_tests` sweep … keeps the hygiene gate where it belongs\")",
    "problem": "Verified at `crates/discovery/tests/boundary_tests.rs:116-130`: the sweep walks every `schemas/*/v*/fixtures/valid/` today, so the re-scope deletes the gate's hold on every other format's valid fixtures — a future `steam/`-bearing `schemas/profiles/…/valid/*.toml` passes green. And the mitigation's single `const` cannot span the two harnesses it names: `crates/core/tests/common/mod.rs` is not reachable from `crates/discovery/tests/boundary_tests.rs` (separate crates, separate test binaries).",
    "fix": "Keep the sweep whole and exclude only the catalog format's own directories — a small local exclusion list in `boundary_tests.rs`, the `const` staying at core's skip sites — instead of narrowing the sweep to two `data/` subdirs."
  },
  {
    "severity": "nit",
    "location": "Options considered — \"the rejected location dodges neither harness consumer — the conformance walk nor the `data/`-walking source-key sweep\"",
    "problem": "The reasoning is inverted for the first half: moving the format spec out of `schemas/` dodges the conformance walk entirely, which is A9's whole point; only the `data/`-walking sweep survives, because 026-S1 fixes the seed path whatever the spec's location is.",
    "fix": "\"The rejected location dodges only the conformance walk — the seed at `data/catalog/v1/` still trips the `data/`-walking sweep, and `026-C3` fixes the spec path anyway.\""
  },
  {
    "severity": "nit",
    "location": "Chosen design → The admin face and auth — \"Enablement is judged from the address actually bound — the shipped `probe()` predicate `local_addr().ip().is_loopback()`, not a literal-spelling comparison\"",
    "problem": "That call is not shipped: `probe()`'s predicate is `addr.ip().is_loopback()` on a resolved outbound address (apps/discovery-cli/src/catalog.rs:248) and `local_addr` appears nowhere in the codebase; the same design deletes that shipped refusal at T-008, so \"the shipped `probe()` predicate\" points an implementer at a block the amendment removes. The loopback judgment is also absent from the five-block copy list in §`apps/registry`.",
    "fix": "\"The predicate is `IpAddr::is_loopback()`, the one `probe()` uses today at `catalog.rs:248`, applied here to the listener's `local_addr()`\" — and either add it to the copy list or mark it as the registry's own."
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
Write your report to docs/designs/026-registry-service/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → Budgets / The admin face and auth — \"`MAX_REQUEST_BODY_OCTETS` (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally hold also fits an upsert and no per-record `source_keys` bound is needed)\" and, in §Catalog document and its spec, the cap placed only in the loader's list: \"a document over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets)\"","problem":"The corpus document cap has no write-time rule anywhere in the design, while the cache cap now has one (\"after a write the registry serializes the cache … until the buffer is ≤ `026-B2` octets\"). An accepted `POST /v1/admin/games` can therefore persist a corpus whose serialized length passes 16,777,216, and the next start refuses the whole corpus by name — every accepted mutation on it is lost, which falsifies the sentence this revision just added to §Concurrency (\"a restart cannot lose an accepted mutation (#11)\"). Both reachable paths are newly opened by this revision: (a) the body cap was raised from 65,536 to the document cap, so a single legal upsert (026-C3 bounds each source_key by 025-B1 and the design now names no per-record bound) can push an empty corpus past the cap in one accepted request; (b) the document cap was raised to the ADR's \"tens of thousands of games\" scale, so a root corpus living at the cap is now a supported configuration (under r1's 1 MiB it was already an error), making ordinary curation across the line the normal operator path. 026-C4 requires \"every accepted mutation bumps `revision` and persists atomically\", yet the design never states what POST answers when the resulting document would be unloadable: #10 pins only \"upsert validates + bumps `revision` + atomic rename\" and #11's over-cap refusal is a per-request body bound, not a resulting-document bound. T-006 implements this with no pinned answer.","fix":"Replace \"validated against `026-C3` and refused by name; bumps `revision`; persists atomically through the tmp-and-rename discipline\" with \"validated against `026-C3` and refused by name; the write is then completed by serializing the whole document and refusing, by name, `MAX_CATALOG_DOCUMENT_OCTETS` as a `422` that leaves `revision`, the model and the file untouched; only then does `revision` bump and the tmp-and-rename run (the loader's over-cap refusal stays as the same rule's read side)\", add that refusal to `schemas/catalog/v1/spec.md`'s roster next to the record-level ones, and pin in #10: \"an upsert whose serialized corpus would pass `MAX_CATALOG_DOCUMENT_OCTETS` is the named `422`, `revision` unchanged, no tmp written\"."},{"severity":"nit","location":"Chosen design → Budgets — \"after a write the registry serializes the cache once and drops the oldest-`fetched_at` entries … until the buffer is ≤ `026-B2` octets and the entry count ≤ `026-B1`, then renames that buffer into place, so the octet test is the document's exact length and not a sum of per-entry totals\"","problem":"The two halves pull apart: measuring \"the document's exact length\" after each drop requires a re-serialization per eviction, while subtracting each dropped entry's length from one buffer is exactly the \"sum of per-entry totals\" the clause disclaims.","fix":"Replace \"serializes the cache once\" with \"serializes the cache, and re-serializes after each dropped entry (at most one extra pass per eviction)\", leaving the rest of the sentence."},{"severity":"nit","location":"Chosen design → The admin face and auth, Concurrency — \"a persist-only lock to cut that latency joins §Deferred's lift if it ever earns its keep\"","problem":"§Deferred's lift entry is \"Lifting the five copied invariant blocks … into a shared crate\" and carries no persist lock; a latency mechanism is deferred to a list that does not contain it, and that lift is a code-sharing refactor rather than a latency item.","fix":"Add \"and, if #11's N-upsert latency ever matters, the persist-only lock §Concurrency deletes\" to that §Deferred bullet, or drop the pointer and state the \"reader may wait on one local write\" latency as accepted."},{"severity":"nit","location":"Chosen design → Catalog document and its spec, harness list — \"`fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique` is **not** guarded … which is the only coverage behind the Windows `Yes` row's case-insensitive-filesystem claim (#19)\"","problem":"The Compatibility matrix's Windows row makes no case-insensitivity claim in either its r1 or r2 form (\"`<data>` = `%APPDATA%` … owner-only modes are a documented no-op … cache and local-entries paths …\"); the case-collision claim is #19's (\"**Windows** and **macOS** behavior is proven by the env-closure seam (#13) and case-unique fixture names\"), so the rationale cites a matrix claim that is not there.","fix":"Say \"the only coverage behind #19's case-unique-fixture-name claim for a case-insensitive filesystem\", or add the case-collision note to the Windows row so the pointer lands."}]}
```
