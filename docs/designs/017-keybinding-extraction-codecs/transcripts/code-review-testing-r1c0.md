# Transcript: code-review:testing r1c0

## Provenance
- stage: code-review:testing (round r1c0)
- agent: codereview/code-testing
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-24T06:55:02.123Z
- ended: 2026-11-24T07:03:49.619Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":147682,"output":31095,"reasoning":0,"cache.read":5798773,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff fc28ab94f313d079222d61438bf5b9074a1b8039...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/017-keybinding-extraction-codecs.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/017-keybinding-extraction-codecs/reviews/code/testing-r1c0.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-24T06:32:09.463Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (testing seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/code/testing-r1c0.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only. The entire response must be a single valid JSON object with no prose, explanation, markdown, or additional text before or after it.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"crates/codecs/tests/fixtures/unreal/ (fixture trees present: steam-userdata, proton-prefix, bom-utf16, defaults-only, bindings-absent, broken-tuple); apps/discovery-cli/tests/cli_bindings_status_tests.rs:96-127","problem":"The `symlinked-prefix` fixture tree named in §The first family and Test strategy #6 does not exist, and the scenario it exists to pin — the Compatibility matrix row 'Steam library on NTFS/exFAT with symlinked prefixes … reads as family-confirmed-files-absent (exit 4)' and Risks row 3 — is reached by no test. Exit 4 is pinned only by a located file that removes its own pairs, which is 'files present, nothing stated'. Under crates/codecs/src/paths.rs:127-136 a link template root is dropped, producing no tree, no claim and therefore claim-refused (3) per apps/discovery-cli/src/bindings.rs:224-235, so the documented degradation is both unpinned and, as written, a different status.","fix":"Add the `symlinked-prefix` tree with its links created at test time behind #[cfg(unix)] (#[cfg(not(unix))] early return, as §The first family specifies) plus one CLI subprocess test running `bindings <slug>` against a Proton prefix whose `steamuser` root is a link, asserting the named status and its exit code — whichever of 3/4 the design settles on."},{"severity":"minor","location":"crates/codecs/src/codec.rs:152 (Display arm :186); produced at crates/codecs/src/read.rs:27,36,45 and crates/codecs/src/families/unreal_ini.rs:109","problem":"`ExtractionRefusal::UnreadableFile` is the only variant of the published refusal enum that no test reaches; every other variant is asserted by name somewhere in the new suites, and a workspace-wide grep for `UnreadableFile` returns only src/ hits, so the unreal_ini.rs:109 branch (game_key() declining a tree root that spells neither template) is dead in test terms.","fix":"Add one test that calls UnrealIni.extract on a hand-built Detection whose UserCandidates.tree_root spells neither resolver template (the /synthetic/tree shape in tests/registry.rs:32) and asserts matches!(refusal, ExtractionRefusal::UnreadableFile { .. }) plus that the printed line names the variant."},{"severity":"minor","location":"crates/codecs/src/snapshot.rs:167, :136, :299, :199, :352, :226, :266; schemas/bindings/v1/fixtures/invalid/","problem":"Loader refusals that schemas/bindings/v1/spec.md states — `format` other than \"bindings\", a body that is not a JSON document, an absolute `provenance.files` entry (§Shape/§Privacy: files are tree-relative), an empty `game` or empty token, an entry/`provenance` that is not an object — have neither a fixture nor a direct test. Every case Test strategy #10 enumerates is covered, but since spec §Fixtures makes the corpora the conformance currency, an implementer cannot tell these rules from legal input, and deleting an arm keeps the suite green.","fix":"Add the matching invalid fixtures (files-absolute-files.json, format-not-bindings-format.json, game-empty-game.json, document-not-json.json) named per the convention tests/snapshot.rs:239 reads; the existing every_invalid_schema_fixture_is_refused_naming_its_key loop picks them up with no new test body."},{"severity":"minor","location":"apps/discovery-cli/src/bindings.rs:237-242, :299-303, :250-254, :270-272","problem":"cli_bindings_status_tests.rs pins statuses 0/3/4/5 through the non-probe route only, so the reachable probe-mode failure — `bindings <slug> --probe` on a game no family claims, and its {\"status\":\"claim-refused\",\"claims\":[]} document — is untested (--probe --json is tested only for ok). Separately, the `registry::family()` None arm and the `status == Ok && documents.is_empty()` arm cannot be reached through the shipped registry, so no test can exist for them as written.","fix":"Add a probe-mode status test per failure kind (table mode exit 3 with one `status:` line; --probe --json parsing to status/claims), and either drop the two unreachable arms or comment the invariant that makes them defensive-only."},{"severity":"minor","location":"schemas/README.md (two-JSON-formats passage, new seven-budget bullet with its aggregate product, the hygiene skip sentence); no test in the diff reads the file","problem":"Test strategy #15 names these README statements as contract-tree pins, and the repo pins comparable text elsewhere (crates/core/tests/ledger_annotations.rs:129-151 asserts the README bound bullet against drift), but nothing in the change reads the README: reverting the passage to one JSON format or deleting the codecs budget family and its aggregate sentence leaves JSON_FIXTURE_FORMATS, the conformance walk, the boundary sweep and every Rust suite green.","fix":"Add one ledger_annotations-style test: read schemas/README.md, assert the two-format passage names `catalog` and `bindings`, assert the budget bullet states the seven MAX_* names with values and the (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES product, and assert the hygiene sentence names schemas/bindings/."},{"severity":"nit","location":"crates/codecs/src/paths.rs:76-99; crates/codecs/src/families/unreal_ini.rs:203-231; tests paths.rs:118,135 and unreal_ini.rs:225","problem":"Test strategy #3 and #6 say skipped entries are 'skipped and counted', but no count is observable anywhere — not in a refusal, not in --probe output — so the tests pin only that the walk stops and the hidden candidate is never located, and the 'counted' fact is unpinned.","fix":"Either surface the counts (a probe column or a budget-named line, making them testable) or record the wording divergence in the Revision log so the operator ledger does not carry a fact the tree cannot show."},{"severity":"nit","location":"crates/codecs/tests/common/mod.rs:101 (TempRoot::files_below); crates/codecs/tests/honesty.rs:36-43 (honest_games)","problem":"Two harness helpers are dead in every target, beyond the per-target deadness the module-level allow(dead_code) documents: files_below is never called (the byte-snapshot checks live in cli_bindings_tests.rs:295), and honest_games is #[allow(dead_code)] with a fixture list that matches neither loop below it, implying broken-tuple is honesty-audited when it only ever refuses.","fix":"Delete both, or use files_below for a never-write byte check in the codec suites and drive the honesty loops from honest_games()."}]}
```
