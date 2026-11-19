# Transcript: code-review:security r1c0

## Provenance
- stage: code-review:security (round r1c0)
- agent: codereview/code-security
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-19T22:01:18.350Z
- ended: 2026-11-19T22:12:30.197Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":238859,"output":29129,"reasoning":0,"cache.read":2094471,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/026-registry-service/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 217c46fda9eb95230c8df62233eca8ef7b6ffb71...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/026-registry-service.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/026-registry-service/reviews/code/security-r1c0.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-19T21:53:05.514Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the security checklist.
Write your report to docs/designs/026-registry-service/reviews/code/security-r1c0.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only. The entire response must be a single valid JSON object with no prose, explanation, markdown, or additional text before or after it.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "apps/registry/src/http.rs:310-329 (write_response); apps/registry/src/state.rs:253-279; apps/registry/src/cache.rs:136-171", "problem": "No outbound response budget. constants.rs:31-33 says MAX_CANDIDATES is \"the most candidates one answer may carry, in either direction\", and the proposal's Goal (:21) plus §The serving face step 3 require the answer to stay inside A1's caps (body ≤ MAX_RESPONSE_OCTETS = 65_536, ≤ 256 candidates; 02-proposal.md:43,252; Test strategy #16's \"caps\"). Only Catalog::candidates truncates (corpus.rs:218-231); the cache-hit and stale paths return entry.candidates whole, Cache::load bounds no per-entry candidate list, and write_response measures nothing. A contract-valid corpus — 256 records riding the 128/128 bounds that fixtures/valid/on-every-length-bound.json declares loadable and sharing one source_key — answers 256×280+257 = 71,937 octets, which apps/discovery-cli's read_response refuses as over-budget (so the shipped CLI fails on a legal corpus, and a caching instance's own upstream reader turns a legal root answer into an upstream error and then a cold 503); separately, a cache document carrying thousands of candidates for one slug is served whole up to the 1 MiB 026-B2 size, per request. spec.md:101-103 waves at exactly this without closing the response seam, and no test pins it (response_shape.rs:236-239 uses a three-record corpus; candidates_join.rs:135-178 truncates short names only).", "fix": "Cap the per-entry candidate list at the cache write seam (upstream.rs already refuses > MAX_CANDIDATES) and again at Cache::load as a named refusal, and measure the serialized candidate body against MAX_RESPONSE_OCTETS in candidates_answer/write_response, answering a named refusal (or narrowing the stable first-256 window) rather than emitting a body the pinned reader rejects. Add a wide corpus at the 128/128 bounds to tests/response_shape.rs and an over-cap cache entry to tests/cache_document.rs."}, {"severity": "minor", "location": "apps/registry/src/main.rs:165-170 (flag arm at :154)", "problem": "Only the space-separated spelling is parsed, so the common `--admin-token=S3CRET` form falls into the unknown-flag arm and prints \"`--admin-token=S3CRET` is not a flag this registry has\" to stderr (main.rs:65-68), writing the configured bearer token into terminal scrollback, journald or CI logs from one typo — checklist item 4's rule that no secret reaches a log line.", "fix": "Split each argument on its first `=` and match the flag name (accepting or refusing `--admin-token=` by name), and echo only the flag name in the unknown-flag refusal, never the value; pin with a unit case asserting the refusal line omits the value."}, {"severity": "minor", "location": "apps/registry/src/http.rs:234-260 (called at :161), with constants.rs:24-29 and :56", "problem": "read_request reads the declared body before routing, and the enablement/admits checks happen only afterwards (serve.rs:162-191). An unauthenticated peer needs no valid token to make the server declare-and-buffer MAX_REQUEST_BODY_OCTETS = 16 MiB per connection — one per admitted slot, i.e. ~1 GiB of buffered body per burst of MAX_CONCURRENT_CONNECTIONS = 64 sockets — before answering 401. The proposal sets the two budgets separately (§Budgets) and never takes their product as a pre-credential allowance.", "fix": "Read the header block first and, for /v1/admin/**, decide is_enabled/admits from the already-complete headers and answer 401/404 before reading the declared body (close rather than drain); or give the admin upsert its own body cap well below the document cap and keep 16 MiB load-side only. Pin in tests/admin_auth.rs that an unauthenticated oversized POST is refused without consuming the body."}, {"severity": "minor", "location": "apps/registry/src/admin.rs:111-118 and :210-222; apps/registry/src/serve.rs:162-194", "problem": "With the defaults (--host 127.0.0.1, no token) the admin face is enabled and admits returns true for an uncredentialled request (admin.rs:134-136). POST /v1/admin/games parses its body as JSON whatever the Content-Type, and nothing checks Origin or Sec-Fetch-Site, so a simple cross-origin request — which needs no CORS preflight this server would answer — carries a full record. A page in a browser on the operator's own machine, DNS-rebound to 127.0.0.1, can blind-upsert into a caching instance's local-entries document, which by 026-L5 answers with zero upstream requests, is never merged and never expires: one write pins an attacker's game_id for a key the user's `discovery list` later trusts. Only the ephemeral default port and the unreadable response mitigate today.", "fix": "Keep 026-C5's posture but close the browser path: require `Content-Type: application/json` on POST /v1/admin/games (a non-simple type, so a cross-origin caller must preflight and no OPTIONS route is answered), and refuse a carried admin request that carries an Origin header; pin both in tests/admin_auth.rs."}, {"severity": "minor", "location": "apps/registry/src/http.rs:262-305 (find_header_end at :300-305)", "problem": "read_header_block reads one octet at a time and calls find_header_end(&buffer) — bytes.windows(4).position(...) from offset 0 — after every octet. A maximal MAX_REQUEST_HEADER_OCTETS = 65,536 block costs ~2×10^9 4-byte comparisons (seconds of CPU) for 64 KiB of input, per connection, and the 60 s REQUEST_READ_TIMEOUT (constants.rs:67-71) lets the peer hold the admitted slot throughout. The design's own inbound budget thus becomes the cheapest network-to-CPU amplification available, against the low-and-slow exposure §Risks (02-proposal.md:456) claims those budgets bound; the crate's own upstream reader already does it linearly (upstream.rs:131 uses head.ends_with(b\"\\r\\n\\r\\n\")).", "fix": "Test only the last four octets per iteration (buffer.ends_with(b\"\\r\\n\\r\\n\")) or resume the scan from buffer.len() - 4, keeping the cap check; add a bounded-time assertion to the request-budget case in tests/persist_and_concurrency.rs."}, {"severity": "nit", "location": "apps/registry/src/persist.rs:194-218", "problem": "save does write_all → flush → chmod → rename with no sync_all() on the tmp handle and no parent-directory sync. On a filesystem that does not order data before the rename's commit, a power loss can leave a zero-length or stale document for a mutation the admin face already answered 200 for, weakening §Concurrency's \"a restart cannot lose an accepted mutation\" (02-proposal.md:315-326) and checklist item 6's atomicity requirement. It mirrors the shipped matches.rs precedent, hence nit.", "fix": "Call handle.sync_all() before dropping the handle (no-op branch on non-unix), optionally sync the parent after the rename, and pin the call in tests/persist_and_concurrency.rs; or state the durability limit in schemas/catalog/v1/spec.md's persistence paragraph."}]}
```
