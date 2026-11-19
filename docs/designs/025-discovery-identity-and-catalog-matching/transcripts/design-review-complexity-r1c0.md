# Transcript: design-review:complexity r1c0

## Provenance
- stage: design-review:complexity (round r1c0)
- agent: review/design-complexity
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T01:24:25.708Z
- ended: 2026-11-19T01:48:54.861Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":985462,"output":23671,"reasoning":0,"cache.read":224904,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/025-discovery-identity-and-catalog-matching/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/025-discovery-identity-and-catalog-matching.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/complexity-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/025-discovery-identity-and-catalog-matching.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/complexity-r1c0.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "Chosen design → Emission — `crates/discovery/src/steam_common.rs` (with Test strategy #4): \"the identity gate becomes bound-only: `slug.len() > bindforge_core::MAX_DISCOVERY_SLUG_OCTETS` is the refusal (`025-L3`)\"; \"The plugin-side skip case at the same bound is asserted separately.\"", "problem": "For the only emitter shipped this run the key is Source::Steam.lookup_key(appid) and parse_manifest \"requires 1–20 ASCII digits\", so the longest slug any input can produce is `steam/` + 20 = 26 octets — 102 short of the 128 bound. The plugin-side comparison is unreachable dead code, and its test case can only be written by hand-building an AppManifest whose appid the parser already refuses, pinning a path no input reaches. `025-L3` names one emit boundary (\"The product checks only `025-B1` at the emit boundary\") and the Goals say the constant \"is the only thing the emit boundary checks\" — that is `refuse_invalid_slugs`, where rows from any `dyn DiscoveryPlugin`, including a future Epic key, are aggregated.", "fix": "Drop the plugin-side `slug.len()` comparison and test #4's last sentence, leaving the 1–20-digit cap plus the one CLI emit boundary to hold `025-B1`. If the defence-in-depth line is kept instead, replace that sentence with the 21-digit skip case test #3 already asserts, so no case fabricates an input the parser refuses."}, {"severity": "minor", "location": "Goals / Non-goals vs Chosen design → Catalog client and gate: Goals \"a connectivity gate with one named refusal\" vs \"Two named refusals, one gate: `error: catalog unreachable: <reason>` … `error: catalog transport unavailable: <reason>`\"", "problem": "The document promises one refusal name in its Goals and ships two, and the second name exists for the loopback endpoint policy — the one mechanism in the design that no Goal states (its rationale appears only in the Options entry and the Risks table, while it costs an error variant, a compatibility-matrix row, and Test strategy #9's policy cases).", "fix": "Add one Goals bullet naming the policy (\"this build's client reaches a loopback authority only; `https` and non-loopback `http` refuse by name\") and make the gate count agree: either say \"two named refusals\" in Goals, or collapse to one name `catalog unreachable: <reason>` whose reason string carries the flag/env/ADR-026 text the Risks mitigation already promises, deleting `TransportUnavailable` rather than adding a Goal for it."}, {"severity": "minor", "location": "Chosen design → Corpus boundary and ledger: \"A corpus sweep asserting no file under `data/` or `schemas/*/v*/fixtures/valid/` contains `steam/`, `epic/`, or `local/` — Rollout 4's … pinned literally against the shipped corpus\" (with `!is_slug_name(friendly_name)` in `boundary_tests.rs`)", "problem": "`025-L5` is asserted three ways and the sweep is the only non-structural one: `!is_slug_name(slug)` already proves it (`lookup_key` always inserts a `/`, which `is_slug_name` refuses — crates/core/src/placement.rs:95-105) and `id-source-key.toml` is refused by the existing conformance walk. The sweep instead constrains unrelated files: the two files under `data/` are prose READMEs, and this design's own store location `$HOME/.local/share/bindforge` contains the substring `local/`, so the first README documenting where data lives fails an unrelated gate. The `!is_slug_name(friendly_name)` clause is contingent for the same reason — it holds only because every current fixture name carries a space or a capital (\"Example Game\", \"Secondary Title\", \"BrokenMarker900201\"), so a lowercase single-word fixture name would fail an assertion about the fixtures, not the design.", "fix": "Delete the corpus sweep and the `!is_slug_name(friendly_name)` clause, keeping `!is_slug_name(slug)` plus the invalid fixture — which is what Rollout 4 names (\"boundary fixtures assert no source key or friendly name reaches a manifest or profile\"); the friendly-name half of `025-L5` stays held by the rule that it is written only to the JSON row and the store."}, {"severity": "nit", "location": "Options considered — Transport, option (a): \"on A17's unverified but plausible reading it puts a C-toolchain build script in the `x86_64-pc-windows-msvc` compile-check leg, which is the gate this run must not break\"", "problem": "The rejection of a maintained client rests on a premise the assumption register itself marks \"plausible but unexercised\" (A17), and the option never considers such a client without its TLS provider (`default-features = false`), whose lack of native code is exactly what would answer the premise; what the premise buys is the hand-rolled `TcpStream` request/response parser, its six parse refusals, the hostile-response risk row, and Test strategy #10's parse cases.", "fix": "Either exercise the premise once on the `x86_64-pc-windows-msvc` check leg, or restate the decision on the ground the document already relies on elsewhere — `Cargo.lock` stays frozen (Rollout 7: \"if one appears, it is a finding\") and repo practice prefers hand-rolled text parsing to a new dependency (crates/core/tests/gate_contract.rs: \"Hand-rolled because a suite that pulls in a YAML parser would be asserting against a dependency\") — so the std-only parse stands on verified grounds."}]}
```
