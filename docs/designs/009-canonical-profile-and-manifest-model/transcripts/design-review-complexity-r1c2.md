# Transcript: design-review:complexity r1c2

## Provenance
- stage: design-review:complexity (round r1c2)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T23:00:17.543Z
- ended: 2026-11-11T23:03:57.610Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 7b2ddc6d10df95a81e738c734fe5c9673d530181 5bbcc09d65524a3e2c522e58501451480f23d8ee` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/009-canonical-profile-and-manifest-model.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Transforms produce bases — \"Each invocation must match its registry entry's declared arity and the grammars of its arguments … a mismatch is the shipped `InvalidValue { key, message }`, one `fixtures/invalid/` case per shape\" + Test strategy #10 \"an unknown transform name is rejected at parse (`InvalidValue`), before `apply_policy` runs\"",
    "problem": "new (r1). The new invocation-validation rule is placed in two crates at once. Policy documents are parsed by `bindforge-core` (the section states it, because the harness needs a reader per format directory), the malformed cases are `schemas/policy/v1/fixtures/invalid/` files rejected with a `CoreError` variant (Test #1), `InvalidValue` is a shipped core variant, and Adversarial posture says \"no registry entry is reached with a bad argument\" — so the arity/hand/placement-id/pattern/transform-name checks run in core, ahead of the registry. Rollout 3 puts \"the registry — each entry carrying its declared arity and argument grammars\" in `bindforge-transforms`. The text therefore implies one of two permanent costs: core hard-codes the v1 transform vocabulary (a closed list in the wrong crate, widened every time transforms grow, dissolving ADR-009's split where transforms owns policy machinery), or core grows a registry-driven validation hook — an abstraction layer with one implementation (ch… [elided by the checkpoint cap]",
    "fix": "State that core's policy reader validates only the shape it owns (`stack` = array of non-empty arrays of strings, first element the transform name) and that `apply_policy` validates each invocation against its registry entry's arity and argument grammars. Reword §4's \"a mismatch is the shipped `InvalidValue`\" as an `apply_policy` outcome and Test #10's \"rejected at parse … before `apply_policy` runs\" as rejected by `apply_policy` before any transform runs; the per-shape cases move to `crates/transforms/tests/policy_semantics.rs`, and `schemas/policy/v1/fixtures/invalid/` keeps the refused-TOML and bad-shape cases core can see. This replaces the parse-time clause rather than adding to it."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Input vocabulary + Files created — \"`schemas/README.md` carries the admitted keyboard-name list — the single enumeration the graded set below and Test strategy #3 iterate\" / \"the admitted keyboard-name list (the enumeration Test strategy #3 iterates)\"",
    "problem": "regression (partial closure of my r0 coverage finding). The r1 closure names two different enumerations for one test. The Authoring bar defines the graded set as content-derived — \"every key the shipped placements and the seed manifest's `default` rows reference, extended only by review\" — and Test #3 reads \"every key in the graded set is graded in both hands\"; the README list is the admitted US-ANSI name set, i.e. the whole keyboard. Read as written, Test #3 iterates the full admitted list, which is the coverage obligation the Revision log records as removed (\"the full-keyboard coverage clause is gone from the authoring bar and Test #3\"), so authored corpus and CI surface grow back with the name list and the new `off-reach` default loses its point.",
    "fix": "One clause in both places: the README list is the admitted vocabulary (the grammar domain for `keyboard:<name>`) and carries no grading obligation; Test #3's enumeration is the graded set, derived from the loaded `data/placements/` keys plus the seed manifest's `default` rows."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Adversarial posture — \"New `CoreError` variants, each pinned by a fixture: … `MergeConflict`\", with the r1 line \"`fork` … and `merge` … land with their suite in the ADR-012 slice\"",
    "problem": "new residue of the accepted deferral. The variant list is the first slice's, and `MergeConflict` is its only entry with no producer in that slice: a merge conflict comes from `merge()`, not from a document reader, so no `fixtures/invalid/` case can pin it — the \"each pinned by a fixture\" claim and Test #1's per-fixture rejection clause are unsatisfiable for it, and the shipped variant is constructed by nothing until ADR-012 (checklist #6).",
    "fix": "Split the sentence: the shipped variants (the list minus `MergeConflict`) ship with fixtures in this slice; `MergeConflict` lands with `fork`/`merge` in the ADR-012 slice, next to the `patch/v1/spec.md` rule that already names it."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Manifests state game facts + Test strategy #8 — \"`docked` (hard-docked actions a binding move cannot escape)\" / \"every `exclusive` and `docked` member appears in the `action` rows\"",
    "problem": "new in the changed text. r1 widened `validate` to `docked`, so its only v1 consumer is a membership check; the stated rule — a binding move cannot escape a docked action — is read by nothing. `relocate-cluster` matches by action pattern plus an isometry precondition and `swap-polarity` is taxonomy-gated, so a policy can relocate an action the manifest declares unmovable (checklist #6: a documented invariant with no consumer).",
    "fix": "One clause naming the honored owner: either add `docked` matches to `relocate-cluster`'s precondition (→ `Skipped`, naming the action), or state that `docked` is enforced at apply time, gate that with ADR-012, and demote the \"cannot escape\" claim in §5 to a reference to that owner."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/009-canonical-profile-and-manifest-model.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/complexity-r1c2.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → 4. Transforms produce bases — \"`apply_policy(base: &Resolved, policy: &PolicyDoc, ctx: &Ctx) -> PolicyOutcome` with `PolicyOutcome { base: Resolved, skipped: Vec<Skipped> }`\" + \"`apply_policy` validates every invocation against the table … and a mismatch is the shipped `InvalidValue { key, message }` naming the invocation, raised before any transform runs\"","problem":"new (residue of my accepted r1 closure). Moving the invocation check out of core's reader left the pinned signature unable to carry the pinned rejection: `apply_policy` returns a bare `PolicyOutcome`, which has no error channel, so the `CoreError::InvalidValue` that §4, Adversarial posture (\"the invocation table's arity and grammars in `apply_policy`\") and Test strategy #10 all require has nowhere to go. The Goals pin function signatures as the deliverable, so an implementer hits a contradiction between two statements of the same section. The r1 theme (validation split across crates) is closed; this is only the return channel.","fix":"Make the return type `Result<PolicyOutcome, CoreError>` in that one signature (replacing `-> PolicyOutcome`), and state that the whole-`stack` table check runs first, so its `InvalidValue` arrives as `Err` while inapplicable transforms keep returning `Ok(PolicyOutcome)` with `skipped` populated. One signature, no new type."},{"severity":"minor","location":"Chosen design → 2. Input vocabulary + Reach classes loader rules + Test strategy #3 — \"`schemas/README.md` carries the admitted keyboard-name list: the admitted `keyboard:` vocabulary the reach tables draw from, enforced by the reach loader (a key outside it is `InvalidValue`)\" / \"every key is a `keyboard:` id in the admitted list, else `InvalidValue`\"","problem":"new. The admitted list gained its first consumer this round, which makes true a duplication the same paragraph denies. The list is prose in `schemas/README.md`, and no machine-readable admitted-name table exists anywhere in the repo (nothing under `crates/` or `schemas/` names a `keyboard:` key today), so `reach.rs` must carry a hard-coded US-ANSI name set of its own — a second key-name table kept in step with the README by eye. Test strategy #3 pins only the behavior of one out-of-list key, never the list's contents, so README and code can drift in either direction, and \"No second key-name table is invented here\" is false as the text stands.","fix":"One clause naming where the machine-readable admitted list lives (a public constant in the crate that owns input ids, e.g. `keyboard_names()`, which `reach.rs` and every input-id reader consults), replacing the prose-list-only reading, with the README pointing at it so \"no second table\" stays true. Extend Test #3's existing clause to assert the loader's admitted set equals the README enumeration, so the pair is pinned."},{"severity":"nit","location":"Rollout 2 vs Chosen design → Adversarial posture — \"the entry sets and the `CoreError` variants named in Adversarial posture\"","problem":"new. The Adversarial-posture sentence now excludes one variant from this slice (\"each pinned by a fixture in this slice except `MergeConflict` — it lands with `fork`/`merge` in the ADR-012 slice\"), while Rollout 2's blanket phrase names all thirteen variants as shipping in the core step, so the r2 carve-out reads as a contradiction at the rollout step.","fix":"Two words in Rollout 2: \"the `CoreError` variants named in Adversarial posture, except the ones it defers to ADR-012\"."}]}
```
