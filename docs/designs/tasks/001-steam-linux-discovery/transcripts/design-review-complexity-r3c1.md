# Transcript: design-review:complexity r3c1

## Provenance
- stage: design-review:complexity (round r3c1)
- agent: review/design-complexity
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T03:40:20.024Z
- ended: 2026-11-18T03:41:34.735Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":23172,"output":10342,"reasoning":0,"cache.read":187495,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/tasks/001-steam-linux-discovery/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 3. This pass's scope: the sections changed since you last reviewed: run `git diff b08c55894c68e616e5536c73cc7a9921db842309 61907ce31bc8cb18f436caf4fcf96f29ea9c488d` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/tasks/T-001-steam-linux-discovery.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/tasks/001-steam-linux-discovery/reviews/design/complexity-r3c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "Assumption register A9 + Chosen design §2 Root selection — \"`std::env::home_dir()` yields the home directory (on Unix `$HOME`, else the passwd entry)\" / \"`home` comes from `std::env::home_dir()` (A9)\"",
    "problem": "On the toolchain this repo pins (rust-toolchain.toml channel = \"1.85.0\"), std::env::home_dir carries the banner \"Deprecated since 1.29.0: … Consider using a crate from crates.io instead\" (the attribute was removed only in 1.87.0), so the call in from_host() emits the warn-by-default `deprecated` lint, which .github/workflows/check.yml:31 (`cargo clippy --workspace --all-targets -- -D warnings`) promotes to an error. The design's gate claims therefore cannot all hold: either §10's \"`Cargo.lock` stays byte-identical\" + Test strategy #10 + Rollout #3's \"no lock diff\" check fail, or the zero-dependency posture fails, or an undeclared `#[allow(deprecated)]` escape hatch appears at implementation time. A9's cited evidence is today's std doc page, which post-dates the undeprecation and does not describe the pinned floor.",
    "fix": "In §2 and A9 name `std::env::var(\"HOME\")` as the source of `home` — `host_candidates(home, env)` already models that shape, so no new mechanism appears, Cargo.lock stays byte-identical, and §7's existing \"no home directory → `Ok(vec![])`\" branch covers the unset case (only the passwd fallback is given up). Alternatively keep home_dir() but state here that its single call site carries `#[allow(deprecated)]` with a comment naming the 1.87 floor that retires it; if `dirs` is chosen instead, amend §10, A15, Test #10 and Rollout #3 in the same edit."
  },
  {
    "severity": "minor",
    "location": "§4 KeyValues reader + §10 Dependencies and gates — \"Files larger than 1 MiB are skipped as malformed (A16)\" and Risks' \"Both caps are single named constants\"",
    "problem": "These caps are not this design's own numbers: they are the repo's documented shared parse budget (\"larger than 1 MiB or nested deeper than 16 levels are rejected before parsing\" in every schemas/*/spec.md), already implemented as MAX_DOCUMENT_BYTES / MAX_NESTING_DEPTH in crates/core/src/toml.rs:36-39. The proposal re-declares the pair as plugin-local numbers with no cross-reference and no word on why crates/discovery does not take the bindforge-core edge to reuse them, so a reviewer cannot tell whether the match is intentional and the next budget change has two unlinked copies.",
    "fix": "One sentence in §4 tying both caps to the documented budget (\"the schemas/README.md parse budget restated as plugin-local constants, because crates/discovery deliberately has no bindforge-core dependency\"), which also records the reuse decision §10's no-new-dependency posture rests on. Do not add the dependency for this — it would break §10's byte-identical-lock claim for two constants."
  },
  {
    "severity": "minor",
    "location": "§1 Modules and public surface — \"Every `pub` item above is exercised by a test in *Test strategy*; the rest (scan loop, composition, comparison key) stays private\"",
    "problem": "That sentence is the stated justification for a 13-item public surface, and it is false for one item: `SteamLinuxPlugin::from_host()` is pub but no test calls it — Test strategy #5 tests the pure `host_candidates`, #7/#8 use `from_roots`, and #10 states \"no test reads the developer's home directory\". The only production entry point (§8's default path, Rollout #4's \"Default on merge\") sits outside the suite while being cited as covered.",
    "fix": "Amend the sentence to name the exception: \"every pub item except `from_host()`, whose candidate list #5 pins and whose host probe Rollout #5 checks manually\" — rather than justifying public surface with a coverage claim the test list does not support."
  },
  {
    "severity": "nit",
    "location": "§1 (\"`keyvalues.rs` — platform-neutral, no Steam knowledge\") + Compatibility matrix Windows row — \"`keyvalues.rs` is written platform-neutral to be reusable\"",
    "problem": "The reuse rationale points at a consumer that does not exist (Deferred: \"`SteamWindowsPlugin` … no task filed\"), i.e. speculative justification for public API surface, when the actual reason the module must be pub is the crate's existing external-integration-test convention (crates/discovery/tests/{slug,registry,plugin,game}_tests.rs).",
    "fix": "State the test-convention reason in §1 and keep the platform-neutral wording as a property of the code rather than as a promised cross-plugin API."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/tasks/T-001-steam-linux-discovery.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/tasks/001-steam-linux-discovery/reviews/design/complexity-r3c1.md. That file is the only path this session
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
