# Transcript: design-review:correctness r3c1

## Provenance
- stage: design-review:correctness (round r3c1)
- agent: review/design-correctness
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T03:37:32.256Z
- ended: 2026-11-18T03:40:30.052Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":33404,"output":13979,"reasoning":0,"cache.read":271095,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/tasks/001-steam-linux-discovery/reviews/design/correctness-r3c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "blocker",
    "location": "Chosen design §3 Library selection within a root — \"`<root>/steamapps` itself, always\" + \"a `path` is used as-is when absolute and otherwise joined to the directory holding the VDF\" + \"`appmanifest_*.acf` files are read only as entries directly inside a selected library's `steamapps/`\" (cross-refs: §2 Canonical path rule \"the `\"0\" → root` duplicate\", §5 install_path composition, §9 Fixtures \"Four games survive from this tree\", Test strategy #3/#6/#8)",
    "problem": "The three rules place a \"library\" at two different levels, so the fixture and the anti-silent-zero guarantee cannot both hold. Since manifests are enumerated at `<library>/steamapps/` and install_path is `l/steamapps/common/<installdir>` (A2, and the cited steamlocate library.rs — `Library::from_dir` reads `path.join(\"steamapps\")`, `resolve_app_dir` appends `steamapps/common` — confirms a library is the parent of steamapps/), the always-included entry must be `<root>`, not `<root>/steamapps`. Consequences as written: (a) `<root>/steamapps` fails §2's own gate (\"dropping entries … that lack the required `steamapps` directory\", i.e. it demands `<root>/steamapps/steamapps`), so the guaranteed scan of the root's own steamapps is dropped — Test #3's \"a root whose `libraryfolders.vdf` is absent still yields its own `steamapps`\" and Risks row 2's mitigation both fail; (b) in the fixture, `\"0\" → \".\"` joined to the VDF's directory resolves to `<root>/steamapps` (one level too deep) while `\"1\" →… [elided by the checkpoint cap]",
    "fix": "State one level: libraries are library folders (parents of steamapps/); the always-included entry is `<root>` itself, and §2's root gate (`<root>/steamapps` is a directory) is what guarantees the scan; join relative `path` values to `<root>` (the VDF directory's parent) and change the fixture's `\"1\"` entry to `\"secondary\"` so both entries resolve under that single rule portably. §9, Test #3/#6/#8 and §2's de-dupe sentence then hold as written."
  },
  {
    "severity": "major",
    "location": "§2 Root selection — \"`home` comes from `std::env::home_dir()` (A9)\" and Assumption register A9 (\"verified\", basis https://doc.rust-lang.org/std/env/fn.home_dir.html)",
    "problem": "On the toolchain this repo pins (rust-toolchain.toml channel = 1.85.0) `std::env::home_dir` still carries `#[deprecated(since = \"1.29.0\")]`; the deprecation was removed only in 1.87.0 (rust-lang/rust#137327), and the 1.85.0 docs page shows \"Deprecated since 1.29.0\". Any call site emits `use of deprecated function`, which `cargo clippy --workspace --all-targets -- -D warnings` (A18, .github/workflows/check.yml:31, and Test strategy #10's own gate) turns into a build failure. A9's basis is the current-stable docs page, which no longer shows the banner, so the row is not supported for the pinned toolchain.",
    "fix": "Read home via `std::env::var_os(\"HOME\")`, treating absent/empty as \"no home directory\" (which §7 already maps to `Ok(vec![])`) — no dependency, so A15 holds — or keep the call behind `#[allow(deprecated)]`; re-cite A9 to https://doc.rust-lang.org/1.85.0/std/env/fn.home_dir.html and note that on 1.85.0 `HOME=\"\"` yields `Some(\"\")`, leaving relative candidates that only the canonicalize gate removes."
  },
  {
    "severity": "major",
    "location": "§7 Failure policy and the one error path — \"at least one root was selected but **no** library under any selected root could be scanned\" vs Test strategy #7 — \"a root that has a `steamapps` directory whose entries are all unparseable returns exactly one `Err(DiscoveryError::PluginError(_))`\"",
    "problem": "§7's own first paragraph files \"a manifest that fails to parse\" under skipped files and reserves the error for libraries that could not be scanned, so code implementing §7 literally returns `Ok(vec![])` for the very scenario Test #7 demands an `Err` for (the steamapps directory was scanned; its manifests just failed to parse). The all-unreadable-library case that does satisfy §7 cannot be produced from a read-only checked-in fixture (§9: \"No test writes to the filesystem\", and a mode change would not survive checkout). Therefore Risks row 4's claim that \"*Test strategy* #7 pins both sides\" is unimplemented, and \"Steam present, every manifest broken\" still prints nothing.",
    "fix": "Restate the trigger as: at least one root was selected, zero games were emitted, and at least one file was skipped as unreadable or malformed. This matches the message §7 already specifies (it counts skipped files), makes Test #7's all-unparseable fixture the Err side, and leaves the Ok(vec![]) side unchanged (no roots, nonexistent root, Steam installed with no games → zero skipped files)."
  },
  {
    "severity": "minor",
    "location": "§1 Modules and public surface — \"Every `pub` item above is exercised by a test in *Test strategy*\"",
    "problem": "`SteamLinuxPlugin::from_host()` is the one public item that no test in Test strategy calls, and the document concedes this itself in Risks row 5: \"`from_host()` cannot be unit-tested\", with `host_candidates` asserted instead. The blanket sentence contradicts that row.",
    "fix": "Qualify it: \"every `pub` item except `from_host()`, whose candidate list is asserted path-by-path through `host_candidates` (Test strategy #5) and whose probe is §2's root gate\"."
  },
  {
    "severity": "minor",
    "location": "§2 Canonical path rule — \"then sorting, then de-duplicating\" … \"so no library is ever scanned twice\"",
    "problem": "The rule is scoped to \"every set of directories the plugin scans\", i.e. one set per root, so a library registered by two separately selected roots (native plus Flatpak/Snap installs pointing at the same /run/media/… or /mnt/… library — the multi-layout case the A8 candidate list exists for) is selected twice and every game in it is emitted twice as identical rows in both the table and `--json`; the \"never scanned twice\" sentence is false as stated.",
    "fix": "Apply the canonical path rule to the union of library paths across all selected roots (one de-duped set per `discover()` call, assembled in the private scan loop) rather than one set per root; Test #3's per-root `libraries_in_root` assertions are unaffected."
  },
  {
    "severity": "nit",
    "location": "§8 CLI wiring (`apps/discovery-cli`) — \"registered in place of the deleted `DummyPlugin`\"",
    "problem": "`DummyPlugin` is the only user of `use discovery::plugin::DiscoveryPlugin;` (apps/discovery-cli/src/main.rs:4); removing it leaves an unused import, which fails the `-D warnings` gate, since register_plugin's `Box<dyn DiscoveryPlugin>` argument supplies the trait-object type without the import.",
    "fix": "One clause: \"`DummyPlugin` and its `DiscoveryPlugin` import are removed together, and the trait is not re-imported\"."
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
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/tasks/001-steam-linux-discovery/reviews/design/correctness-r3c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"nit","location":"§8 CLI wiring (`apps/discovery-cli`) — \"its `DiscoveryPlugin` import goes with it rather than being re-imported\"","problem":"Re-raised against my own r3 nit, now narrowed by the revised text: the sentence covers only one of the three imports `DummyPlugin` solely uses. `apps/discovery-cli/src/main.rs` names `DiscoveredGame` (line 3) and `DiscoveryError` (line 5) only inside `DummyPlugin`'s `fn discover` (line 30), and `register_plugin`'s `Box<dyn DiscoveryPlugin>` argument (crates/discovery/src/engine.rs:16) needs no import at the call site, so implementing §8's sentence as written leaves two unused imports that `cargo clippy --workspace --all-targets -- -D warnings` (A18, Test #10) turns into a build failure. All five of my r3 blocker/major/minor findings are otherwise closed for real; the compiler forces this one, so no design decision is at stake.","fix":"Widen the clause: \"`DiscoveredGame` and `DiscoveryError`, whose only other user is its `discover` signature, go with it too.\""}]}
```
