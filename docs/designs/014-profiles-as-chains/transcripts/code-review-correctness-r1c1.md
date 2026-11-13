# Transcript: code-review:correctness r1c1

## Provenance
- stage: code-review:correctness (round r1c1)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T15:41:36.853Z
- ended: 2026-11-13T15:43:53.951Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/014-profiles-as-chains/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 4844669384fd046eebfd47960283e178990617a4..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/014-profiles-as-chains.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "blocker",
    "location": "crates/core/src/chain_ops.rs:763 (emit_composition, 746-789)",
    "problem": "The `exchanged` partner guard tests the wrong key (`exchanged.contains(other)` while the code inserts `action`), so the second half of any reciprocal base-binding pair always hits `composition_fault`. `emitted.swap` can therefore never be non-empty: every `merge` whose composed result keeps a net exchange refuses as a conflict, including plainly compatible ones where the two patches touch disjoint actions (`p1 = swap a b`, `p2 = set c k` must merge into one patch with `set [c k]` + `swap [a b]`). That contradicts the proposal's merge contract (\"Compatible overlaps are not conflicts\", deterministic element order including swap elements) and breaks ADR-014 Decision Three's `merge` for a whole input class. No test sees it: no merge test asserts a non-empty `swaps_of(&merged)` (chain_ops.rs:1984 asserts it empty), and the two swap-carrying merge tests (:2017, :2046) expect a refusal either way.",
    "fix": "Refuse only on a non-reciprocal composition and skip the partner of an already-emitted pair: `if !reciprocal { return Err(composition_fault(action, other)); } if exchanged.contains(other) { continue; } exchanged.insert(action.clone()); emitted.swap.push((action.clone(), other.clone()));`. Add the missing pin: `p1 = swap a b` with `p2 = set c k` (or `clear d`) merges to one patch carrying the swap plus the second patch's element, and resolves over the base to the same layout as the two-patch chain; keep the `swap X Y` under `clear X` case refusing via the `!reciprocal` branch."
  },
  {
    "severity": "major",
    "location": "crates/core/src/error.rs:141",
    "problem": "A shipped refusal string was edited to satisfy a new test: `CoreError::Cycle`'s Display text gained \"(`Cycle`)\". The proposal states \"No `CoreError` variant is removed or changed; one is added\", the compatibility matrix pins existing `resolve_chain` refusals as untouched, and A15 pins the shipped guard as delivered. The only driver is the needle list in the new assertion at crates/core/tests/chain_ops.rs:1281 (`[\"Cycle\", \"link-a\", \"link-b\"]`), which the original message cannot satisfy — and which no message can satisfy for the two link ids since `Cycle` carries no payload, so the test passes on the `\"Cycle\"` needle alone via `assert_refusal_names`' `any()`. A frozen contract string should not move to make a test green.",
    "fix": "Revert the arm to `write!(f, \"chain derives from itself\")` and drop `\"Cycle\"` from that needle list; the variant is already asserted at chain_ops.rs:1277-1280, which is what Test strategy #3 pins."
  },
  {
    "severity": "minor",
    "location": "crates/core/src/chain_ops.rs:394-408 (check_link_version); test comment at crates/core/tests/chain_ops.rs:1404-1427",
    "problem": "The proposal's Options says the one-directory rule \"reuses `check_dir_version` (A5)\". The code re-implements it and answers `UnsupportedVersion` naming profile's majors, while every shipped loader applying this same rule answers `InvalidValue` (document.rs:1337-1352 via reach.rs:160, placement.rs:160, action.rs:235-241) — the rule the codebase keeps as one definition precisely so loaders \"cannot drift into two spellings of it\" (document.rs:1354-1358). The new test's doc comment claims the chain refuses \"exactly as the shipped loaders already refuse it (`reach_v2.rs`)\", which is false: reach_v2.rs:626-657 pins `InvalidValue` for the identical fault. It fails closed either way, so this is contract consistency plus a wrong comment.",
    "fix": "Call `check_dir_version(directory, &link.version, &format!(\"{expected}.toml\"))` and delete `check_link_version`; or, if `UnsupportedVersion` is the intended chain answer (which schemas/profile/v2/spec.md:120-123 states), record that divergence in the proposal's Options and correct the test comment to name it."
  },
  {
    "severity": "minor",
    "location": "crates/core/src/chain_ops.rs:334-335 with inherit_provenance at :650-656",
    "problem": "`merge` copies `seeded-from`/`generated-by` from the operands onto the merged document — behavior the proposal's merge contract never asks for (it pins only id, inherited derives/for, and the composed delta lists). `generated-by` means \"a label naming what produced the document\", so carrying p1's label asserts that whatever produced p1 produced the merge result: a false fact in a label the design insists is never displayed as authority (ADR-014 Decision Four).",
    "fix": "Leave both provenance labels absent on merge output, or add the inheritance rule to the proposal with a test stating which operand is inherited and why."
  },
  {
    "severity": "minor",
    "location": "crates/core/src/chain_ops.rs:118-119 with open_link at :347-352",
    "problem": "`load_chain` applies the missing-link rule to the head as well, so a missing head answers `ChainLinkMissing { id: <file stem> }`. The proposal's step 1 is `parse_file(head)` and A13 assigns read failures to `Io`, whose variant carries the path for `source_path`; `ChainLinkMissing` deliberately omits the path, so a consumer cannot log which head was missing, and the echoed value is a raw, grammar-unvalidated file stem rather than a profile id. The ancestor rule A18 is about is correct and pinned (:348-351, tests/chain_ops.rs:1322-1346).",
    "fix": "Resolve the head through `parse_file` (its `Io` already carries the path) and apply `ChainLinkMissing` only to links the walk constructed from a `derives` element."
  },
  {
    "severity": "nit",
    "location": "crates/core/src/chain_ops.rs:714-738 (apply_lists)",
    "problem": "`Binding::Base` is never unbound, so the symbol model always moves a `swap` while `resolve`'s rule 3 makes it a no-op when a side is unbound. The effect is over-refusal only (`p1 = set X a`, `p2 = swap X Y` refuses, although over a base leaving Y unbound the composition is just `set X a`), and the comment says the assumption is deliberate — but the proposal says merge composes \"exactly, not heuristically\" and never states the assumption.",
    "fix": "State it in one sentence of the merge section (\"the base is assumed to bind both sides of an exchange; a merge that would need otherwise refuses\") so the refusal is predictable to a third-party implementer."
  },
  {
    "severity": "nit",
    "location": "crates/core/src/document.rs:639",
    "problem": "The proposal says the v2 writer \"branches on `version` alone\", but `serialize_profile_link` skips `kind` and `for` when `kind` is `None`. Unreachable from a parse, but `Profile`'s fields are public, so a consumer can build a v2 model with `kind: None` and get text that will not re-parse — the one case the function's own doc comment promises not to produce.",
    "fix": "Write `kind`/`for` unconditionally on the v2 branch and document that a v2 model must carry a kind, or state the precondition in the doc comment."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/code/correctness-r1c1.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "nit", "location": "crates/core/src/chain_ops.rs:302-309 (`merge` doc), :720-731 (`apply_lists` doc); contract side: 02-proposal.md → Chain operations (`merge`, 460-475)", "problem": "My round-1 nit asked for one sentence in the merge contract stating that the composition assumes the base binds both sides of any exchange either patch names, and that a merge needing otherwise refuses. The sentence now exists in the rustdoc for `merge` and for `apply_lists`, and the behavior it describes is correct and deterministic (`set X a` under `swap X Y` refuses, as does `swap X Y` under `clear X`, both pinned at tests/chain_ops.rs:2207). It is still absent from the proposal's merge paragraph, and `schemas/profile/v2/spec.md` has no merge section, so the only normative text a third-party implementer has for `merge` remains the proposal's \"composes ... exactly, not heuristically\", which does not disclose that a pair with one side re-bound by the other patch is a refusal rather than a composition. The docs half of the fix sits outside this diff's declared scope (`docs/designs` excluded), so no code is wrong; this is a documentation gap that survives the revision.", "fix": "Add to the proposal's merge paragraph: \"the base is assumed to bind both sides of any exchange either patch names; a composition whose answer would depend on the base's bindings has no one-patch form and refuses\" — and, if merge semantics are meant to be conformable by a non-Rust implementer, mirror that sentence in spec.md where the rest of the load-level contract already lives."}]}
```
