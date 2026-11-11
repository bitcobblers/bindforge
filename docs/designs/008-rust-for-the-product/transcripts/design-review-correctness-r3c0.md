# Transcript: design-review:correctness r3c0

## Provenance
- stage: design-review:correctness (round r3c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:58:37.622Z
- ended: 2026-11-11T19:02:07.203Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the ratification pass c0: revisions landed after you approved; you are checking only that they did not break what you approved of round 3. This pass's scope: the sections changed since you last reviewed: run `git diff 67a6a1a380395c0cc041d0478f3213660b33681e bbece027b5eac36f3907aff1df1dd96c51c9f1b8` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Chosen design → Gates — \"installing `cargo-deny` at a pinned version and running `cargo deny check advisories`\" (with Workspace shape's unchanged `rust-toolchain.toml` with `[toolchain] channel = \"1.85.0\"`)",
    "problem": "new: the run is defined inside a checkout whose toolchain file pins 1.85.0, and rustup's toolchain-file override applies to every cargo invocation in that tree, including `cargo install`. Every current cargo-deny release (0.19.x–0.20.2, crates.io `rust_version`) declares MSRV 1.88.0, so the pinned release the job installs cannot build under 1.85.0, and the design's only channel-bump conditions (\"a reviewed dependency upgrade or a published toolchain security advisory\") do not authorize raising the pin for a dev tool that is not a workspace dependency. A25 covers the product graph only, so the new obligation's toolchain requirement is decided nowhere in the document.",
    "fix": "In the Gates sentence name the job's own toolchain: \"installs `cargo-deny` at a pinned version under a `rustup`-installed toolchain at or above that release's `rust-version` (the check.yml 1.85.0 pin is not used by this job)\", plus one assumption-register row for the pinned cargo-deny release's MSRV. Leave check.yml and the workspace floor unchanged."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Gates — \"That same run is the trigger for the `rust-toolchain.toml` security-advisory bump (Workspace shape)\"",
    "problem": "new: the run cannot observe that event. `cargo deny check advisories` matches the Cargo.lock dependency graph against the RustSec advisory DB, which carries advisories for crates published on crates.io; the toolchain is not a Cargo.lock package, so a published toolchain security advisory — the second bump condition Workspace shape allows — never appears in its output. The same sentence concedes the DB is incomplete for dependencies while assuming it is complete for the toolchain.",
    "fix": "Decouple the two: \"that same weekly cadence is when the maintainer checks the Rust security announcements for a `rust-toolchain.toml` bump (Workspace shape); the run itself only reports Cargo.lock crate advisories.\""
  },
  {
    "severity": "minor",
    "location": "Chosen design → Gates — \"a `deny.toml` `skip` names the advisory id it waives\" (Rollout 4 creates the root `deny.toml`)",
    "problem": "new: `skip` is not an advisory-waiver key. cargo-deny's advisories check waives through `[advisories] ignore` (advisory id, `{ id = \"RUSTSEC-…\", reason = … }`, or `crate@version`); `skip`/`skip-tree` belong to `[bans]` and take crate names, and cargo-deny flags unused/unknown advisory-section config. The design names a mechanism the tool does not provide for the waiver it commits to.",
    "fix": "Replace \"`skip`\" with \"`ignore`\" in that clause (the `{ id = …, reason = … }` form if the waiver should carry its reason). No other change."
  },
  {
    "severity": "nit",
    "location": "Test strategy #5 vs Chosen design → Gates — \"every command listed in the Gates block succeeds on the Linux CI runner, with nothing installed beyond the pinned toolchain\"",
    "problem": "re-raised: the referent widened since r1 — the Gates section now also contains `cargo deny check advisories`, which the same paragraph says \"needs a tool outside the pinned toolchain\", and `npm ci && npm run check`. Read as the section, #5 demands commands the section itself excludes from that promise, and the Revision log claim that #5 \"cannot drift from the gate\" no longer holds for the added content.",
    "fix": "Name the code fence rather than the section: \"every command in the Gates code fence (`cargo fmt`/`clippy`/`test`) succeeds …\"."
  },
  {
    "severity": "nit",
    "location": "Revision log → \"Chosen design → Gates (r1, r1c3)\" — \"advisory scanning stays out of it (unpinned tool outside the rustup toolchain, live advisory DB)\"",
    "problem": "new: the retained \"unpinned tool\" reason contradicts the same entry's r1c3 clause and the body, which commits to \"installing `cargo-deny` at a pinned version\". The other two reasons (outside the rustup toolchain; live DB) carry the exclusion on their own.",
    "fix": "Drop \"unpinned\" from that parenthetical."
  },
  {
    "severity": "nit",
    "location": "Chosen design → TypeScript boundary — \"checks Tauri's own repository advisory list — the crates.io/RustSec data the scheduled `cargo deny` scan reads does not carry every v2 advisory (A27)\"",
    "problem": "new: the obligation is one-time at desktop-design landing and has no home afterwards — not in the weekly job (A27 says that job is blind to exactly these advisories), not in Rollout 5's repo-practices rules, and not in the Deferred bullets that enumerate what the desktop design owns (\"Webview UI framework, `apps/desktop` npm toolchain, and its exact workspace-members path\").",
    "fix": "One clause on that Deferred bullet: \"…and the recurring check of Tauri's own repository advisory list — owned by the desktop design\"."
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
Write your report to docs/designs/008-rust-for-the-product/reviews/design/correctness-r3c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[]}
```
