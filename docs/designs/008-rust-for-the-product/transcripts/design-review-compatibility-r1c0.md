# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:22:28.220Z
- ended: 2026-11-11T18:26:40.442Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/008-rust-for-the-product.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/design/compatibility-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Compatibility matrix — macOS row: \"| macOS | unknown | BindForge commits nothing; Tauri v2 itself supports 10.15+ (A16); desktop design decides |\" (same unknown in \"| Contributor dev env | Linux + Windows yes; macOS unknown |\")","problem":"A matrix row whose status is `unknown` is not a compatibility statement, and it contradicts the document's own Deferred section, which says macOS is \"excluded for v1 per Compatibility matrix\" while the matrix says unknown. The contributor cell's unknown also leaves a macOS contributor unable to tell whether the three cargo gate commands are expected to work for them.","fix":"Give both macOS cells a definite status without taking on macOS work: set the product row to `no (v1 decision)` (the status the Deferred text already claims) and the contributor row to `Linux + Windows yes; macOS no (v1)`; if macOS contributor builds are wanted instead, name a macOS leg in Test strategy #8."},{"severity":"minor","location":"Compatibility matrix / Chosen design — no row or sentence names an OS input API (matrix has rows for Windows/Linux/Flatpak/macOS/CI only)","problem":"The checklist requires X11 vs Wayland vs Windows Raw Input vs macOS HID to be named per platform or the abstraction justified; the document never states which. Its own scope implies the answer — bindings travel as files under `schemas/` and `crates/scanner` does \"installed-game detection, config fingerprinting (`compatdata` enumeration per A19)\", i.e. file reads, not event capture — but the justification is absent, so a later reader cannot tell whether input capture is out of scope or unaddressed.","fix":"Add one sentence to the matrix notes (or Context): BindForge reads and writes binding files only and never reads or injects input events, so no X11/Wayland/Raw Input/HID API is targeted; any future component that captures input must name its API per platform in its own matrix."},{"severity":"minor","location":"Compatibility matrix, Linux row — \"| Linux (deb-based, WebKitGTK 4.1) | yes | dev deps per Tauri list (A16); shipping artifacts deb + AppImage |\"","problem":"The row's support is scoped by packaging (\"deb-based\") while its own notes commit an AppImage artifact, which exists for non-deb distros, so the row both claims and contradicts deb-only support. Arch, Fedora and Steam Deck/SteamOS (read-only, not deb-based) get no status even though the Proton-prefix work that makes them relevant is in this design's member table (\"`crates/scanner` … (`compatdata` enumeration per A19)\").","fix":"Relabel the row to the platform set, e.g. \"Linux (glibc distros; artifacts deb + AppImage)\", and add a Steam Deck/SteamOS row with a status (`no (v1)` is fine if that is the decision) so the matrix states which Linux users get an artifact."},{"severity":"minor","location":"Test strategy #8 — \"the cargo gate must be green on the Linux CI runner; the `rust-toolchain.toml` auto-install is verified once on Linux and once on a Windows contributor machine during rollout\"","problem":"The matrix claims Windows support twice (\"Windows 10 (1803+) / 11 — desktop era | yes (target)\" and \"Contributor dev env | Linux + Windows yes\"), but the only Windows platform check named is a rustup pin install; nothing establishes that `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` are green on Windows, where path-length limits and case-insensitive filesystems are exactly what a Linux-only gate first breaks on.","fix":"Add `windows-latest` to the `rust` job's runner matrix in `.github/workflows/check.yml` (or fold the same three commands into the one-time Windows verification in Rollout step 7), and say so in #8; otherwise downgrade the Windows contributor cell to `yes (toolchain only; gate unverified)`.","fix_note":""},{"severity":"minor","location":"Gates (ADR-008 clause 4), CI paragraph — \"`orchestration` (`npm ci && npm run check` in `tools/orchestration`)\"","problem":"This design creates the repo's first CI config (A23) and therefore owns the Node version running the pipeline gate, but no Node floor is named, so the gate silently changes Node version when the runner image updates — asymmetric with the Rust side, whose goal is \"A toolchain floor and pin so every contributor and CI job builds identically\" (`rust-version = \"1.85\"` plus `rust-toolchain.toml channel = \"1.85.0\"`).","fix":"Name the Node floor once: a `setup-node` step with an explicit `node-version` (or a root `.nvmrc` the job reads), recorded in the `repo-practices` section added by Rollout step 5. No Node API above a floor is introduced, so the finding is the missing floor, not a violation."},{"severity":"nit","location":"Formats contract, Harness — \"walks every `schemas/*/v*/fixtures/{valid,invalid}` directory\" (and Test strategy #1)","problem":"The fixture tree and harness ship with this design, and per the matrix the harness also runs on contributor Windows (case-insensitive, path-length-limited) while CI runs only on Linux; nothing constrains fixture file names, so fixtures differing only in case, or using non-ASCII names, pass on Linux and silently collide or fail locally.","fix":"Add one clause to the versioning rule (mirrored in `schemas/README.md`): fixture file names are lowercase ASCII, `-`-separated, and case-unique within a fixture directory."}]}
```
