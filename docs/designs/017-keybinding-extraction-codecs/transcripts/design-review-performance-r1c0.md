# Transcript: design-review:performance r1c0

## Provenance
- stage: design-review:performance (round r1c0)
- agent: review/design-performance
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T03:39:05.303Z
- ended: 2026-11-24T03:46:48.763Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":38721,"output":14319,"reasoning":0,"cache.read":221509,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/017-keybinding-extraction-codecs.md,
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
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/performance-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/017-keybinding-extraction-codecs.md.
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/performance-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"§The first family / §Read discipline and budgets — \"`detect` walks each `UserTree` depth-first in byte order to `MAX_LOCATE_DEPTH`, collecting `*.ini` files whose path ends `Saved/Config/<one directory>/<name>.ini` ... capped at `MAX_CANDIDATE_FILES` per tree\"","problem":"The cap bounds the result set, not the traversal, and two of the three UserTree roots are whole user-directory trees: `NativeHost` is \"$XDG_CONFIG_HOME when set and absolute, else $HOME/.config\" and `ProtonPrefix` is `<library>/steamapps/compatdata/<appid>/pfx/drive_c/users/steamuser` (the entire emulated profile). The worst case is the zero-match case — a game the family does not have natively — where the walk descends the full tree to depth 8, calls `file_type()` on every entry, and never reaches the 32-file cap that would stop it early. No constant bounds directories or entries visited, so the command path carries an unbounded synchronous walk per invocation; the aggregate sentence counts only opened files and reads as a bound where none exists. This is a new cost shape: the shipped scan is shallow (one `read_dir` of `<library>/steamapps` at crates/discovery/src/steam_common.rs:195 plus size-capped VDF/ACF reads), so no existing posture already bounds a deep walk.","fix":"Add one design-time constant beside the other two (e.g. `MAX_LOCATE_ENTRIES_PER_TREE`: entries examined below a `UserTree` root, over-cap entries skipped and counted the way `steam_common`'s `skipped` counter already does, degrading to `family-confirmed-files-absent`), name it in the §Read discipline budget table and in the aggregate sentence, and add a prune clause: the grammar's four trailing components (`Saved/Config/<dir>/<name>.ini`) mean the walk need not descend a directory whose remaining depth budget is below that suffix."},{"severity":"major","location":"§`bindings/v1` / §Read discipline and budgets — \"`entries` ≤ `MAX_SNAPSHOT_ENTRIES`\" with table row \"`MAX_SNAPSHOT_ENTRIES = 4096` | one snapshot's entries | `017-B2`\", read against \"Entries are keyed by the `Command` string verbatim ... and carry the distinct `Name` tokens verbatim as `inputs`\"","problem":"The budget counts grouped actions, not bindings, so no constant bounds a snapshot's input-token count, pair count or serialized size. A 1 MiB INI is inside `MAX_BINDING_FILE_OCTETS`, so no file budget fires; ~15,000 lines of `+Bindings=(Name=\"KEY_<n>\",Command=\"Action\")` produce one entry with 15,000 inputs, `SnapshotEntryBudget` never triggers, and across `MAX_CANDIDATE_FILES` per tree the in-memory token set is bounded only by bytes read. Peak is worse than one document because `extract` returns every user tree's snapshot at once (`Result<Vec<BindingSnapshot>, ExtractionRefusal>`) across up to 18 trees. `017-B2`'s purpose is the pair count (\"a game with more bindings than this does not exist today\"), and the Risks table names \"five named budgets\" as the mitigation for \"Untrusted config files: enormous, deeply nested, UTF-16, symlinked out of the tree, over budget\", which this unit choice defeats.","fix":"Define the `017-B2` count over distinct (action, input-token) pairs instead of `entries` array length — the same `SnapshotEntryBudget` check site evaluated on the pair total — and state in §`bindings/v1` that `entries` array length is additionally bounded by it. No new constant, no new refusal name."},{"severity":"minor","location":"§Read discipline and budgets — \"one extraction opens at most `MAX_USER_DIRS_PER_GAME × MAX_CANDIDATE_FILES` files, so its worst-case read cost is `16 × 32 × MAX_BINDING_FILE_OCTETS` (512 MiB) for one game, offline, one game at a time\"","problem":"The product omits two of the three template families: `MAX_USER_DIRS_PER_GAME` bounds only the `SteamUserdata` trees, while `ProtonPrefix` and `NativeHost` each contribute one more tree with its own `MAX_CANDIDATE_FILES` budget, and `extract` returns one document per tree. The README amendment is instructed to copy this sentence (\"states the aggregate product\"), so `schemas/README.md`'s budget family would ship a ceiling understated by 64 files / 64 MiB. The figure also ignores the per-record multiplier: step 3 runs `from_record` → `claim` → `extract` \"Per matching record\" and \"More than one record matching ... each is processed and rendered\", so N matching records repeat the same `NativeHost` walk N times (that tree's root does not depend on the record's root or library).","fix":"State the product as `(MAX_USER_DIRS_PER_GAME + 2) × MAX_CANDIDATE_FILES` files (576 at the current constants), naming the `+2` as the `ProtonPrefix` and `NativeHost` trees, plus one clause that the worst case multiplies by the number of matching records."}]}
```
