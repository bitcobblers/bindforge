# Transcript: code-review:security r1c0

## Provenance
- stage: code-review:security (round r1c0)
- agent: codereview/code-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T19:20:20.395Z
- ended: 2026-11-11T19:36:37.937Z
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
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the security checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/code/security-r1c0.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": ".github/workflows/advisories.yml:28", "problem": "The scheduled advisory scan cannot run in this checkout: rust-toolchain.toml:7 pins channel 1.85.0 and rustup applies the toolchain file to `cargo install`, but cargo-deny 0.20.2 declares rust_version 1.88.0 on crates.io (every release from 0.18.4 up declares 1.88.0), so the install step fails and `cargo deny check advisories` never executes. This workflow is the repo's only recurring look at Cargo.lock (Chosen design -> Gates; Rollout 4) and the named trigger for the toolchain security-advisory bump, so a weekly job no PR depends on silently stops providing any dependency-advisory coverage once axum/sqlx land; OSV reports no advisory for cargo-deny itself, so this is a dead control rather than an unpatched advisory.", "fix": "Run the tool on its own toolchain while keeping the product pin: `rustup toolchain install 1.88.0` then `cargo +1.88.0 install cargo-deny --version 0.20.2 --locked` and `cargo +1.88.0 deny check advisories` (or RUSTUP_TOOLCHAIN=1.88.0 for that job), and note in the header comment that the tool's toolchain is independent of the product pin. Alternatively pin 0.18.3, the newest release with MSRV 1.85.0, and record its staleness bound."}, {"severity": "major", "location": "crates/core/src/document.rs:144-201", "problem": "The grammar is not closed, contradicting the contract shipped in the same diff: schemas/profile/v1/spec.md:22 and schemas/reach-classes/v1/spec.md:30 both say \"Any other entry is rejected\", but parse_profile and parse_reach_classes read only the keys they know and silently ignore every other entry collected by toml::read_document (toml.rs:45-81), and no invalid fixture exercises it. The boundary the design's adversarial guard exists to close therefore accepts uploads carrying forbidden entries: third-party readers written to spec.md reject documents core accepts, hostile uploads can smuggle entries a future major or another implementation interprets, and serialize() drops the unknown entries so a hub-rewritten profile silently changes meaning.", "fix": "Make the grammar closed: after the format dispatch, validate entries.keys() against a per-format allow list (profile: format, version, id, derives; reach-classes: format, version, hand, classes) and reject with a typed error naming the key (e.g. a CoreError::UnknownEntry variant), then add schemas/profile/v1/fixtures/invalid/unknown-entry.toml so the rejection is part of the language-neutral corpus. If open-ended entries are intended for forward compatibility, amend both spec.md files and schemas/README.md in the same commit instead."}, {"severity": "minor", "location": "crates/core/src/document.rs:243-258", "problem": "quoted() escapes only \" \\\\ \\n \\r \\t and writes everything else literally, but parse_string accepts \\b -> U+0008 and \\f -> U+000C (toml.rs:161-162) and stores them in the value. So core serializes a raw control byte inside a quoted string that its own reader then refuses (toml.rs:175): the Test strategy #1 round trip breaks for any valid fixture using \\b or \\f, and a document parsed from another user gets control bytes injected into the re-serialized text, so a more lenient downstream reader reads a different value than core stored.", "fix": "Emit \\\\b and \\\\f beside \\\\n/\\\\r/\\\\t in quoted(), and escape any remaining C0 or DEL code point as \\u{...} (or reject them at toml.rs:161-162 so the reader cannot produce them) so reader and writer agree as the doc comment claims; add a round-trip unit case with \\b in the id."}, {"severity": "minor", "location": "crates/core/src/toml.rs:23-26", "problem": "The DoS half of the adversarial guard is pinned by nothing: MAX_DOCUMENT_BYTES/MAX_NESTING_DEPTH and DocumentTooLarge/NestingTooDeep appear only in src/ (no test or fixture references them), so deleting the 1 MiB check at toml.rs:39 or the depth check at toml.rs:229 leaves the cargo gate green, while the chain half has three tests and two fixtures. The budget is also applied to an already-materialised &str, so a future entry point that reads an upload fully before calling parse pays the memory before the guard fires.", "fix": "Add crates/core/tests/parse_budget.rs asserting DocumentTooLarge for a 1 MiB + 1 byte document and NestingTooDeep at 17 levels (in-memory generated cases, since an oversized fixture would break the corpus's own size rule), and add a capped-read entry helper (Read::take(MAX_DOCUMENT_BYTES + 1)) so future entry points inherit the cap."}, {"severity": "nit", "location": ".github/workflows/check.yml:14", "problem": "Neither check.yml job nor advisories.yml:20-24 sets timeout-minutes, so a PR adding a workspace member with a stalling build script, or a hung `cargo install cargo-deny`, occupies an ubuntu runner for the 6h default. With `pull_request` and `permissions: contents: read` this is runner exhaustion rather than privilege escape.", "fix": "Add timeout-minutes: 20 to the `rust` job and 30 to `orchestration` and `advisories`."}]}
```
