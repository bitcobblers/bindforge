# complexity review r1c2 (dissent pass: scope = `02-proposal.md` blob 9ff2ec7 → 07e8b3f, plus the full current document)

verdict: request-changes

## Scope and method

Diffed the two named blobs, then read the whole current document and its Revision log. Re-checked
each of my r1 findings against the revised text. Verified the design's claims about shipped code with
read-only source inspection only: `crates/discovery/src/game.rs:66-89` (`lookup_key` /
`spells_lookup_key`), `apps/discovery-cli/src/matches.rs:229,282`,
`apps/discovery-cli/src/catalog.rs:234-254` (`probe()`'s `addr.ip().is_loopback()`, line 248) and
`:438` (the client's request carries `Connection: close`),
`crates/core/tests/conformance.rs:337-352` + `crates/core/tests/common/mod.rs:421`
(`is_fixture_name` has exactly one call site, inside
`fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique`), and
`crates/discovery/tests/boundary_tests.rs:110-156` (the sweep's roots are `data/` recursively plus
every `schemas/*/v*/fixtures/valid`, with the `checked > 0` guard). No run record was used as review
input.

## r1 findings — disposition

All ten are answered in the changed text, and I found no regression among them:

- **major** eviction key + bookkeeping — closed. §Budgets now uses `026-B1`'s own oldest-`fetched_at`
  key, takes the octet test from the write buffer's length ("no bookkeeping of its own"), and deletes
  the per-entry octet count, the running total, the insertion sequence and the tie-break; #5 pins
  oldest-`fetched_at` and a written document ≤ 1 MiB; the clock-jump guard is kept and named as the
  clock-jump answer.
- **major** registry-private key grammar — closed at `step 1`, the corpus refusal list, the §Budgets
  cache-load clause, #3 and #4; `local/Steam.SE` is now an admitted spelling. One residual clause
  remains, filed below as a nit.
- **minor** realistic per-entry band — closed: ~150 octets, ~7,000 slugs inside 1 MiB, `026-B1` named
  as the binding cap for ordinary answers; the "#5 and `cached_slugs` expect that, not 5,000"
  instruction is gone and #5 agrees with it.
- **minor** `MAX_CONCURRENT_CONNECTIONS` valueless — closed: 64 in §Budgets and cited by value in #11
  and the §Risks row.
- **minor** health read as probe trigger — closed in all three places (§Caching mode, §The serving
  face, #12).
- **minor** fixture-name test guarded *and* governing — closed, and verified: `is_fixture_name` really
  has the single call site the new text claims, in the fixture-name test, so "not guarded + the
  directory-aware predicate" is the smaller correct change; the `boundary_tests` roots I re-checked
  match the "keep both roots, skip only `data/catalog/` + `schemas/catalog/`" wording.
- **minor** `--refresh-interval` with no consumer — closed by #6's clause (spelling nit below).
- **minor** persist-only lock — closed: one write lock through serialize→tmp→rename→swap, the lock
  deleted, the latency idea moved to §Deferred, and #11 pins the persisted-equals-model outcome.
- **nit** `probe()` quote — closed; the cited `catalog.rs:248` is the `is_loopback` line.
- **nit** `discovery` dependency / `winreg` — closed in §`apps/registry` and §Deferred.

## Findings

- [major][new] §Budgets — "`MAX_REQUEST_BODY_OCTETS` (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a
  loadable corpus could legally hold also fits an upsert and no per-record `source_keys` bound is
  needed)" together with "`MAX_CATALOG_DOCUMENT_OCTETS` (16,777,216 — ≈120,000 records …),
  `fs::metadata`-gated before the read" — the equivalence is this round's new decision, and it closes
  only one direction: the corpus's 16 MiB cap exists **at load only** (§Catalog document's refusal
  list, §Budgets' metadata gate, #1), while §The admin face lets accepted mutations grow that same
  document with no counterpart check. So a root can be driven past the size its own loader refuses and
  then not restart until the operator hand-edits the JSON — at exactly the scale the ADR projects
  ("tens of thousands of games, each with several store keys", the basis for the ≈120,000-record
  sizing), growth across the cap is foreseeable, not exotic. This is the identical reasoning §Options
  used to reject refuse-the-file-alone for the cache ("it does not stop a legal-but-large corpus … from
  exceeding 1 MiB **between writes**"), and the asymmetry is nowhere recorded as a decision; #10 and
  #11 pin revision and persist order, never size. The bound the design declined to add (a per-record
  `source_keys` count) is what the enlarged body cap is supposed to make unnecessary. Fix: reuse the
  mechanism §Budgets just defined rather than adding one — "the corpus persist applies the same write
  buffer check: a mutation whose serialized document exceeds `MAX_CATALOG_DOCUMENT_OCTETS` is refused
  by name, no rename" (the persist is already under the one write lock per §Concurrency), add that
  refusal to §Catalog document's list and one case to #11; if instead the asymmetry is deliberate,
  say so in one clause and restore the r1 per-record bound with the 64 KiB body cap so the loadable-
  corpus claim is not load-only.

- [minor][new] §`apps/registry` modules — "`serve.rs` — accept loop (one thread per admitted
  connection, covering that connection's whole request loop, up to `MAX_CONCURRENT_CONNECTIONS`,
  over-cap closed at accept…)" plus §Budgets' "one thread per admitted connection for that
  connection's whole life, so ≤64 live threads and ≤64 in-flight upstream hops" — the r1 text said one
  thread per connection; r2 adds a per-connection **request loop**, a mechanism with no consumer: the
  only client this serving face must satisfy sends `Connection: close` and one request per connection
  (`apps/discovery-cli/src/catalog.rs:438`, A19), and §http.rs, the response writer, never states
  whether a connection is closed after the answer. The added phrase also cuts against the bound it was
  written to state, since a keep-alive loop lets an idle client hold one of the 64 admitted slots for
  the connection's whole life (bounded only by the read timeout), which is when the "≤64 in-flight
  upstream hops" claim matters most. Fix: replace the phrase with "one request-response per admitted
  connection, then close, echoing the shipped client's `Connection: close`", and add that close to
  §http.rs — or, if keep-alive is intended, name it as such with the idle-slot timeout that bounds it,
  in the same sentence.

- [nit][new] §Test strategy #6 — "with the fresh/stale pair run under `--refresh-interval 1s`" (with
  §CLI surface's "`--refresh-interval` overrides `026-B3` (default
  `REGISTRY_UPSTREAM_TTL_SECONDS = 3_600`)") — the flag's value spelling is never defined, and the test
  this round added now depends on it: the budget is a count of seconds, so a `u64`-seconds parser
  rejects `1s` and #6's new case fails on a parse error rather than on policy. Fix: state the unit at
  the knob once ("`--refresh-interval <seconds>`") and write the case as `--refresh-interval 1`.

- [nit][new] §The serving face step 1 — "Path safety is the only extra refusal, at its own scope:
  after the first `/`, one non-empty component with no further `/` or `\`, none equal to `.` or `..`,
  and no control byte, space, or invalid-UTF-8 sequence" — the `space` item is the one entry in the
  list that is not path safety: it cannot traverse or split a path, and it cannot break the one-line
  refusal discipline the control-byte ban already protects (a space is not a separator, and keys stay
  "opaque in-memory strings, never joined onto paths" by the same sentence). It is also the clause
  that can still 404 a key the shipped pair admits — `spells_lookup_key`
  (`crates/discovery/src/game.rs:84-89`) and `Source::lookup_key` (`game.rs:66-68`) impose nothing on
  the external id, the shipped store checks only that pair plus `025-B1`
  (`apps/discovery-cli/src/matches.rs:229,282`), and the client puts any such key on the wire escaping
  a space as `%20` — where `local/<dir>`, the ADR's own example of the key the admin face exists to
  curate, is the natural home of a space-bearing id. Blast radius is small (nothing shipped emits one
  today), so nit. Fix: drop "space" from the list, or keep it and name in one clause the derivation
  law it enforces for `local/` keys.

## FYI (no severity)

- §Concurrency's new "A reader may wait on one local write" is written for the corpus persist; whether
  the `RwLock<Cache>` write lock is held across the upstream hop (the `2 s`/`5 s` fetch) is still
  unstated, and that is the path a caching-mode reader would actually stall behind.
- The body budget's move from 65,536 to 16,777,216 is a 256× larger inbound allocation on a plain-HTTP
  64-thread server; how a 16 MiB streamed body sits against the read timeout is a performance/security
  question, not this seat's.

## Value check

Proportionate. With the cache's three bookkeeping parts, the persist-only lock and the registry-private
key grammar all gone, the surviving machinery reads as one crate, one lock per document, one key rule
shared with the shipped pair, five named copies each pinned by a named test, and six constants with
values; the two open items above are one missing refusal and one phrase about a request loop, not new
subsystems.
