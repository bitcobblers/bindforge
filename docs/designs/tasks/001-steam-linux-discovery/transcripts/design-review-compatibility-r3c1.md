# Transcript: design-review:compatibility r3c1

## Provenance
- stage: design-review:compatibility (round r3c1)
- agent: review/design-compatibility
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T03:37:32.259Z
- ended: 2026-11-18T03:40:19.947Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":30845,"output":13109,"reasoning":0,"cache.read":119294,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/tasks/001-steam-linux-discovery/reviews/design/compatibility-r3c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "Assumption register A9 and Chosen design §2 Root selection — \"`home` comes from `std::env::home_dir()` (A9)\"; §10 — \"`#[path]`-free, `unsafe`-free, no `std` feature flags required on the 1.85 floor (A15)\"",
    "problem": "On the toolchain this repo pins the call is deprecated, so the design's own gate claim is false as written. rust-toolchain.toml pins channel 1.85.0, and library/std/src/env.rs at tag 1.85.0 carries `#[deprecated(since = \"1.29.0\", note = \"This function's behavior may be unexpected on Windows. ...\")] pub fn home_dir() -> Option<PathBuf>`; the undeprecation landed after the floor (Rust 1.86.0 notes: \"Undeprecate env::home_dir\"). A9's \"verified\" basis is the unversioned doc.rust-lang.org/std/env/fn.home_dir.html — current stable, which no longer shows the deprecation — so the register records a verification against a newer toolchain than the one CI uses. Concretely: the first `cargo clippy --workspace --all-targets -- -D warnings` run (the Clippy step of the `rust` job, A18) fails on the `deprecated` lint at that one call site, and §10's dependency posture forecloses the usual outs — \"`crates/discovery/Cargo.toml` and `apps/discovery-cli/Cargo.toml` gain nothing, so `Cargo.lock` stays byte… [elided by the checkpoint cap]",
    "fix": "State the call-site decision in §2 in one clause: call `std::env::home_dir()` under a scoped `#[allow(deprecated)]` on `from_host()`, noting the lint is an artifact of the 1.85.0 floor (on Unix that function is exactly `$HOME`, else the passwd entry, which is all §2 needs) — or drop the std call and read `std::env::var_os(\"HOME\")` with a documented empty-value rule, which is the same text the empty-`$HOME` finding needs. Then correct A9's basis to the versioned 1.85.1 page so the register records the floor it was verified against, and add the `deprecated` lint to §10's posture sentence rather than leaving \"no `std` feature flags required on the 1.85 floor\" as the whole account of the floor."
  },
  {
    "severity": "minor",
    "location": "Chosen design §2 Root selection — \"`home` comes from `std::env::home_dir()` (A9); when it returns `None`, the candidate list is empty and the plugin discovers nothing\"",
    "problem": "That is the only guard on the one path input the design takes from the environment, and on the pinned toolchain it is not the guard the wording implies: 1.85.0 documents home_dir() as returning the value of HOME \"if it is set (including to an empty string)\", so a container, service or cron context with `HOME=` yields Some(\"\") rather than None. §2's candidates then become `.local/share/Steam`, `.steam/steam`, `.steam/root`, `.var/app/com.valvesoftware.Steam/…`, `snap/steam/common/…` — relative paths — and both the \"kept only if `<candidate>/steamapps` is a directory\" probe and the canonical path rule resolve them against the process working directory, so `discovery list` from a directory that happens to contain a `.steam/steam/steamapps` tree treats that tree as an installed Steam root. This contradicts the Compatibility matrix row \"Linux, Steam not installed / no home directory | Yes | `Ok(vec![])`, no error, no stderr output (§7)\", and it is the one place where the discipline §2 alrea… [elided by the checkpoint cap]",
    "fix": "Extend §2's rule by one clause to this input: `host_candidates` returns an empty candidate list when `home` is empty or relative. Name the case in Test strategy #5 next to the existing \"a relative `XDG_DATA_HOME` does not (A9)\" assertion. `host_candidates` is already pure and already asserted path-by-path, so no new seam, filesystem write, or dependency is introduced."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix row \"Linux, libraries on arbitrary mounts (`/run/media/…`, `/mnt/…`, ZFS) | Yes | Absolute `path` values are accepted as-is; an unmounted library is skipped, not fatal\", read against Test strategy #3 and §3 — \"a `path` is used as-is when absolute and otherwise joined to the directory holding the VDF\"",
    "problem": "This is the only `Yes` matrix row whose distinguishing mechanism no named test exercises, which is exactly what checklist item 7 asks for. §3's rule has two branches and the fixture reaches only the relative one: §9's libraryfolders.vdf holds `\"0\" → \".\"` and `\"1\" → \"../secondary\"`, Test strategy #3 asserts exactly those two, and §9 explains why the absolute branch cannot appear in a checked-in file (\"a checked-in fixture cannot contain this machine's absolute paths\"). A fabricated absolute value cannot cover it either, because the canonical path rule's `steamapps`-directory gate would drop it and the assertion would pass without reaching \"accepted as-is\". The `--steam-root` seam does not close the gap either — it supplies roots, which bypass `libraries_in_root`'s VDF branch. The row's second clause (\"an unmounted library is skipped, not fatal\") is unclaimed too: #7's two cases are a nonexistent root and an all-unparseable `steamapps`.",
    "fix": "Make the branch a pure function in §1's public surface — `pub fn resolve_library_path(vdf_dir: &Path, value: &str) -> Option<PathBuf>` — with `libraries_in_root` calling it, replacing the status quo where the branch lives inline and can only be reached through a machine-specific file. Then extend Test strategy #3 with the two assertions the row needs on nameable values: an absolute value such as `/bindforge-abs/Media/SteamLibrary` resolves to itself, and `\"1\"` under `tests/fixtures/steam/minimal/steamapps` resolves inside the fixture. This also matches §1's existing rule that \"every `pub` item above is exercised by a test in *Test strategy*\"."
  },
  {
    "severity": "nit",
    "location": "Test strategy #8 — \"returns exactly four games — `unknown/example-game`, `unknown/mixed-case-game`, `unknown/nameless-install`, `unknown/secondary-title`\" (and #6's \"`slug == normalize_slug(display_name)`\")",
    "problem": "Every display name the suite pins is ASCII, so nothing in the run records what a Linux user with an accented or non-Latin store name gets. normalize_slug (crates/discovery/src/slug.rs:1-35) tests characters with `char::is_alphanumeric`, so such names do survive into the slug instead of collapsing to `unknown/` with an empty tail — good behaviour, but it is an accident of the existing helper rather than an expectation this design pins, on the one input (§5's `display_name`) that a real Steam install supplies in the user's locale.",
    "fix": "Give one fixture manifest an accented `name` (e.g. `\"Les Misérables\"`), add its row to #8's expected list, and its `unknown/les-misérables` slug to #6's output contract. The slug format stays ADR-022's deferred concern; this only pins that non-ASCII names do not degrade to `unknown/`."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/tasks/001-steam-linux-discovery/reviews/design/compatibility-r3c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Compatibility matrix row \"Linux, libraries on arbitrary mounts (`/run/media/…`, `/mnt/…`, ZFS) | Yes | Absolute `path` values are accepted as-is; an unmounted library is skipped, not fatal\", read against §3 \"a `path` is used as-is when absolute and otherwise joined to the directory holding the VDF\" and Test strategy #3 \"the root itself and `<root>/secondary` are both returned … the `\"0\" → \"..\"` entry adds no duplicate against the always-included root\"","problem":"Re-raised (unchanged by this diff even though §3 and #3 were both rewritten): this remains the only `Yes` matrix row whose distinguishing mechanism no named test exercises (checklist item 7). The fixture reaches only the relative-join branch; a fabricated absolute value cannot survive the canonicalize + `steamapps` gate, so \"accepted as-is\" is unobservable through libraries_in_root; the `--steam-root`/`from_roots` seam supplies roots and bypasses the VDF branch. The row's second clause (\"an unmounted library is skipped, not fatal\") is likewise unclaimed: #7's two cases are a nonexistent root and an all-unparseable `steamapps`.","fix":"Name the branch: `pub fn resolve_library_path(vdf_dir: &Path, value: &str) -> Option<PathBuf>` in §1's public surface with `libraries_in_root` calling it (or an in-file `#[cfg(test)]` unit test over a private helper of that name, keeping §1's surface unchanged). Extend Test #3 with two assertions on nameable values: an absolute value such as `/bindforge-abs/Media/SteamLibrary` resolves to itself, and `\"1\"` under the fixture VDF dir resolves inside the fixture."},{"severity":"nit","location":"Test strategy #8 — \"returns exactly four games — `unknown/example-game`, `unknown/mixed-case-game`, `unknown/nameless-install`, `unknown/secondary-title`\" (and #6's \"`slug == normalize_slug(display_name)`\")","problem":"Re-raised (the diff does not touch #6/#8/§9 and the Revision log records no rejection): every pinned display name is ASCII, so nothing pins that accented or non-Latin store names survive `normalize_slug`'s `char::is_alphanumeric` character test instead of degrading to `unknown/`, on the one input (§5's `display_name`) real Linux Steam installs supply in the user's locale.","fix":"Give one fixture manifest an accented `name` (e.g. \"Les Misérables\"), add its row to #8's expected list and its `unknown/les-misérables` slug to #6's output contract; slug format itself stays ADR-022's deferred concern."}]}
```
