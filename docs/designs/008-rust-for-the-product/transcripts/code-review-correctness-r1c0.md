# Transcript: code-review:correctness r1c0

## Provenance
- stage: code-review:correctness (round r1c0)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T19:20:20.393Z
- ended: 2026-11-11T19:33:40.637Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/008-rust-for-the-product/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff 881d3b48641d7dbb6114de03c5770bc12073bc45...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/008-rust-for-the-product.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/code/correctness-r1c0.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "crates/core/src/document.rs:144-201", "problem": "The reference parser silently accepts any number of unknown top-level entries, contradicting the grammar tables shipped in the same diff (schemas/profile/v1/spec.md:22 and schemas/reach-classes/v1/spec.md:30: \"Any other entry is rejected\"). This is the one place bindforge-core is more permissive than the contract it is the reference for, it runs against the Adversarial-guard premise that the invalid-fixture corpus is the contract for rejection behavior, and no invalid fixture exercises an unknown entry, so crates/core/tests/conformance.rs:77-93 cannot detect it. A third-party reader written from spec.md rejects documents the hub-side reference implementation accepts.", "fix": "After the required entries are read, reject every key outside the format's grammar table with a typed CoreError (InvalidValue { key, message: \"not part of the <format>/v1 grammar\" } or a dedicated UnknownEntry variant), and add schemas/profile/v1/fixtures/invalid/unknown-entry.toml and schemas/reach-classes/v1/fixtures/invalid/unknown-entry.toml so the corpus pins the rule for every implementer."}, {"severity": "major", "location": "crates/core/src/toml.rs:38-83 (also toml.rs:52-58, toml.rs:251)", "problem": "The proposal fixes the contract as \"Formats are TOML (ADR-009) over TOML v1.1.0 (A20)\", but the in-tree reader accepts a hand-rolled line-oriented subset and rejects legal TOML v1.1.0 — table headers, dotted keys, multi-line arrays, floats, hex integers. That narrows which documents the language-neutral contract admits: anyone using a standard TOML parser (ADR-009/ADR-012 authors, third-party readers) accepts a multi-line `derives = [...]` profile that bindforge-core refuses with Malformed. The narrowing is stated only in the new contract-tree docs (schemas/README.md:31-38, the two spec.md grammar sections), not against the proposal's format claim, and this seat could not confirm it is recorded in 03-build-notes.md.", "fix": "Either declare the subset as a deviation in 03-build-notes.md with the dependency-free reason and reword schemas/README.md to say the v1 grammar is a subset of TOML v1.1.0 (not \"TOML over TOML v1.1.0\"), or parse with the real `toml` crate and keep the subset as fixture guidance only. If the build notes already record it, this reduces to the README wording."}, {"severity": "minor", "location": "crates/core/tests/conformance.rs:56-60", "problem": "Valid reach-classes fixtures are parsed and then skipped with `continue`, so they are never load-asserted. The proposal (\"Reach-class table fixtures are asserted by load + distinctness (Test strategy #4)\") and the spec shipped in this diff (schemas/reach-classes/v1/spec.md:41-44) both promise a load assertion, while reach_tables.rs:26-31 loads only the version directory's left.toml/right.toml data. Per-hand distinctness is pinned for the data files and lost for the corpus that ADR-009 will extend.", "fix": "For each reach-classes/v1/fixtures/valid/ fixture, assert the parsed document is a ReachClassTable whose hand matches the harness expectation (or drive load_reach_tables over a temp directory holding the fixture), keeping the invalid-fixture rejection; or record in 03-build-notes.md that the scaffold asserts those fixtures by parse + guard only and correct the spec wording to match."}, {"severity": "minor", "location": "crates/core/src/toml.rs:23-26,39-43,229-233", "problem": "The parse-budget guards (MAX_DOCUMENT_BYTES / MAX_NESTING_DEPTH and the DocumentTooLarge / NestingTooDeep variants) are exercised by nothing — no test and no fixture references them (only src/ does). The design decides this budget here as a constraint ADR-009 must adopt and publishes it for third-party readers in schemas/README.md:46-47, yet only the chain bound is boundary-tested; dropping the size check or off-by-one-ing the nesting limit keeps the whole suite green, and the harness deliberately cannot carry a 1 MiB fixture.", "fix": "Add crates/core/tests/parse_budget.rs (unit, no fixture) asserting parse rejects 1 MiB + 1 bytes with DocumentTooLarge and accepts exactly 1 MiB, and that 17 nesting levels yield NestingTooDeep while 16 do not — written against the constants, the way chain_guard.rs writes against MAX_CHAIN_DEPTH."}, {"severity": "minor", "location": "crates/core/tests/conformance.rs:159-163; crates/core/src/document.rs:100-103,174-180", "problem": "The design's deliverable is a workspace green behind one cargo gate (Test strategy #5), and this seat had no toolchain to run it. Two sites look hand-wrapped rather than rustfmt-canonical — document.rs:100-103 and document.rs:174-180 both fit inside 100 columns and rustfmt normally collapses them — and conformance.rs:159-163 passes the simple binding `kind` to `{}`, which clippy::uninlined_format_args (style, on by default) flags under `-D warnings`. Any of these turns `cargo fmt --all -- --check` or `cargo clippy --workspace --all-targets -- -D warnings` red.", "fix": "Run the three gate commands from a clean checkout (Rollout 7 / Test #5) and apply their own output: `cargo fmt --all`, and inline `{kind}` in the conformance.rs assertion message."}, {"severity": "nit", "location": "crates/core/src/toml.rs:117,206", "problem": "The module docs (toml.rs:1-11) claim the reader \"accepts exactly that grammar and rejects everything else\", but parse_key admits `-` inside bare keys and quoted keys, and parse_integer accepts a trailing underscore so `1_` parses as 1 although TOML forbids it. It does not widen the model surface (typed entry reads reject non-strings), so this is precision only.", "fix": "Tighten parse_key/parse_integer to the TOML bare-key and integer rules, or soften the module doc to say the reader is permissive where the grammar is stricter."}, {"severity": "nit", "location": "crates/core/src/lib.rs:21-23 (crates/core/src/toml.rs:23,26)", "problem": "MAX_DOCUMENT_BYTES and MAX_NESTING_DEPTH are `pub` inside the private `toml` module, so unlike MAX_CHAIN_DEPTH they are not part of the crate's public surface, even though schemas/README.md publishes them as the budget third-party readers and the design's future CLI/desktop/hub entry points must apply before a document reaches a writer.", "fix": "Re-export MAX_DOCUMENT_BYTES and MAX_NESTING_DEPTH from lib.rs alongside MAX_CHAIN_DEPTH, and list them in the lib.rs \"public surface\" doc comment."}]}
```
