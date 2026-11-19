# complexity review r1c3 (dissent pass on the r3 revision)

verdict: approve

Scope: `git diff 07e8b3f8 1e8f240c` (02-proposal.md blobs), then the full current
document, plus the Revision log and `docs/adrs/026-registry-service.md`.

## Prior findings — all four closed

- **major** (corpus cap enforced at load only) — **closed**, and by the mechanism I
  asked for: §The admin face and auth now serializes the post-upsert model and
  refuses over-cap as the named `422` before any tmp; §Budgets states the cap at
  "both ends of its life"; §Catalog document cross-refs the write side; #10 pins
  it. No per-record `source_keys` bound is needed any more, because the merged
  document is what is checked (each key is still bounded by §The serving face step
  1's rule, so total keys are bounded by the document cap). Residual ordering
  hole reported below as [minor][new].
- **minor** (per-connection request loop with no consumer) — **closed**: §serve.rs
  drops "covering that connection's whole request loop", §http.rs now closes the
  connection after "one answer per connection, echoing the shipped client's own
  `Connection: close` (A23)". A23 verified: `apps/discovery-cli/src/catalog.rs:438`
  builds the request with `Connection: close`, and `candidates()` (catalog.rs:408–446)
  opens a fresh `TcpStream::connect_timeout` per call and reads one response, so
  nothing in the shipped pair keeps a connection alive. The 64-slot idle-hold I
  flagged is gone, and "≤64 in-flight upstream hops" now follows from the text.
- **nit** (`--refresh-interval` unit) — **closed**: §CLI surface now says
  "`--refresh-interval <seconds>` — a bare unsigned count of seconds", and #6 runs
  `--refresh-interval 1`.
- **nit** (space in the path-safety refusal list) — **closed** as proposed: the
  clause is deleted from §The serving face step 1 and the Revision log records the
  reason ("traverses nothing and would `404` a key the shipped pair admits").

Checklist 1–6 re-run over the changed text: no new package/module/interface or
dependency (A23 is an assumption row, not a mechanism); the new write-time size
check reuses the existing §Budgets constant and the existing single write lock
rather than adding machinery; the revision deletes three things it had carried as
deferred ideas or duplicates (the persist-only-lock pointer, the Defaults block,
the "026-S1 mandates the keys" gloss), which is net complexity reduction.
Checklist 7: complexity still looks proportionate to what ADR-026 buys — one
crate, ten named modules, five private constants, two documents, no new deps.

## Findings

- [minor][new] §The admin face and auth — "the model with the upsert is then
  serialized whole, and one over `MAX_CATALOG_DOCUMENT_OCTETS` (§Budgets) is the
  named `422` … leaving model, `revision` and file untouched … Only then does
  `revision` bump and the tmp-and-rename persist run" — the buffer that is checked
  is not the buffer that is written: `revision` is a serialized field of the
  document (§Catalog document's `{version, revision, games}`; the ADR's `026-C3`
  "bumped by every mutation"), so the persisted document is the checked buffer
  plus whatever octets the bump adds, and at the boundary (≈16,777,215 octets, or a
  revision crossing a power of ten) an accepted upsert persists a corpus a few
  octets over the cap that its own loader then refuses — the exact state §Budgets
  now claims cannot happen ("so no accepted mutation can persist a corpus its own
  loader refuses"). The two clauses also disagree about whether the bump is inside
  the checked serialization: "the model with the upsert" reads as if it includes
  it, "Only then does `revision` bump" says it does not. Fix: one clause —
  serialize the *candidate* copy with the bump already applied, check that buffer,
  and have the persist write exactly that buffer, swapping the model (new record +
  new `revision`) in only after the rename; then #10 pins the boundary inclusive at
  the cap with the revision already incremented. No new mechanism: the cache
  document's §Budgets check has no such hole because `{version, entries}` carries
  no `revision`, which is what makes the corpus the one place needing the ordering
  stated.
- [nit][new] §Test strategy #10 — "an upsert whose serialized corpus would pass
  `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422` with model, `revision` and file
  untouched" — "would pass" reads as "satisfies the cap", which inverts the case;
  the document's own refusal voice is "over" (#1 "document over
  `MAX_CATALOG_DOCUMENT_OCTETS`", #5 "load over `026-B2` is refused"), and the same
  word is used in the r3 Revision-log line. A test engineer pinning #10 literally
  writes the acceptance boundary backwards. Fix: "would **exceed**
  `MAX_CATALOG_DOCUMENT_OCTETS`" here and in the Revision-log clause.
- [nit][new] §Deferred — "Lifting the five copied invariant blocks … into a shared
  crate so both binaries call one copy, which also gives
  `Source`/`spells_lookup_key` that shared home" — the r3 trim deleted this entry's
  tail, "and retires `apps/registry`'s `discovery` edge with its Windows-only
  `winreg` transitive", while §`apps/registry` (unchanged) still promises the
  `winreg` edge is "taken for that predicate and retired by the deferred lift
  below", a pointer to text that no longer makes the claim. The claim is true —
  `crates/discovery/Cargo.toml:39` carries `winreg = { workspace = true }` — so the
  promise, not the fact, is the casualty. Fix: restore the retired-transitive
  clause in the same §Deferred sentence (preferred; it is the only stated payoff of
  the lift beyond deduplication), or drop "and retired by the deferred lift below"
  from §`apps/registry`.

## FYI (no severity)

- §Budgets' new "serializes the cache and re-serializes after each dropped entry —
  one extra pass per eviction at most" is honest about pass count but not about
  pass cost (each pass re-serializes up to 1 MiB at write time); since one lookup
  adds one entry, the loop is 1–2 iterations in practice. Performance seat's call,
  not a complexity one.
- `MAX_REQUEST_BODY_OCTETS` = 16 MiB times `MAX_CONCURRENT_CONNECTIONS` = 64 is a
  ~1 GiB worst-case inbound buffer on a std-only server; unchanged text, but it
  becomes the load-bound number the performance seat may want to price now that the
  document cap is enforced post-merge as well.
