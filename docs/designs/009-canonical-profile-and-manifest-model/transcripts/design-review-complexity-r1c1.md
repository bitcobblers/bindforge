# Transcript: design-review:complexity r1c1

## Provenance
- stage: design-review:complexity (round r1c1)
- agent: review/design-complexity
- model: strata/swift-1.5-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T22:54:09.696Z
- ended: 2026-11-11T22:57:11.516Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff cc78a21d26a8a556fb75f7206e403d8e8618c604 7b2ddc6d10df95a81e738c734fe5c9673d530181` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design → Transforms produce bases, `schemas/policy/v1/` entries — \"`hand` (`left`/`right`, required)\"",
    "problem": "Configuration knob with no consumer (checklist #6). No rule, transform, `Ctx` field, or Test strategy reads the policy's `hand`: `relocate-cluster` takes `<hand>` in its own invocation, `swap-polarity` and `swap-mouse-buttons` take no hand, `Ctx { taxonomy, reach, placements }` carries none, and Test strategy #11 never exercises it. ADR-009 Decision 4 puts the hand inside the invocation, not on the policy. Because a format's entry set is closed and minor bumps are additive only (schemas/README.md), a required unreadable entry shipped in v1 costs a major bump to remove, and is ambiguous for a policy with two relocate invocations on opposite hands.",
    "fix": "Delete `hand` from the policy/v1 entry table and the example; the `<hand>` invocation argument on `relocate-cluster` is the only spelling the registry reads."
  },
  {
    "severity": "major",
    "location": "Chosen design → Manifests state game facts — \"then role-qualified quoted keys: `action:<id>` = the game-native action name (the join), `default:<id>` = …, `exclusive:<group>` = id array\"",
    "problem": "This is the design's one relaxation of the shipped closed-entry-set invariant into an open prefix rule (the Risks row concedes it), and it has no recorded alternative in Options considered. The shipped mechanism is a flat allow-list (`PROFILE_ENTRIES` + `reject_unknown_entry`, crates/core/src/document.rs:39-43,156-170); the prefix form forces new machinery in the manifest reader — enumerated prefixes, a distinct unknown-prefix rejection, its fixture, a spec clause, and Test #1's unknown-prefix case — i.e. permanent complexity to keep a published entry set open (checklist #3/#6).",
    "fix": "Encode the three roles as nested arrays, which the design already accepts at nesting level 2 (A13; `stack` and `swap` use it): `action = [[\"move.steer-left\", \"Steer Left\"]]`, `default = [[\"combat.melee\", \"keyboard:f\"]]`, `exclusive = [[\"ui\", \"ui.map\", \"ui.inventory\"]]`. Every row is `[id, value…]`, the entry set stays closed, `validate`'s checks are unchanged, and the prefix enumeration, unknown-prefix error path, its fixture and the Risk row all go away."
  },
  {
    "severity": "major",
    "location": "Chosen design → Reach classes and placements (Authoring bar) and Test strategy #3 — \"both hands graded over the full admitted keyboard name set\" / \"every admitted keyboard name is graded in both hands\"",
    "problem": "The largest authored artifact in the design has consumers only for a fraction of it. The reach bound reads grades for cluster source/target keys, the emphasis reading reads the grade of a game's `default:` input, and the asymmetry claim needs a few mirrored keys; function/navigation/lock keys are graded only to satisfy the coverage rule, and each added row must also clear the \"grades non-decreasing in grid distance from that hand's home-graded keys\" check, so CI surface grows with corpus size. Because \"a key absent from the hand's table is `UnknownInput`\", absence is illegal, which forces exhaustive 2×N authoring in the first seed slice (Rollout 4a) — a design ~30-50% larger on the data side than the goals require (checklist #2/#3/#5).",
    "fix": "State a graded set (the keys the shipped placements and manifest defaults reference, extended by review) and treat a key absent from a hand's table as `off-reach` for that hand instead of `UnknownInput` — which also makes the emphasis reading total for any game default. Drop the full-set coverage clause from the authoring bar and Test #3, keeping coordinate-agreement and monotonicity checks over the graded subset. This keeps the authored-grading methodology the r0 log defends and shrinks only its volume."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Transforms produce bases — \"`mouse-swap` (boolean, default **false** — the ADR's opt-in)\" with \"accepted in a stack only when the policy declares `mouse-swap = true`, else the document is rejected (`MouseSwapNotOptedIn`)\"",
    "problem": "A knob whose only consumer is the gate on its own transform: a policy that lists `swap-mouse-buttons()` has already declared the intent, in the same document, explicitly. The ADR's \"default off\" is satisfied by stack membership. The knob therefore buys nothing and costs an entry, a `CoreError` variant, a fixture and a Test #11 clause (checklist #6).",
    "fix": "Drop `mouse-swap` and `MouseSwapNotOptedIn`; state in policy/v1/spec.md that a policy opts in by listing `swap-mouse-buttons()` in `stack`."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains — \"`revision` (a `<major>.<minor>` string)\" as an entry on `profile` and `patch`",
    "problem": "No consumer. Chain identity is `id` + `derives`; the pin's revision half names the manifest's `revision` (that one is consumed by the drift mechanism); nothing compares a profile's own revision. The Rollout defaults line \"`revision` absent = unpinned\" describes `for`, not this entry, so the knob's meaning is unstated as well as unread (checklist #6).",
    "fix": "Keep `revision` on manifest/v1 only; drop it from profile/v1 and patch/v1 (it is additive, so a minor bump can add it when a consumer exists) and correct the defaults line to \"`for` absent = not bound to a game\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Profiles are chains + Rollout step 5 — \"`fork(index, root, rung) -> PatchDoc`\" and \"`merge(base: &Resolved, a: &PatchDoc, b: &PatchDoc)` … overlap is `MergeConflict`\", shipped with Test strategy #8",
    "problem": "Checklist #5: the merge/fork surface has no v1 caller — the CLI front end for diff/fork/merge is Deferred to ADR-012 — so the first merged slice carries multi-document algebra plus its suite ahead of need. Chain resolution and `diff` do have consumers (drift CI, the model walk).",
    "fix": "State the merge rule in patch/v1/spec.md (chain order, per-action single touch, disjointness for a legal merge) so the contract stays complete, ship `resolve` + `diff` in the first slice, and land `fork`/`merge` with Test #8 in the ADR-012 slice that has the caller."
  },
  {
    "severity": "nit",
    "location": "Files created by this design — `schemas/taxonomy/v1/actions.toml` and `schemas/reach-classes/v1/{left,right}.toml` vs `data/{manifests,profiles,policies,placements}/`",
    "problem": "Two homes for reviewed data with no stated rule (lookup tables in the contract tree, curated content under `data/`), which Rollout 6 then has to justify in a repo-practices line.",
    "fix": "One sentence in schemas/README.md: the contract tree holds lookup tables a loader reads at a fixed path; `data/` holds content discovered by walking."
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
Write your report to docs/designs/009-canonical-profile-and-manifest-model/reviews/design/complexity-r1c1.md. That file is the only path this session
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "Chosen design → Transforms produce bases — \"Each invocation must match its registry entry's declared arity and the grammars of its arguments … a mismatch is the shipped `InvalidValue { key, message }`, one `fixtures/invalid/` case per shape\" + Test strategy #10 \"an unknown transform name is rejected at parse (`InvalidValue`), before `apply_policy` runs\"", "problem": "new (r1). The new invocation-validation rule is placed in two crates at once. Policy documents are parsed by `bindforge-core` (the section states it, because the harness needs a reader per format directory), the malformed cases are `schemas/policy/v1/fixtures/invalid/` files rejected with a `CoreError` variant (Test #1), `InvalidValue` is a shipped core variant, and Adversarial posture says \"no registry entry is reached with a bad argument\" — so the arity/hand/placement-id/pattern/transform-name checks run in core, ahead of the registry. Rollout 3 puts \"the registry — each entry carrying its declared arity and argument grammars\" in `bindforge-transforms`. The text therefore implies one of two permanent costs: core hard-codes the v1 transform vocabulary (a closed list in the wrong crate, widened every time transforms grow, dissolving ADR-009's split where transforms owns policy machinery), or core grows a registry-driven validation hook — an abstraction layer with one implementation (checklist #6).", "fix": "State that core's policy reader validates only the shape it owns (`stack` = array of non-empty arrays of strings, first element the transform name) and that `apply_policy` validates each invocation against its registry entry's arity and argument grammars. Reword §4's \"a mismatch is the shipped `InvalidValue`\" as an `apply_policy` outcome and Test #10's \"rejected at parse … before `apply_policy` runs\" as rejected by `apply_policy` before any transform runs; the per-shape cases move to `crates/transforms/tests/policy_semantics.rs`, and `schemas/policy/v1/fixtures/invalid/` keeps the refused-TOML and bad-shape cases core can see. This replaces the parse-time clause rather than adding to it."}, {"severity": "minor", "location": "Chosen design → Input vocabulary + Files created — \"`schemas/README.md` carries the admitted keyboard-name list — the single enumeration the graded set below and Test strategy #3 iterate\" / \"the admitted keyboard-name list (the enumeration Test strategy #3 iterates)\"", "problem": "regression (partial closure of my r0 coverage finding). The r1 closure names two different enumerations for one test. The Authoring bar defines the graded set as content-derived — \"every key the shipped placements and the seed manifest's `default` rows reference, extended only by review\" — and Test #3 reads \"every key in the graded set is graded in both hands\"; the README list is the admitted US-ANSI name set, i.e. the whole keyboard. Read as written, Test #3 iterates the full admitted list, which is the coverage obligation the Revision log records as removed (\"the full-keyboard coverage clause is gone from the authoring bar and Test #3\"), so authored corpus and CI surface grow back with the name list and the new `off-reach` default loses its point.", "fix": "One clause in both places: the README list is the admitted vocabulary (the grammar domain for `keyboard:<name>`) and carries no grading obligation; Test #3's enumeration is the graded set, derived from the loaded `data/placements/` keys plus the seed manifest's `default` rows."}, {"severity": "minor", "location": "Chosen design → Adversarial posture — \"New `CoreError` variants, each pinned by a fixture: … `MergeConflict`\", with the r1 line \"`fork` … and `merge` … land with their suite in the ADR-012 slice\"", "problem": "new residue of the accepted deferral. The variant list is the first slice's, and `MergeConflict` is its only entry with no producer in that slice: a merge conflict comes from `merge()`, not from a document reader, so no `fixtures/invalid/` case can pin it — the \"each pinned by a fixture\" claim and Test #1's per-fixture rejection clause are unsatisfiable for it, and the shipped variant is constructed by nothing until ADR-012 (checklist #6).", "fix": "Split the sentence: the shipped variants (the list minus `MergeConflict`) ship with fixtures in this slice; `MergeConflict` lands with `fork`/`merge` in the ADR-012 slice, next to the `patch/v1/spec.md` rule that already names it."}, {"severity": "minor", "location": "Chosen design → Manifests state game facts + Test strategy #8 — \"`docked` (hard-docked actions a binding move cannot escape)\" / \"every `exclusive` and `docked` member appears in the `action` rows\"", "problem": "new in the changed text. r1 widened `validate` to `docked`, so its only v1 consumer is a membership check; the stated rule — a binding move cannot escape a docked action — is read by nothing. `relocate-cluster` matches by action pattern plus an isometry precondition and `swap-polarity` is taxonomy-gated, so a policy can relocate an action the manifest declares unmovable (checklist #6: a documented invariant with no consumer).", "fix": "One clause naming the honored owner: either add `docked` matches to `relocate-cluster`'s precondition (→ `Skipped`, naming the action), or state that `docked` is enforced at apply time, gate that with ADR-012, and demote the \"cannot escape\" claim in §5 to a reference to that owner."}]}
```
