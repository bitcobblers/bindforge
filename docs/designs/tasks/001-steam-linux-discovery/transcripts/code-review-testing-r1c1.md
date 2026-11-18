# Transcript: code-review:testing r1c1

## Provenance
- stage: code-review:testing (round r1c1)
- agent: codereview/code-testing
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-18T04:34:28.839Z
- ended: 2026-11-18T04:36:10.016Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":23775,"output":13037,"reasoning":0,"cache.read":380792,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/tasks/001-steam-linux-discovery/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 752fd5abc9e4e88b57a98ef1c23c31daa3983c13..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/tasks/T-001-steam-linux-discovery.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/tasks/001-steam-linux-discovery/reviews/code/testing-r1c1.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-18T04:31:16.470Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (testing seat):
[
  {
    "severity": "major",
    "location": "crates/discovery/src/game.rs:8 (asserted at apps/discovery-cli/tests/cli_tests.rs:121)",
    "problem": "The diff adds `#[serde(rename_all = \"kebab-case\")]` to `Platform`, re-spelling `DiscoveredGame.platform` on the JSON channel from \"SteamLinux\" to \"steam-linux\" (and, with no `alias`, dropping acceptance of the old spelling on deserialize), against three explicit statements in 02-proposal.md — Context:14-15 (\"no change to `DiscoveredGame`, `Platform`\"), Non-goals:35, and §8:262 (\"the printing code is untouched, so the table header and JSON shape are unchanged\"). The testing defect is the guard layer: the only assertion of the new spelling in the repo is cli_tests.rs:121, written against post-change behavior (the base cli_tests.rs asserted only `[]`, as this diff's removed lines show), and crates/discovery/tests/game_tests.rs — the data-model test, untouched here — has no serde assertion at all. So the suite cannot distinguish a ratified rename from an accidental derive edit: a revert or further rename in game.rs is caught only by a downstream CLI test, and nothing tests that previously … [elided by the checkpoint cap]",
    "fix": "Make the contract explicit at its own layer: either drop the derive attribute from this additive task and assert \"SteamLinux\" at cli_tests.rs:121, or keep the ADR-022-aligned kebab form (docs/adrs/022-game-discovery.md:34) and add a cited test in crates/discovery/tests/game_tests.rs pinning both directions (`serde_json::to_value(Platform::SteamLinux) == \"steam-linux\"` plus a round-trip, with `#[serde(alias = \"SteamLinux\")]`/`\"SteamWindows\"` if prior output must keep deserializing), correcting §8:262's \"JSON shape ... unchanged\" in the same change."
  },
  {
    "severity": "minor",
    "location": "crates/discovery/tests/steam_linux_tests.rs (missing test); property at crates/discovery/src/steam_linux.rs:61-71",
    "problem": "§2's canonical statement that all selected roots' libraries form one set per `discover()` call — \"no library is ever scanned twice, including by two roots sharing one\" — has no test; Test strategy #3 pins de-duping only within one root (the `\"0\" → \"..\"` case at steam_linux_tests.rs:161-171). Hoisting `canonical_paths` so libraries de-dup per root instead of per call leaves all 40 tests green while duplicating rows for a `~/.steam/root` plus real-root host. §8's documented repeatable `--steam-root` is likewise passed only once by each CLI test.",
    "fix": "Add a unit test with `from_roots([fixture(\"minimal\"), fixture(\"minimal/secondary\")])` asserting exactly the same four rows with no duplicate `unknown/secondary-title`, plus one CLI test passing `--steam-root` twice over the same fixture tree and asserting the unchanged four-row table."
  },
  {
    "severity": "minor",
    "location": "crates/discovery/src/steam_linux.rs:89 (sort key) and crates/discovery/tests/fixtures/steam/",
    "problem": "§5's \"Identical slugs from two libraries stay two rows\" and the `install_path` tie-break of the `(slug, install_path)` sort are untested: the fixture yields four distinct slugs, so discover_output_honours_the_game_output_contract (steam_linux_tests.rs:408-417) exercises only the slug component, and no fixture produces a duplicate slug at all. A slug-only sort, or de-duping by slug, would pass the suite.",
    "fix": "Add a checked-in `appmanifest_900006.acf` under `minimal/secondary/steamapps/` with `\"name\" \"Example Game\"` (same name, other library) and extend the #8 integration expectation to five rows, `unknown/example-game` appearing twice in install_path order — pinning both the two-rows rule and the tie-break with one fixture entry."
  },
  {
    "severity": "minor",
    "location": "crates/discovery/src/steam_linux.rs:198-247 (`scan_library`/`discovered_game`) and the fixture set",
    "problem": "§6's explicit accepted cost — \"No `StateFlags` bit test and no non-game filter, so Proton builds and the Steam Linux Runtime appear in `discovery list` as rows\" (Risks: likelihood High) — is unpinned. Every fixture manifest carries `\"StateFlags\" \"4\"` and every fixture name looks like a game, so introducing a StateFlags bit test or a name denylist keeps all tests green. The design's Test strategy names no item for §6, so this is a gap the test plan itself left.",
    "fix": "Add one fixture manifest with a non-ready `StateFlags` (e.g. `\"StateFlags\" \"6\"`) and a runtime-shaped `installdir`, and assert in the #8 integration test that it is still emitted, cited `// proves: Chosen design §6`."
  },
  {
    "severity": "minor",
    "location": "crates/discovery/src/keyvalues.rs:21,77 and crates/discovery/src/steam_linux.rs:285-312",
    "problem": "The 1 MiB read cap exists twice (keyvalues::MAX_DOCUMENT_BYTES at parse time; read_text_file's pre-read `metadata` check) and neither is covered: no test parses an oversize document and no fixture approaches the size. §4 makes these two named constants the entire per-file work bound (A16) and §7 puts the skipped count in the error message precisely because these caps bite, yet inverting a comparison or drifting a constant breaks no test.",
    "fix": "Add a cited unit test that `parse` on a 1 MiB + 1 byte document returns `KeyValueError { line: 1, .. }`, and assert `MAX_DOCUMENT_BYTES` equals the shared parse-budget constant §4 says it restates, so the two cannot drift."
  },
  {
    "severity": "minor",
    "location": "crates/discovery/src/steam_linux.rs:152-160 (`libraries_in_root_with_skips`, absolute `path` branch)",
    "problem": "The `path.is_absolute()` arm — the shape every real `libraryfolders.vdf` uses, and the Compatibility matrix's \"libraries on arbitrary mounts — Yes\" row — is never exercised: the fixture VDF carries only `\"..\"` and `\"../secondary\"` (§9 explains why a checked-in fixture cannot hold machine absolute paths). `PathBuf::join` replacing on an absolute argument mutes the regression risk, but the promised behavior remains unverified and a change in library-path resolution would pass silently.",
    "fix": "Use the escape hatch §9 already sanctions: under `env!(\"CARGO_TARGET_TMPDIR\")` build a one-library tree with a `steamapps/` directory and a `libraryfolders.vdf` whose numeric block carries that absolute path, then assert `libraries_in_root` returns it as-is."
  },
  {
    "severity": "nit",
    "location": "crates/discovery/src/steam_linux.rs:273-280 (`usable_install_dir`)",
    "problem": "The `contains('\\0')` refusal goes beyond §5's enumerated rejection set (absent, empty, `.`, `..`, absolute, `/`) and is the only guard in that function with no test — parse_manifest_rejects_unusable_installdir (steam_linux_tests.rs:110-119) covers the five documented cases only.",
    "fix": "Add `\"\\0\"` (and a whitespace-only value) to the existing `for bad in [...]` list so the extra refusal is a stated expectation rather than an unclaimed branch."
  },
  {
    "severity": "nit",
    "location": "crates/discovery/src/steam_linux.rs:75-87 (§7 error message)",
    "problem": "§7 specifies the single error message names the selected roots and the count of skipped files. discover_of_all_broken_root_returns_plugin_error_naming_root_without_contents asserts the root name and the no-contents half, but never the count — the field that separates \"Steam present, everything under it unreadable\" from \"nothing installed\" under A14's swallowed `eprintln!`. A bug reporting zero skips (e.g. counting only one of the two skip sites) passes.",
    "fix": "Assert the message contains the expected skip count (2 for the `all-broken` fixture: one unbalanced-brace and one unterminated-string manifest)."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/tasks/001-steam-linux-discovery/reviews/code/testing-r1c1.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only. The entire response must be a single valid JSON object with no prose, explanation, markdown, or additional text before or after it.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"request-changes","findings":[{"severity":"major","location":"crates/discovery/src/game.rs:5-8 (asserted only at apps/discovery-cli/tests/cli_tests.rs:121)","problem":"OPEN from r1, unaddressed by this revision: the diff's `#[serde(rename_all = \"kebab-case\")]` on `Platform` still contradicts the current 02-proposal.md — Context:14-15 (\"no change to `DiscoveredGame`, `Platform`\"), Non-goals:35, §8:262 (\"the table header and JSON shape are unchanged\") — and the guard layer is unchanged: crates/discovery/tests/game_tests.rs (19 lines at HEAD) still has no serde assertion, so the suite still cannot distinguish a ratified rename from an accidental derive edit, cannot catch a revert of the derive or a further rename, and nothing tests deserialization of the old \"SteamLinux\" spelling. Checklist item 3 (each previously rejected major now has a test that would have caught it) fails: the revision touched only the steam_linux.rs §7 hunk, no test or game.rs change.","fix":"Make the contract explicit at its own layer: either drop the derive attribute from this additive task and assert \"SteamLinux\" at cli_tests.rs:121, or keep the ADR-022-aligned kebab form and add a cited test in crates/discovery/tests/game_tests.rs pinning both directions (`serde_json::to_value(Platform::SteamLinux) == \"steam-linux\"` plus a round-trip, with `#[serde(alias = \"SteamLinux\")]`/`\"SteamWindows\"` if prior output must keep deserializing), correcting §8:262's \"JSON shape ... unchanged\" in the same change."},{"severity":"minor","location":"crates/discovery/tests/steam_linux_tests.rs (missing test); property at crates/discovery/src/steam_linux.rs:62-70","problem":"OPEN from r1, unchanged by the revision: §2's canonical statement that all selected roots' libraries form one set per `discover()` call (\"no library is ever scanned twice, including by two roots sharing one\") has no test; only the within-one-root `\"0\" → \"..\"` de-dup is pinned. Hoisting `canonical_paths` to a per-root call would keep every test green while duplicating rows for a `~/.steam/root` plus real-root host, and no CLI test passes `--steam-root` twice.","fix":"Add a unit test with `from_roots([fixture(\"minimal\"), fixture(\"minimal/secondary\")])` asserting exactly the same four rows with no duplicate `unknown/secondary-title`, plus one CLI test passing `--steam-root` twice over the same fixture tree and asserting the unchanged four-row table."},{"severity":"minor","location":"crates/discovery/src/steam_linux.rs:91 (sort key); crates/discovery/tests/fixtures/steam/","problem":"OPEN from r1, unchanged by the revision: §5's \"Identical slugs from two libraries stay two rows\" and the `install_path` tie-break of the `(slug, install_path)` sort are still untested — the fixture yields four distinct slugs, so the integration test exercises only the slug component; a slug-only sort or slug de-duping passes the suite.","fix":"Add a checked-in `appmanifest_900006.acf` under `minimal/secondary/steamapps/` with `\"name\" \"Example Game\"` (same name, other library) and extend the #8 integration expectation to five rows with `unknown/example-game` appearing twice in install_path order."},{"severity":"minor","location":"crates/discovery/src/steam_linux.rs (`scan_library`/`discovered_game`) and the fixture set","problem":"OPEN from r1, unchanged by the revision: §6's accepted cost — Proton builds and the Steam Linux Runtime appearing in `discovery list` as rows (Risks: likelihood High) — is still unpinned; every fixture manifest carries `\"StateFlags\" \"4\"` and a game-like name, so adding a StateFlags bit test or name denylist keeps all tests green.","fix":"Add one fixture manifest with a non-ready `StateFlags` (e.g. `\"StateFlags\" \"6\"`) and a runtime-shaped `installdir`, and assert in the #8 integration test that it is still emitted, cited `// proves: Chosen design §6`."},{"severity":"minor","location":"crates/discovery/src/keyvalues.rs (MAX_DOCUMENT_BYTES) and crates/discovery/src/steam_linux.rs (read_text_file)","problem":"OPEN from r1, unchanged by the revision: the 1 MiB read cap exists twice and neither leg is covered — no test parses an oversize document and no fixture approaches the size; inverting a comparison or drifting either constant breaks no test, though §4 makes these constants the entire per-file work bound (A16).","fix":"Add a cited unit test that `parse` on a 1 MiB + 1 byte document returns a `KeyValueError`, and assert `MAX_DOCUMENT_BYTES` equals the shared parse-budget constant §4 says it restates, so the two cannot drift."},{"severity":"minor","location":"crates/discovery/src/steam_linux.rs:154-160 (`libraries_in_root_with_skips`, absolute `path` branch)","problem":"OPEN from r1, unchanged by the revision: the `path.is_absolute()` arm — the shape every real `libraryfolders.vdf` uses and the Compatibility matrix's \"libraries on arbitrary mounts — Yes\" row — is never exercised; the fixture VDF carries only relative paths. A change in library-path resolution would pass silently.","fix":"Use the escape hatch §9 sanctions: under `env!(\"CARGO_TARGET_TMPDIR\")` build a one-library tree with a `steamapps/` directory and a `libraryfolders.vdf` whose numeric block carries that absolute path, then assert `libraries_in_root` returns it as-is."},{"severity":"nit","location":"crates/discovery/src/steam_linux.rs (`usable_install_dir`)","problem":"OPEN from r1, unchanged by the revision: the `contains('\\0')` refusal goes beyond §5's enumerated rejection set and remains the only guard in that function with no test case.","fix":"Add `\"\\0\"` (and a whitespace-only value) to the existing `for bad in [...]` list in parse_manifest_rejects_unusable_installdir so the extra refusal is a stated expectation."},{"severity":"nit","location":"crates/discovery/src/steam_linux.rs:84 (§7 message); crates/discovery/tests/steam_linux_tests.rs:464-492","problem":"OPEN from r1 and worsened by this revision: §7's skip-count component is still unasserted, and the revision changed the one user-visible string §7 prescribes — `{} file(s) skipped as unreadable or malformed` → `{} entries skipped as unreadable or malformed` — with no test change. No test pins the count, the count-bearing noun, or any message wording beyond the root name, so the drift from §7's stated \"count of skipped files\" to \"entries\" is an unpinned contract change and further wording edits stay silent. A bug reporting the wrong count still passes.","fix":"In discover_of_all_broken_root_returns_plugin_error_naming_root_without_contents, assert the message contains the expected skip count (`; 2 ` for the all-broken fixture: two malformed manifests) and pin the fixed message tail, cited `// proves: Chosen design §7`."},{"severity":"nit","location":"crates/discovery/src/steam_linux.rs:77 (`!self.roots.is_empty()` clause added by this revision)","problem":"NEW from the changed code: the added clause cannot change behavior — `skipped` is only incremented inside the `if !self.roots.is_empty()` scan block at lines 61-71, so `skipped > 0` already entails a selected root; the roots-empty + skipped>0 combination that would distinguish it is unconstructible, so no test can exercise it. That makes it an untestable addition (checklist item 7) while the new comment claims it spells out a §7 clause the suite cannot verify.","fix":"Either drop the redundant clause (the scan-block guard plus `skipped > 0` already encode the three §7 clauses) or make the roots-selected precondition real (e.g. count skipped roots) so a test can pin each clause."}]}
```
