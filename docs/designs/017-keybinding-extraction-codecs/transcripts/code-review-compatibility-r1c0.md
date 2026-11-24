# Transcript: code-review:compatibility r1c0

## Provenance
- stage: code-review:compatibility (round r1c0)
- agent: codereview/code-compatibility
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-24T06:32:09.540Z
- ended: 2026-11-24T06:55:02.021Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":326235,"output":32773,"reasoning":0,"cache.read":4751176,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff fc28ab94f313d079222d61438bf5b9074a1b8039...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/017-keybinding-extraction-codecs.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/017-keybinding-extraction-codecs/reviews/code/compatibility-r1c0.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-24T06:32:09.463Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/code/compatibility-r1c0.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "crates/codecs/src/families/unreal_ini.rs:194-243 (esp. 197, 217, 240-242)", "problem": "The locate walk rebuilds every child path from `entry.file_name().to_string_lossy()` (`dir.join(&name)`) instead of `entry.path()`. For an entry whose name is not valid UTF-8 — legal on ext4 and ordinary on the exFAT/NTFS libraries the Compatibility matrix lists as supported — the joined path names something that never existed: a grammar-shaped file falls into the `Err(_)` arm written for \"a match the reader refuses is located on path grammar alone\", so an invented path is recorded as a located candidate and `extract` later refuses the whole game as `UnreadableFile` (face exit 5, `extraction-refused`); a non-UTF-8 directory is queued as a nonexistent path, so its whole subtree is silently skipped with no skip-count to account for it. §The first family's \"extract reads only located files\" and the matrix's supported-layout posture are both unmet; no fixture exercises it because every fixture name is ASCII.", "fix": "Keep `entry.path()` for the queue and for `candidates.push`, use `entry.file_name()` (OsStr bytes) as the sort key, and split the `Err` arm: locate-on-path-grammar only for `FileOverBudget`/`UndecodableText`, skip-and-count for `UnreadableFile`."}, {"severity": "minor", "location": "crates/codecs/src/families/unreal_ini.rs:506-511 (with crates/codecs/src/snapshot.rs:305-308)", "problem": "`tree_relative` emits `strip_prefix(..).to_string_lossy()`, i.e. platform-native separators, into the contract's `provenance.files`. On the Windows host the face runs on (matrix row: \"the root-relative trees resolve and are stat'd\", and `<root>/userdata/<steamid>/<appid>` is the Windows Steam layout), the same layout states `Saved\\Config\\WindowsNoEditor\\Engine.ini` instead of the `/`-spelled document the spec and fixtures state, and the byte-order sort of `files` changes (`\\`=0x5C vs `/`=0x2F) — breaking the field's own stated portability rule (§`bindings/v1`: \"the same layout states the same document on any machine\"). The read side already assumes `/` (`.max(file.matches('/').count() + 1)`), so the normalization exists on one half only.", "fix": "Build the contract spelling once with `components().map(|c| c.as_os_str().to_string_lossy()).join(\"/\")` and use it for the document, the sort and refusal messages; keep absolute native paths only in `--probe` output, and pin the `/`-spelling in a loader/honesty assertion."}, {"severity": "minor", "location": "crates/codecs/src/paths.rs:127-136 (comment at 133-135)", "problem": "The comment promises \"A17's broken-prefix case degrades to files-absent\", and the Compatibility matrix's NTFS/exFAT row promises `family-confirmed-files-absent` (exit 4). Shipped behavior differs: a link-anchored prefix yields no tree, `unreal_ini::detect` returns `None` when nothing is located (unreal_ini.rs:73-90), and `appids()` is empty (unreal_ini.rs:67-71), so no `appid-table` claim exists to produce status 4 — the game answers `claim-refused` (exit 3). The safety half of the row (no hang, no out-of-tree read) does hold.", "fix": "State the degradation truthfully: correct the comment and the matrix row (via the §Rollout 8 ledger hand-off) to `claim-refused`/exit 3 with §The registry's reason, or give the row a real route to exit 4."}, {"severity": "nit", "location": "crates/codecs/src/families/unreal_ini.rs:251-258", "problem": "`grammar_match` compares `Saved`, `Config` and the `.ini` suffix byte-exactly, an unstated case-sensitivity assumption on the case-insensitive filesystems the face can run on (Windows row). Neither the doc comment, `schemas/bindings/v1/spec.md`, nor §The first family records that the grammar is case-exact, so a `saved/config` or `X.INI` tree is silently out of grammar.", "fix": "One sentence in the `grammar_match` doc comment (and in spec.md, since it is a locate-time contract fact) naming the match as case-exact by design — or match the extension case-insensitively and say so."}, {"severity": "nit", "location": "crates/codecs/tests/paths.rs:140-181 and crates/codecs/tests/unreal_ini.rs:183-224", "problem": "§The first family names a `symlinked-prefix` fixture tree with a `#[cfg(not(unix))]` early return; no such tree ships (fixture set: steam-userdata, proton-prefix, bom-utf16, defaults-only, bindings-absent, broken-tuple) — link coverage moved to temp-dir links, and only the `paths.rs` test has the compile-only `cfg(not(unix))` arm; the `unreal_ini.rs` link test has none. Linux coverage is equivalent (verified: `cargo test -p bindforge-codecs --locked` → 64 tests, symlink tests execute), so this is plan drift rather than a gap.", "fix": "Record the shipped shape in `03-build-notes.md` (links created at test time, no committed `symlinked-prefix` tree) and add the matching `#[cfg(not(unix))]` arm to the `unreal_ini` link test."}]}
```
