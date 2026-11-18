# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T02:53:37.683Z
- ended: 2026-11-18T02:56:46.315Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":54623,"output":27344,"reasoning":0,"cache.read":803269,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/tasks/001-steam-linux-discovery/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
verify what the design claims about the code, and the report at docs/designs/tasks/001-steam-linux-discovery/reviews/design/correctness-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/tasks/T-001-steam-linux-discovery.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/tasks/001-steam-linux-discovery/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict": "request-changes", "findings": [{"severity": "blocker", "location": "Chosen design §3 Library selection — \"- `<root>/steamapps` itself, always.\"", "problem": "The root-as-library inclusion returns a directory one level below what every other section treats as a 'library'. §3's closing line ('`appmanifest_*.acf` files are read only as entries directly inside a selected library's `steamapps/`'), §5's composition (`l.join(\"steamapps\").join(\"common\")`), A2, and the Goals bullet ('a guaranteed scan of each root's own `steamapps/`') all require a library entry to *contain* `steamapps/`, and §2's canonical path rule drops entries lacking one. As written `<root>/steamapps` requires `<root>/steamapps/steamapps`, so it is dropped or scanned at the wrong place, the root's own manifests are never read, and the design's defence against the A12 silent-zero bug is inert. Confirmed against A2/A3's own source (steamlocate-rs/src/library.rs: `Library::from_dir` reads `path.join(\"steamapps\")`; `resolve_app_dir` = path/steamapps/common/<installdir>). Failure scenario: a single-library native install (`~/.local/share/Steam`, no secondary library) yields zero rows, violating T-001's Done-when 'Running `discovery list` on a Linux machine with Steam correctly identifies installed games', and Test strategy #8 ('returns exactly four games') and #3 are unsatisfiable as specified.", "fix": "Make the inclusion \"`<root>` itself, always\" and state in §3 that entries returned by `libraries_in_root` are library roots each containing a `steamapps/` directory, replacing the current off-by-one entry so §2's gate, §3's scan rule and §5's join describe the same directory level."}, {"severity": "major", "location": "§9 Fixtures — \"# modern shape: \\\"0\\\" → \\\".\\\", \\\"1\\\" → \\\"../secondary\\\"\"", "problem": "The `\"0\"` entry contradicts §3's join rule ('otherwise joined to the directory holding the VDF' = `<root>/steamapps`), so `\".\"` resolves to `<root>/steamapps`, not `<root>`; only `\"1\" → \"../secondary\"` is written against that base. The `\"0\"` entry therefore contributes nothing that survives the `steamapps` gate, and §9's claim that 'the `\"0\" → \".\"` entry exercises root/library de-duping' plus Test strategy #3's 'adds no duplicate' assertion test nothing.", "fix": "Keep the join base as the VDF's directory and write the fixture entries as `\"0\" → \"..\"` and `\"1\" → \"../secondary\"`, replacing the current `\".\"`; `\"..\"` canonicalizes to the same directory as §3's root inclusion, which is the de-duping the section claims to exercise, and makes the primary library's manifests reachable."}, {"severity": "major", "location": "§7 Failure policy and the one error path vs Test strategy #7 — \"The single `Err` case: at least one root was selected but **no** library under any selected root could be scanned.\"", "problem": "The predicate is about libraries being unscannable, and §7's skip list already places 'a manifest that fails to parse' on the continue side; Test strategy #7 pins the opposite case ('a root that has a `steamapps` directory whose entries are all unparseable returns exactly one `Err(DiscoveryError::PluginError(_))`'). There the library scanned successfully and yielded nothing, so §7 returns `Ok(vec![])` and the design's own test fails. §7's error text ('the count of files skipped as unreadable or malformed') points the other way again. The alternative reading ('no games emitted from a selected root') would make a legitimately empty install an error, which §7's 'nothing installed' sentence forbids. Risks row 4 names Test #7 as the mitigation for 'a degraded scan prints nothing', so silently changing either side loses the mitigation.", "fix": "Restate the predicate over candidate files: 'at least one library was selected, at least one `appmanifest_*.acf` was found under it, every one was skipped as unreadable/non-UTF-8/unparseable, and no game was emitted', keeping 'no manifests at all' on the `Ok(vec![])` side, so §7 and Test #7 describe one condition."}, {"severity": "major", "location": "§2 Root selection and assumption A9 — \"`home` comes from `std::env::home_dir()` (A9)\"", "problem": "On the pinned toolchain (`rust-toolchain.toml` channel 1.85.0) `std::env::home_dir` still carries `#[deprecated(since = \"1.29.0\")]`: the 1.85.0 reference page shows the deprecation banner and the 1.85.0 release notes defer removal to a later release (undeprecated in 1.87 per the `home` crate). The call site emits a `deprecated` warning, and A18's gate `.github/workflows/check.yml:31` (`cargo clippy --workspace --all-targets -- -D warnings`) turns it into a hard failure, so §10's 'gates pass' and Test strategy #10 are false as written. A9 is marked `verified` against the unversioned doc page, which no longer shows the deprecation and so does not support the pinned-toolchain fact the design depends on.", "fix": "Replace the sentence with `std::env::var_os(\"HOME\")` (documenting that the passwd fallback is not used on the pinned floor), or keep `home_dir()` behind a named `#[allow(deprecated)]` with a reason; either way record the pinned-toolchain deprecation status as its own assumption row instead of folding it into A9."}, {"severity": "minor", "location": "§9 Fixtures vs Test strategy #3 and #7 — \"Read-only, checked-in, one tree used by both packages\"", "problem": "Three roots the tests assert are absent from the fixture tree, and §9 forbids creating them at runtime ('No test writes to the filesystem'): #3's 'a root with no `steamapps` directory yields an empty list', #3's 'a root whose `libraryfolders.vdf` is absent still yields its own `steamapps`', and #7's 'a root that has a `steamapps` directory whose entries are all unparseable' — the latter is the only pin for §7's single error path.", "fix": "Extend the §9 tree with two more roots under `fixtures/steam/` — `empty/` (no `steamapps`) and `broken/steamapps/` holding two truncated `appmanifest_*.acf` — and cite them from Test strategy #3 and #7."}, {"severity": "nit", "location": "§1 Modules and public surface — \"Every `pub` item above is exercised by a test in *Test strategy*\"", "problem": "`SteamLinuxPlugin::from_host()` is exercised by no test (deliberately: Test #10 states 'no test reads the developer's home directory'), and `name()` returning `\"steam-linux\"` is asserted nowhere.", "fix": "Scope the sentence to 'every `pub` item except `from_host()`, the thin wrapper over the `host_candidates` + probe pair Test #5 pins', or add a `name()` assertion to Test #8."}, {"severity": "nit", "location": "§9 Fixtures — \"the crate's tests directly, the CLI's through `env!(\\\"CARGO_MANIFEST_DIR\\\")` (A19)\"", "problem": "A19 yields the root of the test's own package, so from `apps/discovery-cli` the shared tree is reached only by a `../../crates/discovery/tests/…` hop the section never states; read as 'the fixture root' it produces a path that does not exist.", "fix": "Name the join explicitly: `env!(\"CARGO_MANIFEST_DIR\") + \"/../../crates/discovery/tests/fixtures/steam/minimal\"`."}]}
```
