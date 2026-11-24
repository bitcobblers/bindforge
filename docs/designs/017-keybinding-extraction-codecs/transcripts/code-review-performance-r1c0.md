# Transcript: code-review:performance r1c0

## Provenance
- stage: code-review:performance (round r1c0)
- agent: codereview/code-performance
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-24T06:44:20.550Z
- ended: 2026-11-24T06:58:26.361Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":78411,"output":22202,"reasoning":0,"cache.read":1314845,"cache.write":0}
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
Your report at docs/designs/017-keybinding-extraction-codecs/reviews/code/performance-r1c0.md is yours: re-reading it is allowed, through your
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
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/code/performance-r1c0.md, and write no other file: an audit checks
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"crates/codecs/src/families/unreal_ini.rs:226-243","problem":"The locate walk's per-tree open cap gates on `candidates.files.len()` — the LOCATED set — not on the number of files opened. A path-grammar match that decodes cleanly but lacks `[/Script/Engine.PlayerInput]` is opened via `read::open_capped`, buffered whole, decoded and scanned, and never advances the counter that gates the next open, so opens per tree are bounded only by `MAX_LOCATE_ENTRIES_PER_TREE` (16,384) rather than `MAX_CANDIDATE_FILES` (32). This contradicts the aggregate budget this same diff ships in `schemas/README.md:202-208` (\"the product caps the *opens* too — so the worst case is ... (544 MiB) read per game\") and `schemas/bindings/v1/spec.md:74-78`, and the proposal's §The first family (\"At most `MAX_CANDIDATE_FILES` path-grammar matches per tree ... are opened through `read::open_capped` and `read::decode` for that content test\"). Practical worst case: every `…/Saved/Config/<dir>/*.ini` under each walked tree is opened and buffered at up to 1 MiB each — reachable without an adversary, since a Proton prefix tree reaches the grammar at 7 relative components (`AppData/Local/<Game>/Saved/Config/<platform>/<name>.ini`, within `MAX_LOCATE_DEPTH` = 8), so one query reads every game's config in that prefix. The design's accepted 544 MiB/game risk therefore does not bound what shipped, and no test can see it: `tests/unreal_ini.rs:262-280` uses 33 matches that all state the section, the one input where \"matches located\" and \"files opened\" coincide.","fix":"Count opens independently of locating: add `let mut opened = 0usize;` beside `examined`, test `if opened >= MAX_CANDIDATE_FILES { continue; }` before `read::open_capped`, and increment it for every grammar match the walk attempts to test, so the skip-and-count matches the README's \"caps the opens too\". Add the missing pin: N grammar-matching `.ini` files that do NOT state the section plus one that does, sorted past the cap, asserting the stating file is not located."},{"severity":"minor","location":"crates/codecs/src/families/unreal_ini.rs:232 and crates/codecs/src/families/unreal_ini.rs:117-121,516-533","problem":"`Detection` carries paths rather than contents, so `extract` re-reads and re-decodes each located file that `detect` already opened, buffered and scanned for the content test. Even with the open cap fixed, one `discovery bindings <slug>` run reads 2× the `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` = 544 MiB per game that `schemas/README.md:202-208` states as the worst case, because the aggregate sentence counts only one pass.","fix":"Either carry the already-decoded text on `UserCandidates` (or a private per-`detect` read cache keyed by path) so `extract` folds what `detect` read, keeping `read_and_decode` only for the located-on-path-grammar-only case; or state the doubled pass in `schemas/README.md` and `schemas/bindings/v1/spec.md` as `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS`, noting `--probe` pays one pass and the extracting face two."},{"severity":"minor","location":"crates/codecs/src/families/unreal_ini.rs:329-333","problem":"`Fold::remove` allocates a two-String key and then unconditionally `retain`s over the whole working set (up to `MAX_SNAPSHOT_ENTRIES + 1` = 4,097 pairs) for every `-Bindings=`/`!Bindings=` line — including removals of pairs the set never held, which consume no `admitted` budget and are therefore bounded only by `MAX_BINDING_FILE_OCTETS`. A 1 MiB file of ~40-byte removal tuples is ~25k lines, each scanning ~4k pairs ≈ 10^8 predicate evaluations, and `extract` folds up to 32 located files per tree across up to 17 trees: O(removal lines × working set), where the named budgets were meant to be the only bound on untrusted config input.","fix":"`seen` and `pairs` are maintained in lockstep by `add`/`remove`/`clear`, so membership is exact: build the key once, `if !self.seen.contains(&key) { return; }` before touching `pairs`, and reuse the one key for the set removal and the retain predicate. Better, keep the working set in a `BTreeSet<(action, input)>` so removal is O(log n) at no cost — `entries()` (unreal_ini.rs:343-358) already sorts by action then input byte order."},{"severity":"minor","location":"crates/codecs/src/families/unreal_ini.rs:194-206","problem":"The walk collects `read_dir`'s entire output into `Vec<(String, DirEntry)>` and sorts it before the loop applies the `examined >= MAX_LOCATE_ENTRIES_PER_TREE` gate, so transient allocation and the O(m log m) sort are bounded by the largest directory under the tree root, not by the budget `lib.rs:53-55` presents as bounding the walk (\"entries the locate walk examines below one `UserTree` root\"). The `ProtonPrefix` anchor (`paths.rs:117-137`) is a home-directory tree, precisely where large entry-count directories live.","fix":"Test the budget at the top of the queue loop before `read_dir` (an over-budget walk should not enter another directory) and cap the listing to the remaining budget (`entries.take(room)`) — determinism is unaffected, since which entries a truncated walk examines is already unspecified beyond the stop-and-count law pinned at `tests/unreal_ini.rs:231-256`. Otherwise document beside `MAX_LOCATE_ENTRIES_PER_TREE` that it bounds examined entries, not the transient listing."},{"severity":"nit","location":"crates/codecs/src/registry.rs:40-60","problem":"When a family's `appids()` contain `ctx.appid` and its `detect` returns `None`, pass 2 iterates the same slice and calls `detect(ctx)` on that family again — for `unreal-ini` a full `user_trees` enumeration plus a `locate` walk of every tree per record. Unreachable in shipped configuration (`appids()` empty, unreal_ini.rs:67-71) and live the moment `017-R4` lands appid routing data, the seam §The registry was built to receive.","fix":"Collect the family ids whose `detect` ran in pass 1 and skip them in pass 2 (or iterate pass 2 over `ordered` minus the pass-1 candidates)."},{"severity":"nit","location":"apps/discovery-cli/src/bindings.rs:224-235 (with 137-148)","problem":"Each matching record is built with the plugin's whole root set (bindings.rs:187), so a slug installed in two libraries resolves the same `userdata/<steamid>/<appid>` trees twice and `registry::claim` redoes the complete `user_trees` + `locate` walk — including the duplicate file reads — for the second record; `MergedClaim::absorb` drops the duplicate groups only after that cost is paid, and `extract`'s `dedup_by(tree_root)` dedups documents, not walks.","fix":"Memoize the claim per `(appid, roots)` across the record loop, or skip a record whose `(appid, plugin root set)` was already claimed, keying on the tree identity §Determinism already uses."},{"severity":"nit","location":"crates/codecs/src/read.rs:58-72","problem":"`decode` builds the String via `String::from_utf8(bytes.to_vec())` / `bytes[3..].to_vec()`, allocating and memcpy-ing a second full-size copy of a buffer `open_capped` already owns; both call sites (unreal_ini.rs:232, unreal_ini.rs:529-531) discard the `Vec<u8>` immediately. With the detect/extract double pass that is four full-file buffers per located file per run.","fix":"Add `read::decode_owned(Vec<u8>)` (or take `Vec<u8`) so the no-BOM path moves the buffer into `String::from_utf8`, keeping the borrowed-slice `decode` for the pure seam the suites call directly."}]}
```
