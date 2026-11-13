# Transcript: code-review:testing r1c0

## Provenance
- stage: code-review:testing (round r1c0)
- agent: codereview/code-testing
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T05:08:06.983Z
- ended: 2026-11-13T05:17:51.099Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/011-action-taxonomy-and-core-skeleton/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 4aeb57c2188fbdfc1ca68d99080e4d2f3950d945...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/011-action-taxonomy-and-core-skeleton.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (testing seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/011-action-taxonomy-and-core-skeleton/reviews/code/testing-r1c0.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": ".github/workflows/check.yml:29-33 (gate leg; report-wide)", "problem": "Testing checklist item 6 is not evaluable in this seat: shell access was denied, so `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` were never run and the report carries no `test result:` tail. Per the skill, a checklist item I could not evaluate is a finding, not a pass. Nothing in the static review (import/usage audit against `-D warnings`, fixture-name audit against common::is_fixture_name and the case-unique rule, hand-trace of every new expectation against src/) suggests a red gate, but nothing here proves green either — and the proposal's own Risk row rates 'the new suite never runs locally' High likelihood.", "fix": "Post the merge PR's `rust` job `test result:` lines for crates/core/tests/action.rs and crates/core/tests/action_ids.rs (plus the fmt and clippy legs) into the outstanding evidence checklist at docs/designs/008-rust-for-the-product/03-build-notes.md as Rollout 5 states, and run Test strategy #7's Windows contributor leg once, before merge."}, {"severity": "minor", "location": "crates/core/src/action.rs:229-238 (is_major_dir false arm of load_action_registry)", "problem": "The refusal for a directory that is not named as a major is a branch no test reaches. Every load_action_registry call in the new suite passes `v1` or `v2` (crates/core/tests/action.rs:140, :533; crates/core/tests/action_ids.rs:131, :536, :560, :592), so a caller pointing the loader at `latest/`, at the format directory itself, or at a path whose file_name() is empty (`unwrap_or_default()` -> \"\") is guarded by code the gate can never see fail. Checklist 7 counts an untested dead branch as a finding.", "fix": "Add one case to crates/core/tests/action.rs: write registry.toml into a scratch directory named `latest`, assert load_action_registry returns CoreError::InvalidValue naming the major-directory rule, one line, no local path — or drop the branch and let the declared-version comparison carry the check."}, {"severity": "minor", "location": "crates/core/tests/action.rs:26-29 and crates/core/tests/action_ids.rs:32-33", "problem": "Both new suite headers still state that the tests are 'written from the proposal while crates/core/src/action.rs does not exist yet, so they fail until the design is implemented' / 'before the format ships, so they fail until it does'. In this commit src/action.rs, document.rs's action-ids arm and the whole action-ids corpus are present, so the claim is false about the tree a reader is standing in, contradicts the proves: map, and gives a future fixer an excuse to wave away a red run. It is the same class of stale statement the proposal itself treats as a risk ('Stale ADR-009 lineage misleads the next contributor').", "fix": "Rewrite both sentences to say the suite was written from the proposal and is kept green against the shipped src/action.rs, with no assertion permitted to be weakened to make a change pass."}, {"severity": "minor", "location": "crates/core/tests/action.rs:376-385 (refuses_a_non_string_element_in_an_axis_array)", "problem": "The test asserts only that parse returns some error, with no variant and no named entry. Test strategy #3 groups the non-string element with the duplicate case as a refusal 'naming the id', and schemas/action-ids/v1/spec.md:40-42 types each array as 'array of strings'; as written, a reader that refused `7` for an unrelated reason (Malformed, a budget error, UnknownFormat) passes. Every neighbouring refuses_a_* test in the same file pins the variant, so this is an outlier rather than deliberate looseness.", "fix": "Assert matches!(err, CoreError::InvalidValue { .. }) and that the rendered message names the array (`none`), mirroring refuses_a_duplicate_id_within_one_axis_array_naming_the_id at :387."}, {"severity": "nit", "location": "data/manifests/README.md, data/profiles/README.md (new) vs proposal Rollout 3 + A11", "problem": "The tree facts this stage promises — both data/ placeholders present in the commit, and the new version directory carrying spec.md and registry.toml alongside its corpora — are pinned for fixtures only (crates/core/tests/action_ids.rs:161-193). git tracks no empty directory, so the placeholder guarantee rests on prose, while the repo pins comparable tree facts in tests elsewhere (crates/core/tests/gate_contract.rs:517).", "fix": "Extend the tests/action_ids.rs:161 assertion with spec.md and registry.toml presence, and add one repo_root()-relative assertion that data/manifests/README.md and data/profiles/README.md exist."}]}
```
