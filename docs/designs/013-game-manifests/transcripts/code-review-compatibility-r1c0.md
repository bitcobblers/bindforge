# Transcript: code-review:compatibility r1c0

## Provenance
- stage: code-review:compatibility (round r1c0)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T13:38:52.525Z
- ended: 2026-11-13T13:41:48.094Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/013-game-manifests/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff fc71ff96f1143ecc21e01ad20faf22da45360ad2...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/013-game-manifests.md, and your own previous findings quoted below. Do not open
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
Write your report to docs/designs/013-game-manifests/reviews/code/compatibility-r1c0.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "crates/core/tests/manifest.rs:428 and schemas/manifests/v1/fixtures/valid/", "problem": "The format's only free-text column is pinned exclusively with ASCII names. The proposal's basis for the remainder-split row encoding is localized, multi-word game names (A13), yet no valid fixture and none of the 49 new tests carry a non-ASCII native name — grep for non-ASCII over the new corpus, manifest.rs and tests/manifest.rs finds only prose. The two locale-visible properties therefore go unproven against a second implementation: a multi-byte name round-trips byte-exactly only because `quoted` (document.rs:863-881) writes non-control chars raw rather than as \\uXXXX, and MAX_NATIVE_NAME_OCTETS is an octet cap (name.len()), so a 60-character CJK name (180 octets) refuses while a character-counting implementer accepts it. reach-classes/v2 set the precedent of pinning the non-ASCII posture in the corpus; manifests, the format whose whole point is the game's own spelling, ships none.", "fix": "Add a tests/manifest.rs case beside the punctuation test parsing a row whose name is multi-byte (e.g. `aim.camera Caméra \\\"retour\\\"`), asserting the value byte-for-byte and parse(serialize(parse(text))) == parse(text); add a boundary case whose name is under 128 characters but over 128 octets, refused for the octet budget. Preferably also add one fixtures/valid/ document with a localized name, with a line added to the proposal's fixture list and to spec.md's 'Spaces, `#`, and every glyph…' sentence so the language-neutral contract states that non-ASCII is data."}, {"severity": "nit", "location": "crates/core/src/manifest.rs:237-245; asserted at crates/core/tests/manifest.rs:1058-1061", "problem": "The location-rule refusal interpolates {segment}, a component of the caller's local path, into the rendered message, while the proposal states the mismatch 'never echoes a local path (the error contract's channel for paths is source_path)' and error.rs upholds that elsewhere by never rendering CoreError::Io { path }. Today it is bounded by one_line and is the only actionable spelling, but the test that claims the promise asserts only that the full scratch path is absent, so a future edit echoing a parent segment or a truncated display() would keep the suite green while contradicting the contract sentence.", "fix": "Either name only the document's own `id` in the message (the caller supplied the directory), or keep the segment and strengthen the assertion to reject any path separator or display() suffix in the rendered line, so the contract sentence is pinned rather than one scratch root."}, {"severity": "nit", "location": "crates/core/tests/manifest.rs:787", "problem": "The slug-grammar case loop's own doc comment says the rule exists to catch 'a spelling that merely looks lowercase', but the loop tests only an uppercase ASCII spelling, a dot, a leading hyphen and a 64-octet id. A non-ASCII lookalike id (Cyrillic `е`, an accented `joueur-2`, a U+2011 hyphen) is the realistic non-English-contributor mistake, and it is exactly where a directory name and an `id` look equal in a file browser while being unequal byte-for-byte — the name↔location collision the loader's location rule exists to make visible.", "fix": "Add one non-ASCII spelling and one Unicode-hyphen spelling to the loop's list; the byte-based is_slug_name already refuses them, so this pins the claim the comment makes at no behavioral cost."}]}
```
