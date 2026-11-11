# Transcript: design-review:security r1c4

## Provenance
- stage: design-review:security (round r1c4)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:46:08.191Z
- ended: 2026-11-11T18:48:16.844Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the dissent pass c4: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 796b2a63f1997540de4eb8a6c4288cacd77ce505 67a6a1a380395c0cc041d0478f3213660b33681e` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "severity": "major",
    "location": "Chosen design — \"Gates (ADR-008 clause 4)\": \"Per-PR dependency-advisory scanning is deliberately excluded from the gate: it needs a tool outside the pinned toolchain and an advisory database that is live network state\" (and Deferred: \"Dependency-advisory scanning (`cargo deny`, a root `deny.toml`) — no follow-up yet\")",
    "problem": "Regression: the r1 text I approved carried a standing dependency-advisory control (`cargo deny check advisories` against the committed `Cargo.lock`, with `deny.toml` created in Rollout 1). This pass removes it from the gate, from Rollout 1, and from the Revision log's closure, and hands the obligation to a Deferred bullet that names no owner and no cadence — every other Deferred bullet names the document that inherits its concern, this one says \"no follow-up yet\", so no mechanism and no party in the run ever looks at advisories for a lockfile this design commits and freezes with `--locked`. The design still depends on advisory information existing: `rust-toolchain.toml` is bumped when \"a published toolchain security advisory requires it\" (Workspace shape), and the TypeScript boundary's numeric floor is justified as the \"current fix for the IPC-origin advisories\". That justification is already stale on the framework ADR-008 fixes: Tauri GHSA-w28w-mhc8-qvjv (\"Improper Tauri IPC Access Co… [elided by the checkpoint cap]",
    "fix": "Keep the author's objection to the placement — advisory scanning stays out of the three-command per-PR gate — but replace \"no follow-up yet\" with an owner: rewrite the Deferred bullet to name a scheduled, non-PR-blocking workflow running `cargo deny check advisories` against the committed lock with a root `deny.toml`, and add one clause to TypeScript boundary: \"the desktop design pins the newest `tauri` release available when it lands and checks the framework's own security-advisory list, not only crates.io\". No new tooling; if the scheduled job is rejected, the bullet must instead name who checks advisories and on what trigger — the sentence replaces the unowned deferral rather than adding to it."
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
Write your report to docs/designs/008-rust-for-the-product/reviews/design/security-r1c4.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → TypeScript boundary (clause 1; canonical) — \"the desktop design pins the newest `tauri` release available when it lands in `[workspace.dependencies]` (never below 2.11.1)\"","problem":"new — the closure dropped the stale justification but kept the stale number as the floor. A27 in the same document records GHSA-w28w-mhc8-qvjv (High, CVSS 7.6, verified live: affected >= 2.0.0, <= 2.11.5, patched >= 2.11.6), so the floor explicitly permits 2.11.1–2.11.5, and that advisory's mechanism is exactly the control this paragraph relies on: `plugin:__TAURI_CHANNEL__|fetch` is carved out of Tauri's ACL rejection branch, queue is app-wide with old monotonic u32 ids, so a renderer retrieves queued IPC payloads belonging to another webview with no capability check. A floor's only job is to catch downgrades, and this one sits below the patch for the High-severity IPC advisory the paragraph itself cites. Separately, \"newest `tauri` release available\" is ambiguous when read: the newest published `tauri` on crates.io today is 3.0.0-alpha.6 while the newest stable v2 is 2.12.5, and ADR-008 clause 2 fixes \"Tauri v2\", so a literal reading pins an unreleased major's alpha.","fix":"Change the parenthetical to \"(never below 2.11.6, the patch A27 names)\" and qualify the pin as \"the newest stable v2 release available when it lands\". This replaces the 2.11.1 floor rather than adding a second constraint."},{"severity":"minor","location":"Chosen design → Gates (ADR-008 clause 4) — \"its crates.io/RustSec data is not every dependency's only advisory source (A27, TypeScript boundary)\"","problem":"new — the sentence overstates what the newly added control reads. cargo-deny's advisory source is advisory-database git repos (`[advisories] db-urls`, default RustSec/advisory-db); it never reads crates.io or OSV data, which is the comparison A27 is built on. Verified: RustSec's `crates/tauri` directory contains only RUSTSEC-2022-0088 and RUSTSEC-2022-0091 — no v2 advisory at all, so neither GHSA-w28w-mhc8-qvjv nor the 2.11.1 fix GHSA-7gmj-67g7-phm9 is visible to the weekly job. The paragraph's conclusion (check Tauri's own list) is correct but is justified by data the named control never sees, and a reader can easily conclude the scheduled scan enforces the tauri version floor it quotes.","fix":"Name the source in that sentence: \"the RustSec advisory DB it clones (`db-urls`) carries no `tauri` v2 advisory at all, so framework advisories come from Tauri's own list\" — replacing the loose \"crates.io/RustSec data\" phrase."},{"severity":"minor","location":"Chosen design → Gates — \"a `deny.toml` `skip` names the advisory id it waives\"","problem":"new — `skip` is not cargo-deny's advisory-waiver key. Advisory waivers are `[advisories] ignore` entries (`{ id = \"...\", reason = \"...\" }`, optionally `expiry`, with `unused-ignored-advisory` to retire stale waivers); `skip` is the `[bans]` key. An implementer writing `skip` under `[advisories]` gets a config error, and the auditable-waiver property the sentence promises is not provided by the key it names.","fix":"Rename and add the self-limiting fields: \"a `deny.toml` `[advisories] ignore` entry names the advisory id it waives, with a `reason` and an `expiry`\" — one rename plus two fields, replacing the current clause."},{"severity":"nit","location":"Chosen design → Gates — \"installing `cargo-deny` at a pinned version\"","problem":"new — the version is pinned, but `cargo install` without `--locked` re-resolves the tool's transitive dependencies on every weekly run, making this the one CI step that pulls freshly resolved registry code that never enters the repo's `Cargo.lock` — the artifact the rest of the design freezes with `--locked` (Workspace shape; Risks row 1).","fix":"Write it as `cargo install cargo-deny --locked --version X.Y.Z`, replacing the bare \"at a pinned version\" wording."}]}
```
