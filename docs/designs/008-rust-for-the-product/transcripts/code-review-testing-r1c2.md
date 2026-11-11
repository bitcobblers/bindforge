# Transcript: code-review:testing r1c2

## Provenance
- stage: code-review:testing (round r1c2)
- agent: codereview/code-testing
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T20:26:30.421Z
- ended: 2026-11-11T20:33:39.565Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/008-rust-for-the-product/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff a8b3aeda622ff4da60863c46e704091e6a74707b..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/008-rust-for-the-product.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (testing seat):
[
  {
    "severity": "blocker",
    "location": "crates/core/src/toml.rs:28-31 (BTreeMap used at :97, :104)",
    "problem": "`read_document` returns `Result<BTreeMap<String, Value>, CoreError>` and constructs `BTreeMap::new()`, but the module imports only `std::fs`, `std::io::Read`, `std::path::Path`, and `crate::error::CoreError` — `BTreeMap` is not in scope (error[E0412]/[E0433]). This is library source, so `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` fail at the first member: every gate command is red and zero tests execute, re-opening round-1 blocker 1 (the test corpus is again dead) in a new location.",
    "fix": "Add `use std::collections::BTreeMap;` to `crates/core/src/toml.rs`, then run all three gate commands under the pinned 1.85.0 toolchain and paste the `test result:` lines into the build-notes checklist."
  },
  {
    "severity": "blocker",
    "location": "crates/core/tests/conformance.rs:119",
    "problem": "`load_corpus_table` maps `table.classes().iter()` through `|class| class.to_string()`, but `ReachClass` (reach.rs:61-64) derives only Clone/Debug/PartialEq/Eq and implements no `Display`, so `ToString` is unavailable: error[E0599]. The flagship Test #1 conformance target fails to build.",
    "fix": "Use `class.id.clone()` (or `class.id.to_string()`) in the map, or implement `Display` for `ReachClass`; then the comparison against the textually-read `classes` list holds."
  },
  {
    "severity": "blocker",
    "location": "crates/core/tests/reach_tables.rs:114-121",
    "problem": "`left.classes()` is `&[ReachClass]` and `slice::contains` takes `&ReachClass` (T = ReachClass), but the assertions pass `&\"near\"` etc. — type `&&str`, error[E0308] six times. The rewritten Test #4 assertions that replaced the round-1 pointer tautology do not compile, so the suite meant to prove the round-1 fix is itself a build failure.",
    "fix": "Compare ids exactly: `assert_eq!(left.classes().iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), [\"near\", \"mid\", \"extended\"])` and `[\"near\", \"mid\", \"cross\"]` for the right hand, keeping the `hand()` checks and the `assert_ne!`."
  },
  {
    "severity": "major",
    "location": "docs/designs/008-rust-for-the-product/03-build-notes.md:59-68",
    "problem": "The gate-verification checklist (fresh-clone Linux leg, Windows leg, advisories workflow_dispatch run) is still entirely unexecuted — no run of the three gate commands has ever been recorded on any toolchain, in either implementation stage or this revision. With the toml.rs compile error, no toolchain run could pass today; checklist item 6 (run the gate yourself) remained unevaluable a third round, which counts as a finding, not a pass.",
    "fix": "After the compile fixes: install rustup, run the three gate commands under 1.85.0 on a fresh clone with no DATABASE_URL, run the Windows leg once, do one manual advisories dispatch, and paste the command tails into the three dated checklist bullets (Rollout 7)."
  },
  {
    "severity": "minor",
    "location": "crates/core/src/lib.rs:34,31 (hand_matches_file_name, parse_reader re-exports)",
    "problem": "The exported `hand_matches_file_name` — which the build notes' own test-stage item 2 asked `parse_budget.rs` to cover as a pure function — is called by no test, and the stream half of the input budget (`parse_reader`/`read_reader`, the hub-upload entry point the Adversarial guard commits to) is never exercised. Public API and a budget path without a consumer, per checklist item 7.",
    "fix": "Add a unit test for `hand_matches_file_name` (truth table over \"left\"/\"right\"/mismatch/wrong-case) and a `parse_reader` case feeding `Cursor::new(bytes_of_exactly(MAX_DOCUMENT_BYTES + 1))` expecting `DocumentTooLarge`, plus an in-budget stream parsing."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/code/testing-r1c2.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "blocker", "location": "crates/core/tests/gate_contract.rs:330", "problem": "`deny.contains(format!(\"[{section}]\"))` passes a `String` by value where `str::contains` needs `P: Pattern`; the Pattern implementors are `&str`, `&&str`, `char`, `&String`, `&[char]`, `[char; N]`, `&[char; N]` and `F: FnMut(char) -> bool` — never `String` — so the `gate_contract` target fails E0277 (\"expected a `FnMut(char)` closure, found `String`\"). With `--all-targets`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` stay red and the Test #5/#6 pin suite, including the new `cargo deny --format json check` regression guard, never executes: my round-1 blocker 1-3 class (the suite does not build, so zero tests run) is re-opened in a fourth location by the same root cause.", "fix": "`deny.contains(&format!(\"[{section}]\"))` (or `.as_str()`), then grep the workspace for the same shape (`.contains|find|split|replace|starts_with|trim_matches|strip_prefix` with a `format!`/`to_string()` argument) — `tests/common/mod.rs:316` is a `char` pattern and is fine."}, {"severity": "major", "location": "docs/designs/008-rust-for-the-product/03-build-notes.md:68-80 (gate-verification checklist, with :48-66)", "problem": "The three dated bullets — fresh-clone Linux leg with no `DATABASE_URL` (Rollout 7 / Test strategy #5), the Windows leg (#7), and one manual `workflow_dispatch` of `advisories.yml` — are still unchecked and unexecuted in the fourth stage to touch this tree; no gate command has ever run against it, so Test #5 and #7 have no evidence and testing-checklist item 6 remained unevaluable a fourth round (it is unevaluable for this seat too: the shell is denied here, so no green tail could be pasted). With the gate_contract compile error, no such run could pass today either — the pattern of hand-verification in place of running the gate has missed a compile error in each of its three rounds.", "fix": "After the compile fix: install rustup, run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` under the pinned 1.85.0 on a fresh clone with no `DATABASE_URL`; run the same three once on a Windows contributor machine; do one manual advisories dispatch with the \"produced no report\" step green; paste the `test result:` lines and command tails into the three bullets. If no stage can run a toolchain, say so as a blocker rather than a \"not ours to edit\" note."}, {"severity": "minor", "location": "crates/core/src/error.rs:71 (`CoreError::source_path`), error.rs:35 (`NotUtf8`)", "problem": "Checklist item 7 — public surface with no consumer. `source_path` is the documented channel for the `Io` path that `Display` deliberately omits, yet no test pins the pair: `tests/reach_tables.rs:180-183` checks only that the path is absent from the message, never that `source_path()` carries it. `NotUtf8` is reachable only via `parse_reader`, which the revision gave tests without a non-UTF-8 case.", "fix": "Two in-memory cases in `tests/parse_budget.rs`: `parse_reader(Cursor::new(&[0xff, 0xfe, 0x00, 0x00]))` expecting `NotUtf8`, and `parse_file(&scratch.path().join(\"absent.toml\"))` asserting `err.source_path()` names the path while `err.to_string()` does not."}]}
```
