# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T02:56:46.349Z
- ended: 2026-11-18T02:59:20.219Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":37019,"output":23236,"reasoning":0,"cache.read":491493,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/tasks/001-steam-linux-discovery/reviews/design/compatibility-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
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
re-open one only if it contradicts docs/tasks/T-001-steam-linux-discovery.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/tasks/001-steam-linux-discovery/reviews/design/compatibility-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"blocker","location":"Chosen design §3 Library selection within a root, with §5 Manifest to `DiscoveredGame` — quote: \"`<root>/steamapps` itself, always.\" with \"install_path = l.join(\"steamapps\").join(\"common\").join(m.install_dir)\"","problem":"The document uses \"library\" in two mutually exclusive senses. §3's closing sentence (\"`appmanifest_*.acf` files are read only as entries directly inside a selected library's `steamapps/`\"), §5's join, and §2's canonical path rule (library paths are \"dropp[ed] ... that lack the required `steamapps` directory\") all treat a returned entry as a library root, but the always-included entry `<root>/steamapps` is the steamapps directory itself. As written: that entry fails §2's own gate unless `<root>/steamapps/steamapps` exists, so the always-scan that Risks row 2 and A12 lean on to avoid the silent-zero bug silently drops the root's own library; if the gate is instead read as not applying, the root's manifests are read from `<root>/steamapps/steamapps/appmanifest_*.acf`, a path no real Steam tree has, and the emitted install_path is one level too deep. The §9 fixture (manifests directly in `steamapps/`) then yields one game, not the four Test strategy #8 asserts, while #6's \"ends with `steamapps/common/<installdir>`\" passes vacuously and hides it. Consequence for compatibility: the matrix row \"Linux, native Steam under `$HOME` ... | Yes\" and the task's Done-when clause \"Running `discovery list` on a Linux machine with Steam correctly identifies installed games\" (docs/tasks/T-001-steam-linux-discovery.md:24) fail for the common single-library install.","fix":"Pick one convention in §3: `libraries_in_root` returns library roots (each containing a `steamapps/` directory), and the always-included entry is `root` itself, gated on `root/steamapps` being a directory; §2's gate and §5's join then stay as written. Update §9's fixture VDF entries to `\"0\" → \"..\"` and `\"1\" → \"../../secondary\"` (joined to the VDF's own directory `<fixture>/steamapps`), which keeps \"0\" exercising root/library de-duping and \"1\" exercising the relative join."},{"severity":"major","location":"Chosen design §2 Root selection — quote: \"`home` comes from `std::env::home_dir()` (A9)\" (with §10 Dependencies and gates, Test strategy #10, A9/A15/A18)","problem":"On the toolchain this repo pins (rust-toolchain.toml channel 1.85.0), `std::env::home_dir` is still `#[deprecated(since = \"1.29.0\", note = \"This function's behavior may be unexpected on Windows. Consider using a crate from crates.io instead.\")]` (verified in library/std/src/env.rs at the 1.85.0 tag); the un-deprecation the 1.85.0 release notes promised landed only later. A call from `crates/discovery` therefore emits the warn-by-default `deprecated` lint, and .github/workflows/check.yml:31 runs `cargo clippy --workspace --all-targets -- -D warnings`, so Test strategy #10, Rollout step 3 and the matrix's \"CI `rust` job ... | Yes\" row cannot pass as designed; §10/A15 (no new dependency, byte-identical Cargo.lock) rule out the crates.io replacement the note asks for. Same clause, smaller mismatch: A9 cites the current std docs (\"$HOME ... if it is set (and not an empty string)\"), while 1.85.0's text says \"including to an empty string\", so an empty $HOME yields Some(\"\") and every §2 candidate becomes a CWD-relative path that can pass the steamapps gate against an unrelated directory.","fix":"Replace that clause with `home` = `std::env::var_os(\"HOME\")`, treating unset-or-empty as \"no home\" (empty candidate list, so §7 returns `Ok(vec![])`), leaving `host_candidates(home, env)` and its test unchanged; a bare `#[allow(deprecated)]` clears the lint but leaves the empty-$HOME behaviour."},{"severity":"minor","location":"Chosen design §2 Canonical path rule, with Compatibility matrix row \"Linux, native Steam under `$HOME` (incl. `~/.steam/*`) | Yes\" — quote: \"`std::fs::canonicalize` resolves the symlink aliases (`~/.steam/root` → the real root) ... so no library is ever scanned twice\"","problem":"That Yes row depends on de-duping the ~/.steam/steam, ~/.steam/root and ~/.steam/debian-installation aliases of a single install (§2 candidate 2), but Test strategy #3 pins only the VDF `\"0\" → \".\"` case, and §9's \"No test writes to the filesystem\" bars a symlink fixture, so the alias property the row claims is never asserted (checklist item 7: a platform test per supported row).","fix":"Name a test that passes the same directory to `from_roots` in two spellings that canonicalize identically (the fixture root and `<fixture>/steamapps/..`) and asserts the game count is unchanged — no symlink, no filesystem write."},{"severity":"minor","location":"Compatibility matrix row \"Linux, libraries on arbitrary mounts (`/run/media/…`, `/mnt/…`, ZFS) | Yes\" — quote: \"Absolute `path` values are accepted as-is\"","problem":"No test in Test strategy exercises the absolute branch of §3's path rule: #3 covers only the relative `../secondary` entry, and §9 states a checked-in fixture \"cannot contain this machine's absolute paths\", so the one code path that makes this row Yes is untested (checklist item 7).","fix":"Add a pure `pub fn library_path(vdf_dir: &Path, value: &str) -> Option<PathBuf>` to §1's surface holding §3's stated rule (absolute as-is, else join to the VDF's directory, drop empty), have `libraries_in_root` call it, and assert both branches in Test strategy #3 — reachable from the crates/discovery/tests/ integration tests the design keeps, consistent with §1's \"Every `pub` item above is exercised by a test\". Otherwise downgrade the row to Partial."}]}
```
