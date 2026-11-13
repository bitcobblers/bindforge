# complexity review r2

verdict: approve

## Scope

Ratification/confirmation pass, round 2, editorial seat diff only:
`docs/designs/014-profiles-as-chains/reviews/design/editorial-r2.diff` (13 hunks),
read against `02-proposal.md`, `docs/adrs/014-profiles-as-chains.md`, the
proposal's Revision log, and my own prior findings (none — first complexity
pass on this text). Inputs limited to those named by the caller.

## Checklist walk

1. **Reuse (`tools/`, existing code).** No hunk adds a package, module, or
   interface. The diff touches prose inside decisions that already name their
   reuse (`check_dir_version` in "Where ancestor documents come from", the
   shipped pair encoding, `ActionId::parse`/`InputId::parse`, the slug grammar,
   the `reach_classes_entries` neighbour for `profile_entries(version)`). No new
   surface to check against the repo.
2. **Every moving part traces to a Goal.** The diff introduces no moving part:
   all hunks are punctuation, word choice, or line reflow. Nothing new to trace.
3. **A 30–50% smaller design.** Unchanged by this diff — the structural choices
   it touches (single-directory rule, `refuse` on v1 links, one new cap, no
   aggregate cap) were already the thin answer; the editorial pass did not
   re-inflate any of them.
4. **New dependencies.** None, and the diff does not touch the dependency claim
   ("std-only, no new dependency, `Cargo.lock` byte-unchanged").
5. **Deferred second half.** The one Deferred hunk — "the matrix stays
   deferred exactly as design 013 left it — needed before the first pinned
   chain ships" — keeps the same deferral boundary as before the pass
   (`needs one` → `needed`). Consistent with the ADR, whose "Not decided here"
   list and Consequences impose no CI obligation on this design; the load-time
   pin-equality refusal covers the ADR's named re-pinned-base cost. Not
   re-opened.
6. **Knobs with no consumer / one-implementation abstraction.** The only
   budget text the diff edits is "one per link — totalling at most
   `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB)". "totalling" binds the
   product to the documents opened; it adds no constant and no consumer.
   `MAX_PATCH_DELTA_ELEMENTS` and `MAX_GENERATED_BY_OCTETS` are untouched here
   and both have consumers (the delta-list cap, the `generated-by` grammar row).
7. **Value check.** Proportionate, and this pass moved it in the thinning
   direction: it removed a stray space from `forged-log-line.toml` (the fixture
   name in Test strategy 7 now matches the corpus listing in "The contract"),
   and it made the fixed-list-order sentence state the rule as a verb
   ("`set` → `clear` → `swap` applies per patch") instead of a dangling
   fragment. No complexity added anywhere in the diff.

## Findings

None. No hunk changes technical meaning, so no previously-closed theme is back
and no section cited earlier was made false.

Verified non-changes worth naming for the reviser's confidence: "Chosen: refuse"
(line 19 of the diff) preserves the single-major answer; "An absent file is
`ChainLinkMissing` naming the id" preserves the A18 named refusal and the
id↔name equality rule; the `merge` reflow keeps the non-representability
refusal and the identity-cancelling case identical to Revision log r1; the
Test strategy 5 reflow keeps the same assertions.

## FYI (no severity)

- Cross-file loading — "a shell that `load_chain` refuses as
  `patch-empty-deltas` until the caller fills one element" — the inserted
  `that` leaves the clause without a main verb (fragment). Meaning is
  recoverable, so it is a language regression from this editorial pass, for the
  editing seat, not a complexity finding.
- Cross-file loading — "An absent file is `ChainLinkMissing` naming the id" —
  grammatically a file is equated with an error variant; "an absent file refuses
  as" would match the phrasing used everywhere else. Language, for the editing
  seat.
- Chain operations — the reflowed `merge` line
  "over every action either touches — exactly, not heuristically — and" runs
  past the wrapping used elsewhere in the document; formatting only.
