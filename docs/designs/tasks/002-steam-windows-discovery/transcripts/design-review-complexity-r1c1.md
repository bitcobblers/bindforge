# Transcript: design-review:complexity r1c1

## Provenance
- stage: design-review:complexity (round r1c1)
- agent: review/design-complexity
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T05:55:48.157Z
- ended: 2026-11-18T05:59:49.011Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":37420,"output":15154,"reasoning":0,"cache.read":288537,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/tasks/002-steam-windows-discovery/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff d5bafff8b0c75a6a9ae1fbb2c9da1c8c8167c537 b12a4dd76acc0cfadddc1198d7e0c3e0e3747419` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/tasks/T-002-steam-windows-discovery.md,
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
verify what the design claims about the code, and the report at docs/designs/tasks/002-steam-windows-discovery/reviews/design/complexity-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "Chosen design §8 (else-branch: \"engine.register_plugin(Box::new(SteamLinuxPlugin::from_roots(steam_root.iter().cloned())));\") and the Deferred bullet \"An explicit 'these roots are Windows Steam roots' CLI seam that would emit `steam-windows` rows from any host (§8): no follow-up yet.\"",
    "problem": "[new] The deferral contradicts the task's Done-when: \"The `steam-windows` plugin is wired into `apps/discovery-cli` (`discovery list`) alongside `steam-linux`, and `--steam-root` keeps working for it\" (docs/tasks/T-002-steam-windows-discovery.md:65-67), whose subject is the steam-windows plugin. Under §8 the `--steam-root` branch never registers `SteamWindowsPlugin`, so no `steam-windows`-tagged row can ever result from the flag, and the design defers exactly that behavior. The proposal's rebuttal (doubling CLI assertions, \"a platform tag that is a fiction on a Linux runner\") is unsound: the protected assertions live in `apps/discovery-cli/tests`, while the task freezes only the Linux plugin's tests (T-002:34-35), and the design's own rule \"the row's platform tag is the plugin's\" gives the doubled rows a defined meaning.",
    "fix": "Register both plugins on the roots branch: add `engine.register_plugin(Box::new(SteamWindowsPlugin::from_roots(steam_root.iter().cloned())));` beside the Linux one in §8, update the row-count assertions in `apps/discovery-cli/tests/cli_tests.rs` to the doubled expectation, and delete the Deferred bullet. If the author instead reads the clause as merely \"the flag keeps working on Linux\", surface that as an explicit task-deviation question for the runner rather than keeping it inside the document's Deferred list."
  },
  {
    "severity": "minor",
    "location": "Test strategy #2 — \"`steam_windows::usable_install_dir` and `steam_linux::usable_install_dir` differ on no other case.\"",
    "problem": "[new] The test requires `steam_linux::usable_install_dir` to be public; it is private today (crates/discovery/src/steam_linux.rs:275), is absent from A15's preserved-surface list, and Non-goals freeze \"Any change to `SteamLinuxPlugin`'s ... `pub` items\" — so either §1's public-surface-preservation claim breaks or the test cannot be written. The new public item would also have no consumer besides this differential test, growing public surface without need.",
    "fix": "Test the one-rule difference solely through the already-public `parse_manifest` on both sides (the first half of #2, backslash `installdir` yielding `None` vs `Some`, already establishes it) and delete the `usable_install_dir` comparison clause from #2; keep `usable_install_dir` `pub(crate)` in `steam_common`."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/tasks/T-002-steam-windows-discovery.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/tasks/002-steam-windows-discovery/reviews/design/complexity-r1c1.md. That file is the only path this session
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "Chosen design §8 — \"#[cfg(windows)] engine.register_plugin(Box::new(SteamWindowsPlugin::from_roots(steam_root.iter().cloned())));\" and Options — \"§8 instead feeds the flag to the plugin of the compiled-for host, where the task's ':65-67' clause is met\"", "problem": "[new] The branch the revision now cites as satisfying the task's Done-when \"`--steam-root` keeps working for it\" (docs/tasks/T-002-steam-windows-discovery.md:65-67) is executed by no gate the design names. It is compiled out on Linux, so *Test strategy* #9/#10 cannot reach it; #12 only type-checks it (cargo check --all-targets compiles the bin but runs nothing); and #13's manual Windows check enumerates only \"the default install, any secondary library, the `\\\\?\\` prefix in `install_path`, and a non-default install path from the registry\" — no `--steam-root` — with *Rollout* #5 pointing at the same list. The revision therefore replaced the Deferred bullet with a Done-when clause resting on one statement of unexercised code, unlike the document's own convention of naming the lone gate per Windows-only row (Compatibility matrix gnu row: \"*Rollout* #4 runs the `cargo check` for it once\"). My r1 major is not re-raised as such: the deferral is gone and the doubling rebuttal checks out against apps/discovery-cli/tests/cli_tests.rs:68-72, :91, :124, which all pass --steam-root at the fixture and assert exactly four rows.", "fix": "Add \"`discovery list --steam-root <that install>`\" to *Test strategy* #13's enumerated checks and name #13 in *Rollout* #5 as the lone gate for §8's Windows roots branch; point §8's \"keeps working for it\" sentence at that gate instead of at #12."}, {"severity": "minor", "location": "Chosen design §5 — \"`HKLM\\SOFTWARE\\Valve\\Steam` → `InstallPath` as two ordered candidates: `LocalMachineValveSteam32` then `LocalMachineValveSteam64` (A4); values are appended in that order\" vs Test strategy #3 — \"both keys present (HKCU `SteamPath` first, HKLM `InstallPath` second, then the two env candidates)\"", "problem": "[new] The r1 seam change made both HKLM views unconditional candidates, but #3's ordered-list enumeration was not updated: it still expects four entries where the revised rule yields five once both views answer. The two new consequences of always appending both views are also unpinned — the same path reported by both views (§5's \"collapses both views reporting one path\" is prose only; #3's dedupe clause covers registry-vs-env equality only, though the BTreeSet in canonical_paths, crates/discovery/src/steam_linux.rs:322-333, does back it), and distinct paths from the two views, which now scans two roots where the superseded short-circuit scanned one.", "fix": "Restate #3's \"both keys present\" expectation as the five-entry ordered list and add one enumerated case — both HKLM views reporting, same path vs distinct paths — with the expected candidate list for each."}, {"severity": "nit", "location": "Chosen design §3 — \"Which HKLM view to ask first is `host_candidates`' policy, not the reader's (§5)\"", "problem": "[new] \"ask first\" carries over the superseded fallback wording (r0 asked the 64-bit view \"if the first was `None`\"); under §5's always-append rule there is an order, not a first/second branch, and Test strategy #8 asserts \"exactly the three `RegistryKey` variants\" are queried, which a short-circuiting implementation would violate whenever the 32-bit view answers.", "fix": "Say \"which HKLM views to ask, and in what order\" so the sentence cannot be read as license for a short-circuit that contradicts #8."}]}
```
