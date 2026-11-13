# Transcript: code-review:performance r1c0

## Provenance
- stage: code-review:performance (round r1c0)
- agent: codereview/code-performance
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T13:44:23.447Z
- ended: 2026-11-13T13:46:28.319Z
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
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/013-game-manifests/reviews/code/performance-r1c0.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "crates/core/src/manifest.rs:385 (cap applied at :394)", "problem": "The MAX_GROUP_MEMBERS budget is checked after the group element is fully split: `let fields: Vec<&str> = raw.split(' ').collect();` materializes one entry per space before the refusal. A single `exclusive-groups` element is bounded only by MAX_DOCUMENT_BYTES (the toml reader has no per-value or per-element bound, toml.rs:314-345), so `exclusive-groups = [\"<1 MiB of spaces>\"]` transiently allocates ~1,048,576 &str entries (~16.8 MB) before refusing — a ~16x amplification of the input budget on the untrusted-input path the design itself names (A14, hostile-manifest risk row). `parse_docked` (:470-475) already avoids this pattern with split_once + contains.", "fix": "Bound the split lazily before collecting: `if raw.split(' ').take(MAX_GROUP_MEMBERS + 1).count() > MAX_GROUP_MEMBERS { …refuse… }`, and derive the `< 2` check from `raw.matches(' ').count() + 1`; collect only after both caps pass. The existing 17-member case (tests/manifest.rs:750-760) keeps covering the refusal."}, {"severity": "minor", "location": "crates/core/src/manifest.rs:283-284, 374-375, 435-436, 462-463, 504-505", "problem": "For every array-shaped entry the length budget is checked on the already-cloned array: string_array (document.rs:840-857) allocates a fresh String per element, and only then does check_row_count compare the length against MAX_MANIFEST_ROWS / MAX_CONSTRAINT_GROUPS. A hostile `actions = [\"\", \"\", …]` under the 1 MiB budget materializes ~350k elements twice (~8 MB of clone on top of the reader's own ~8 MB) before the O(1)-knowable fault refuses. The budget is therefore a model-level bound, not the allocation bound its doc comment advertises (manifest.rs:40-44: \"what stops a hostile manifest driving the load-time join over an unbounded table\").", "fix": "Check the borrowed array's length before cloning it: the length is already on `Value::Array(items)`. Add a small helper (e.g. `array_items(key, value, cap) -> Result<&[Value], CoreError>`) or an inline `if let Value::Array(items) = value { if items.len() > cap { … } }` ahead of `string_array` in each of the five parse functions, keeping the refusal message byte-identical."}, {"severity": "nit", "location": "crates/core/src/manifest.rs:171-180, 288, 296, 406, 413, 441, 480, 509", "problem": "Every one-fact-per-entry rule is a linear scan over the values collected so far (two `iter().any(…)` per actions row, `dead_keys.contains(…)` per docked row, nested `groups.iter().any(|g| g.inputs.contains(…))`), i.e. quadratic in entry length. All are bounded because the cap check precedes each loop — worst case ~65k predicate calls for a max-size `actions` array, ≤ 256×73 for the dead/docked disjointness scan, ~2.6k contains calls for groups — so this costs sub-millisecond once per load and needs no change today. The exposure is the design's own growth path (registry grows in batches, so MAX_MANIFEST_ROWS may ride a future major) where nothing marks these scans as the cost.", "fix": "State the bound in a comment beside each scan, or use a BTreeSet<ActionId>/BTreeSet<String> sidecar for the id and name uniqueness checks — which also matches the set-membership posture ADR-015's precondition checks are described as needing."}, {"severity": "nit", "location": "crates/core/src/manifest.rs:344, 562-578 (decode_base64 at 586-627)", "problem": "Two comments overstate what the budgets buy. (a) MAX_NATIVE_NAME_OCTETS is documented as \"what bounds a refusal that has to echo the name\" (:54-57), but the bound is the render-time 80-char cut in one_line (error.rs:132-134, 155-169): the refusal String is built with the whole value first, so an oversized name allocates up to the document budget before being truncated (same for the duplicate/slug echoes at :290, 301, 442, 482, 510, 539, 546). (b) check_fingerprint decodes the entire base64 body before checking the digest length, so a hostile 1 MiB body drives Vec::with_capacity(786432) and ~786k pushes before the \"32 octets\" refusal; the fingerprint is the only new shape with no length budget at all.", "fix": "Reword the two comments to name the render-time bound, and in check_fingerprint reject a body whose length is not the 44 chars a 32-octet digest encodes to before calling decode_base64 — or export a fingerprint length budget so the format matches the proposal's \"named parse budgets for every new shape\"."}]}
```
