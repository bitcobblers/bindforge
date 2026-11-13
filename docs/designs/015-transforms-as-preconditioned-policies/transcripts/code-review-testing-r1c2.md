# Transcript: code-review:testing r1c2

## Provenance
- stage: code-review:testing (round r1c2)
- agent: codereview/code-testing
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T17:31:26.079Z
- ended: 2026-11-13T17:33:41.019Z
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
Your own previous findings (testing seat):
[
  {
    "severity": "minor",
    "location": "crates/core/src/document.rs:1430-1433 (schemas/policy/v1/spec.md:89-90)",
    "problem": "The parse-level \"no repeats\" rule for relocate-cluster targets is stated in the proposal and in the shipped spec.md and enforced in code, but has no test and no invalid fixture, breaking the corpus invariant \"every grammar refusal has its fixture\" (A22) that every other policy refusal honors.",
    "fix": "Add fixtures/invalid/relocate-target-repeats.toml (a within-cap row repeating one input), cite it in spec.md's fixture index, and add it to INVALID_FIXTURE_NAMES in crates/core/tests/policy.rs; at minimum add an inline value_refusal case in policy.rs."
  },
  {
    "severity": "minor",
    "location": "crates/transforms/src/apply.rs:495-496",
    "problem": "Dead branch: the None arm of the axis_kind match (\"is not an id the loaded registry lists\") is unreachable — the match set is built from registry.ids(), and registry_from_entries gives every listed id one of the three kinds, so axis_kind answers Some for every matched id. Untestable dead branches are findings per the testing checklist.",
    "fix": "Collapse the match to the Some spelling or replace the arm with a debug_assert!, keeping a single message path."
  },
  {
    "severity": "minor",
    "location": "crates/transforms/src/apply.rs:508-510",
    "problem": "The untouchable guard's swap-polarity path is reachable (the guard is unconditional over every row, spec.md:176-179) but has no test: guard cases exist for relocate (four shapes) and mouse (docked), none for polarity.",
    "fix": "Add an apply.rs case — e.g. docked-actions = [\"move.steer-left keyboard:a\"] with swap-polarity move.steer-* — asserting reserved-input naming the offender and a byte-equal layout."
  },
  {
    "severity": "minor",
    "location": "crates/transforms/src/apply.rs:724-727",
    "problem": "The not-keyboard verdict is pinned only on the source arm; the target arm of first_not_keyboard — promised by Chosen design → The stack (\"any source or target input not keyboard:*\") — is untested, though mouse:/gamepad: targets parse cleanly as closed-vocabulary input ids.",
    "fix": "Add a relocate-cluster row with mouse:left as a target, asserting not-keyboard naming mouse:left."
  },
  {
    "severity": "minor",
    "location": "checklist item 6 (cargo fmt / clippy -D warnings / cargo test --workspace --locked), whole change",
    "problem": "Could not evaluate: the review environment rejected every shell invocation (Permission denied: shell), so the gate commands could not be run from this seat and no test-result tail can be pasted. Per the review rule an un-evaluable checklist item counts as a finding, not a pass.",
    "fix": "Rely on the pipeline's independent runner leg (Test strategy #11, the merge PR's rust job) to post the green test result lines; no code change suggested — a hand check of test imports against the new public surfaces (lib.rs re-exports, GradeBound.0, PlacementCheck fields, TagPattern::spelling, Transform::kind) found no compile-level inconsistency."
  },
  {
    "severity": "nit",
    "location": "crates/transforms/tests/apply.rs:499-523",
    "problem": "Test strategy #5 promised \"a match that includes aim.camera → axis-kind naming the id\"; the case uses move.steer-right in a scratch registry's continuous-lateral array instead. The deviation is forced by the shipped tag grammar (prefix form needs a literal '-' in the second segment, exact form matches one id so match-count fires first), but no test touches the shipped continuous-lateral id and the none-kind spelling of the same message (apply.rs:491-494) is unexercised.",
    "fix": "Record the reason in the test's docstring, and optionally add a scratch-registry case with two none-array ids under one prefix tag to pin the none spelling."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/015-transforms-as-preconditioned-policies/reviews/code/testing-r1c2.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict": "approve", "findings": [{"severity": "nit", "location": "crates/transforms/src/apply.rs:287-306; crates/transforms/tests/apply.rs:1133-1162", "problem": "The new entry refusal's docstring makes a determinism claim — the refusal names the first action the base's set answers twice — but the pinning test carries exactly one duplicated action, so which of two differently-duplicated actions a report names is unpinned, against the suite's stated posture that which offender a report names is part of the determinism surface.", "fix": "Extend a_base_binding_one_action_twice_refuses_naming_the_duplicated_action with a second duplicated action appearing later in the set array and assert the refusal message names the first duplicate in element order."}, {"severity": "nit", "location": "crates/transforms/src/apply.rs:347-355 (swap_mouse, apply.rs:614-638)", "problem": "The fix's amended model explicitly documents first-answer action_on as intended swap-mouse-buttons semantics ('the action it exchanges is the one it found on the button') while the grammar admits a base binding one mouse button to two actions; no test pins that multi-holder mouse-button shape (what exchanges, which holder stays), so the newly-documented semantic is reachable but unpinned.", "fix": "Add a swap-mouse-buttons case whose base binds a third action to mouse:left alongside the exchanged action, asserting the documented first-answer reading and that the unexchanged holder keeps its binding."}]}
```
