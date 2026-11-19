# complexity review r1 (dissent pass c1)

verdict: request-changes

Scope: the caller's blob diff `4a75e3d… 9ff2ec7` of `02-proposal.md`, then the
full current document. Checklist: `references/complexity.md` (reuse, Goal
traceability, smaller-design sketch, dependencies, deferral, knobs/one-member
abstractions, value). Inputs: `02-proposal.md`, `docs/adrs/026-registry-service.md`,
the Revision log, my own r1 findings, plus read-only inspection of
`crates/core/tests/{conformance.rs,common/mod.rs}`,
`crates/discovery/tests/boundary_tests.rs`, `crates/discovery/src/game.rs`,
`crates/discovery/Cargo.toml`, `apps/discovery-cli/{Cargo.toml,src/catalog.rs,src/matches.rs}`,
`data/**`.

## Previous-round findings — disposition

- **major #1 (source-key sweep missed by the harness list) — CLOSED, verified.**
  §Catalog document and its spec now names
  `crates/discovery/tests/boundary_tests.rs::the_shipped_corpus_carries_no_source_key_spelling`
  and re-scopes it to `data/manifests` + `data/profiles` with the doc-comment and
  the `checked > 0` guard re-pinned; the Options paragraph names both harness
  consumers. Checks: the sweep is the only `data/`-walking consumer in the repo
  (path-restricted grep for `join("data")`/`walk_files` returns that one test),
  and the re-scoped set still carries 2 files (`data/manifests/README.md`,
  `data/profiles/README.md`), so the re-pinned `checked > 0` holds.
- **major #2 (reuse language vs a bin-only crate) — CLOSED, verified.**
  `apps/discovery-cli` declares no `[lib]` and ships only `src/{main,catalog,matches}.rs`;
  the design now names `bindforge-core` + `discovery` + `serde_json` and lists the
  five copied invariants as copied-and-re-pinned, with one deferred lift. The
  dependency sentence is code-true (`Source`/`spells_lookup_key` are `pub` in
  `crates/discovery/src/game.rs`; the four core items cited are `pub`). Residual
  wording nit below.
- **minor #3 (unvalued thresholds) — CLOSED for the two named values**
  (`MAX_REQUEST_BODY_OCTETS` 65,536; `fetched_at` ∈ `[now − 31_536_000 s, now + 86_400 s]`).
  The same revision introduced a third unvalued constant: new finding C.
- **minor #4 (no refresh trigger) — CLOSED**, and the closing sentence creates a
  contradiction with the health section: new finding D.
- **minor #5 (knobs with no consumer) — HALF CLOSED.** `--local-entries` is gone
  and #13 now owns relocation through the env seam. `--refresh-interval` is
  still untested: new finding F.
- **nit #6 (conformance loop list) — HALF CLOSED**: the truncation loop is out and
  `is_fixture_name` gains the directory argument. The replacement text is
  self-contradictory about the fixture-name test: new finding E.
- **nit #7 (`EXTERNAL_JSON_FORMATS` + meta-test) — CLOSED**: one `const` naming
  `catalog`, and the Risks row follows it.

## Findings

- [major][new] §Budgets — "evicts the oldest entry by **insertion sequence**
  (ties by `fetched_at`, so a backwards clock jump cannot make fresh data the
  eviction target)" + "The in-memory cache holds each entry's serialized octet
  count with a running total … O(1) plus evictions, never a whole-document
  re-serialization" — the r1 rewrite changed the eviction key `026-B1` names:
  the ADR says "over the cap the oldest `fetched_at` is evicted"
  (`docs/adrs/026-registry-service.md`, 026-B1); here `fetched_at` is demoted to
  a tie-break. Test strategy #4 keeps the cache document "sorted by `slug` byte
  order", so after any restart the insertion sequence *is* slug order and
  eviction silently becomes alphabetical — an entry is discarded because its
  slug sorts low, not because it is old, and #5 as written pins that wrong key
  for `T-007`. And the machinery bought nothing: the same section says "The file
  is rewritten write-through after each lookup that adds an entry", so every
  write already serializes the whole ≤ 1 MiB document (≈ 16 fat entries, ≈ 90
  by this section's own realistic figure) — the "never a whole-document
  re-serialization" target is a cost the design pays anyway; the per-entry
  totals, the running total and the insertion counter (plus the tie-break that
  exists only to defuse clock jumps) are three bookkeeping parts traced to no
  Goal, and they make "provably inside both caps" rest on a sum of entry
  lengths, which is not the document's length. Smaller design that meets
  `026-B1`+`026-B2` unchanged: serialize the cache once at write time, take
  `bytes` from that buffer's length (exact, no per-entry totals), evict the
  oldest `fetched_at` until count and bytes both fit, and keep the load-time
  re-check; delete the running total, the per-entry octet count and the
  insertion sequence. Fix: replace the insertion-sequence clause with the
  ADR's `fetched_at` key and the write-buffer byte check as above, keep the
  backwards-jump guard where it already belongs (the load bound + "expired
  rather than immortal"), and change #5 to pin eviction by `fetched_at`.

- [major][new] §The serving face step 1 — "the registry's one key validator
  tightens it: after the first `/`, exactly one non-empty `.`-free component, no
  embedded `/` — the grammar that keeps a `..` or a separator from ever asking
  the filesystem for a path" + the same paragraph's "Lookup keys are opaque
  in-memory strings, never joined onto paths" — a second, registry-private
  spelling rule for keys the repo already validates in one place. The shipped
  validator is `Source::spells_lookup_key` + `len ≤ MAX_DISCOVERY_SLUG_OCTETS`
  with no character or dot rule (apps/discovery-cli/src/matches.rs:222-234 and
  282-283; `game.rs:84-89` admits any non-empty external id after a closed-set
  token), so `local/Steam.SE` is a legal key the user's own store keeps and the
  registry's fourth grammar site refuses at three seams (candidate query,
  upsert via §Catalog document and its spec, cache load via §Budgets). Its only
  stated purpose is path safety, which the next sentence disclaims — keys are
  never joined onto paths — and the cited precedent, `is_slug_name`, governs
  `game_id`, already bound by A4. So the added part is a private grammar with no
  consumer and a cost in contract divergence. Fix: keep the two shipped checks
  (`spells_lookup_key` + `025-B1`) as the registry's whole key rule, and if the
  path-safety defense is wanted keep only its actual scope in one clause — "a
  path component equal to `.` or `..`, or a further `/` after the first, is the
  `404`" — then drop the flat `.`-ban from step 1, from the corpus-load refusal
  list and from #3/#4's dot-bearing case; one spelling, shared, at every seam.

- [minor][new] §Budgets — "One saturated entry is `MAX_RESPONSE_OCTETS` (≈64
  KiB), so `026-B2` alone caps the working set at ≈16 fat entries (≈90 at
  realistic ~40-octet candidates) — accepted; #5 and `cached_slugs` read expect
  that, not 5,000." — the realistic band is off by an order of magnitude and no
  per-entry figure is shown. `A1` pins `MAX_RESPONSE_OCTETS = 65_536`
  (catalog.rs:23), so the 16-entry figure holds only for a body at the cap; a
  realistic entry — slug + `{game_id, name}` + `fetched_at` + JSON framing — is
  on the order of 10² octets, so ≈ 10³–10⁴ entries fit inside 1 MiB and
  `026-B1`'s 5,000 is the cap that binds for realistic answers, exactly as #5
  states. The sentence is load-bearing twice: it is what makes the ≈16-entry
  working set "accepted" and it tells the `T-007` test author what to expect
  from `cached_slugs`, where it contradicts #5's `count ≤ 5_000` in the same
  list. Fix: name the assumed per-entry octet figure and say which cap binds in
  which band (byte cap near `MAX_CANDIDATES`, `026-B1` for ordinary answers),
  and delete the "#5 … expect that, not 5,000" instruction.

- [minor][new] §Budgets — "`MAX_CONCURRENT_CONNECTIONS` (over-cap closed at
  accept, one thread per admitted connection)" (with `serve.rs` and Risks) —
  the only one of the five new constants with no value, while #11 pins behavior
  against it ("over-`MAX_CONCURRENT_CONNECTIONS` is closed at accept"). It is
  also the only bound on live thread count, and in caching mode each admitted
  connection may hold an upstream hop open for the mirrored `2 s`/`5 s`
  timeouts, so the pair sets the worst-case concurrent-outstanding-fetch count
  the design never states. §Budgets' stated job is that the budgets close
  against each other, and the task author has to invent the number. Fix: give
  the value in the same sentence (e.g. 64) and cite it in #11 by value, like
  the other four.

- [minor][new] §Caching mode — "probed **at most once per TTL**, fired lazily by
  the first lookup or health read after expiry on that request's own thread" vs
  §The serving face "`GET /v1/health` … Answering it reads the **last-known**
  upstream verdict … it does not perform a live upstream probe per call" — the
  r1 sentence that supplied the missing trigger made a health read a probe
  trigger, which is exactly what the health section promises it never is; the
  two sections now contradict each other on the one route the ADR pins
  (`026-C2`). Fix: drop "or health read" (lookups alone drive the probe; a
  health read only reports the last-known verdict), or move the health trigger
  into the health paragraph and qualify it as "at most one probe per TTL".

- [minor][new] §Catalog document and its spec — "guard the fixture-tree loops —
  `every_valid_fixture_…`, both `every_invalid_fixture_…` message tests, and
  `fixture_file_names_…` (which keeps governing the `.json` fixtures: lowercase
  dash-separated names, case-unique) — behind the core-TOML check so catalog
  fixtures are not parsed as TOML" — the bullet both guards the fixture-name test
  for catalog and says it keeps governing catalog's `.json` fixtures; those are
  opposite instructions. Verified in the shipped harness: that loop
  (`conformance.rs::fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique`)
  walks `format_dirs()` and asserts `is_fixture_name(name)` per file, so guarded
  = uncontrolled names, ungoverned; and `is_fixture_name` (common/mod.rs:421)
  hard-codes the `.toml` suffix with no format argument, so leaving it unguarded
  without the new argument fails the walk. Fix: one sentence per loop — the two
  parse/rejection loops are guarded for the `catalog` format, while
  `fixture_file_names_…` is **not** skipped and instead calls the
  directory-aware `is_fixture_name(dir, name)`, which is what keeps the name and
  case-uniqueness rule on the `.json` fixtures.

- [minor][re-raised] §CLI surface — "`--refresh-interval` overrides `026-B3`
  (default `REGISTRY_UPSTREAM_TTL_SECONDS = 3_600`)" — the r1 pass retired the
  unconsumed `--local-entries` knob but this one still has no consumer anywhere
  in the Test strategy: #6 pins fresh/stale/cold against an injected fake, #12
  pins health, #17 chains a caching instance, and none shortens the TTL. The
  knob itself stays — `026-B3` mandates it ("overridable by `--refresh-interval`
  at the operator's call") — so only the missing pin is open, and the same
  interval now also drives the TTL-gated health probe, which nothing observes.
  Fix: one clause in #6 — start caching with `--refresh-interval 0` (or 1 s) and
  pin that an entry that was fresh becomes stale on the next query and triggers
  the refresh — and note there that the override also re-times the probe.

- [minor][new] §Concurrency (open Q10) — "mutations take the write lock, mutate
  the model, and perform the atomic tmp-and-rename under a separate persist-only
  lock, so serving reads never wait on filesystem IO" — r1 added a third lock
  domain for a benefit no Goal or ADR unit states (the cache is bounded at ≤ 1 MiB
  and the corpus is a local file: a reader waiting on one local write is not a
  cost any Goal names), and the split decouples persist order from model order —
  two accepted mutations can reach the tmp-and-rename in the opposite order, so
  the file can end at the older revision while memory holds the newer, and after
  a restart an accepted mutation is gone; nothing in #10/#11 pins persist order,
  so it ships either way. Fix: perform the tmp-and-rename while holding the
  model write lock (the r0 wording, one lock, order guaranteed) and delete the
  persist-only lock; if read latency ever earns its keep it is a named
  refinement next to the copy lift in §Deferred, not a v1 lock.

- [nit][new] §The admin face and auth — "Enablement is judged from the address
  actually bound — the shipped `probe()` predicate `local_addr().ip().is_loopback()`,
  not a literal-spelling comparison" — the quoted expression is not the shipped
  one: `probe()` (catalog.rs:234-248) applies `is_loopback()` to each *resolved*
  address and `local_addr` appears nowhere in that function, so a `T-006` author
  going to the named precedent will not find it. Fix: "the same
  `std::net::IpAddr::is_loopback` test `probe()` applies to each resolved
  address, applied here to the serving listener's `local_addr()`".

- [nit][new] §`apps/registry` — "and on `discovery` for `Source` and
  `spells_lookup_key`" — the new dependency edge is for a two-line token
  predicate, and it pulls the whole engine crate into a serving binary, including
  its Windows-only `winreg` dependency (crates/discovery/Cargo.toml:38-39), which
  the registry never uses; the Compatibility matrix's `Windows | Yes` row says
  nothing about it. Reusing the authoritative predicate is the right call, so
  this is only a record-keeping ask. Fix: either say in that sentence that the
  dep is taken for the predicate alone and the transitive Windows cost is
  accepted, or name the alternative once next to the existing lift in §Deferred
  (give `Source`/`spells_lookup_key` the same shared home as the other copied
  blocks so `apps/registry` depends on `bindforge-core` only).

## Checklist notes

1. Reuse: the shipped `spells_lookup_key` + `025-B1` pair and the shipped
   persist/load discipline are the reuse candidates; the design now names them
   honestly as copies, but adds a rival key grammar instead of reusing the
   shipped one (major #2 above).
2. Goal traceability: the insertion-sequence/running-total machinery and the
   persist-only lock trace to no stated Goal (majors/minors above).
3. Smaller design: dropping the running-total bookkeeping for a single
   write-time serialize, and the persist lock for the write lock, removes three
   moving parts from caching and concurrency with no contract change; the whole
   rest of the plan (one binary, std-only, hand-framed HTTP, no new runtime deps)
   is already close to minimal.
4. Dependencies: `serde_json` (already shipped in the sibling binary) plus the
   `discovery` edge — acceptable, one nit above.
5. Deferrable half: caching mode (`T-007`) is already sequenced after root and
   admin, so the first merged slice stays thin. No finding.
6. Knobs/abstraction: `--refresh-interval` still unconsumed; the earlier
   one-member format registry is gone.
7. Value: proportionate overall — the added complexity concentrates in the r1
   Budgets machinery and the private key grammar, both of which are cheaper to
   remove than to build, test and keep honest against two ADR units.

## FYI (no severity)

- §The serving face step 2 moves join ordering into the load-time index
  ("postings are built already ordered by `game_id` byte order"), which makes the
  admin upsert/delete path responsible for keeping each posting list sorted;
  nothing in #2/#10 pins that, and the previous wording ("truncated to the first
  `MAX_CANDIDATES`") put the sort at request time where it is trivially correct.
  Correctness/testing territory.
- §Budgets sets `MAX_REQUEST_BODY_OCTETS` (65,536) against a body that is one
  whole `026-C3` record, while `026-C3` bounds neither `source_keys` count nor
  record size (verified: no such cap in core's exported constant family) — so a
  legal record between 64 KiB and the 1 MiB document cap loads from disk but
  cannot be upserted. Correctness territory; a per-record bound in
  `schemas/catalog/v1/spec.md` would close it.
