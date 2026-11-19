# performance review r2

verdict: approve

## Scope

Ratification-style editorial confirmation pass, round 2, performance seat. Scope
is the editorial diff `docs/designs/026-registry-service/reviews/design/editorial-r2.diff`
(239 lines, 22 hunks) applied to `02-proposal.md`, read against
`docs/adrs/026-registry-service.md` and the proposal's own Revision log. No
prior findings were carried into this pass, so nothing here is re-raised or a
regression. Inputs used: the diff, the proposal, the ADR, the Revision log, and
read-only inspection of `apps/discovery-cli/src/catalog.rs` to re-confirm the
wire-budget facts the touched sentences lean on. No other run record opened.

## Checklist walk

1. **Input-path latency budget** — the only latency-bearing text the diff
   touches is the `MAX_CONCURRENT_CONNECTIONS` parenthetical in §Budgets,
   re-flowed from "answers the one request §http.rs closes" to "answers exactly
   one request before `http.rs` closes the connection". Meaning-preserving, and
   it now states the request-per-connection bound in the same words as the
   `http.rs` module bullet ("one answer per connection") and A23's shipped
   `Connection: close` — re-confirmed in code at `catalog.rs:438`. No new
   per-event allocation, table rebuild, or synchronous IO is introduced.
   Request-read/write timeouts, the accept-time over-cap close, and the
   `MAX_REQUEST_HEADER_OCTETS`/`MAX_REQUEST_BODY_OCTETS` pair are untouched.
2. **Structure costs (O(n²) over unbounded n)** — no hunk touches the
   slug→candidates index build, the join probe, or the corpus scale arithmetic.
   The §Catalog document hunk only re-casts the write-side cap clause ("on the
   write side, §The admin face and auth applies that cap to every upsert"),
   which keeps both-ends enforcement intact and matches §Budgets and #10.
3. **Memory growth / bounds** — `MAX_CATALOG_DOCUMENT_OCTETS` (16,777,216),
   `026-B1` 5,000, `026-B2` 1,048,576 and `MAX_CONCURRENT_CONNECTIONS` 64 are
   unchanged by the diff. The `026-C3` loader refusal list in Test-strategy #1
   gains "the following" and keeps the over-cap-document entry.
4. **Startup cost** — `fs::metadata`-gated corpus/local-entries read, the
   load-built index, and the absent-cache-is-empty rule are outside the diff.
5. **Cache invalidation, correct not just fast** — the §Caching mode hunk is
   punctuation only (comma → semicolon in "a health read only reports the
   verdict as it stands; it never fires the probe"). The once-per-TTL lazy
   probe, the `--refresh-interval` override, the dual write-time eviction and
   the load-time `026-B2` re-check are unmodified.
6. **Research baseline gap** — not applicable to this pass: no expected number,
   budget or timing in the diff changed, and `01-research.md` was not among the
   inputs the caller allowed for this seat. Recorded as unevaluated rather than
   passed.

Diff-touched claims spot-checked against code (read-only): `MAX_RESPONSE_OCTETS
= 65_536` (`catalog.rs:23`), `MAX_CANDIDATES = 256` (`catalog.rs:26`), the
header-block and body budget refusals (`catalog.rs:299`, `:353`), the candidate
count refusal (`catalog.rs:380`) and the `Connection: close` request line
(`catalog.rs:438`) — all as A1/A23 state, so the re-flowed concurrency sentence
rests on facts that still hold.

One wording point worth recording as *checked and resolved by the diff itself*:
Test-strategy #10 changed "an upsert whose serialized corpus with its bumped
`revision` would **pass** `MAX_CATALOG_DOCUMENT_OCTETS`" to "would **exceed**"
it. The old verb could be read as *clearing* the check, which would have
described the `422` firing on the wrong side of the cap — the write-time octet
invariant that keeps a corpus from growing past what its own loader accepts.
The editor's verb is the correct one and now matches §The admin face ("one over
`MAX_CATALOG_DOCUMENT_OCTETS` … is the named `422`"). This is an improvement to
a performance-relevant clause, not a defect.

## Findings

None. No blocker, major, minor, or nit from the performance seat: the diff is a
language pass that introduces no new cost, no new bound, and no falsified
timing or budget claim, and it repaired the one clause whose wording could have
inverted the write-time document cap.

## FYI (no severity)

- §Budgets, `MAX_CONCURRENT_CONNECTIONS` hunk — the re-flow leaves an unfilled
  short line ("threads and ≤64 in-flight") inside the parenthetical, and Test
  items #9/#12/#13/#15/#16 shift their continuation indent from 5 spaces to
  3–4, so sibling items now hang at different widths. Renders identically in
  markdown; ragged source only. That is the editorial seat's surface, not a
  performance concern.
- The Revision log's r4 line about the post-`revision`-bump buffer and the
  `422` is the authority for #10's cap wording; a later pass that re-syncs the
  log could note the "would exceed" verb choice there. Not a finding.
