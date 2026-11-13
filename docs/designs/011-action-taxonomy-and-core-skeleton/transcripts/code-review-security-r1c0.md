# Transcript: code-review:security r1c0

## Provenance
- stage: code-review:security (round r1c0)
- agent: codereview/code-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T05:05:21.652Z
- ended: 2026-11-13T05:07:18.577Z
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
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the security checklist.
Write your report to docs/designs/011-action-taxonomy-and-core-skeleton/reviews/code/security-r1c0.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "schemas/action-ids/v1/fixtures/invalid/ (missing case); rule at schemas/action-ids/v1/spec.md:116-117 and crates/core/src/action.rs:109-131, action.rs:194-206", "problem": "The diff adds the contract's largest new echo site of untrusted text — the offending id, attacker-controlled up to the parse budget — but no invalid fixture exercises it. Every action-ids invalid fixture carries a short id with no control character, so the harness legs every implementer runs (conformance.rs:223-245: one line, no control character) pass for a reader that echoes a multi-kibibyte id verbatim and never truncates. The escape/truncate rule for ids exists only in this format's spec prose and in Rust tests a third-party reader does not run (tests/action.rs:453-491), whereas profile/v1 already pins the same rule with fixtures/invalid/forged-log-line.toml.", "fix": "Add one one-fault invalid fixture for action-ids/v1 modelled on profile/v1's forged-log-line.toml: a `none` entry holding one id of several hundred characters containing a `\\u0007` and a `\\n` escape, so the corpus's one-line/no-control-character legs bind the escaping for ids; extend the bound leg (conformance.rs's truncation test) with a length assertion driven by that fixture, and register the file in INVALID_FIXTURES (tests/action_ids.rs:113-128) and in 02-proposal.md's corpus list."}, {"severity": "minor", "location": "crates/core/src/action.rs:212-247 (load_action_registry), crates/core/src/lib.rs:37, schemas/action-ids/v1/spec.md:14-19, 02-proposal.md:192-196", "problem": "The contract rule that a registry.toml from anywhere else is data and 'only the copy shipped in this versioned directory is ever consulted for axis_kind' is enforced nowhere. load_action_registry checks internal consistency only (format arm, directory named v<digits>, declared version equals the directory name), and it is public API. The directories the diff's own READMEs declare untrusted (data/manifests/README.md:9-13, data/profiles/README.md:9-13) could be passed as `dir` and yield an attacker-chosen taxonomy, including the discrete-lateral set ADR-015 will act on; no test in the diff would fail, since tests/action.rs:138-145 and tests/action_ids.rs:541-606 deliberately load from scratch directories, i.e. from 'elsewhere'.", "fix": "Make the classification mechanical: give consumers a path-free entry point that resolves the shipped tree (e.g. load_shipped_action_registry()) and keep the path-taking loader crate-internal/test-only, or, if the Deferred 'how an installed artifact locates schemas/ data' question forces the path-taking loader to stay public, state the caller obligation in spec.md ('a caller may pass only the directory the artifact ships with it') and record in 03-build-notes.md an open item binding ADR-013/014 to prove the registry source is the installed copy, not a synced or uploaded tree."}, {"severity": "nit", "location": "schemas/README.md:68-73; new echo sites at crates/core/src/action.rs:249-257, action.rs:201-205, action.rs:239-245", "problem": "The canonical rejection-message contract enumerates the quoted document parts as 'a declared format, a declared version, an entry name'. The diff adds an id value as a fourth quoted part — the only one whose length the writer controls up to 1 MiB — and a reader implementing the README's list literally would treat it as outside the escape-and-truncate duty. The obligation for ids appears only in the format-local spec (action-ids/v1/spec.md:116-117), which is the arrangement this contract tree otherwise rejects ('a narrowing that only lives in one implementation is not a contract', schemas/README.md:39-40).", "fix": "Extend schemas/README.md:68-71 with one clause naming an id value among the quoted parts, so the escape-and-truncate obligation is stated once, canonically, for every part of a document a rejection may quote."}]}
```
