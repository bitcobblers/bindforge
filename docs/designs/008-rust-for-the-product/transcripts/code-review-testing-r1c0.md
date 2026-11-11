# Transcript: code-review:testing r1c0

## Provenance
- stage: code-review:testing (round r1c0)
- agent: codereview/code-testing
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T19:24:34.828Z
- ended: 2026-11-11T19:35:53.130Z
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
Your own previous findings (testing seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/code/testing-r1c0.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "blocker", "location": "crates/core/tests/common/mod.rs:177", "problem": "`fs::read_dir()` yields `io::Result<DirEntry>`, so `entries.map(|entry| entry.path())` does not compile. Because every suite pulls in `mod common;`, all four test targets (conformance, version_gate, chain_guard, reach_tables) fail to build: `cargo test --workspace --locked` exits 101 with error[E0599] and zero tests execute, so the entire Test strategy #1-#4 corpus is dead code today.", "fix": "Handle the Result — `.filter_map(|entry| entry.ok().map(|entry| entry.path()))` (or propagate as `CoreError::Io`) — then run `cargo test --workspace --locked` and record the `test result:` lines in the build notes."}, {"severity": "blocker", "location": "Cargo.toml:7", "problem": "`members = [\"crates/*\", \"apps/*\"]` names a directory with no member crate. Git does not track empty directories, so a fresh clone — i.e. every CI runner, and Test strategy #5's fresh-clone premise — has no `apps/` at all, and all three gate commands abort at manifest load (`failed to read .../apps/*/Cargo.toml: No such file or directory`) before any crate builds. The one-gate promise fails at its first command.", "fix": "Set `members = [\"crates/*\"]` for this tree, with a comment noting `apps/*` is added by ADR-012 when `apps/cli` lands (and `apps/desktop/src-tauri` in the desktop era); the proposal's glob is a target state, not this tree's state."}, {"severity": "major", "location": "crates/core/src/document.rs:91", "problem": "The gate is not demonstrably green on any toolchain: `clippy::manual_contains` fires under `-D warnings` at document.rs:91 (exit 101), and `cargo fmt --all -- --check` reports 13 hunks across src/lib.rs:24-28, src/reach.rs:16-19, src/document.rs, src/error.rs, src/toml.rs and four test files. rustup is absent in this environment, so rust-toolchain.toml's pinned 1.85.0 channel has never been applied and no run of the three gate commands has ever been green.", "fix": "Honor rust-toolchain.toml (install rustup), run `cargo fmt --all`, rewrite line 91 as `supported_majors().contains(&version)`, and re-run all three gate commands under 1.85.0 with their tails recorded."}, {"severity": "major", "location": "crates/core/src/toml.rs:23,26,39-43,229-233", "problem": "The adversarial parse budget this design commits to the contract (schemas/README.md:46-47: >1 MiB rejected, >16 nesting levels rejected; CoreError::DocumentTooLarge / NestingTooDeep in error.rs:18-20) has no fixture and no test — grep over crates/core/tests finds neither constant nor variant. These are unenforced contract guards for the untrusted uploads the ADR's hub consequence depends on.", "fix": "Mirror the chain guard: an invalid fixture under schemas/profile/v1/fixtures/invalid/ nesting arrays to limit+1 (e.g. nesting-past-limit.toml), plus a tests/parse_budget.rs asserting NestingTooDeep at limit+1 and acceptance at the limit, and DocumentTooLarge for a MAX_DOCUMENT_BYTES+1 document built in memory rather than committed."}, {"severity": "minor", "location": "crates/core/tests/conformance.rs:15-20", "problem": "Test strategy #5, #6 and #7 are cited by no test: #1-#4 all carry `// proves: Test strategy #N`, while #5-#7 exist only as prose here and as comments in .github/workflows/check.yml:4-6. Checklist item 1 counts an unevaluated strategy item as a finding.", "fix": "Add crates/core/tests/gate_contract.rs citing #5/#6 at the level code can pin: read .github/workflows/check.yml and assert both jobs (`rust`, `orchestration`) exist, every third-party action is pinned to a 40-char SHA, `permissions: contents: read` is set per job, and nothing references DATABASE_URL/Postgres. Record the #5 fresh-clone and #7 Windows legs as dated rollout checks with their command output in the build notes (Rollout 7)."}, {"severity": "minor", "location": "crates/core/tests/reach_tables.rs:81", "problem": "The distinctness assertion is `!std::ptr::eq(left, right)` — two elements of the same Vec always have distinct addresses, so the test cannot fail on the bug it claims to guard (both hands loaded from the same data). The loader's file-name/hand-conflict branch (reach.rs:92-111) and Io branch (reach.rs:80-83) are unreachable by any test, and the grammar exclusions schemas/README.md:36-38 declares (table headers, dotted keys, multi-line arrays, floats, hex) — rejected at toml.rs:52-58 — have no invalid fixture.", "fix": "Replace the pointer check with `assert_ne!(left.classes(), right.classes())` plus exact expected class lists (left.toml is near/mid/extended, right.toml near/mid/cross); cover the hand/file-name conflict via an invalid fixture (hand-mismatch.toml) or a pure function exposing that rule; add one invalid fixture per excluded syntax (table-header.toml, dotted-key.toml, multi-line-array.toml, float-value.toml, hex-integer.toml)."}, {"severity": "minor", "location": "schemas/profile/v1/fixtures/valid/basic-profile.toml:5", "problem": "The only valid profile fixture has `derives = []` and a plain ASCII id, so the re-serialization half of Test strategy #1 runs on one trivial document. document.rs:243-257 does not escape \\b (0x08) or \\f (0x0c) that toml.rs:161-162 decodes, and toml.rs:175-177 rejects the raw control character — a document containing \\b or \\f parses, serializes to text its own parser refuses, and the harness never sees such a string.", "fix": "Add a valid profile fixture with a non-empty derives list and an id containing `\\\"`, `\\\\`, `\\t`, `\\b` (and/or a unit round-trip over a string table), so the existing round-trip assertion catches the serializer gap."}, {"severity": "nit", "location": "deny.toml:15-36", "problem": "[bans], [licenses] and [sources] have no consumer: advisories.yml:30 runs only `cargo deny check advisories`. The license allow-list reads like an enforced policy but never runs — a config knob without a consumer.", "fix": "Either run `cargo deny check` (all sections) in the scheduled job, or delete the three sections and state that the scheduled scan is advisories-only."}, {"severity": "nit", "location": "crates/core/tests/conformance.rs:99-120", "problem": "The zero-fixture rule's own trigger is unexercised: the assertions run only against the real tree, and schemas_root() (common/mod.rs:36-43) is hard-wired, so nothing can show the rule fires. Relatedly, common/mod.rs:17's blanket `#![allow(dead_code)]` hides helpers that stop being used across all four targets.", "fix": "Parameterize format_dirs()/fixtures() on a root path and add a case against a scratch tree with an empty fixtures/valid/ asserting the run fails; scope the dead_code allow to the items that need it."}]}
```
