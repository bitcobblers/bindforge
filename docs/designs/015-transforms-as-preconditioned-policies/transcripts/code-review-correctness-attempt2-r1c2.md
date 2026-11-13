# Transcript: code-review:correctness:attempt2 r1c2

## Provenance
- stage: code-review:correctness:attempt2 (round r1c2)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T17:25:19.064Z
- ended: 2026-11-13T17:27:30.192Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/015-transforms-as-preconditioned-policies/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 39d4fcb589fd14ce2c81ea01572c84dbd4b28d39..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/015-transforms-as-preconditioned-policies.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "crates/transforms/src/apply.rs:94-100 (offender field; spec at schemas/policy/v1/spec.md:208-211)",
    "problem": "The proposal declares NotApplicable.offender as \"a grammar-checked id, never raw text\" and the spec.md shipped in this diff states a report carries \"an offender that is always a grammar-checked id — never raw document text\", but the implementation fills it with composed sentences (\"`keyboard:d` is bound to `combat.fire`, which the row never named\"), and the field's own doc comment still says \"the id, tag, or key the row tripped over\". A third-party implementer following spec.md produces a different report for the same fault; nothing in the gate pins report text.",
    "fix": "Reconcile one side: amend spec.md's sentence (and the field doc comment) to say the offender names the grammar-checked id/tag/key at fault plus a bounded reason line, or put the checked id in `offender` and carry the explanation in a separate field, updating the proposal."
  },
  {
    "severity": "minor",
    "location": "crates/transforms/src/apply.rs:276-309 (Layout::new, input_of, action_on, rebind)",
    "problem": "profile/v2 makes repeated action ids in one link legal data and core's `resolve` resolves them last-wins (chain_ops.rs:167-170), but Layout reads and writes the FIRST element of a repeated action. For a base like set = [\"move.steer-left keyboard:a\", \"move.steer-left keyboard:b\"], a relocate rebinds the stale first element, leaves the effective binding untouched, and reports Applied — a row that did nothing while looking as though it did something.",
    "fix": "Collapse the working layout by last-wins in Layout::new (keeping each element's position in the order-preserving output), or refuse at refuse_entry a base whose `set` binds one action twice, naming the action."
  },
  {
    "severity": "minor",
    "location": "crates/transforms/src/apply.rs:500-510",
    "problem": "The untouchable guard is specified over every action a row touches, but swap_polarity builds `moves` with filter_map over bound matched actions only, so guard_fault's docked_at(action) leg never sees a matched action the base leaves unbound. With `swap-polarity move.steer-*`, steer-left hard-docked and unbound, the row falls through the (Some, Some) test and reports Applied writing nothing instead of reserved-input (relocate cannot hit this because unbound-action fires first). No mangling, but the wrong one of the ten reasons.",
    "fix": "Check guard.docked_at over every id in `matched` (a small loop before the filter_map), not only over the bound subset."
  },
  {
    "severity": "minor",
    "location": "crates/transforms/src/apply.rs:665",
    "problem": "produce() copies `seeded_from` from the source base. The proposal's output rules enumerate kind, id, derives, for, set and generated-by and never mention seeded_from, so a produced base asserts a seeding provenance its layout no longer has, with no stated reason for carrying it.",
    "fix": "Either leave seeded_from: None in the produced base, or state the carry-forward in spec.md → The output (and the build notes) with the reason."
  },
  {
    "severity": "nit",
    "location": "crates/transforms/src/apply.rs:232-238",
    "problem": "The non-base entry refusal tests `!matches!(base.kind, Some(ProfileKind::Base))`, which also covers a profile/v1 document carrying no `kind`, but the message is the fixed string \"`patch` is not a base\".",
    "fix": "Spell what the document actually carried (or `none`) in the message."
  },
  {
    "severity": "nit",
    "location": "crates/transforms/src/apply.rs:104-105",
    "problem": "The doc comment says \"A fourteenth reason is a policy/v2 question\"; the closed set is ten reasons (apply.rs:107-143, pinned by transforms/tests/zz_api_probe.rs:85-96), so the next reason is the eleventh.",
    "fix": "Reword to \"an eleventh reason\"."
  },
  {
    "severity": "nit",
    "location": "schemas/policy/v1/fixtures/valid/exact-tag-policy.toml:1-6",
    "problem": "The fixture comment reasons about `move.strafe-left` never matching `move.strafe-left-extra`, but the fixture's exact-tag rows are `swap-polarity move.steer-left` and `relocate-cluster move.strafe-right …`; the corpus comment is contract text for a third-party reader.",
    "fix": "Name the ids actually on the rows."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/015-transforms-as-preconditioned-policies/reviews/code/correctness-r1c2.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only.

Previous attempt's gate failure (log excerpt): independence violation: read a review report: docs/designs/015-transforms-as-preconditioned-policies/reviews/code/*.
Do not repeat the behavior that caused that failure.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"crates/transforms/src/apply.rs:103-109, 766-772 (offender field; contract at schemas/policy/v1/spec.md:216-222; proposal → The report)","problem":"Unchanged from round 1 (F2's fix did not touch it): spec.md still states a report carries \"an offender that is always a grammar-checked id — never raw document text\", but every fault() fills the field with a composed sentence (\"`keyboard:s` is bound to `combat.fire`, which the row never named\"), the field's doc comment still says \"The id, tag, or key the row tripped over\", and tests/common/mod.rs:346-355 now asserts that \"which sentence names it is not part of the contract\". A third-party implementer of policy/v1 produces a different report for the same fault, and nothing in the gate pins report text.","fix":"Reconcile one side: amend spec.md's sentence (and the field doc comment) to say the offender is the grammar-checked id/tag/key at fault plus a bounded reason line, or put the checked id in `offender` and carry the explanation in a separate field, updating the proposal."},{"severity":"minor","location":"crates/transforms/src/apply.rs:583-593, 595-600 (swap_polarity; guard_fault at 644-652; spec.md:183-186)","problem":"Unchanged from round 1: `moves` is still built by filter_map over bound matched ids only, so guard_fault's docked_at(action) leg never sees a matched id the base leaves unbound. With `swap-polarity move.steer-*`, a base binding only `move.steer-right keyboard:d`, and a manifest `docked-actions = [\"move.steer-left keyboard:m\"]`, the row falls through the (Some, Some) test and reports Applied writing nothing, where the unconditional guard promises reserved-input naming the docked input. Relocate cannot hit this because unbound-action fires first.","fix":"Check guard.docked_at over every id in `matched` (a small loop before the filter_map), not only over the bound subset."},{"severity":"minor","location":"crates/transforms/src/apply.rs:748 (produce; schemas/policy/v1/spec.md:226-230)","problem":"Partly addressed from round 1: the seeded_from carry-forward is now declared with a reason in 03-build-notes.md, but spec.md → The output still enumerates only kind, id, derives, for, set and generated-by. The output contract a non-Rust implementer follows therefore omits a field this implementation copies, so the produced documents differ.","fix":"State the carry-forward in spec.md → The output (with the reason the build notes give: a seeded base would otherwise lose that provenance through one application), or set seeded_from: None in the produced base."},{"severity":"nit","location":"crates/transforms/src/apply.rs:350-355, 614-638 (action_on / swap_mouse; spec.md:211-214)","problem":"Made contract-visible by this range's fix: c1 establishes that a base binding one input to several actions is legal and pinned, and the c1 probe case shows swap-mouse-buttons exchanging only the action action_on finds first while other holders on that button stay put. spec.md's clause (\"the actions currently bound to `mouse:left` and `mouse:right`\") does not say which holder the row moves, so a third-party implementer that exchanges every holder answers a different produced base for the same inputs.","fix":"Add one sentence to spec.md's swap-mouse-buttons clause: the row exchanges the action each button carries first in `set` order and leaves any other holder of that button untouched."},{"severity":"nit","location":"crates/transforms/src/apply.rs:243-249","problem":"Unchanged from round 1: the non-base entry refusal tests `!matches!(base.kind, Some(ProfileKind::Base))`, which also covers a document carrying no `kind`, but the message is the fixed string \"`patch` is not a base\".","fix":"Spell what the document actually carried (or `none`) in the message."},{"severity":"nit","location":"crates/transforms/src/apply.rs:113-114","problem":"Unchanged from round 1: the doc comment says \"A fourteenth reason is a `policy/v2` question\" while the closed set is ten reasons (enum at 116-152, pinned by tests/zz_api_probe.rs:85-96), so the next reason is the eleventh.","fix":"Reword to \"an eleventh reason\"."},{"severity":"nit","location":"schemas/policy/v1/fixtures/valid/exact-tag-policy.toml:1-6","problem":"Unchanged from round 1: the fixture comment reasons about `move.strafe-left` never reaching `move.strafe-left-extra`, but the fixture's rows are `swap-polarity move.steer-left` and `relocate-cluster move.strafe-right … keyboard:i`; this comment is contract text for a third-party reader.","fix":"Name the ids actually on the rows."}]}
```
