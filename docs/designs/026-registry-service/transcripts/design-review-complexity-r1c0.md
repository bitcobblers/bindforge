# Transcript: design-review:complexity r1c0

## Provenance
- stage: design-review:complexity (round r1c0)
- agent: review/design-complexity
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T18:46:34.641Z
- ended: 2026-11-19T18:50:24.486Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":59211,"output":23432,"reasoning":0,"cache.read":998251,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/complexity-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/026-registry-service.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/026-registry-service/reviews/design/complexity-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"§Options considered (\"the harness carve-out is the honest cost, not the format choice\") + §The seed corpus + §Catalog document and its spec (\"`schemas/catalog/v1/` ships `spec.md` + `fixtures/valid/*.json`\")","problem":"The JSON-carve-out file list names only crates/core/tests changes and misses a second shipped consumer of the same trees: crates/discovery/tests/boundary_tests.rs::the_shipped_corpus_carries_no_source_key_spelling walks every file under data/ recursively plus every schemas/*/v*/fixtures/valid/ (walk_files recurses) and fails if any contains the literal `steam/`, `epic/` or `local/`. Since Source::spells_lookup_key (crates/discovery/src/game.rs:84) admits only those three tokens, no valid catalog document can live under those trees: the 026-S1 seed \"each carrying its `steam/<appid>` source_key\" and every valid schemas/catalog/v1/fixtures/valid/*.json both trip it, so T-004 as scoped cannot pass `cargo test --workspace --locked`. The options comparison also priced the choice on an incomplete cost: the sweep covers data/ as well, so the rejected \"just don't put it in schemas/\" alternative would not have avoided it either.","fix":"Add the amendment to that same bullet list: re-scope that sweep to data/manifests and data/profiles (or exempt data/catalog/ and schemas/catalog/v1/fixtures/), re-pin its doc-comment and its `checked > 0` guard, and restate the Options paragraph naming both harness consumers. 026-S1 mandates the keys, so the sweep moves, not the seed."},{"severity":"major","location":"§`apps/registry` — one crate, two modes (\"It depends on `bindforge-core` for the grammar and name bounds\") + §Caching mode (\"reuses the shipped client's response discipline with its own private constants\") + §CLI amendment (\"mirroring `parse_endpoint`\") + §Deferred (\"pinned by the same `StoreLayout` env-closure seam\")","problem":"The reuse language describes sharing the crate layout cannot provide: apps/discovery-cli is a bin-only crate (its Cargo.toml declares no [lib]; apps/ holds only src/main.rs, src/catalog.rs, src/matches.rs), so the response reader and count caps, parse_endpoint, game_id_is_valid (matches.rs:354), StoreLayout/data_dir/store_path (matches.rs:46-103) and the tmp-and-rename save/owner_only_new_file discipline are unreachable from apps/registry — the design ships a second copy of five shipped invariants and defers the lift for exactly one of them. Meanwhile the one validator that is importable, Source::spells_lookup_key, lives in crates/discovery, a dependency the dependency sentence never names, so Test strategy #8's \"runs through `game_id_is_valid`\" names a function in another crate's binary.","fix":"In §Chosen design replace \"reuses\"/\"mirroring\"/\"the same … seam\" with one short shared-vs-copied list naming, per block (response discipline, endpoint gate, game_id predicate, <data> rule, atomic persist, source-key spelling), whether the task lifts it to a shared crate (bindforge-core or a small new one) or duplicates it plus a pinning test; and state the real dependency line for apps/registry (bindforge-core + discovery + serde_json). Smaller design: lift the three store/IO blocks once so both binaries call the same code, which also removes the duplicated env-closure seam Test #13 and the Compatibility matrix rely on."},{"severity":"minor","location":"§Budgets (\"`MAX_REQUEST_BODY_OCTETS` (a POST body cap)\") and the same section's (\"a `fetched_at` that is not a non-negative integer within a plausible-clock bound\")","problem":"Two of the design's own refusal thresholds are named without a value, while Test strategy #11 (\"MAX_REQUEST_HEADER_OCTETS/MAX_REQUEST_BODY_OCTETS refusals\") and #4 (\"a `fetched_at` outside the load bound is refused\") pin behavior against them. A section whose stated goal is that the budgets close against each other cannot establish that for an unvalued cap, and the task author must invent the number the proposal exists to decide.","fix":"State both numbers in the same sentence (e.g. body cap = 65_536 mirroring the header cap; fetched_at accepted within [now − horizon, now + skew] with both named) and cite them from #4/#11 by value, not by constant name alone."},{"severity":"minor","location":"§Caching mode (\"Upstream **reachability** (the `health.upstream` gauge) is gauged by the upstream's own `/v1/health`, but that verdict is **cached and refreshed on the same TTL**\")","problem":"No trigger is named for that refresh, and the module plan (main/serve/http/corpus/admin/cache/upstream/endpoint/constants, serve.rs = \"accept loop (one thread per connection…)\") has no scheduler. Either the /v1/health probe never runs, leaving health.upstream derived only from lookup outcomes and not honoring the 026-C6 clause the section cites, or a background timer arrives as an unplanned tenth moving part. The same paragraph's claim that \"A cold query therefore does not pay a separate upstream health round trip\" only holds while the verdict is unexpired.","fix":"One sentence naming the trigger — the probe runs at most once per TTL, fired lazily by the first lookup after expiry on that request's own thread, with no timer thread — and reword the round-trip claim as \"no additional health round trip per query beyond the one TTL-gated probe\"."},{"severity":"minor","location":"§CLI surface (\"overridable by `--local-entries <path>`\", \"`--refresh-interval` overrides `026-B3`\")","problem":"Neither knob has a consumer anywhere in the Test strategy: #14 pins only --upstream, #16 passes --corpus, #13 reaches the paths through the env seam, and no item starts a caching instance with a relocated entries document or a shortened TTL. --refresh-interval is 026-B3-mandated and --local-entries is invented beyond 026-C1's flag list, so the design pays for two knobs it never exercises.","fix":"Add one clause to #6 or #17 per knob — a TTL override that expires a cached entry early, and a caching run with --local-entries on a scratch path proving the admin mutation lands there — or drop --local-entries and let the tests relocate via HOME/XDG_DATA_HOME as #13 already does."},{"severity":"nit","location":"§Catalog document and its spec (\"guard `every_valid_fixture_…`, `every_invalid_fixture_…` and the single-line/truncation loops behind the core-TOML check\")","problem":"Of the loops named, only every_valid_fixture_re_serializes_to_a_semantically_equal_value, every_invalid_fixture_is_rejected_with_a_structured_error and every_invalid_fixture_is_rejected_with_a_single_line_message walk the fixture tree; the truncation case (a_rejection_truncates_an_untrusted_value_rather_than_echoing_it) parses a hand-built document, so guarding it adds a dead conditional. The list also omits the call site the .json rule actually needs: fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique asserts is_fixture_name(name) for every file in the tree, and is_fixture_name takes a bare name with no format argument.","fix":"Name the three walking loops plus the fixture-name test, drop \"truncation\", and say whether is_fixture_name gains a format argument or the call site is guarded."},{"severity":"nit","location":"§Catalog document and its spec (\"recognize a small `EXTERNAL_JSON_FORMATS = [\"catalog\"]` set\") + §Risks (\"pin it with a test that the list is the single source\")","problem":"A general external-format registry plus a meta-test about the list's own use, built for exactly one member — an abstraction layer with one implementation.","fix":"Use one const naming `catalog` at the skip site with the schemas/README.md paragraph as the record, and generalize when a second JSON format actually arrives; keep as written only if the panel wants the forward-compat on the record."}]}
```
