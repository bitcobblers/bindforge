# correctness review r1 (panel pass c0)

verdict: request-changes

Scope: whole of `02-proposal.md`. Inputs: the proposal, ADR-009, its Revision
log, 01-research.md, and the cited repository basis for the assumption
register. No prior correctness findings exist (first pass).

Checklist notes:
1. Assumption register — spot-checked the verified rows against their cited
   basis: A1–A10 (schemas/README.md, profile + reach-classes spec.md,
   toml.rs, document.rs, chain.rs, conformance.rs, Cargo.toml files),
   A12/A13 (toml.io v1.1.0), A14 (RFC 6902), A19 (PCGamingWiki Returnal).
   All spot-checked bases support their rows. A21 and A24 are marked
   unverified and the body defers accordingly — no unverified assumption is
   stated as fact.
2. URL spot-check (anti-hallucination gate) — https://toml.io/en/v1.1.0
   (live; bare-key charset, dotted-key→nested-table, nested arrays all as
   claimed), https://www.rfc-editor.org/rfc/rfc6902 (live; sequential
   op-order as claimed), https://www.pcgamingwiki.com/wiki/Returnal (live;
   "Left-Handed and Right-Handed presets" + axis inversion as claimed).
   pubmed.ncbi.nlm.nih.gov/25029899 and slsa.dev/provenance/v1 could not be
   fetched (fetch proxy 403, origin not reached) — not evaluable, not
   contradicted; their rows' claims are modest and match 01-research.md.
3. Internal contradictions — findings 1, 2, 3, 5.
4. Edge cases — empty sets, name collisions (DuplicateId, GeometryConflict,
   AxisKindConflict, MergeConflict), absent platform, escape-carrying ids
   (verified the shipped escaped-strings.toml fixture, two ancestors as
   claimed) are all covered; the one gap is finding 4.
5. Interfaces — one fact / one name holds for based-on↔derives[0],
   version↔revision, the CoreError variant list, and the registry; the
   breaks are findings 1–3 and 5.
6. The design decides all five ADR-009 contracts, the crate shape, the chain
   algebra, and the drift mechanism; the one ADR deviation (serde) is stated
   deliberately, not silently.
7. Revision log — every r0 entry was checked; no rejected finding is
   re-raised. Finding 1 is not in the log: the log records the round-trip
   split's intent, not its mechanism, and the mechanism is what fails.

## Findings

- [major][new] Chosen design → Encoding rules (Round-trip split) + Test strategy #1 — "pure lookup data (`taxonomy`, `reach-classes`, `placement`) is asserted by load, using the exemption `reach-classes/v1/spec.md` already states" — the shipped harness grants that exemption only to the `reach-classes` format: `conformance.rs:63` gates it on `dir.format == "reach-classes"` and every other format's valid fixtures go through parse→serialize→re-parse→equality (conformance.rs:47-78, and A8 as verified states exactly that). So taxonomy, placement, and policy fixtures are re-serialized by the unmodified harness; a fixture authored under the exemption this rule claims for them (e.g. key order the serializer does not preserve) fails the harness, contradicting Goals ("machine-checkable by the existing harness with no harness change") and Test strategy #14 ("requires no change to `crates/core/tests/conformance.rs`"). The split also leaves `policy` in neither half, although the harness round-trips it and it is a person-edited document. Fix: in the Encoding rules and Test strategy #1, state that `taxonomy`, `placement`, and `policy` fixtures must re-serialize to an equal value (as the harness enforces today for every format except reach-classes), and reserve "asserted by load" for the seed data files (`actions.toml`, `left.toml`/`right.toml`), which live outside `fixtures/` and are load-checked by the model tests. This replaces the claimed exemption mechanism; it adds nothing.
- [minor][new] Chosen design → Manifests (`validate`) + Test strategy #9 — "`validate(manifest, &Taxonomy)` enforces: … a profile's `seeded-from` names a declared preset" — the function's declared inputs are the manifest and the taxonomy; a profile's `seeded-from` is a fact about a *profile* against the manifest, so the rule as attributed cannot be evaluated by this signature. Test strategy #9 repeats the mismatch by folding the seeded-from case into "manifest validation" with "its own fixture". Fix: move that one condition out of `validate(manifest, &Taxonomy)` and into the chain-resolution/drift step — "`resolve` rejects a `seeded-from` the pinned manifest does not declare, naming the profile id" — and mirror the sentence in Test #9; it adds no mechanism (resolve already loads the pinned manifest).
- [minor][new] Chosen design → Encoding rules (`version` bullet) vs Profiles are chains and Manifests — the canonical rule allows a document revision of "dotted `<major>.<minor>[.<patch>]` string", while the profile section pins "a `<major>.<minor>` string" and the `for` pin validates its revision half as "a dotted `<major>.<minor>`". A manifest declaring the permitted patch component (`revision = "0.1.2"`) can never be named by a `game@revision` pin, so the same fact has two shapes and one is unpinnable. Fix: in the Encoding rule drop the `[.<patch>]` option — the v1 revision grammar is `<major>.<minor>` for every document that can be pinned — replacing the current sentence.
- [minor][new] Chosen design → Transforms (`relocate-cluster`) — "the matched actions' current inputs form a contiguous cluster for `<hand>` in `base`" — "contiguous cluster" is a predicate no section defines over the data the design ships (reach tables carry class+row/col; `placement::validate` computes isometry and reach bound). The Options section rejects a review-only legality claim precisely because "two implementations could disagree"; the same argument applies to an undefined precondition predicate that decides apply-vs-`Skipped`. Fix: define the precondition in existing data — the matched actions' current input set must be isometric (same offset vectors) to the named placement's source-key set, else `Skipped` — replacing the word "contiguous".
- [nit][new] Chosen design → Profiles are chains (Chain guard) and the Risks row — "32 links is a base plus 31 patches" — the shipped `resolve_chain` rejects only when `derives.len() > MAX_CHAIN_DEPTH` (chain.rs:27), so the accepted maximum chain is 32 derives links = 33 documents, a base plus 32 patches; "links" must mean derives entries for Test strategy #5's boundary ("exactly `MAX_CHAIN_DEPTH` links resolves and one link deeper is `ChainTooDeep`") to be true, which makes the "base plus 31" phrasing off by one. Fix: state the unit once — `MAX_CHAIN_DEPTH` counts `derives` links; the maximum chain is a base plus 32 patches (33 documents) — and carry it into the Risks row.

## FYI (no severity)

- The design deviates deliberately from ADR-009's "(serde-based)" parenthetical
  (Options, with A9/A10 basis). The deviation is stated, not silent — but
  ADR-009 is `status: proposed`, so the ADR's own text should be reconciled
  with the accepted design at acceptance time. Cross-document concern, not
  this seat's.
- The seed `wasd-to-oksemike` placement maps four keys (W/A/S/D); the OKL;
  cluster's fifth key (`keyboard:semicolon`) appears in no shipped placement.
  A content choice for the seed slice, noted for the data-review slices.
- `Ctx { taxonomy, … }` leaves the `taxonomy` field untyped while its siblings
  carry types; cosmetics for the implementation pass.
