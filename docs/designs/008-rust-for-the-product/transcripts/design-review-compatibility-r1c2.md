# Transcript: design-review:compatibility r1c2

## Provenance
- stage: design-review:compatibility (round r1c2)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:39:05.251Z
- ended: 2026-11-11T18:40:03.448Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 6e4ea214d807aebc3faf5a2e28068a7b30b54411 796b2a63f1997540de4eb8a6c4288cacd77ce505` (both are blob ids of 02-proposal.md), then read the full current document.
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
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "Gates (ADR-008 clause 4) — gate block \"cargo deny check advisories\"; Rollout 1 \"Add root `Cargo.toml`, `rust-toolchain.toml` (contents above), and `deny.toml`\"; Rollout 6 \"contributor prerequisite: rustup one-liner and the Linux system-deps list (A16)\"; Test strategy #5",
    "problem": "The gate gained a fourth command whose binary is nowhere acquired or pinned. cargo-deny is not a rustup component and not a workspace dependency, so neither `rust-toolchain.toml channel = \"1.85.0\"` nor the committed `Cargo.lock` + `--locked` story covers it: the `rust` CI job as specified has no install step, the contributor prerequisite list omits it, and Rollout 5 records \"the cargo gate commands\" without saying how each is obtained. Its advisory database is live network state, so an unchanged PR can flip red the day RUSTSEC publishes — the one gate step outside the design's own goal that \"every contributor and CI job builds identically\". Test #5's one-gate promise still enumerates only fmt, clippy and test, so after this revision the promise no longer covers the whole gate and nothing exercises the new command anywhere. New in this revision.",
    "fix": "Name acquisition and pin once in Gates and repeat in Rollout 4/6: `cargo install cargo-deny@<pinned> --locked` (or a SHA-pinned install action) in the `rust` job, same pin in the contributor prerequisite; pin the advisory database revision in `deny.toml` (`[advisories] database = <rev>`) or state that the command is network-dependent. Then add `cargo deny check advisories` to Test #5's enumeration, or replace #5's three-command list with \"the gate commands listed in Gates\". Dropping the command from the per-PR gate also closes it, if Gates says so explicitly."
  },
  {
    "severity": "minor",
    "location": "Test strategy #7 / Rollout 7 — \"the gate commands and the `rust-toolchain.toml` auto-install are verified once on Linux and once on a Windows contributor machine during rollout, which is where Windows gate greenness is established\"",
    "problem": "Narrower re-raise of the r1 Windows finding: the chosen mechanism is the one I proposed, but the verification is asserted to happen \"during rollout\" while no rollout step performs it — step 7 is \"Verify Test strategy #5 on a fresh clone\", and #5 is the Linux fresh-clone gate. The matrix cell `Contributor dev env | Linux + Windows yes` therefore rests on unassigned work, which is precisely where path-length and case-insensitive-filesystem breakage surfaces.",
    "fix": "Extend Rollout 7: \"Verify Test strategy #5 on a fresh clone and the Windows leg of Test strategy #7 on a Windows machine before announcing to contributors.\""
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix — \"| Postgres (hub runtime, hub era) | yes (target) | compile-time path per the Options sqlx decision; server version pinned by ADR-014 |\"",
    "problem": "After the hub-era `.sqlx` test was removed from Test strategy, this is the only row with a positive status and no test anywhere in the design; the Windows `yes (target)` row at least gets Test #7's \"the desktop packaging matrix is exercised only when the desktop design lands\". Checklist item 7 (a platform test per claimed-supported platform) is satisfied only by omission here.",
    "fix": "One clause in that row's Notes: \"exercised by ADR-014's test strategy, see Deferred\" — so both `yes (target)` rows name where they get exercised."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/design/compatibility-r1c2.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"nit","location":"Chosen design → Workspace shape / Gates / Test strategy #5 — \"`rust-toolchain.toml` with `[toolchain] channel = \"1.85.0\"`\" … \"`rust` (the three gate commands above, and nothing installed beyond the pinned toolchain (A11))\" … \"every command listed in the Gates block succeeds … with nothing installed beyond the pinned toolchain\"","problem":"New in this revision: the revised text makes \"nothing installed beyond the pinned toolchain\" load-bearing — it is the stated reason advisory scanning is excluded and now part of Test #5's one-gate promise — but two of the three gate commands depend on rustup components that the committed `channel` pin does not name: `cargo fmt` needs `rustfmt`, `cargo clippy` needs `clippy`. They are present only because rustup's default install profile happens to include them, so a `--profile minimal` contributor install (or any toolchain installed without those components) fails the gate for a setup reason, not a code reason. A11's basis covers the channel pin only. The three prior compatibility findings (gate-tool acquisition, unassigned Windows verification, Postgres row exercise) are all closed: the gate is back to the three toolchain-shipped commands, Rollout 7 schedules the Windows leg of Test #7, and the Postgres row cites ADR-014's strategy; dropping `cargo deny` does not contradict ADR-008 clause 4, which fixes only clippy + fmt as the check gate. Fix: name the components beside the channel in Workspace shape — `[toolchain] channel = \"1.85.0\"` plus `components = [\"clippy\", \"rustfmt\"]` — so the \"nothing installed beyond the pinned toolchain\" claim is guaranteed by the committed file instead of by the installer's default profile.","fix":"Extend the `rust-toolchain.toml` contents in Workspace shape to declare `components = [\"clippy\", \"rustfmt\"]` alongside `channel = \"1.85.0\"`; this replaces the reliance on rustup's default install profile that Gates and Test #5 now assert away."}]}
```
