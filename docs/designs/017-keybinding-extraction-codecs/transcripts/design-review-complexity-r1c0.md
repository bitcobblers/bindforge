# Transcript: design-review:complexity r1c0

## Provenance
- stage: design-review:complexity (round r1c0)
- agent: review/design-complexity
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T03:44:30.339Z
- ended: 2026-11-24T03:50:28.058Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":48317,"output":28006,"reasoning":0,"cache.read":1063618,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/017-keybinding-extraction-codecs.md,
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
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/complexity-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/017-keybinding-extraction-codecs.md.
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/complexity-r1c0.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "§Discovery plumbing (`crates/discovery`) — \"`src/steam_common.rs`: `SteamCore::scan()` returns `Vec<ScanRecord>`; `discover()` becomes `Ok(self.scan()?.into_iter().map(|r| r.game).collect())`\"", "problem": "The chosen seam threads per-row root ownership through the code path that produces `list`'s byte-pinned rows (`crates/discovery/src/steam_common.rs:90-130`), and that one choice generates the run's only High-impact compatibility risk (\"The `steam_common` refactor changes what `list` prints\"), Test strategy 1, Rollout 1's re-pin caveat, and a new cross-crate public type that `crates/discovery/tests/zz_api_probe.rs` does not name. It also hands `GameContext` a single owning root, which is not what `017-C2` asks for (\"every steam root the scan resolved — main root and library roots separately\") — an unregistered fourth divergence from the source wording, since the Context section registers only three.", "fix": "Leave `discover()` untouched and add one read-only inherent accessor per Steam plugin returning the two sets the scan already computes (the canonicalized `SteamCore` roots, plus the existing `libraries_in_root_with_skips` at `steam_common.rs:137`, reachable only through that accessor because `mod steam_common;` is private at `lib.rs:6`); make `GameContext` carry `roots: Vec<PathBuf>` / `libraries: Vec<PathBuf>` and have `user_trees` de-duplicate resolved trees by path. This replaces the `ScanRecord` struct, the ownership-preserving rewrite of the shared `canonical_paths` dedupe, the two `scan_records()` wrappers, Test strategy 1 and the `list`-output Risk row, and satisfies `017-C2`'s wording literally instead of by ownership inference."}, {"severity": "minor", "location": "§`bindings/v1` — Bounds — \"`user` one of the three spellings above\"", "problem": "The contract validates a launcher-context vocabulary (1–20 digits | `steamuser` | `host`), turning the resolvers' internal knowledge into a format rule. `017-R2`'s Windows contexts, whose user directory is not `steamuser` (A17), then require a `bindings` major rather than a new resolver, and the loader carries a rule no Goal asks for. Test strategy 10's \"a non-digit `user` under a userdata-shaped label\" makes the coupling concrete: the document has no tree-kind key, so the rule is only implementable as this run's invented pattern union.", "fix": "State `user` as an opaque label under the bounds every other string already rides (1–`MAX_NATIVE_NAME_OCTETS`, no control character, no leading/trailing space) and keep the three spellings as prose about the resolvers; delete the enumeration from the bounds and that one invalid fixture."}, {"severity": "minor", "location": "§Read discipline and budgets — \"`MAX_USER_DIRS_PER_GAME = 16` … `MAX_LOCATE_DEPTH = 8` … design-time\"", "problem": "These are the run's only two budgets without an ADR unit (the proposal says so), and no text establishes the values: the layouts they must cover are `unverified` in the register (A15, A17) and no surveyed layout depth or per-machine SteamID count is recorded. The property that cannot be established from the document is that `MAX_LOCATE_DEPTH = 8` is at least the depth of the deepest layout the family claims to resolve — which is the sole reason the locate walk is generic rather than template-shaped.", "fix": "Add one sentence beside each constant naming the deepest surveyed layout path (native, Proton, userdata) and the observed SteamID count per machine it was sized from, and extend Test strategy 6 to assert a miss at `surveyed_depth + 1`. Keep the constants; ground them."}, {"severity": "minor", "location": "§Test strategy 16 and Rollout 9 — \"and the workspace `Cargo.lock` is unchanged\" / \"any `Cargo.lock` change is a finding\"", "problem": "As worded the pin cannot pass: adding `crates/codecs` to the workspace necessarily changes `Cargo.lock` (a new `[[package]] bindforge-codecs` entry plus new edges from it and from `apps/discovery-cli`), and `serde_json` is not in `[workspace.dependencies]` (`Cargo.toml:20-24`), so the new crate declares it directly. The intended claim (A12: no new external crate, no version change) is true — `serde_json` is already resolved at `Cargo.lock:317` and the `crates/*` glob exists at `Cargo.toml:12`.", "fix": "Restate both places as \"the only `Cargo.lock` diff is the new member's own entry and its edges — no new external package and no version change\", so the gate asserts the property that was meant."}, {"severity": "nit", "location": "§`GameContext` — \"`pub platform: discovery::game::Platform,`\"", "problem": "No rule in the design reads it: the three tree templates are anchored on `root`/`library`/env, and the Windows `claim-refused` answer comes from `from_record`'s digit-appid gate plus no located trees, not from the platform value.", "fix": "Name the clause that consumes `platform`, or drop the field until `017-R2` needs it; `017-C2` asks for the row's identity parts, not its platform."}, {"severity": "nit", "location": "§The CLI face — \"the shared table renderer, generalized from `COLUMNS.len()` to a `&[String]` header so `list`'s five columns and their pinned widths are unchanged\" alongside item 6's \"`discovery list` is untouched\"", "problem": "The two sentences describe editing the renderer `list` renders through (`apps/discovery-cli/src/main.rs:377-420`, where `Row`/`Widths` are fixed-size arrays keyed to `COLUMNS`) while claiming `list` is untouched. Generalizing is the smaller option over a second width algorithm; only the claim is wrong.", "fix": "Say \"`list`'s rendering code is refactored to slice widths with byte-identical output, pinned by `cli_output_polish_tests` and `cli_json_mode_tests`\" instead of \"untouched\"."}]}
```
