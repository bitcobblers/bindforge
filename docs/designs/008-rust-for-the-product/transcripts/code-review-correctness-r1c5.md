# Transcript: code-review:correctness r1c5

## Provenance
- stage: code-review:correctness (round r1c5)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T20:59:50.231Z
- ended: 2026-11-11T21:12:52.976Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/008-rust-for-the-product/02-proposal.md.
This is the ratification pass c5: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 42d019b292b97c70c49f656a0b8d5304bf9b5008..HEAD -- ':(exclude)docs/designs', read against the full current document.
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
    "severity": "minor",
    "location": "docs/designs/008-rust-for-the-product/03-build-notes.md:62-66,118-120,252-296",
    "problem": "The notes still describe as pending the four test edits this diff actually ships: conformance.rs:122 now uses class.id.clone(), reach_tables.rs:112-128 asserts the exact id lists, gate_contract.rs:301 pins the fixed `cargo deny --format json check` form, and parse_budget.rs carries the stream budget half. The claims that the load assertion 'does not compile yet (conformance.rs:119)', that clippy/test 'stay red' until the test pass, and the future-tense 'Test changes the next stage must make' section are false about HEAD; the changed test code makes them stale.",
    "fix": "Mark build-notes test items 1-4 as landed (noting the hand_matches_file_name truth table landed in reach_tables.rs, not parse_budget.rs), drop the conformance.rs:119 'does not compile yet' caveat from the reach-classes deviation entry, and leave only the dated first-party-toolchain gate checklist outstanding."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/code/correctness-r1c5.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": ".github/workflows/advisories.yml:54-58", "problem": "The added permission scope's stated reason is a tool fact that does not hold: the comment says actions/upload-artifact cannot run without `actions: read`. Uploading an artifact produced by the same run needs no `actions` scope; the `actions` namespace is what cross-run/cross-repo artifact reads need (`actions: read`) and what delete/overwrite needs (`actions: write`) — hence the ecosystem convention of `actions: read` on downloaders and `actions: write` on uploaders. The scope is therefore inert here, and if an explicit upload scope were wanted the level would be `write`, not `read`, so the comment points the next editor at the wrong knob precisely if the dated `workflow_dispatch` check reports a 403 at the upload step. It also puts the job's permissions one line beyond the proposal's \"Both workflows declare `permissions: contents: read` at the job level\" (Gates), which build-notes:281 records but the design does not.", "fix": "Drop the causal claim: either remove the `actions: read` line (the upload works on `contents: read`, which the dispatch checklist bullet then proves) or set `actions: write` with a comment naming it as the artifact-upload scope and have the design owner mirror that in the Gates paragraph."}, {"severity": "nit", "location": ".github/workflows/advisories.yml:83,95,108", "problem": "The captured report file is named `cargo-deny-report.json` while, since this round, it holds a SARIF 2.1.0 document — which is what the guard greps for (`\"runs\"`) and what the build notes' checklist bullet expects (\"a `cargo-deny-report` artifact whose file holds a SARIF document\"). Anyone opening the artifact with JSON tooling gets a SARIF payload under a `.json` name.", "fix": "Rename the redirect target to `cargo-deny-report.sarif` in the scan step, the guard's `grep`/`head`, and the upload step's `path:` — one string in all three places, which gate_contract.rs:457-498 already enforces by construction."}, {"severity": "nit", "location": "crates/core/tests/parse_budget.rs:238", "problem": "`matches!(err, CoreError::NotUtf8 { .. })` matches a unit variant with a struct pattern. It compiles on the pinned 1.85.0 toolchain, but it is a default-on clippy warning (`clippy::unneeded_struct_pattern`) on clippy 1.86+, and the gate is `cargo clippy --workspace --all-targets -- -D warnings`. The design's own anticipated toolchain floor bump (Risks: `rust-version` and `rust-toolchain.toml` move together) would then turn the gate red on this line.", "fix": "Write `matches!(err, CoreError::NotUtf8)` — the rest of the suite already uses braces only where fields exist."}]}
```
