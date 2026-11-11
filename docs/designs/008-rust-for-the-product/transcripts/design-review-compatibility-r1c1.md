# Transcript: design-review:compatibility r1c1

## Provenance
- stage: design-review:compatibility (round r1c1)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:32:29.694Z
- ended: 2026-11-11T18:37:28.429Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 04a4ae1a2d2f75dc291f14e573660a483f51bbaa 6e4ea214d807aebc3faf5a2e28068a7b30b54411` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "location": "Compatibility matrix — macOS row: \"| macOS | unknown | BindForge commits nothing; Tauri v2 itself supports 10.15+ (A16); desktop design decides |\" (same unknown in \"| Contributor dev env | Linux + Windows yes; macOS unknown |\")",
    "problem": "A matrix row whose status is `unknown` is not a compatibility statement, and it contradicts the document's own Deferred section, which says macOS is \"excluded for v1 per Compatibility matrix\" while the matrix says unknown. The contributor cell's unknown also leaves a macOS contributor unable to tell whether the three cargo gate commands are expected to work for them.",
    "fix": "Give both macOS cells a definite status without taking on macOS work: set the product row to `no (v1 decision)` (the status the Deferred text already claims) and the contributor row to `Linux + Windows yes; macOS no (v1)`; if macOS contributor builds are wanted instead, name a macOS leg in Test strategy #8."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix / Chosen design — no row or sentence names an OS input API (matrix has rows for Windows/Linux/Flatpak/macOS/CI only)",
    "problem": "The checklist requires X11 vs Wayland vs Windows Raw Input vs macOS HID to be named per platform or the abstraction justified; the document never states which. Its own scope implies the answer — bindings travel as files under `schemas/` and `crates/scanner` does \"installed-game detection, config fingerprinting (`compatdata` enumeration per A19)\", i.e. file reads, not event capture — but the justification is absent, so a later reader cannot tell whether input capture is out of scope or unaddressed.",
    "fix": "Add one sentence to the matrix notes (or Context): BindForge reads and writes binding files only and never reads or injects input events, so no X11/Wayland/Raw Input/HID API is targeted; any future component that captures input must name its API per platform in its own matrix."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix, Linux row — \"| Linux (deb-based, WebKitGTK 4.1) | yes | dev deps per Tauri list (A16); shipping artifacts deb + AppImage |\"",
    "problem": "The row's support is scoped by packaging (\"deb-based\") while its own notes commit an AppImage artifact, which exists for non-deb distros, so the row both claims and contradicts deb-only support. Arch, Fedora and Steam Deck/SteamOS (read-only, not deb-based) get no status even though the Proton-prefix work that makes them relevant is in this design's member table (\"`crates/scanner` … (`compatdata` enumeration per A19)\").",
    "fix": "Relabel the row to the platform set, e.g. \"Linux (glibc distros; artifacts deb + AppImage)\", and add a Steam Deck/SteamOS row with a status (`no (v1)` is fine if that is the decision) so the matrix states which Linux users get an artifact."
  },
  {
    "severity": "minor",
    "location": "Test strategy #8 — \"the cargo gate must be green on the Linux CI runner; the `rust-toolchain.toml` auto-install is verified once on Linux and once on a Windows contributor machine during rollout\"",
    "problem": "The matrix claims Windows support twice (\"Windows 10 (1803+) / 11 — desktop era | yes (target)\" and \"Contributor dev env | Linux + Windows yes\"), but the only Windows platform check named is a rustup pin install; nothing establishes that `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` are green on Windows, where path-length limits and case-insensitive filesystems are exactly what a Linux-only gate first breaks on.",
    "fix": "Add `windows-latest` to the `rust` job's runner matrix in `.github/workflows/check.yml` (or fold the same three commands into the one-time Windows verification in Rollout step 7), and say so in #8; otherwise downgrade the Windows contributor cell to `yes (toolchain only; gate unverified)`."
  },
  {
    "severity": "minor",
    "location": "Gates (ADR-008 clause 4), CI paragraph — \"`orchestration` (`npm ci && npm run check` in `tools/orchestration`)\"",
    "problem": "This design creates the repo's first CI config (A23) and therefore owns the Node version running the pipeline gate, but no Node floor is named, so the gate silently changes Node version when the runner image updates — asymmetric with the Rust side, whose goal is \"A toolchain floor and pin so every contributor and CI job builds identically\" (`rust-version = \"1.85\"` plus `rust-toolchain.toml channel = \"1.85.0\"`).",
    "fix": "Name the Node floor once: a `setup-node` step with an explicit `node-version` (or a root `.nvmrc` the job reads), recorded in the `repo-practices` section added by Rollout step 5. No Node API above a floor is introduced, so the finding is the missing floor, not a violation."
  },
  {
    "severity": "nit",
    "location": "Formats contract, Harness — \"walks every `schemas/*/v*/fixtures/{valid,invalid}` directory\" (and Test strategy #1)",
    "problem": "The fixture tree and harness ship with this design, and per the matrix the harness also runs on contributor Windows (case-insensitive, path-length-limited) while CI runs only on Linux; nothing constrains fixture file names, so fixtures differing only in case, or using non-ASCII names, pass on Linux and silently collide or fail locally.",
    "fix": "Add one clause to the versioning rule (mirrored in `schemas/README.md`): fixture file names are lowercase ASCII, `-`-separated, and case-unique within a fixture directory."
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
Write your report to docs/designs/008-rust-for-the-product/reviews/design/compatibility-r1c1.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Gates (ADR-008 clause 4) — gate block \"cargo deny check advisories\"; Rollout 1 \"Add root `Cargo.toml`, `rust-toolchain.toml` (contents above), and `deny.toml`\"; Rollout 6 \"contributor prerequisite: rustup one-liner and the Linux system-deps list (A16)\"; Test strategy #5","problem":"The gate gained a fourth command whose binary is nowhere acquired or pinned. cargo-deny is not a rustup component and not a workspace dependency, so neither `rust-toolchain.toml channel = \"1.85.0\"` nor the committed `Cargo.lock` + `--locked` story covers it: the `rust` CI job as specified has no install step, the contributor prerequisite list omits it, and Rollout 5 records \"the cargo gate commands\" without saying how each is obtained. Its advisory database is live network state, so an unchanged PR can flip red the day RUSTSEC publishes — the one gate step outside the design's own goal that \"every contributor and CI job builds identically\". Test #5's one-gate promise still enumerates only fmt, clippy and test, so after this revision the promise no longer covers the whole gate and nothing exercises the new command anywhere. New in this revision.","fix":"Name acquisition and pin once in Gates and repeat in Rollout 4/6: `cargo install cargo-deny@<pinned> --locked` (or a SHA-pinned install action) in the `rust` job, same pin in the contributor prerequisite; pin the advisory database revision in `deny.toml` (`[advisories] database = <rev>`) or state that the command is network-dependent. Then add `cargo deny check advisories` to Test #5's enumeration, or replace #5's three-command list with \"the gate commands listed in Gates\". Dropping the command from the per-PR gate also closes it, if Gates says so explicitly."},{"severity":"minor","location":"Test strategy #7 / Rollout 7 — \"the gate commands and the `rust-toolchain.toml` auto-install are verified once on Linux and once on a Windows contributor machine during rollout, which is where Windows gate greenness is established\"","problem":"Narrower re-raise of the r1 Windows finding: the chosen mechanism is the one I proposed, but the verification is asserted to happen \"during rollout\" while no rollout step performs it — step 7 is \"Verify Test strategy #5 on a fresh clone\", and #5 is the Linux fresh-clone gate. The matrix cell `Contributor dev env | Linux + Windows yes` therefore rests on unassigned work, which is precisely where path-length and case-insensitive-filesystem breakage surfaces.","fix":"Extend Rollout 7: \"Verify Test strategy #5 on a fresh clone and the Windows leg of Test strategy #7 on a Windows machine before announcing to contributors.\""},{"severity":"nit","location":"Compatibility matrix — \"| Postgres (hub runtime, hub era) | yes (target) | compile-time path per the Options sqlx decision; server version pinned by ADR-014 |\"","problem":"After the hub-era `.sqlx` test was removed from Test strategy, this is the only row with a positive status and no test anywhere in the design; the Windows `yes (target)` row at least gets Test #7's \"the desktop packaging matrix is exercised only when the desktop design lands\". Checklist item 7 (a platform test per claimed-supported platform) is satisfied only by omission here.","fix":"One clause in that row's Notes: \"exercised by ADR-014's test strategy, see Deferred\" — so both `yes (target)` rows name where they get exercised."}]}
```
