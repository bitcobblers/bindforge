# performance review r1

verdict: request-changes

Scope: whole document (full panel, c0). Inputs used: `02-proposal.md`,
`docs/adrs/027-hosted-registry-with-store-resolvers.md`, the Revision log.
`01-research.md` was not in this pass's allowed inputs, so checklist item 6
(quote the research baseline) is answered from the ADR's own budgets and the
proposal's numbers only; where the design cites "the research's Q5 finding" or
the 164k–200k band (A5) I take the numbers as the proposal states them.
Claims about the shipped code were verified with read-only inspection of
`apps/registry/src/{state,corpus,persist,constants,serve}.rs`.

## Findings

- [major][new] §Concurrency — "inside it, a writer takes its document's `RwLock`
  write half, mutates the model, serializes, tmp-and-rename, and swaps — the
  shipped single-lock persist posture, so persist order cannot diverge from
  mutation order." combined with "Readers (serving lookups) take document read
  halves only." — the mint path is placed on the *client-request* path while
  inheriting a cost model built for operator-frequency admin writes, and the
  lock scope makes every concurrent non-curated query wait behind it. Verified
  in the shipped code the design copies: `state.rs::upsert` holds
  `catalog.write()` across `apply_upsert` (a full `Catalog::clone()`),
  `Catalog::upsert` → `rebuild_postings()` (a full `O(records × keys)` index
  rebuild — `corpus.rs:280-291, 359-365`), `to_bytes()` (which first builds a
  `Vec<serde_json::Value>` DOM cloning every field — `corpus.rs:339-357`), and
  `persist::save` (write + `sync_all()` fsync + rename — `persist.rs:184-233`).
  The design then raises the document 4× (`MAX_ISSUED_DOCUMENT_OCTETS =
  67_108_864` against the shipped `MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216`,
  `constants.rs:27`) and sizes it at "~30 MiB" of live records, and the writer
  is now a first-time `steam/<appid>` key from any client, serialized on one
  `mutations` mutex with at most `MAX_CONCURRENT_CONNECTIONS = 64` threads
  deep. Consequences the document nowhere quantifies: (a) per-mint cost grows
  linearly with document size (clone + postings rebuild + DOM serialize + fsync
  of the whole file), so filling to `027-B3`'s 200,000-record cap is quadratic
  aggregate work — order 2×10¹⁰ element operations and terabytes of writes to
  one file (≈ ½ × 200k × 30 MiB); (b) while a mint holds the issued document's
  write half, policy step 2 cannot take its read half, so every query that
  misses the corpus stalls behind that fsync — and the corpus is the shipped
  six-record seed (A13, Rollout item 2), so on the hosted instance that is
  effectively all traffic, up to 64 deep, over-cap connections closed at accept
  with no status line (`serve.rs:37-46`); (c) the same mutex also serializes
  corpus admin mutations, so an operator upsert queues behind mint backlog.
  The §Risks row states the exposure ("every first-time key rewrites a
  multi-megabyte issued document under the mutation lock", Likelihood High) and
  then rests the mitigation on a false premise: "the wait is one local write,
  already an accepted `026` posture" — in `026` this rewrite path was reachable
  only through the authenticated admin face, never from `GET /v1/candidates`.
  There is also no input-path latency budget anywhere in the document
  (checklist item 1): the design budgets octets, pages, seconds and record
  counts but never a per-request or per-mint cost, so "the cap is an ordinary
  operating state" (own Risks row 2) and "one mint costs a whole-document
  rewrite" are asserted compatible without a number. Fix: keep `027-C4`'s
  persist-before-serve and the single `mutations` mutex, but narrow the lock
  scope — inside the `mutations` mutex (which is what `027-C4`'s "the same write
  lock that serializes admin mutations" names, and which is what preserves
  persist order) build the new document, serialize it and pay
  tmp-and-rename, and take the document's `RwLock` write half only for the
  mutate-and-swap; this replaces the sentence "the shipped single-lock persist
  posture, so persist order cannot diverge from mutation order" as the
  justification for holding the write half across file IO, and no reader ever
  waits on an fsync again. Pair it with the mint path's index update made
  incremental (insert the new key's posting rather than `rebuild_postings()`
  over all records) and the write serialized straight into the buffer instead
  of through a `Vec<serde_json::Value>` DOM, and put a measured figure in
  §Budgets — the issued-document rewrite cost at 100k and 200k records and the
  mint throughput that implies — so the §Deferred "batched issued writes"
  entry is a deferral against a number rather than against an assumption. The
  §Risks mitigation text "already an accepted `026` posture" must go, since it
  is the sentence doing the load-bearing work.

- [major][new] §The store pull / §Budgets — "Per page, up to
  `STEAM_PAGE_ATTEMPTS` (3) attempts separated by `STEAM_RETRY_SECONDS`; a page
  that fails all attempts fails the pull." with `STEAM_RETRY_SECONDS = 3_600` —
  the hourly retry separation is defined once and the pull rules are shared by
  the startup pull and the background refresh, so the startup path can hold the
  process from binding for hours on a transient store fault. The socket
  timeouts bound only the attempt itself (`STEAM_CONNECT_TIMEOUT = 2 s`,
  `STEAM_READ_TIMEOUT = 30 s`), and no overall pull deadline is stated, so
  `027-C3`'s "pulled in full before the process binds" plus the §The steam
  snapshot resolver rule "a present-but-unloadable snapshot that the pull then
  cannot replace is a named startup refusal" means one flaky page costs two
  one-hour sleeps before the operator learns anything, and the worst legal case
  is 12 pages × 3 attempts × 32 s + 12 × 2 × 3,600 s ≈ a day of silence on a
  host that is running `registry serve` and listening on nothing. The hourly
  cadence is right for the refresh thread's retry loop; it is wrong as an
  inter-attempt gap inside a startup path with no deadline. Fix: give the
  startup pull its own short inter-attempt backoff (seconds-scale, a named
  constant in the §Budgets family) plus one named overall pull deadline that
  converts an unfinished pull into the existing named startup refusal, and
  restrict `STEAM_RETRY_SECONDS` to the refresh thread's retry-after gap — the
  "separated by `STEAM_RETRY_SECONDS`" clause becomes the named startup
  backoff, and no startup path keeps a 3,600 s sleep.

- [minor][new] §Issued entries / §The steam snapshot resolver — "`<data>/bindforge/registry/issued-v1.json` is a
  `026-C3` catalog document — same shape, same loader refusals (via the same
  `corpus.rs::Catalog` type)" and "`MAX_ISSUED_DOCUMENT_OCTETS = 67_108_864`,
  sized for `027-B3`'s 200,000 records at typical ~150-octet size (~30 MiB …)"
  — the design budgets each document's bytes separately but never totals the
  process, and the load discipline it reuses costs several copies of the file
  at once. `corpus.rs::load_at` reads the whole file into a `Vec<u8>`, parses
  it into a `serde_json::Value` DOM, and then `from_document` clones each
  record twice into the `BTreeMap` and rebuilds the postings
  (`corpus.rs:186-275`) — file bytes + DOM + records + postings live together.
  The worst-case boot reads corpus ≤ 16 MiB + issued ≤ 64 MiB + retired ids
  ≤ 32 MiB + snapshot ≤ 32 MiB of JSON, then a single mint adds a full model
  clone, a DOM of it, and the post-`to_bytes` buffer on top. Every document is
  individually bounded, so memory growth (checklist item 3) is finite, but the
  design states no peak-resident posture or startup time bound to match the
  octet discipline it applies everywhere else. Fix: one sentence in §Budgets
  carrying the aggregate figure — the sum of the four document caps as the load
  ceiling and the model-clone-plus-buffer multiple a mint adds — so the
  operator-facing capacity number exists and the `027-B1`/`B3` budgets are read
  as bytes-on-disk rather than as memory.

- [minor][new] §Issued entries (retired ids) — "**Every** delete — issued or
  corpus — appends the removed record's `game_id` there before the mutation's
  persist." with `MAX_RETIRED_IDS_OCTETS = 33_554_432` and the file "in byte
  order" — the bookkeeping document's cost per mutation and the mint's
  membership test are both left unspecified, and the shipped shape makes each
  one a full-document event: an `O(n)` insert-and-resort of a byte-ordered list
  plus a second whole-file tmp-and-rename inside the same mutation-lock hold as
  the issued (or corpus) document rewrite — so an issued `DELETE` pays two
  multi-megabyte fsyncs under the mutex, a corpus `DELETE` newly inherits the
  retired-file rewrite it did not have in `026`, and the mint's
  "held-set is `games ∪ retired` across both catalog documents" probe is given
  no structure while the snapshot's join is explicitly given one ("a sorted
  vector plus an appid index"). Frequency is admin-level so impact is bounded.
  Fix: name the in-memory shape (a set with ordered insert, membership by
  binary search — the same affordance the snapshot sentence names) and state in
  one clause that a delete's cost is two bounded document rewrites, admin-only.

- [nit][new] §Budgets — "A6's daily allowance is ~6,000× the worst pull — no
  separate quota machinery" — the ratio is measured against a successful
  12-page pull, but by this design's own rules a worst pull is
  `STEAM_PAGE_ATTEMPTS` (3) attempts on each of 12 pages = 36 calls, and the
  refresh thread's hourly retry loop can spend 24 × 36 = 864 calls/day against
  A6's 100,000. The conclusion survives (≈ 115× inside the allowance even
  there), the factor of ~2 does not. Fix: restate the figure against the
  attempt-shaped worst pull (36 calls, ~2,800×) and note the retry-loop
  daily spend, so the "no quota machinery" conclusion is arithmetic the reader
  can reproduce.

## Checklist walkthrough

1. Input-path latency budget — **absent**; folded into the first finding (the
   design budgets every other axis but never per-request or per-mint cost).
2. Structure costs / O(n²) — first finding: `rebuild_postings()` per mint plus
   the whole-document rewrite makes fill-to-cap quadratic in the design's own
   `027-B3` numbers.
3. Memory growth — bounded per document; aggregate peak unstated (third
   finding). No unbounded cache: the snapshot, issued and retired documents all
   carry caps, and there is no per-key cache to invalidate.
4. Startup cost — the pull's retry cadence (second finding) and the untotalled
   document reads (third finding). The metadata-gated, read-before-parse rule is
   correctly carried over (A2, `persist::read_capped`) and is not a finding.
5. Cache invalidation — largely not applicable: `027-L1` deliberately removes
   the caching face, and the snapshot's "invalidation" is the age bound plus the
   `RwLock` publish-after-rename, which is stated and coherent. One point in the
   design's favour worth recording: the delta merge "insert keeping ascending
   order" over a 200k vector is an `O(n)` memmove per merged app at
   once-a-day frequency — inside budget, no finding.
6. Research baseline — cannot be quoted (out of this pass's inputs). The
   numbers the design quotes against it (A5's 164k–200k band against the 12 ×
   50k = 600k page budget) are internally consistent; the missing research
   cross-check is recorded here as missing evidence, not as a finding, since the
   first finding stands on the shipped code and the ADR's own budgets alone.

## FYI (no severity)

- §Issued entries routes the issued document through "the same
  `corpus.rs::Catalog` type": the load API already takes the octet cap as a
  parameter (`corpus.rs:170-184`), so a 64 MiB cap works, but
  `MAX_REQUEST_BODY_OCTETS` is defined as `MAX_CATALOG_DOCUMENT_OCTETS`
  (`constants.rs:29-32`) — an implementer reaching for "the catalog document
  cap" would silently bind the wrong number. Complexity/correctness seat's
  call whether the design owes a sentence.
- §Issued entries' retired-ids sentence says the append happens "before the
  mutation's persist", but the shipped persist shape is one buffer per
  document per mutation; a delete now spans two documents, so "in one
  mutation-lock persist" (§The curation face) is not a single save as written.
  Ordering/durability question for the correctness seat.
- §The steam snapshot resolver's "An out-of-order file is refused, not
  resorted" is a load-time `O(n)` check over 200k entries — cheap and correct;
  no performance objection, recorded only because the ADR's `027-C3` does not
  demand the refusal.
