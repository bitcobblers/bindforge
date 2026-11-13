# Transcript: code-review:compatibility r1c0

## Provenance
- stage: code-review:compatibility (round r1c0)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T05:02:40.140Z
- ended: 2026-11-13T05:05:21.612Z
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
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/011-action-taxonomy-and-core-skeleton/reviews/code/compatibility-r1c0.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"crates/core/src/action.rs:221 (with schemas/action-ids/v1/spec.md:14-19 and 02-proposal.md:287)","problem":"The registry's file name is resolved by the filesystem: `dir.join(\"registry.toml\")` never inspects the name it opened, so on a case-insensitive filesystem — the platform the Compatibility matrix calls `yes` — a version directory holding `Registry.toml` loads as the registry while the same tree is refused with an `Io` error on the Linux CI runner. The directory half of the same identity check is byte-exact (`is_major_dir` at action.rs:289 refuses `V1` on every platform), so name matching is asymmetric inside one function, and spec.md never states which rule a third-party reader implements. No fixture or test pins either behaviour.","fix":"Pick one rule and state it: either add a byte-exact-name sentence to spec.md's \"Registry data\" and compare `file_name()` in the loader, or state that file lookup follows the filesystem's case rules and the shipped tree uses lowercase names, and mirror that in the matrix's Windows note."},{"severity":"minor","location":"02-proposal.md:328-331 (Test strategy #7) and 02-proposal.md:287 (Compatibility matrix, Windows row)","problem":"Checklist item 7 is unevaluable rather than a pass. A16 (02-proposal.md:65) records that no stage of this run has invoked a toolchain, so the two new targets and the new corpus have never executed anywhere; the diff rightly declines a fake test-file leg (crates/core/tests/action_ids.rs:20-26), and nothing in the change runs the Windows pass or names who runs it and when. The matrix's \"Windows 10/11 contributor: yes\" therefore rests on inspection only, and no `test result:` line exists for `action` or `action_ids` yet.","fix":"Post the CI job's `test result:` lines for the `action` and `action_ids` targets into this run's build notes, and give the Windows leg an owner and a date; if it cannot be scheduled in this stage, mark the Windows row's evidence as inspection-only in the matrix note."},{"severity":"nit","location":"schemas/action-ids/v1/spec.md:21-35 with crates/core/tests/grammar_closure.rs:131-152","problem":"The new spec tells this format's readers that table headers, dotted keys, multi-line arrays, floats and hex integers are legal TOML that the grammar excludes and rejects, but none of the 14 invalid fixtures carries one of those cases, and the suite that pins them walks `profile/v1/fixtures/invalid/` only. Fixtures are the conformance currency for the language-neutral corpus, so an implementer reading only `action-ids/v1/` gets the claim with no leg to check it against.","fix":"Point the excluded-construct walk at every `schemas/*/v*/fixtures/invalid/` (or add the five cases to `action-ids/v1/`), or add one sentence to spec.md naming `schemas/profile/v1/fixtures/invalid/` as where the shared-construct cases live."},{"severity":"nit","location":"crates/core/src/action.rs:229-232","problem":"`dir.file_name().and_then(|name| name.to_str()).unwrap_or_default()` turns a final path component that is not valid UTF-8 (ordinary on Linux, reachable on Windows for a name with an unpaired surrogate) into `\"\"`, and the refusal then says \"a registry directory is named for a major, like `v1`\" — it misnames the fault, pointing the operator at the file's contents instead of at their own path.","fix":"Keep the `OsStr` and compare bytes for the expected major, or separate the two cases so an undecodable directory name gets a message saying the directory name is not usable text."}]}
```
