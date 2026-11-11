# Transcript: design-review:correctness r1c3

## Provenance
- stage: design-review:correctness (round r1c3)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:40:03.554Z
- ended: 2026-11-11T18:43:27.140Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the ratification pass c3: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 6e4ea214d807aebc3faf5a2e28068a7b30b54411 796b2a63f1997540de4eb8a6c4288cacd77ce505` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/008-rust-for-the-product.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "Test strategy #1 vs Formats contract harness paragraph — harness: \"Reach-class table fixtures are asserted by load + distinctness (Test strategy #4), not re-serialization.\" vs Test #1: \"for every directory under `schemas/*/v*/fixtures/valid/`, `bindforge-core` parses the fixture and re-serializes to a semantically equal value\"",
    "problem": "The r1 closure gave reach-classes/v1 a fixtures/valid/ directory, so Test #1's unqualified glob now sweeps reach-class valid fixtures into the re-serialization assertion that the harness paragraph explicitly says does not apply to them. The two sentences directly contradict for that one directory; an implementer cannot tell whether the conformance harness must round-trip reach tables. New, created by the revised text.",
    "fix": "Scope Test #1 to the document formats: \"for every `profile`/`patch`/`manifest`/`policy` directory under `schemas/*/v*/fixtures/valid/`…\", and append \"reach-class fixtures follow #4\", mirroring the harness sentence."
  },
  {
    "severity": "nit",
    "location": "Goals / Non-goals — \"The model's semantics (taxonomy, reach tables, chains, policies, manifest contents) — owned by ADR-009\"",
    "problem": "Re-raised residue of my r1 major (which is substantively closed): the guard paragraph and the Deferred clause now carve out the depth guard, error variant names, and input budget as decided by this design, but the Non-goals bullet still lists `chains` unqualified, restating the ownership the guard paragraph just partially reserved. My r1 fix asked to narrow \"Deferred/Non-goals\"; only Deferred was narrowed. Residual wording ambiguity only — the specific guard sentence resolves the ownership question.",
    "fix": "Mirror the Deferred phrasing in the Non-goals bullet: \"chains, except the depth guard and its boundary fixtures owned by this design\"."
  },
  {
    "severity": "nit",
    "location": "Rollout step 5 — \"the cargo gate commands, the Node floor, the capability-file rule (own commands only; no `fs`/`shell`/`dialog` plugins), tests-under-each-crate\"",
    "problem": "New in the revised text: the design never states a Node floor anywhere; the only Node statement is \"Node version pinned explicitly in a `setup-node` step\" (Gates). Rollout 5 instructs the repo-practices Rust section to record a value this design has not decided.",
    "fix": "Either state the Node floor explicitly (e.g. the LTS line matching tools/orchestration), or rewrite the Rollout 5 item as \"the Node version pinned in the `orchestration` CI job\"."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/008-rust-for-the-product.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/design/correctness-r1c3.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "Chosen design → Gates / Test strategy #5 — Gates: \"`rust` (the three gate commands above, and nothing installed beyond the pinned toolchain (A11))\"; #5: \"every command listed in the Gates block succeeds on the Linux CI runner, with nothing installed beyond the pinned toolchain\"", "problem": "new: the added \"nothing installed beyond the pinned toolchain\" promise holds only if the pinned toolchain carries rustfmt and clippy. The toolchain file this design fixes (\"`rust-toolchain.toml` with `[toolchain] channel = \\\"1.85.0\\\"`\", copied verbatim in Rollout 1) requests no components, and rustup installs a toolchain-file toolchain with the machine's configured profile — the rustup toolchain-file page notes the profile falls back to a possibly-changed default, and the profiles page recommends the `minimal` profile \"in CI\" (and on Windows); minimal ships only rustc, rust-std, cargo, so `cargo fmt` and `cargo clippy` fail on such a clone. Test #5's one-gate promise and Test #7's Windows/contributor leg therefore depend on an environment setting the design never decides.", "fix": "Make the components explicit in the Workspace shape toolchain file: `[toolchain] channel = \"1.85.0\"` plus `components = [\"rustfmt\", \"clippy\"]` (additive regardless of profile). No other text changes."}, {"severity": "nit", "location": "Compatibility matrix (Postgres row) vs Deferred — \"server version pinned and tested by ADR-014's strategy, see Deferred\"", "problem": "new: the revised row points at `## Deferred / out of scope`, but that section's ADR-014 bullet lists only \"Hub DB schema, migrations, upload policy, auth, voting, plus the sqlx offline-cache mechanism and the hub-path `cargo sqlx prepare --check` job\" — Postgres server-version pinning and testing is not among the handed-over obligations, so the new pointer resolves to nothing. The r1 row asserted the same ownership without invoking Deferred, so the dangling cross-reference is created by this revision.", "fix": "Append one clause to the ADR-014 Deferred bullet: \"plus Postgres server version pinning and its test\"; leave the matrix wording unchanged."}, {"severity": "nit", "location": "Test strategy #5 vs Chosen design → Gates — \"every command listed in the Gates block succeeds on the Linux CI runner, with nothing installed beyond the pinned toolchain\"", "problem": "new: \"the Gates block\" is new wording (r1 spelled out the three commands). The Gates section also lists `npm ci && npm run check` for the `orchestration` job, so read as the section, #5 demands the npm commands run \"with nothing installed beyond the pinned toolchain\" — which the same Gates paragraph contradicts by pinning a Node version in a `setup-node` step.", "fix": "Name the code fence rather than the section: \"every command in the Gates code fence (`cargo fmt`/`clippy`/`test`) succeeds …\"."}]}
```
