# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T05:55:48.158Z
- ended: 2026-11-18T06:00:11.277Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":34680,"output":16445,"reasoning":0,"cache.read":353194,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/tasks/002-steam-windows-discovery/reviews/design/correctness-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Assumption register, row A1 (\"Everything after root selection is the same on Windows as on Linux: … and library `path` values written with doubled backslashes. | verified | …github.com/Playmoir/knowledge-base/blob/main/steam-library-integration.md\")",
    "problem": "The citation is dead: github.com/Playmoir/knowledge-base and the GitHub API both return 404 (no such repository), as does the raw.githubusercontent.com form. It is the only live-checkable support for the doubled-backslash `path` spelling and the second of A1's two bases (the other is a StackOverflow question that returns 403 to a non-browser client, so it is inconclusive rather than confirming). A1 is the row that licenses the document's central move — putting everything after root selection into one shared module — so a `verified` row resting on a nonexistent source fails the register's own standard. A19 and §2's second-location paragraph cite it too, but A19 stands on its other basis, which I verified live (lutris util/steam/config.py reads config/libraryfolders.vdf).",
    "fix": "In A1, swap the Playmoir URL for a live source that shows a Windows libraryfolders.vdf `path` value with escaped backslashes, or split the `path`-spelling clause out of A1 and mark it `unverified`, noting that A2's verified `\\`/`\"` unescape (keyvalues.rs:186-207) is what makes either spelling arrive usable."
  },
  {
    "severity": "major",
    "location": "§5 Root candidates (\"2. `HKLM\\SOFTWARE\\Valve\\Steam` → `InstallPath`: the 32-bit-view result, then the 64-bit-view result if the first was `None`\") against §1's `pub enum RegistryKey { CurrentUserValveSteam, LocalMachineValveSteam }` and Test strategy #3 (\"HKLM-32-bit-view only; HKLM-64-bit-view only\")",
    "problem": "The three sections disagree about who owns the WOW64 view choice. §3 puts the 32-then-64 fallback inside `registry_string_value`, which is `#[cfg(windows)]`-only, so `host_candidates` gets exactly one value per key and cannot order \"the 32-bit-view result, then the 64-bit-view result\" as §5 states. And a fake keyed by `(RegistryKey, &str)` (§4's HashMap over that key) cannot express 32-view-only versus 6-view-only, so the two named Test strategy #3 cases are unwritable as specified. Consequence: the view-ordering policy — the answer to A4 and the stated reason for rejecting the hardcoded `Wow6432Node` spelling — is covered only by #12's type-check and #13's manual run.",
    "fix": "Put the view in the seam so `host_candidates` orders it: add `LocalMachineValveSteam32` and `LocalMachineValveSteam64` variants (candidate order HKCU, HKLM-32, HKLM-64, envs) and reduce `registry_string_value` to \"open this key with these flags, read one REG_SZ\", keeping #3's two cases as written; or delete those two cases from #3 and state in §5 and Risks that view ordering is compile- and manually-verified only."
  },
  {
    "severity": "major",
    "location": "§8 CLI wiring, else-branch comment \"// `--steam-root` is the Linux seam from T-001: the row's platform tag is the plugin's, so the plugin is the one that follows the flag.\"",
    "problem": "With `--steam-root` the Windows plugin is not registered at all, so no `steam-windows` row can ever come from an explicit root. That contradicts the task's Done-when clause docs/tasks/T-002-steam-windows-discovery.md:65-67 — \"…and `--steam-root` keeps working for it\" — where \"it\" is the steam-windows plugin. §8 concedes the gap (\"A user who wants `steam-windows`-tagged rows from an explicit root is a one-line CLI change with its own argument\") and the Deferred item \"An explicit 'these roots are Windows Steam roots' CLI seam\" hands it to no document, so this is a deferral against a stated requirement, not an out-of-scope refinement.",
    "fix": "Smallest fix that keeps the three CLI tests undoubled: in the `--steam-root` branch register `SteamWindowsPlugin::from_roots(steam_root.iter().cloned())` instead of the Linux plugin when compiled for Windows (`#[cfg(windows)]`), so the flag feeds the plugin whose rows that host can produce, and drop §8's self-imposed \"no `#[cfg]` in `main.rs`\". Otherwise register it alongside the Linux plugin and say in §8 and Test strategy #9 that the fixture rows double to eight and cli_tests.rs:68-72/:124 are re-pinned across both tags. Either way, record the reading of :66-67 in the Revision log instead of in Deferred."
  },
  {
    "severity": "minor",
    "location": "§5 Root candidates — \"A relative value is ignored by the same `absolute_env` helper Linux uses (crates/discovery/src/steam_linux.rs:338-342)\"",
    "problem": "`absolute_env` is a private `fn` in `steam_linux.rs`, and §1's inventory of what moves into `steam_common` (library_paths, libraries_in_root_with_skips, scan_library, read_text_file, canonical_paths, discovered_game, parse_manifest/usable_install_dir) omits it, while §1 keeps `host_candidates` in `steam_linux`. As specified, steam_windows's `host_candidates` can neither call nor share that helper, and copying it is the duplication the Done-when clause forbids.",
    "fix": "Add `absolute_env` to §1's moved-items list (`steam_common`, `pub(crate)`), with steam_linux's call sites rewired to it."
  },
  {
    "severity": "minor",
    "location": "§5 Root candidates — \"each only when the variable is set and absolute\", read together with §6's \"`Path::is_absolute` (which reads a Windows path as relative on a Linux host)\" and Test strategy #3",
    "problem": "The env-candidate gate is `absolute_env`, whose predicate is `PathBuf::is_absolute` — host-dependent, as §6 itself states. `host_candidates` is compiled and tested on Linux (§3, Test strategy #3), so on the test host the gate can only pass with POSIX-absolute fake values, leaving the Windows-shaped value the design is about (`C:\\Program Files (x86)`) either untested or silently dropped by the very rule under test.",
    "fix": "State in §5 which values the fake supplies and add one #3 case pinning what a Windows-absolute value does on each host — best done by generalizing the absoluteness check over `SteamTarget` next to `install_dir_rejects_backslash`, so the rule exercised on Linux is the rule that runs on Windows."
  },
  {
    "severity": "nit",
    "location": "§1 sketch — \"impl DiscoveryPlugin for SteamWindowsPlugin { /* delegates to `core`, name() = \\\"steam-windows\\\" */ }\" versus §2's \"prefix is `T::NAME`, which is the same string the plugin's `name()` returns, so the prefix can no longer drift\"",
    "problem": "The §2 no-drift claim only holds if the plugin's `name()` returns `T::NAME`; the §1 sketch shows a second literal spelling of the tag beside `WindowsTarget::NAME`, which is exactly the duplicate §2 says is gone.",
    "fix": "Write `name()` as `self.core.name()` in the §1 sketch for both plugins, so the single-source claim is the shape on the page."
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
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/tasks/002-steam-windows-discovery/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"§5 Root candidates — \"A relative value is ignored by the same `absolute_env` helper Linux uses, shared per §1\" read with Test strategy #3 (re-raised from r0; the sentence was changed but the gap remains)","problem":"The env-candidate gate's predicate is `PathBuf::is_absolute` (crates/discovery/src/steam_linux.rs:341), host-dependent as §6 itself states. `host_candidates` is compiled and unit-tested on Linux, so the fake can only pass POSIX-absolute values, and the Windows branch of the rule — a `C:\\Program Files (x86)`-shaped value kept, a Windows-relative one dropped — is exercised on no host. §5 still does not state what values the fake supplies. Runtime behavior is correct on each host; this is a verification gap for a rule the design depends on, unchanged by the revision.","fix":"In §5, add one sentence stating that the #3 fake supplies POSIX-absolute values and that the Windows-absolute case is covered only by gates #12/#13; or generalize the absoluteness predicate over `SteamTarget` next to `install_dir_rejects_backslash` (§1's existing mechanism) and add one Test strategy #3 case pinning what a Windows-shaped value does on each host."},{"severity":"minor","location":"Test strategy #2 — \"`foo\\bar`, the doubled form `foo\\\\bar` as a real Windows ACF writes it (A2)\"","problem":"The r1 revision split the doubled-backslash spelling out as A1b, marked `unverified` with \"the design assumes no spelling\" — yet this body sentence still asserts the doubled spelling as what real Windows ACFs write, and cites A2, which is the reader's verified `\\`/`\"` unescape behavior (keyvalues.rs:186-207), not a source on what ACFs write. The changed register makes this sentence unsupported; the test case itself stays valid either way, so it is a factual/language defect, not behavioral.","fix":"Reword to \"the doubled form a Windows writer may emit (A1b, spelling unverified)\", leaving the enumerated test cases unchanged."}]}
```
