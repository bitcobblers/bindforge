# complexity review r2

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial language pass),
confirmed against `02-proposal.md` and `docs/adrs/008-rust-for-the-product.md`.
Previous complexity findings: none (this phase is a single language pass per round).

verdict: approve

## Findings

None. Every hunk in the diff is language-only; no hunk adds, removes, or
re-scopes a moving part, and none makes a section this seat approved false.

Hunk-by-hunk confirmation:

- Goals — "with client types" → "with the same compiled types": matches the
  ADR's Consequences clause verbatim ("validates uploaded profiles with the
  same compiled types the client uses"); no new obligation, no new mechanism.
- Gates — "and nothing installed beyond the pinned toolchain" → "with nothing
  installed beyond the pinned toolchain": same one-gate promise, and the
  wording now matches Test strategy #5 exactly, which lowers drift risk.
- Adversarial guard — comma before "with typed errors": only re-attaches the
  phrase to the parse rejection rather than to the nesting level; the decided
  items (1 MiB budget, 16 levels, `Cycle`/`ChainTooDeep`, `MAX_CHAIN_DEPTH = 32`
  as a tunable-but-not-removable floor) are untouched.
- TypeScript boundary — "Node is only ever a prerequisite for" → "Node is a
  prerequisite only for": identical scope restriction on Node.
- Risks / Deferred — "not silently" → "not silent", "never interpolating" →
  "no interpolation of": grammar/parallelism only; the ADR-012 deferral list
  keeps every item it had.
- Revision log — two appended Editor notes (dependency-edge phrasing, "the
  Node floor"): author TODOs, not design content; they add no mechanism.

Checklist walked:

1. Reuse of existing packages/modules: nothing new introduced by the diff.
2. Moving parts trace to a Goal: the pieces already traced (workspace + pin →
   Goal 2; `schemas/` + harness → Goal 3; two CI jobs + `repo-practices` →
   Goal 5; chain guard → Goal 6). The diff adds none.
3. Smaller design: the diff is a net simplification of prose; no structure to
   shrink further at this scope.
4. New dependencies: none added; the advisory-scanning machinery
   (`advisories.yml`, pinned `cargo-deny`, `deny.toml`) is unchanged and is
   Revision-log-documented as replacing an unowned deferral, not adding a
   layer.
5. Deferred second half: unchanged — hub/sqlx, codecs, desktop frontend all
   still deferred per ADR-012/014 and the desktop design; nothing in the diff
   pulls a deferred piece forward.
6. Knobs with no consumer / single-implementation abstractions: no new knob;
   the only config surface touched in prose (`deny.toml` `skip`, Node pin)
   already has a named consumer (maintainer triage; the `setup-node` step).
7. Value check: complexity remains proportionate to what the feature delivers
   — a greenfield workspace, one gate, a fixture contract, and one input
   budget. This pass changed none of it.

## FYI (no severity)

- The two appended Editor notes flag real ambiguities this seat also noticed:
  "Dependency edges point one way: nothing → `bindforge-core`" reads both as
  "core depends on nothing" (intended) and "nothing depends on core"
  (contradicted by the very next clause), and "the Node floor" in Rollout 5
  names a pin whose only consumer is the `setup-node` step in `check.yml`
  without naming where the floor value lives. Both are phrasing/placement
  issues for the author's next round, not added complexity; already recorded
  in the Revision log's editor notes.
