# Transcript: design-review:security r1c0

## Provenance
- stage: design-review:security (round r1c0)
- agent: review/design-security
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T18:40:27.626Z
- ended: 2026-11-19T18:43:36.014Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":43808,"output":12748,"reasoning":0,"cache.read":409144,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/security-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/026-registry-service.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/026-registry-service/reviews/design/security-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design → Serving face — \"Percent-decode `slug` **first**, then apply `025-B1`'s 128-octet cap to the *decoded* bytes and `spells_lookup_key` to it; the corpus/index is keyed by the decoded string\" (same predicate at \"Catalog document and its spec\": \"a `source_key` failing `spells_lookup_key` or `025-B1`\"; cache-load refusal list omits `slug`)","problem":"The admitted lookup-key spelling is too weak for the network-fed seams: `spells_lookup_key` (crates/discovery/src/game.rs) is shape-only — closed source token, one `/`, non-empty external-id, no character or dot rule — and `025-B1` is a length cap, so `slug=local/%2e%2e%2f%2e%2e%2fetc%2fpasswd` decodes to `local/../../etc/passwd` and passes the design's own admission gate for `/v1/candidates`, passes the admin upsert, and is persistable into the corpus and cache documents (whose load refusals name `game_id`/`name` but never `slug`). The design never pins which spellings are accepted nor states the key's data plane, so the property \"dot- and separator-bearing keys can never ask the filesystem for a path\" cannot be established from the text, while the codebase's own precedent (`is_slug_name`'s doc: the grammar exists so `..`/separators never \"ask the filesystem for a path\") makes a path-joined index the pattern a T-005/T-007 implementer reaches for (security checklist 3).","fix":"Pin the admitted spelling — after the first `/`, exactly one non-empty `.`-free component with no embedded `/` — as one named validator applied at the candidate query, the admin upsert, and cache load (adding `slug` to the named cache refusals), and state that lookup keys are opaque in-memory strings that are never joined onto paths, replacing the bare `spells_lookup_key`-plus-length gate at those three seams."},{"severity":"minor","location":"Chosen design → apps/registry module plan — \"`serve.rs` — accept loop (one thread per connection, request-read timeout to bound low-and-slow)\" (with §Risks row \"Low-and-slow / one-inbound-per-outbound exposure on a routable bind\")","problem":"For a publicly reachable root with an anonymous serving face, the pinned defenses bound inbound bytes only: no cap on concurrently open connections (one OS thread each, held to the read timeout or longer), and the named timeout is a read timeout — a peer that never reads the response parks a writer thread indefinitely, so unauthenticated connection buildup can exhaust the process.","fix":"Name `MAX_CONCURRENT_CONNECTIONS` in `constants.rs` with an immediate close when over the cap, and a write-side timeout or overall per-connection deadline, in the `serve.rs` line and that risk row's mitigation."},{"severity":"minor","location":"Chosen design → The admin face and auth — \"`--admin-token` or `BINDFORGE_REGISTRY_ADMIN_TOKEN`\" (cf. Caching mode's \"dropped with one escaped, single-line echo\")","problem":"The design states the escaping posture for echoed untrusted ids but never states the logging posture for the admin token: nothing says the configured token never enters refusal lines, logs, health/stats output, or the `401` path's text; `026-R1` carries transport assurance, not local/accidental disclosure.","fix":"Add one sentence to the admin section: the configured token is never echoed by any refusal, log line, or status surface; `401` responses carry only `WWW-Authenticate: Bearer` and the status line."},{"severity":"nit","location":"Chosen design → Catalog document and its spec — \"Nothing on disk is rewritten on refusal (A7's load discipline)\" (seed corpus loaded by `corpus.rs` from `data/catalog/v1/catalog.json`)","problem":"The corpus is the one document the design loads with no read budget: the cache gets `026-B2` re-checked at load and the match-store precedent caps its read at `STORE_READ_BUDGET_OCTETS` before parsing, but the corpus is checked only by per-record bounds known after the parse, so an oversized or corrupt corpus parses unbounded.","fix":"Mirror the precedent: a private corpus-read byte budget checked on file metadata before the parse, stated in `corpus.rs`'s line."},{"severity":"nit","location":"Chosen design → Caching mode — \"It **refuses redirects** (a `3xx` is an upstream error → `stale`/`503`)\"","problem":"The framing-mirror discipline (reject `Transfer-Encoding`, duplicated/non-decimal `Content-Length`) is pinned only for responses the registry reads; the inbound request reader's framing is left open, and the `026-R1` deployment posture (an edge terminating TLS in front of the root) makes a CL/TE mismatch between that edge and the hand-written reader a request-smuggling seam.","fix":"Extend the mirror sentence to `http.rs`'s request reader: inbound bodies are `Content-Length`-framed only; a request `Transfer-Encoding` is a named refusal."}]}
```
