# Transcript: design-review:performance r1c0

## Provenance
- stage: design-review:performance (round r1c0)
- agent: review/design-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T02:52:44.767Z
- ended: 2026-11-22T03:01:54.494Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":57077,"output":18311,"reasoning":0,"cache.read":401539,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/027-hosted-registry-with-store-resolvers.md,
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/performance-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/027-hosted-registry-with-store-resolvers.md.
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/performance-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"§Concurrency — \"inside it, a writer takes its document's `RwLock` write half, mutates the model, serializes, tmp-and-rename, and swaps — the shipped single-lock persist posture\" and \"Readers (serving lookups) take document read halves only.\"","problem":"The mint is placed on the client-request path while inheriting a cost model built for operator-frequency admin writes, and the lock scope makes concurrent non-curated queries wait behind it. In the shipped code the design copies, state.rs::upsert holds catalog.write() across a full Catalog::clone(), Catalog::upsert's rebuild_postings() (O(records × keys) index rebuild, corpus.rs:280-291,359-365), to_bytes() via a Vec<serde_json::Value> DOM (corpus.rs:339-357), and persist::save's write + sync_all fsync + rename (persist.rs:184-233). This design raises the document 4× over the shipped MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216 (MAX_ISSUED_DOCUMENT_OCTETS = 67_108_864, ~30 MiB of live records) and drives that writer from any first-time steam/<appid> key, serialized on one mutations mutex ≤64 threads deep. Per-mint cost grows linearly with document size, so filling to 027-B3's 200,000 cap is quadratic aggregate work (order 2e10 element ops, ≈½ × 200k × 30 MiB ≈ terabytes written to one file). While a mint holds the issued write half, policy step 2 cannot take its read half, so every query missing the 6-record seed corpus stalls behind that fsync — effectively all hosted traffic — and the same mutex also queues corpus admin mutations. The §Risks mitigation \"the wait is one local write, already an accepted `026` posture\" rests on a false premise: in 026 the rewrite path was reachable only via the authenticated admin face, never GET /v1/candidates. No input-path latency budget exists anywhere in the document (checklist item 1), so the design's own claims that the cap is \"an ordinary operating state\" and that a full-document rewrite per mint are compatible go unquantified.","fix":"Keep 027-C4's persist-before-serve and the single mutations mutex, but narrow the lock scope: inside the mutations mutex (the lock 027-C4 names, and the thing that preserves persist order) build the new document, serialize it and pay tmp-and-rename, and take the document's RwLock write half only for the mutate-and-swap — replacing the clause \"the shipped single-lock persist posture, so persist order cannot diverge from mutation order\" as the justification for holding the write half across file IO, so no reader waits on an fsync. Make the mint's index update an incremental posting insert rather than rebuild_postings(), serialize straight into the buffer instead of through a serde_json::Value DOM, and add a measured figure to §Budgets (issued-document rewrite cost at 100k and 200k records, and the mint throughput it implies) so the §Deferred batched-writes entry defers against a number. Delete the \"already an accepted `026` posture\" mitigation text."},{"severity":"major","location":"§The store pull — \"Per page, up to `STEAM_PAGE_ATTEMPTS` (3) attempts separated by `STEAM_RETRY_SECONDS`; a page that fails all attempts fails the pull.\" with §Budgets \"STEAM_RETRY_SECONDS = 3_600\"","problem":"The hourly retry separation is defined once and the pull rules are shared by the startup pull and the background refresh, so the startup path can keep the process from binding for hours on a transient store fault. The socket timeouts bound only an attempt (STEAM_CONNECT_TIMEOUT 2 s, STEAM_READ_TIMEOUT 30 s) and no overall pull deadline is stated, so 027-C3's \"pulled in full before the process binds\" plus \"a present-but-unloadable snapshot that the pull then cannot replace is a named startup refusal\" means one flaky page costs two one-hour sleeps before the operator sees a refusal, and the worst legal case is 12 pages × 3 attempts × 32 s + 12 × 2 × 3,600 s — roughly a day of a running `registry serve` listening on nothing.","fix":"Give the startup pull its own seconds-scale inter-attempt backoff (a named constant in the §Budgets family, replacing \"separated by `STEAM_RETRY_SECONDS`\" on that path) plus one named overall pull deadline that turns an unfinished pull into the existing named startup refusal, and restrict STEAM_RETRY_SECONDS to the refresh thread's retry-after gap."},{"severity":"minor","location":"§Issued entries — \"same loader refusals (via the same `corpus.rs::Catalog` type)\" and \"`MAX_ISSUED_DOCUMENT_OCTETS = 67_108_864`, sized for `027-B3`'s 200,000 records at typical ~150-octet size (~30 MiB …)\"","problem":"Each document's bytes are budgeted separately but the process total is never stated, and the reused load discipline costs several copies of a file at once: corpus.rs::load_at reads the whole file into a Vec<u8>, parses it into a serde_json::Value DOM, then clones each record twice into the BTreeMap and rebuilds the postings (corpus.rs:186-275), so file bytes + DOM + records + postings coexist. Worst-case boot reads corpus ≤16 MiB + issued ≤64 MiB + retired ids ≤32 MiB + snapshot ≤32 MiB, and one mint adds a full model clone, a DOM of it, and the post-to_bytes buffer on top. Growth is bounded, so memory is finite, but there is no peak-resident or startup-time posture to match the octet discipline applied everywhere else.","fix":"Add one §Budgets sentence carrying the aggregate: the sum of the four document caps as the load ceiling and the model-clone-plus-serialize-buffer multiple a mint adds, so the caps read as bytes-on-disk and an operator capacity number exists."},{"severity":"minor","location":"§Issued entries (retired ids) — \"**Every** delete — issued or corpus — appends the removed record's `game_id` there before the mutation's persist.\" with `MAX_RETIRED_IDS_OCTETS = 33_554_432`, \"in byte order\"","problem":"The bookkeeping document's per-mutation cost and the mint's membership test are both left unspecified, and the shipped shape makes each a full-document event: an O(n) insert-and-resort of a byte-ordered list plus a second whole-file tmp-and-rename inside the same mutation-lock hold as the issued (or corpus) rewrite — so an issued DELETE pays two multi-megabyte fsyncs under the mutex, a corpus DELETE newly inherits a retired-file rewrite it never had in 026, and the mint's \"held-set is `games ∪ retired` across both catalog documents\" probe is given no structure while the snapshot's join is explicitly given one (\"a sorted vector plus an appid index\").","fix":"Name the in-memory shape (ordered set with membership by binary search, the affordance the snapshot sentence names) and state in one clause that a delete costs two bounded document rewrites, admin-frequency only."},{"severity":"nit","location":"§Budgets — \"A6's daily allowance is ~6,000× the worst pull — no separate quota machinery\"","problem":"The ratio is measured against a successful 12-page pull, but by the design's own rules a worst pull is 3 attempts on each of 12 pages = 36 calls, and the refresh thread's hourly retry loop can spend 24 × 36 = 864 calls/day against A6's 100,000. The conclusion survives (~115× inside the allowance) but the stated factor of ~6,000 overstates headroom by about 2×.","fix":"Restate the figure against the attempt-shaped worst pull (36 calls, ~2,800×) and note the retry loop's daily spend, so the \"no quota machinery\" conclusion is reproducible arithmetic."}]}
```
