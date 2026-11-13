# Performance review r3c0

verdict: approve

## Scope

Ratification pass c0. Read only the `f752153..4d07da4` diff of `02-proposal.md`,
the full current document, `docs/adrs/009-canonical-profile-and-manifest-model.md`,
the Revision log, and my r3 findings.

What the diff changes, seat by seat: the Context line names the five contracts
1–5 in section order; the `§N` cross-references become `contract N`; the A16 row
swaps a dead citation for the live home-row article plus ADR-009's Context
observation; `resolve`'s pin-lookup sentence delegates to Id → file; the rest is
punctuation and "Test #N" → "Test strategy #N". None of it touches a cost, a
bound, an allocation, a read set, a cache, or a startup path.

## Checklist walk

1. **Per-event cost** — no input-event API in scope (Non-goals: "Any input-event
   API (injection, capture)"), and the diff adds none. No keystroke-path cost
   introduced.
2. **Structure costs** — chain resolution stays `BTreeMap`-ordered,
   `MAX_CHAIN_DEPTH` reused unchanged; the resolve-bullet rewording delegates the
   pin's index lookup to Id → file, which is already a string-keyed index lookup,
   so no new search cost and no O(n²) shape enters.
3. **Memory growth** — the bounded-read text is untouched: A25, the §3 walk
   sentence ("The walk is bounded by two new public constants beside the shipped
   ones — `MAX_INDEX_ENTRIES`, `MAX_INDEX_DEPTH`"), and the `schemas/README.md`
   guard-section row all read exactly as they did in r3. My r3 minor (the bound
   names the `data/profiles/` and `data/manifests/` walks, while `Ctx` is built
   from `data/placements/` and `data/policies/`, which the new
   data-homes-rule wording calls walked content) is neither addressed by this
   diff nor made worse by it — the sentence it lived in is unchanged, so it is
   not a regression and not newly caused. Under this pass's rule it is not
   re-reported.
4. **Startup cost** — per-document budget unchanged ("No new dependency, no
   per-document budget change, `MAX_CHAIN_DEPTH` unchanged"), the 1 MiB pre-read
   cap and per-file `parse_reader` byte cap unchanged, taxonomy and reach tables
   stay fixed-path loader reads.
5. **Caching/invalidation** — "The walk and drift suites build one `Index` per
   test binary and share it, so the gate is linear in files, not tests" survives
   the rewrite verbatim; the drift gate's invalidation is still a manifest
   `revision` bump caught in `cargo test`.
6. **Research baseline gap** — A24/A25 read exactly as in r3, both still marked
   `unverified` with their basis stated; no number changed, so no new gap.

Cross-reference integrity, which is what the diff's bulk is: every
`contract N` reference resolves to the section the Context ordering promises —
`contract 1` = Action taxonomy (emphasis), `contract 2` = input vocabulary /
authoring bar, `contract 3` = Profiles are chains (stem rule, chain-guard unit,
`MAX_INDEX_ENTRIES`), `contract 4` = Transforms (`apply_policy`'s `Err`),
`contract 5` = Manifests (drift mechanism). No `§N` reference is left dangling.

## Findings

None. Nothing in the revisions breaks a performance property approved in r3, and
no section cited in r3 is made false.

## FYI (no severity)

- A16's basis is now a citation claim about a live URL ("the QWERTY home keys are
  `ASDF` / `JKL;`") — citation liveness is not this seat's check; the ADR text
  it now also cites (`docs/adrs/009-…:28-31`) does carry the placement-split
  observation.
