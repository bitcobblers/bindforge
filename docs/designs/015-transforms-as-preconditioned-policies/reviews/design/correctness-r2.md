# correctness review r2

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r2.diff` against
`02-proposal.md`, cross-checked against
`docs/adrs/015-transforms-as-preconditioned-policies.md` and the proposal's
Revision log. Previous correctness findings: none (editorial phase is a single
language pass per round), so this pass reports only what the changed text
itself makes true.

## Findings

- [minor][new] Revision log (trailing note) + Test strategy #1 — the diff
  appends `Editor notes: Test strategy — "grammar_closure.rs refuses a policy
  document aliasing `transforms_`" is ambiguous; author to clarify.` — the
  ambiguity is real, and the note now leaves it unresolved as the document's
  last line. Test strategy #1's sentence "`grammar_closure.rs` refuses a policy
  document aliasing `transforms_`" does not say which entry spelling is refused
  (`transforms_` as a near-miss of `transforms`, or an alias *of* `transforms_`),
  which refusal is expected, or whether the case is an inline case in
  `grammar_closure.rs` or a corpus fixture — and the `fixtures/invalid/` list
  under "The contract: `schemas/policy/v1/`" contains no alias fixture, only
  `unknown-entry.toml`, while the Delta inventory says `grammar_closure.rs`
  covers "the policy entry set and alias cases". As written, the near-miss-entry
  guard that makes the entry set closed is asserted but not pinned to a named
  artefact. Fix: replace the sentence in Test strategy #1 with the concrete
  assertion — "a policy document carrying an entry named `transforms_` (a
  near-miss of `transforms`) refuses at parse as an unknown entry naming it, as
  an inline alias case in `grammar_closure.rs`, not a corpus fixture" — and
  drop the `Editor notes:` line from the Revision log.

- [nit][new] Non-goals + Deferred / out of scope — "Preconditions composed with
  and/or/not, and any fourth transform kind: both ride a future `policy/v2`
  (Options considered)" (and the matching Deferred bullet, "the version gate
  makes the old-reader refusal automatic (Options considered)") — the citation
  now names a real section, and the fourth-kind half is genuinely argued there
  ("Growing the set rides `policy/v2`: the version gate fails old readers
  closed"), but Options considered never considers and/or/not precondition
  composition, so half of the cited "both" has no referent. Fix: cite the
  option by heading for the kind half ("Options considered → Transform kinds")
  and ground the composition half in the ADR alternative it closes (ADR-015,
  "Conditions as free-form predicates"), or drop the parenthetical for that
  half.

## Checklist coverage

- Assumption register: the diff changes no register row and no row's cited
  basis; the four passages it touches still rest on A9 (channel split), A22
  (space-joined encoding + wrong-count fixtures), A4 (structured
  `PlacementCheck`), and A11 (per-format version gate), which support them.
- URL spot-check: the diff adds, removes, or reworded no external citation; the
  only `unverified` rows (A23) are untouched by it and rest on `01-research.md`,
  which is outside this pass's named inputs. Nothing in scope to fetch.
- Internal contradictions / interface names: "Precondition outcomes are
  **verdict data** on `PlacementCheck`'s channel, never new `CoreError`
  variants" still agrees with the Unchanged list ("**no new `CoreError`
  variant**"), the Report-channel option, and the `apply` entry refusals. The
  softened "with wrong-field-count refusals and fixtures, as
  `set`/`swap`/manifest rows already do" keeps the A22 claim intact; "exactly
  the answer a bool cannot give (A4)" keeps the A4 claim intact.
- Decides what the ADR asked: unaffected by the diff; the three kinds, the
  base-only boundary, the guard over `docked-actions`/`dead-keys`/
  `exclusive-groups`, the reports-and-no-ops posture, `schemas/policy/v1/` and
  `crates/transforms` all remain decided.
- Deferred items: the two touched deferrals (fourth kind, and/or/not
  composition) stay consistent with ADR-015's Decision, which names exactly
  three kinds and rejects free-form predicates; no contradiction, not
  re-opened.
- Previous round's blocker/majors: none on this seat; nothing to re-verify.

## FYI (no severity)

- Citation shape now mixes "(Options considered)" (this diff) with the existing
  "(Options: Tags)" / "(Options: Stack order)" forms — unambiguous either way,
  so this is a style question for the consistency/complexity seat, not a
  correctness defect.
