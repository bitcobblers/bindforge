# Transcript: code-review:correctness r1c1

## Provenance
- stage: code-review:correctness (round r1c1)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T12:44:34.177Z
- ended: 2026-11-13T12:46:14.851Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 2682fb1f740770fdb588db1a3ff69d66e9d1f3d4..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/012-input-vocabulary-and-reach-classes.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "crates/core/src/document.rs:186-213 (parse's version gate and the `profile`/`action-ids` arms; geometry guard at document.rs:531-537)",
    "problem": "supported_majors() is now [v1, v2] but the version gate is still crate-wide and only reach-classes and keyboard-geometry are version-aware, so a document declaring `version = \"v2\"` with `format = \"profile\"` or `format = \"action-ids\"` is accepted and read with the v1 grammar (parse_profile at document.rs:379 and registry_from_entries at action.rs:179 never check the version), and serialize writes the v2 declaration back. Before this diff any v2 declaration was refused, so the diff opens a fail-open accept path on untrusted input; there is no profile/v2 or action-ids/v2 grammar, directory, or fixture. This contradicts the invariant version_gate.rs:9-12 states (a future-major document parsed with v1 semantics is exactly what the gate exists to prevent) and the versioning rule in schemas/README.md:23-30; build note #4 records the hole as an observation rather than closing it.",
    "fix": "Make the gate per-format: a `supported_majors_for(format)` (reach-classes = [v1, v2]; profile, action-ids, keyboard-geometry = [v1]) checked in each arm, which also subsumes the geometry-only refusal at document.rs:531-537; keep supported_majors() as the union so version_gate's majors-equal-directories test still passes. Add a version_gate.rs case pinning a profile declaring v2 as UnsupportedVersion, and state the per-format rule in schemas/README.md."
  },
  {
    "severity": "minor",
    "location": "crates/core/src/document.rs:310-316 (serialize's reach-classes arm)",
    "problem": "The guard `if table.version == MAJOR_V1 && table.keys.is_empty()` sends a v1-versioned table with a non-empty `keys` (constructible by any consumer, since ReachClassTable's fields are public) down the `else` branch, printing `keys = [...]` under `version = \"v1\"` — a document v1's closed entry set refuses to re-parse, exactly the case reach-classes/v1/fixtures/invalid/keys-entry.toml pins and the comment above the branch says must not happen.",
    "fix": "Branch on the version alone: `if table.version == MAJOR_V1 { head } else { format!(\"{head}keys = [{keys}]\\n\") }`, so only a v2 document writes `keys` and serialize output always re-parses."
  },
  {
    "severity": "minor",
    "location": "crates/core/tests/placement.rs:693-716 (two_moves_onto_the_same_target_key_are_illegal)",
    "problem": "The move set `a→s, d→s` has deltas (1,0) and (-1,0), so shape_ok is false on the common-delta rule and placement.rs:335-345's `targets_are_distinct` never decides the case; the doc comment's claim that the moves share one delta is untrue of them. Clause (a)'s distinct-targets half is therefore unpinned — removing that check leaves the suite green.",
    "fix": "Pin it with a set that really carries one common delta and repeated targets, e.g. [(keyboard:a, keyboard:s), (keyboard:a, keyboard:s)] (both (1,0)), asserting `check.delta == Some((1, 0))` alongside `!check.shape_ok` so the two halves of clause (a) are separated."
  },
  {
    "severity": "minor",
    "location": "docs/designs/012-input-vocabulary-and-reach-classes/03-build-notes.md:58-116 (Gate posture, deviation #1, deviation #2's rationale)",
    "problem": "Checklist item 5 fails against this tree: the notes say 'No test files were written or edited', that conformance.rs still hard-codes v1, that grammar_closure.rs carries only three entry lists, and that Test strategy #3/#5 are 'pinned by nothing yet' — but the diff ships tests/input.rs (6), tests/reach_v2.rs (17), tests/placement.rs (17), tests/zz_api_probe.rs (1), a conformance.rs that takes the fixture's own major and compares grades, and v2/geometry entry lists; the suite runs 119 tests green across 14 targets, not the 75 across 10 the notes quote. Deviation #2's justification (the harness loads from a `...corpus-load-*` stand-in directory) is also stale now that the harness loads from `<scratch>/v1|v2`.",
    "fix": "Rewrite the Gate posture with the current per-target `test result:` lines and rewrite deviation #1 to list only the deviations that actually remain, so the deviation record and the merge PR's evidence leg match the shipped code."
  },
  {
    "severity": "nit",
    "location": "crates/core/tests/zz_api_probe.rs (new target) and crates/core/tests/common/mod.rs:329-382 (quoted_array)",
    "problem": "Both are additions outside the proposal's delta inventory and undeclared in 03-build-notes.md, whose deviation #1 names only the three proposed test targets; zz_api_probe.rs is also the one new Rust target not module-mirrored the way the proposal says its three are.",
    "fix": "Name both paths in the build notes' inventory, or fold the probe's public-surface assertions into tests/reach_v2.rs and tests/placement.rs so the inventory stays the three module-mirrored targets."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/012-input-vocabulary-and-reach-classes/reviews/code/correctness-r1c1.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "docs/designs/012-input-vocabulary-and-reach-classes/03-build-notes.md:98-105, 116-140", "problem": "The test-update stage inside this pass's scope (9c1f601) implemented exactly the three test-side cases the notes record as 'not done here' — the version_gate.rs profile-declares-v2 case (rejects_a_document_declaring_a_major_that_only_another_format_ships), the keyboard:é case in tests/input.rs, and the shared-delta repeated-target move set in tests/placement.rs — and added two tests, so the quoted Gate posture is stale: this tree runs 122 tests green (re-run this pass), with version_gate.rs at 6 and reach_v2.rs at 18, not the quoted 119 / 4 / 17. The merge PR's evidence leg will post test result: lines that do not match the notes' quoted block (checklist item 5, same shape as the round-1 finding).", "fix": "Re-run the three gate legs against the final tree and update the quoted per-target test result: lines and the 'not done here' paragraph — the three cases are now done in the test targets; only the ownership note remains accurate."}, {"severity": "nit", "location": "crates/core/tests/common/mod.rs:103-115", "problem": "The test-update stage added format_majors, a second outside-inventory helper shared by version_gate.rs and tests/action.rs, to the file deviation 1 names only for quoted_array — the same bookkeeping gap round 1's nit closed, one helper late.", "fix": "Widen deviation 1's sentence in 03-build-notes.md to name format_majors beside quoted_array."}]}
```
