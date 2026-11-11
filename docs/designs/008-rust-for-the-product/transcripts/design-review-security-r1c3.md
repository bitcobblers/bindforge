# Transcript: design-review:security r1c3

## Provenance
- stage: design-review:security (round r1c3)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:40:03.555Z
- ended: 2026-11-11T18:43:49.288Z
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
Your own previous findings (security seat):
[
  {
    "severity": "minor",
    "location": "Chosen design — \"Adversarial guard\": \"parsing rejects documents larger than 1 MiB or nested deeper than 16 table/array levels with typed errors\" and \"This design decides that input budget, the two error variant names, and the depth bound\"",
    "problem": "The bound itself closes the round-1 finding, but the sentence claiming to decide \"the two error variant names\" names only Cycle and ChainTooDeep; the size and nesting violations are \"typed errors\" with no names, so the third-party conformance obligation this section exists to publish (\"so third-party readers can conform\") is unfulfilled for the two newly added guards. Nothing in this design's tests pins them either: the same paragraph states \"the invalid-fixture corpus is the contract for that behavior\", yet Test strategy #3 boundary-tests only chain depth and no named fixture exercises >1 MiB or >16 levels — the budget is the one guard in the section with no contract behind the claim. Label: new (created by the r1 text).",
    "fix": "Name the two variants beside Cycle/ChainTooDeep, and extend Test strategy #3 into an input-budget test asserting rejection one level past 16 and one byte past the cap (assert the size case in-test, not as a checked-in fixture, so the corpus carries no 1 MiB file)."
  },
  {
    "severity": "minor",
    "location": "Chosen design — \"TypeScript boundary\": \"the desktop design pins a `tauri` floor no lower than 2.11.1 (current fix for the IPC-origin advisories)\"",
    "problem": "The floor closes the round-1 finding, but the parenthetical is already stale and no mechanism keeps framework advisories in view. A later core advisory is published: CVE-2026-95626 / GHSA-6vxm-x265-58qf (Sept 2026) — Tauri's CSP nonce hardening gives zero protection when the app's script-src includes data: or blob:, i.e. arbitrary script execution in the webview, the exact threat capability-based isolation is cited here against. That advisory exists only as a Tauri repository advisory: it is absent from the GitHub global advisory DB and crates.io OSV, so the design's new `cargo deny check advisories` gate will not surface it, and A13 (\"2.12 line\") sits above a numeric floor of 2.11.1. Label: new (the \"current fix\" claim is new text).",
    "fix": "Make the floor move with reality — \"the newest `tauri` release at the time the desktop design lands, no lower than 2.11.1\" — and add one clause, or a Deferred bullet naming the desktop design, requiring the webview CSP to omit data:/blob: from script-src."
  },
  {
    "severity": "nit",
    "location": "Chosen design — \"Gates (ADR-008 clause 4)\": \"`deny.toml` is committed at the workspace root\"",
    "problem": "The advisory gate is added and pinned to the committed lock, but no triage rule accompanies it. The cheap way to keep a red advisory gate green is a `skip` entry in the committed deny.toml, which restores the informal posture the r1 closure replaced; deny.toml is ordinary contributor-editable text with no review bar named. Label: new.",
    "fix": "One clause in the same sentence: \"a `deny.toml` `skip` names the advisory id and a tracking issue for the upgrade that removes it\" — no new tooling."
  },
  {
    "severity": "nit",
    "location": "Chosen design — \"TypeScript boundary\": \"the `fs`, `shell`, and `dialog` plugins are not enabled\"",
    "problem": "The enumeration closes the round-1 capability finding and does cover the dialog-plugin scope-expansion advisory (CVE-2026-95627), but an enumerated ban reads as permission for unlisted plugins, and the two a desktop app reaches for next are `http` and `updater` — the updater has its own 2026 advisories and brings the update-signing secret and artifact-verification posture, which no document in the run names. Label: new.",
    "fix": "Keep the enumeration in the capability-file rule and pair it with a default — \"no other Tauri plugin is enabled without a reviewed capability entry\" — plus a Deferred bullet assigning the updater signing-key posture to the desktop design."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/design/security-r1c3.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design — \"Gates (ADR-008 clause 4)\": \"Per-PR dependency-advisory scanning is deliberately excluded from the gate: it needs a tool outside the pinned toolchain and an advisory database that is live network state\" (and Deferred: \"Dependency-advisory scanning (`cargo deny`, a root `deny.toml`) — no follow-up yet\")","problem":"Regression: the r1 text I approved carried a standing dependency-advisory control (`cargo deny check advisories` against the committed `Cargo.lock`, with `deny.toml` created in Rollout 1). This pass removes it from the gate, from Rollout 1, and from the Revision log's closure, and hands the obligation to a Deferred bullet that names no owner and no cadence — every other Deferred bullet names the document that inherits its concern, this one says \"no follow-up yet\", so no mechanism and no party in the run ever looks at advisories for a lockfile this design commits and freezes with `--locked`. The design still depends on advisory information existing: `rust-toolchain.toml` is bumped when \"a published toolchain security advisory requires it\" (Workspace shape), and the TypeScript boundary's numeric floor is justified as the \"current fix for the IPC-origin advisories\". That justification is already stale on the framework ADR-008 fixes: Tauri GHSA-w28w-mhc8-qvjv (\"Improper Tauri IPC Access Control for Fetch Command\", high, CVSS 7.6, published 2026-09-26) covers >= 2.0.0, <= 2.11.5 and is patched only at 2.11.6 — above the 2.11.1 floor this design publishes — and it exists only as a Tauri-repository advisory, absent from crates.io OSV/RustSec, so the deleted `cargo deny` gate could not have surfaced it either. Net effect: the automated half of the approved posture is gone and the human half was never added. (Separately confirmed, not a blocker: no dependency at the design's stated versions carries a known unfixed advisory — RUSTSEC-2024-0363 is fixed at sqlx 0.8.1 vs the design's 0.9, and CVE-2026-42184/GHSA-7gmj-67g7-phm9 is fixed at tauri 2.11.1, matching the floor.) Label: regression.","fix":"Keep the author's objection to the placement — advisory scanning stays out of the three-command per-PR gate — but replace \"no follow-up yet\" with an owner: rewrite the Deferred bullet to name a scheduled, non-PR-blocking workflow running `cargo deny check advisories` against the committed lock with a root `deny.toml`, and add one clause to TypeScript boundary: \"the desktop design pins the newest `tauri` release available when it lands and checks the framework's own security-advisory list, not only crates.io\". No new tooling; if the scheduled job is rejected, the bullet must instead name who checks advisories and on what trigger — the sentence replaces the unowned deferral rather than adding to it."}]}
```
