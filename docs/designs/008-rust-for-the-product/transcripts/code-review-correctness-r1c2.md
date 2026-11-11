# Transcript: code-review:correctness r1c2

## Provenance
- stage: code-review:correctness (round r1c2)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T20:26:30.418Z
- ended: 2026-11-11T20:33:01.119Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/008-rust-for-the-product/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff a8b3aeda622ff4da60863c46e704091e6a74707b..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/008-rust-for-the-product.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "blocker",
    "location": "crates/core/src/toml.rs:97,104 (imports at toml.rs:28-32)",
    "problem": "The F1/F7 refactor moved `read_document` into the `toml` module but lost its `use std::collections::BTreeMap;` import — the module imports only `std::fs`, `std::io::Read`, `std::path::Path`, and `crate::error::CoreError`, and `BTreeMap` appears at exactly the two use sites. `BTreeMap` is not in the 2024-edition prelude, so the `bindforge-core` lib target fails with E0412: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` both error out, zero tests run, and the proposal's deliverable (a workspace \"compiling green behind one cargo gate\") is unmet. Same failure mode round 1 flagged in tests/common/mod.rs, now in src/.",
    "fix": "Add `use std::collections::BTreeMap;` to toml.rs's import block, then run the three gate commands from the Gates block and tick the still-open build-notes checklist boxes."
  },
  {
    "severity": "blocker",
    "location": "crates/core/tests/conformance.rs:119",
    "problem": "The new `load_corpus_table` — the load-half assertion that closes my round-1 finding F3 and Test strategy #4's corpus half — maps loaded classes with `class.to_string()`, but `ReachClass` (reach.rs:61-64) derives only Clone/Debug/PartialEq/Eq and implements no `Display`, so there is no `ToString`: the `conformance` test target fails to build with E0599, and both the harness walk and the new load assertion never run.",
    "fix": "Use `class.id.clone()` (map to the id field) when building the `Vec<String>`, or add a `Display for ReachClass` impl if id-spelling is meant to be public surface."
  },
  {
    "severity": "blocker",
    "location": "crates/core/tests/reach_tables.rs:111-125",
    "problem": "The content assertion that replaced round 1's tautological `!std::ptr::eq` calls `left_classes.contains(&\"near\")` (and five sibling string literals) on a `&[ReachClass]`. `slice::contains` expects `&ReachClass` and `ReachClass` implements `PartialEq` only with itself, so `&&str` arguments fail with E0308: the `reach_tables` target does not compile, and the Test #4 per-hand distinctness assertion — the one thing pinning the hands' content — never runs.",
    "fix": "Assert on ids: e.g. collect `left.classes().iter().map(|c| c.id.as_str())` and check containment of `\"near\"`/`\"mid\"`/`\"extended\"` (right: `near`/`mid`/`cross`), or pass `&ReachClass { id: \"near\".to_string() }`."
  },
  {
    "severity": "nit",
    "location": "schemas/reach-classes/v1/spec.md:52-56; 03-build-notes.md:96-104,228-230",
    "problem": "After the test pass added conformance.rs's `load_corpus_table`, the shipped contract text still says the corpus walk checks only \"parse, grammar, and the closed entry set\" and that the load rules are asserted against the version directory's own `left.toml`/`right.toml`, and the build notes still list the corpus load half as \"needs a harness change\" / a test change the stage could not make. Both artifacts now understate what the shipped harness asserts.",
    "fix": "Add one clause to spec.md's Fixtures section stating the harness also loads each valid corpus table and files its classes under the hand it declares, and update the build-notes round-1-fix/outstanding-test lists to record that the test pass landed."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/code/correctness-r1c2.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "docs/designs/008-rust-for-the-product/03-build-notes.md:62-66,118-120,252-296", "problem": "The notes still describe as pending the four test edits this diff actually ships: conformance.rs:122 now uses class.id.clone(), reach_tables.rs:112-128 asserts the exact id lists, gate_contract.rs:301 pins the fixed `cargo deny --format json check` form, and parse_budget.rs carries the stream budget half. The claims that the load assertion 'does not compile yet (conformance.rs:119)', that clippy/test 'stay red' until the test pass, and the future-tense 'Test changes the next stage must make' section are false about HEAD; the changed test code makes them stale.", "fix": "Mark build-notes test items 1-4 as landed (noting the hand_matches_file_name truth table landed in reach_tables.rs, not parse_budget.rs), drop the conformance.rs:119 'does not compile yet' caveat from the reach-classes deviation entry, and leave only the dated first-party-toolchain gate checklist outstanding."}]}
```
