# correctness review r2 (dissent pass c2)

Scope: `git diff 9ff2ec7 07e8b3f` on `02-proposal.md`, then the full current
document. Inputs: `02-proposal.md`, `docs/adrs/026-registry-service.md`, the
Revision log, my r1 findings. Checks run against the code with read-only
inspection only (`crates/discovery/src/game.rs`, `crates/discovery/Cargo.toml`,
`crates/discovery/tests/boundary_tests.rs`, `crates/core/tests/conformance.rs`,
`crates/core/tests/common/mod.rs`, `crates/core/tests/slug_bound_corpus.rs`,
`crates/core/tests/version_gate.rs`, `crates/transforms/tests/common/mod.rs`,
`apps/discovery-cli/src/catalog.rs`). No write leg was run; no other run record
was opened.

verdict: request-changes

## Findings

- [major][new] §Budgets / §The admin face and auth — "`MAX_REQUEST_BODY_OCTETS`
  (= `MAX_CATALOG_DOCUMENT_OCTETS`, so any record a loadable corpus could legally
  hold also fits an upsert and no per-record `source_keys` bound is needed)" with
  "`MAX_CATALOG_DOCUMENT_OCTETS` (16,777,216 — ≈120,000 records …)" and §Catalog
  document and its spec's "a document over `MAX_CATALOG_DOCUMENT_OCTETS`
  (§Budgets)" placed **only** in the loader's refusal list — the corpus document
  cap has no write-time rule anywhere in the design, while the cache cap now does
  ("after a write the registry serializes the cache … until the buffer is ≤
  `026-B2`"). An accepted `POST /v1/admin/games` may therefore persist a corpus
  whose serialized length passes 16,777,216, and the next start refuses the whole
  corpus by name — every accepted mutation on it is lost, which falsifies the
  sentence this revision just added to §Concurrency ("a restart cannot lose an
  accepted mutation (#11)"). Two reachable paths, both newly opened by this
  revision: (a) r2 raised the request body cap from 65,536 to the document cap,
  so a *single* legal upsert — `026-C3` bounds each `source_key` by `025-B1` and
  the design now names no per-record bound, so a record carrying ~130k keys is
  inside both rules — moves an empty corpus past the cap in one accepted request;
  (b) r2 raised the document cap to the ADR's "tens of thousands of games" scale,
  which makes a root corpus *living at* the cap a supported configuration (under
  the r1 numbers a corpus at 1 MiB was already an error), so ordinary curation
  across the line is the normal operator path. `026-C4` requires "every accepted
  mutation bumps `revision` and persists atomically"; the design never says what
  `POST` answers when the resulting document would be unloadable, so T-006
  guesses and no pinned test (#10's "upsert validates + bumps `revision` + atomic
  rename", #11's "over-cap body is refused before any mutation") sits on the
  seam — the body cap is a per-request bound, not a resulting-document bound.
  Fix: one clause in §The admin face and auth replacing "validated against
  `026-C3` and refused by name; bumps `revision`; persists atomically" with
  "validated against `026-C3` and refused by name; the write is then completed by
  serializing the whole document and refusing, by name, `MAX_CATALOG_DOCUMENT_-
  OCTETS` as a `422` that leaves `revision` and the file untouched; only then
  `revision` bumps and the tmp-and-rename runs (the loader's over-cap refusal
  stays, as the same rule's read side)", add that refusal to
  `schemas/catalog/v1/spec.md`'s roster alongside the record-level ones, and pin
  it in #10 as "an upsert whose serialized corpus would pass
  `MAX_CATALOG_DOCUMENT_OCTETS` is the named `422`, model and `revision`
  unchanged, no tmp written".

- [nit][new] §Budgets — "after a write the registry serializes the cache once and
  drops the oldest-`fetched_at` entries … until the buffer is ≤ `026-B2` octets
  and the entry count ≤ `026-B1`, then renames that buffer into place, so the
  octet test is the document's exact length and not a sum of per-entry totals" —
  the two halves pull apart: measuring "the document's exact length" after each
  drop needs a re-serialization per eviction, and the alternative — subtracting
  each dropped entry's length from one buffer — is the "sum of per-entry totals"
  the clause disclaims. Fix: replace "serializes the cache once" with "serializes
  the cache, and re-serializes after each dropped entry (at most one extra pass
  per eviction)", keeping the rest of the sentence.

- [nit][new] §The admin face and auth → Concurrency — "a persist-only lock to cut
  that latency joins §Deferred's lift if it ever earns its keep" — §Deferred's
  lift entry is "Lifting the five copied invariant blocks … into a shared crate"
  and carries no such lock, so the deferred mechanism is pointed at a list that
  does not contain it (and the lift is a code-sharing refactor, not a latency
  item). Fix: name it where it is deferred — add "and, if #11's N-upsert latency
  ever matters, the persist-only lock §Concurrency deletes" to that §Deferred
  bullet — or drop the pointer and state the reader-waits-on-one-local-write
  latency as accepted.

- [nit][new] §Catalog document and its spec, harness list — "`fixture_file_names_-
  are_lowercase_ascii_dash_separated_and_case_unique` is **not** guarded … which
  is the only coverage behind the Windows `Yes` row's case-insensitive-filesystem
  claim (#19)" — the Windows row of the Compatibility matrix makes no
  case-insensitivity claim in either its r1 or r2 form ("`<data>` = `%APPDATA%`
  … owner-only modes are a documented no-op … cache and local-entries paths …");
  the case-collision claim is #19's ("**Windows** and **macOS** behavior is
  proven by the env-closure seam (#13) and case-unique fixture names"), so the
  rationale cites a matrix claim that is not there. Fix: say "the only coverage
  behind #19's case-unique-fixture-name claim for a case-insensitive filesystem",
  or add the case-collision note to the Windows row so the pointer lands.

## Closures verified (r1 findings of this seat)

- Concurrency (major): closed for real — the persist-only lock is deleted and the
  persist runs inside the one write lock, with the false "serving reads never wait
  on filesystem IO" promise replaced by "A reader may wait on one local write",
  and #11 gains the N-concurrent-upserts pin the r1 fix asked for. `026-C4`'s
  "every accepted mutation bumps `revision` and persists atomically" now holds
  against mutation loss *between* mutations; the open case is the document-cap
  loss above.
- Compatibility matrix (minor): closed — row 1 is `Linux`, row 4 carries the
  `No`, and the closing sentence names the three `Yes` rows "and no other Unix".
- Health/probe (minor): closed — only lookups fire the probe, "a health read only
  reports the verdict as it stands, it never fires the probe", #12 re-pinned; with
  that, #12's `down`/`last_upstream_fetch: 0` case is no longer order-dependent.
- Eviction key (minor): closed — `026-B1`'s own oldest-`fetched_at` is restored,
  so the ADR divergence and its unrecorded-annotation half of the finding are
  moot; the clock-jump handling is relocated to the `fetched_at` load bound plus
  the TTL guard.
- Corpus cap sizing (minor): closed — 16,777,216 with the ~140-octet arithmetic
  shown and required in `schemas/catalog/v1/spec.md`; the quoted ADR scale
  ("tens of thousands of games, each with several store keys") is verbatim, and
  16,777,216 / 140 ≈ 119,800 ✓.
- Body cap sizing (minor): closed as to the sizing claim — with the cap equal to
  the document cap, any record a loadable corpus can hold does fit an upsert
  (a record inside a ≤-cap document is smaller than the cap), and the
  `corpus.rs`-vs-`POST` disagreement is gone. It reopens a different seam, which
  is the major above.
- `MAX_CONCURRENT_CONNECTIONS` (minor): closed — 64 with the thread and in-flight
  hop story, and the over-cap answer is now checkable: I verified the shipped
  client's `read_response` returns `CatalogError::unreachable("the connection
  closed before the header block ended")` on an immediate close
  (`apps/discovery-cli/src/catalog.rs:290-294`), so "closed at accept with no
  status line, which the shipped CLI reports as its named unreachable" is true.
  (#17's "`catalog unreachable`" for a cold `503` is likewise consistent: the
  non-200 path is `CatalogError::unreachable` too.)
- Key grammar (minor): closed — `spells_lookup_key` is shape-only
  (`crates/discovery/src/game.rs:84-89`: closed token + non-empty remainder, so
  `local/Steam.SE` passes and `steam/a/b` passes, which is exactly why the
  further-separator and `.`/`..` refusals stay), the dot ban is gone with the ADR
  citation, and control byte / space / invalid-UTF-8 are refused; #3 pins both
  the admitted dotted id and the control-byte `404`.
- Repeated `source_key` within a record (minor): closed — added to the `corpus.rs`
  refusal list with its `MAX_CANDIDATES` rationale and to #1's roster.
- Harness guard split (minor): closed and verified — `is_fixture_name`'s single
  call site is inside `fixture_file_names_are_lowercase_ascii_dash_separated_and_-
  case_unique` (`conformance.rs:348` in the test opening at :337), so "the one
  call site is in the fixture-name test below" is true, and the guarded set's
  `every_valid_fixture_…` prefix covers
  `every_valid_fixture_re_serializes_to_a_semantically_equal_value` (:70)
  alongside both `every_invalid_fixture_…` loops (:198, :283); the zero-fixture
  rule (:220) and `the_walk_picks_up_every_version_directory_…` (:404, subset
  assertion) survive a new catalog directory unchanged.
- `boundary_tests` re-scope (minor): closed as asked — verified the sweep's roots
  are `data/` plus every `schemas/<format>/v*/fixtures/valid`
  (`crates/discovery/tests/boundary_tests.rs:115-130`), so "keep both roots and
  skip only files under `data/catalog/` and `schemas/catalog/`" costs the gate no
  other tree and leaves `checked > 0` intact.
- Options-considered inversion (nit): closed — "dodges only the conformance walk".
- `probe()` predicate citation (nit): closed and verified — `addr.ip().is_loopback()`
  per resolved address at `apps/discovery-cli/src/catalog.rs:248`, now cited as
  the precedent for `IpAddr::is_loopback` on `local_addr()` and marked "the
  registry's own judgment, not one of the five copied blocks". The `winreg` claim
  is also true: `crates/discovery/Cargo.toml` carries `winreg` under
  `[target.'cfg(windows)'.dependencies]`.

## FYI (no severity)

- Body cap = document cap means 64 admitted connections may each have up to
  16,777,216 octets buffered, against a 65,536-octet outbound cap — the memory
  and back-pressure story of that pairing belongs to the performance and security
  seats, not to correctness.
- `crates/core/tests/conformance.rs:395-403`'s doc-comment for
  `the_walk_picks_up_every_version_directory_the_contract_tree_ships` asserts "a
  new `schemas/*/v*/` directory is picked up with no harness change" — the
  assertion is a subset check and stays green, but the comment states the
  opposite of this design's carve-out; the harness list could name the comment
  alongside the loops it edits.
- Nothing in this pass's changed text re-opens a Deferred item, and I found no
  Deferred entry contradicting ADR-026.
