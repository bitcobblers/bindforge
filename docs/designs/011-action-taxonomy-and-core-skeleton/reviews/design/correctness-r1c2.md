# correctness review r1c2

verdict: approve

Scope: the c2 diff (`118d4d2..f23e3e4` of 02-proposal.md) plus the full current
document, checked against docs/adrs/011-action-taxonomy-and-core-skeleton.md,
the Revision log, and the correctness seat's own r1 findings. Ratification
pass: findings only where the revised text breaks previously approved material
or falsifies a section cited in r1.

## Findings

- [minor][new] Chosen design — The contract, entry table × The model × Test strategy #5 — "`version` (string, required, equal to the version segment of the directory holding the file)" — this new constraint is written as a per-document entry rule, but the same document keeps fixtures (which are `action-ids` documents) in `schemas/action-ids/v1/fixtures/{valid,invalid}/`, whose holding directories carry no version segment; the check itself lives only in `load_action_registry` ("a check that the declared `version` equals `dir`'s version segment"), and the harness — unchanged — parses fixtures without it. A third-party reader that applies the entry table literally to every fixture either rejects every valid fixture (holding dir `valid/` has no version segment) or must invent the unstated rule "match the nearest ancestor directory named `v<N>`"; and a version-mismatch fixture becomes unexpressible under the literal reading. This is the checklist's same-rule-under-two-shapes case (document constraint in spec.md vs. loader-side check in The model). The Rust-side behavior is unambiguous, so it does not block implementation. Fix: scope the entry-table parenthetical to the registry document and name where the check happens — e.g. "`version` (string, required; `load_action_registry` refuses a `registry.toml` whose `version` differs from the version segment of the versioned directory that holds it; fixtures declare `v1` and are checked by the version gate, not by their directory)".

## Closed-for-real verification (no finding)

- Options considered (Withdrawal): the deleted "ids are never renamed, retargeted, or removed / major-directory withdrawal / aliases deferred" sentences are recoverable by reference — Stability states the rule verbatim, and the Deferred section carries the alias deferral. The citation `schemas/README.md:16-25` × A12 is unchanged from r1.
- Context rewrite (reset reading → Options considered): A3's basis names "Options considered (last option)"; the reset option is that option and its content survived. No orphaned reference.
- Non-goals condensation: every deleted non-goal (axis-kind action, emphasis/placement, manifest id validation, input vocabulary/reach tables, profile chains, codecs/CLI, toml-test-style runner) appears in Deferred / out of scope with its owning ADR, so "written once … under Deferred" holds; the refusal of an `emphasis` entry survives in the spec.md entry bullet, and the quoted clause "emphasis is *not* in the taxonomy" matches ADR-011 line-for-line (docs/adrs/011-…:55).
- Stability grammar rewrite × Risks row: both now say widening rides `action-ids/v2`; the cited Test strategy #6 ("returns 40 ids, every one grammar-valid") does fire if `v1/registry.toml` gains an out-of-grammar id, since the loader refuses a malformed id; the new version-directory check makes the v2 version gate a real mechanism for v1 readers. Consistent.
- `non-ascii-id.toml` (new fixture) matches the id grammar ("ASCII lowercase letter first"), Test strategy #1's new reject case, the toml-test naming rule (A6), and the Windows case-collision note; the new escaped-id echo in the model is consistent with the single-line bounded-message promise and with "refused naming the id" in Test strategy #2/#3.
- Rollout renumbering: the Risks row's "Retargeted in Rollout 4" matches the renumbered rollout (step 4 = lineage retarget); all other `Test strategy #N` cross-references (#2, #6, #7) still point where they claim.
- Compatibility rows (non-Rust, Steam Deck): each now states one status with its Deferred/008 basis; internally consistent and consistent with the Deferred section. (008's matrix itself is outside this pass's named inputs; the revision log records the substitution deliberately.)
- Deferred items still track ADR-011's "Not decided here" list (012/013/014/015/ADR-017) with no contradiction.

## Status of r1 findings (record, not new findings)

- Test strategy #3 "four defined entries" vs. five required entries: text unchanged by the diff; the revised entry table still lists five required entries, so the finding stands as recorded in r1, unchanged.
- Delta inventory lists `chain.rs` under Unchanged while Placeholders and lineage and Rollout 4 edit its module docs: unchanged by the diff; stands as recorded.
- ADR-009 lineage retarget coverage (reach.rs, reach-classes spec.md, profile spec.md not in the retarget list): Goal sentence unchanged; the rollout renumber (5→4) kept the Risks row self-consistent. Stands as recorded.
- `missing-axis-arrays.toml` plural naming: unchanged; stands as recorded.

None of these was broken or worsened by the revisions, so under this pass's scope rule they are not re-reported as findings.

## FYI (no severity)

- The Revision log lists seven decisions but not the Non-goals condensation or the Context rewrite; the document stays self-consistent either way, but a later round's reviewer reading only the log would miss that the non-goal list moved to Deferred.
- "Chosen: the major-directory route whose rule Stability states" (Options considered, Withdrawal) is garbled syntax — meaning is recoverable; language-only fix belongs to the editing pass.
