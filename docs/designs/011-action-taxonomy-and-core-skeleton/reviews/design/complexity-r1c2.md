# complexity review r1c2 (ratification of the r1 revisions)

verdict: approve

Scope: `git diff 118d4d2..f23e3e4` over `02-proposal.md`, then the full current
document. Inputs: `02-proposal.md` (incl. Revision log),
`docs/adrs/011-action-taxonomy-and-core-skeleton.md`, my r1 findings. Checklist:
`references/complexity.md`.

## Checklist walk

1. **New package/module/interface** — unchanged by the revisions: still one new
   module (`crates/core/src/action.rs`) beside the existing `reach.rs`
   pattern, one new format directory beside the two existing ones, no new
   crate. The r1 edits add no new type or interface. Pass.
2. **Every moving part traces to a Goal** — the only genuinely new behaviour in
   the revised text is the declared-`version`-vs-directory check in
   `load_action_registry`. It traces to Goal 1 ("the shipped id registry") and
   to the revised Stability route (the version gate is what refuses a
   `action-ids/v2` document to a v1 reader), and Test strategy #4 consumes it.
   Not YAGNI. Pass.
3. **A 30–50% smaller design** — the r1 revisions moved complexity *out*: the
   "grammar widening is an additive change" posture (which implied a
   per-grammar version distinction) was deleted in favour of the withdrawal
   route the design already had; the Non-goals list collapsed into Deferred
   rather than duplicating it. No smaller design is now available that still
   meets the Goals. Pass.
4. **New dependencies** — none; std-only, `Cargo.lock` byte-unchanged, stated
   again in Rollout 2. Pass.
5. **Deferrable second half** — unchanged: nothing consumes the taxonomy yet,
   and the rollout's thin first slice (corpus + module + tests) is intact after
   renumbering. Pass.
6. **Knobs with no consumer / one-implementation layers** — the one knob with
   no consumer is still `ActionRegistry::contains` (see finding). The new
   fixtures (`non-ascii-id.toml`) and the new unit case each pin an already
   stated grammar rule, so they have a consumer. Pass with the nit below.
7. **Value check** — proportionate. The stage buys a published id set and a
   third contract format for one module, four re-exported names, one extra
   version cross-check, and no harness, CI, or dependency change. The r1 edits
   reduced the number of stated postures rather than adding any.

## Regression check of the revised text

- Non-goals consolidation: every item dropped from the old list reappears in
  Deferred / out of scope (axis kind → ADR-015, emphasis and manifest id
  validation → ADR-013, vocabulary/reach → ADR-012, chains → ADR-014, codecs /
  config discovery / CLI → ADR-017, `toml-test`-style runner contract →
  Deferred), and the two items kept in Non-goals are the ones with no owning
  document. No orphaned scope. Not a finding.
- Withdrawal paragraph now delegates the rule to Stability instead of restating
  it; the rule is still stated once, canonically, in Stability. Same single
  mechanism, less duplication. Not a finding.
- Rollout renumbered 7→6: the Risks row's "Retargeted in Rollout 4" matches the
  new step 4 (lineage retarget), and the two "Test strategy #6/#7" anchors still
  point at the intended items (#6 is the shipped-registry load that a widened id
  in `action-ids/v1/registry.toml` would fail; #7 is the platform leg). No
  dangling cross-reference. Not a finding.
- The source-classification sentences (spec.md Extension and withdrawal,
  `load_action_registry`, the two data READMEs) restate one rule in three
  places but name the canonical copy (`spec.md, above`) rather than adding a
  mechanism; `Document::ActionIds` still parses foreign documents as data, which
  is consistent with "never loaded as the registry". Not a finding.
- Nothing in the diff contradicts ADR-011's Decision or Consequences; the
  Deferred entries remain consistent with the ADR's "Not decided here" list.

## Findings

- [nit][re-raised] Chosen design → "The model: `crates/core/src/action.rs`" —
  "`contains(&ActionId)`, and `axis_kind(&ActionId) -> Option<AxisKind>`" —
  the r1 revisions left this line untouched, so the finding still stands: the
  registry shape ("the arrays *are* the id set, so there is no id without an
  axis kind") makes membership and kind one fact, and the test strategy (#2,
  #6) consumes `ids()`/`axis_kind` only, so `contains` remains the one public
  API knob with no distinct consumer. The new source-classification text did not
  create one. Fix: drop `ActionRegistry::contains` and let callers use
  `axis_kind(id).is_some()`, or name the consumer that needs membership without
  a kind.

## FYI (no severity)

- Chosen design → "The contract" entry table — "`discrete-lateral` /
  `continuous-lateral` / `none` (arrays of strings, all required, may be empty;
  length bounded by the shared parse budget)" — this is a byte/size budget on
  the whole file restated as if it were a per-array length rule; a third-party
  reader implementing `spec.md` alone has no number to honor and would have to
  invent one. That is contract wording, so it belongs to the compatibility or
  correctness seat, not here: within complexity the clause adds no new
  mechanism, since the budget it names is the shipped guard the Context already
  says is left alone.
