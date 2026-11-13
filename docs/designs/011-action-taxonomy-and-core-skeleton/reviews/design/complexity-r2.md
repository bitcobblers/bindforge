# complexity review r2

verdict: approve

Scope: `docs/designs/011-action-taxonomy-and-core-skeleton/reviews/design/editorial-r2.diff`
(the round-2 editorial pass), read against `02-proposal.md`,
`docs/adrs/011-action-taxonomy-and-core-skeleton.md`, and the Revision log.
No previous complexity findings were carried into this pass.

## Checklist evaluation

1. **New package / module / interface, reuse** — no hunk adds one. The diff
   touches Context, Goals, Options considered, one sentence of The model's error
   rule, Placeholders and lineage, and Test strategy #1. The design's moving
   parts (`schemas/action-ids/v1/`, `crates/core/src/action.rs`, two test
   targets, two READMEs) are untouched by this pass, and the proposal already
   grounds each of them in an existing in-repo analogue (`load_reach_tables`,
   `parse_file`, `conformance.rs`'s `schemas/*/v*/` walk, the `profile/v1`
   round-trip) instead of a new one. Nothing reusable is being re-invented.
2. **Every moving part traces to a stated Goal** — the diff adds no moving part.
   The only sentence that describes behaviour ("with an offending id echoed,
   escaped to printable characters") is the Revision-log r1 closure for the
   error rule, re-worded, not widened: still one existing `CoreError` path, no
   new escape mechanism named.
3. **A 30–50% smaller design** — unchanged by this pass; the shippable thinning
   (registry as data over a Rust literal or a `build.rs` table) is already
   decided in Options considered and the diff leaves it intact.
4. **New dependencies** — the diff changes none; the Compatibility matrix's
   "std-only, no new dependency, `Cargo.lock` byte-unchanged" and Rollout 2's
   "no `Cargo.lock` change" stand as written.
5. **Defer half so the first slice stays thin** — unchanged; the second half
   (emphasis, polarity flips, manifest validation, codecs) stays deferred to
   013/015/017 exactly as the ADR's "Not decided here" clause allocates it.
6. **Knobs with no consumer, abstraction with one implementation** — the diff
   adds no knob and no layer. The one scope-bearing reword, "the `ADR-009`
   references in the module docs of `crates/core/src/lib.rs`, `document.rs`, and
   `chain.rs` — the ones that describe future model ownership — are retargeted",
   keeps the edit set at the three module docs the Delta inventory already names
   ("three module-doc lineages"); the em-dashes make the qualifier descriptive
   rather than restrictive, but the touched surface is identical and is
   "Docs only — no behaviour, no fixture, no gate change".
7. **Value check** — proportionate. This pass adds zero mechanism: five hunks
   are punctuation, hyphenation ("build-script-generated"), article and verb
   choice ("adds" for the ungrammatical "gains … to"), and serial commas.
   Complexity is unchanged from the version this seat has no open finding
   against, and it remains small relative to what the stage buys — the ADR's
   one-time bootstrap, taken additively on a spine the ADR-008 run already
   shipped, with no new dependency, no new error variant, and no harness, CI,
   or `Cargo.lock` change.

## Findings

None.

## FYI (no severity)

- Placeholders and lineage — "— the ones that describe future model ownership —"
  — converting the restrictive clause to a non-restrictive one reads as
  retargeting *every* `ADR-009` reference in those three module docs rather than
  only the ownership ones. Correctness/scope seat's call whether a module doc
  contains an `ADR-009` reference that should stay; no complexity cost either way.
- Deferred / out of scope — "Seed profile and manifest sets — ADR-016 (not yet
  written)" sits next to ADR-011's "`data/manifests/` and `data/profiles/` …
  populated by 013/014". Not re-raised as a finding: out of this pass's diff
  scope, and the ADR clause governs which ADR populates the directories, which
  Placeholders and lineage still states as 013 and 014. Noted for the
  compatibility/correctness seats.
