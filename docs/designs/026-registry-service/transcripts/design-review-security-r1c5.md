# Transcript: design-review:security r1c5

## Provenance
- stage: design-review:security (round r1c5)
- agent: review/design-security
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:41:57.019Z
- ended: 2026-11-19T19:46:57.311Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":50328,"output":22671,"reasoning":0,"cache.read":546614,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/security-r1c5.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "nit",
    "location": "Test strategy #3 — \"the cap and the key validator apply to **decoded** `slug` and decoded `DELETE` path segment\"",
    "problem": "The key validator requires `<source-token>/<component>`, so applied literally to a DELETE path segment it would 404 every valid `game_id`; §Admin face correctly pins the DELETE segment to the A4 slug grammar, and the security property (dot/separator rejection via `is_slug_name`, which excludes `.` and `/`) survives, but the test line as written pins the wrong predicate pair.",
    "fix": "In #3, pin `slug` against the key validator plus `025-B1`'s cap, and the DELETE segment against the A4 grammar."
  },
  {
    "severity": "nit",
    "location": "Chosen design → `http.rs` module line — \"a `Transfer-Encoding`, missing or non-decimal length, or an over-cap body is refused before any mutation\"",
    "problem": "Read literally, \"missing length is refused\" contradicts the A1-pinned shipped client, whose GETs carry no `Content-Length`; the hand-written reader needs the zero-body case named (`Transfer-Encoding` ⇒ refusal; `Content-Length` present ⇒ read exactly that many octets within cap; absent ⇒ zero-length body) so the request-side framing rule has exactly one reading at the smuggling seam.",
    "fix": "Add the zero-`Content-Length`-means-zero-body clause so the framing refusal targets body claims only."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Budgets — \"`MAX_CATALOG_DOCUMENT_OCTETS` (1,048,576), `fs::metadata`-gated before the read of the `--corpus` or local-entries file\"",
    "problem": "The cache got a write-time dual-cap invariant, but the corpus cap is checked only at read, so the authenticated admin face can grow the corpus past 1 MiB and the next start refuses to load it — a token-holder-only self-DoS (the token-holder's integrity damage is the already-accepted 026-R1 risk), asymmetric with the cache's stated posture.",
    "fix": "One clause: an upsert whose record would push the document over `MAX_CATALOG_DOCUMENT_OCTETS` is refused by name at write, mirroring \"provably inside both caps after every mutation\"."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/026-registry-service/reviews/design/security-r1c5.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"§Budgets → request budgets — \"`MAX_REQUEST_BODY_OCTETS` (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally hold also fits an upsert and no per-record `source_keys` bound is needed)\"","problem":"The revision moved the inbound request-body bound from the r1 value this seat approved (65,536, \"mirroring the client's own header cap\") to 16,777,216 for every route, because the constant was re-purposed as the document cap; the §Risks row still cites it as the inbound mitigation (\"Low-and-slow / one-inbound-per-outbound exposure on a routable bind … `MAX_REQUEST_HEADER_OCTETS`/`MAX_REQUEST_BODY_OCTETS`\") while the bound it names is 256× larger, and nothing pins where the route/method/`401` decision sits relative to the body read (`http.rs` describes only \"the request reader admits `Content-Length`-framed bodies within the budgets below\"). Concrete failure: on the ADR's public-root posture (non-loopback bind, admin possibly disabled — the socket is still open), an anonymous caller sends 64 concurrent requests each claiming a 16,777,216-octet `Content-Length` body to any route, and a reader that runs before the router buffers and hands `serde_json` ~1 GiB in one std-only process. Deleting the per-record `source_keys` bound removes the old implicit key-count bound that the 64 KiB cap supplied.","fix":"One clause in `http.rs`/§Budgets: the router answers `404`/`405`/`401` from the header block alone, and a body claim on any route other than `POST /v1/admin/games` (whose legitimate bodies are always zero-length) is refused unread; state the worst-case inbound figure (`MAX_CONCURRENT_CONNECTIONS` × body cap) beside the §Risks row it belongs to. Alternatively keep 65,536 as the non-admin body bound and bound `source_keys` per record, which retires the equality instead of widening it."},{"severity":"minor","location":"§Test strategy #3 — \"`025-B1`'s cap and §The serving face step 1's rule apply to **decoded** `slug` and decoded `DELETE` path segment\"","problem":"The rewrite renamed the predicate but kept the wrong pair. A `026-C3` `game_id` is `is_slug_name(id) && len ≤ 128` (A4) and `is_slug_name` is `[a-z][a-z0-9-]*` with no `/` (crates/core/src/placement.rs:107), while `spells_lookup_key` requires a `/` (crates/discovery/src/game.rs:84, `split_once('/')`), so step 1's rule applied to a DELETE segment `404`s every valid `game_id`. Both authoritative texts, revised in this same diff, say otherwise: §The admin face pins \"percent-decodes the path segment, validates the grammar (A4)\", and step 1 now scopes its own rule to \"candidate queries, admin upserts and cache loads alike\" (DELETE absent). #3 therefore contradicts the two sections it exists to pin.","fix":"In #3 pin `slug` against `025-B1`'s cap plus §The serving face step 1's rule, and the `DELETE` path segment against the A4 grammar (`is_slug_name` + 128), keeping the separator/dot-dot/control-byte cases at the query seam."},{"severity":"nit","location":"§Chosen design → `http.rs` — \"a `Transfer-Encoding`, missing or non-decimal length, or an over-cap body is refused before any mutation\" (and #11's \"a `Transfer-Encoding`, `Content-Length`-less, or over-cap body is refused\")","problem":"This pass verified the contradicting fact into the register (A23: the shipped client sends one `GET` per connection carrying `Connection: close`), and the shipped request really is `GET …\\r\\nConnection: close\\r\\n\\r\\n` with no `Content-Length` (apps/discovery-cli/src/catalog.rs:438), yet the request-reader sentence still reads as refusing a missing length. Literally, the reader refuses every `GET /v1/candidates` — the primary route and the one `026-C1` requires to be byte-identical to what that client parses — so the framing rule keeps two readings at exactly the smuggling seam.","fix":"Name the zero-claim case so the refusal targets body claims only: `Transfer-Encoding` ⇒ refused; `Content-Length` present ⇒ read exactly that many octets within cap; absent ⇒ zero-length body."}]}
```
