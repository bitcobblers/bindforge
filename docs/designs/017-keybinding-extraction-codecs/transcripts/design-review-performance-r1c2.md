# Transcript: design-review:performance r1c2

## Provenance
- stage: design-review:performance (round r1c2)
- agent: review/design-performance
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T04:38:44.104Z
- ended: 2026-11-24T04:42:37.723Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":40461,"output":12018,"reasoning":0,"cache.read":293272,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff a4dda6a8542694895102f4a9460835e189e4f08b e068cf6132a6fc1e4d93c99cfa7a7f303e66939f` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/performance-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
[
  {
    "severity": "major",
    "location": "§The first family — \"examining at most `MAX_LOCATE_ENTRIES_PER_TREE` entries per tree and never descending a directory whose remaining depth is under the grammar's four trailing components\"",
    "problem": "The new prune compares the remaining depth budget against the constant 4 without the case where the directory being descended is itself one of the grammar's named components (`Saved`, `Config`, the platform directory), where the components still needed below it are 3, 2 or 1. With `MAX_LOCATE_DEPTH = 8` the rule stops descending at relative depth 5, so a candidate at relative depth 6–8 whose `Saved` sits at relative depth 2–5 is never enumerated. The design's own flagship Proton layout is that shape: the `ProtonPrefix` root `<ctx.library>/steamapps/compatdata/<ctx.appid>/pfx/drive_c/users/steamuser` holds a packaged UE title's config at `AppData/Local/<Project>/Saved/Config/<Platform>/<Game>.ini` (relative depth 7), and the walk must descend `Config` at relative depth 5, where `8 − 5 = 3 < 4` prunes it. That title answers no-claim / `family-confirmed-files-absent` for the layout the row exists to serve, and Test strategy 6's pin \"the `Saved/Config/<dir>/` grammar is found at depth ≤ 8 … [elided by the checkpoint cap]",
    "fix": "Delete the clause — `MAX_LOCATE_DEPTH` plus the newly added `MAX_LOCATE_ENTRIES_PER_TREE` already bound the walk, which was the point of round 1's finding 1 — or restate it name-anchored: a directory at relative depth `d` need not be descended only when neither its own name nor any name within the three levels above it is one of the grammar's components, so the surviving prune costs no locatable file."
  },
  {
    "severity": "major",
    "location": "§The first family / §Read discipline and budgets — \"A candidate is an `*.ini` whose `Path::components()` end in `Saved/Config/<one directory>/<name>.ini` … holding a `[/Script/Engine.PlayerInput]` section with a `Bindings=` line, capped at `MAX_CANDIDATE_FILES` per tree and opened through `read::open_capped` and `read::decode`\"",
    "problem": "The content test is part of candidacy, so every path-grammar match must be opened and decoded before it can be counted as a candidate; the 32-file cap bounds the retained set, not the opens, and after this revision the only thing bounding opens is `MAX_LOCATE_ENTRIES_PER_TREE = 16_384`. Worst case is 16,384 opens of up to `MAX_BINDING_FILE_OCTETS` each per tree (≈16 GiB), across up to 17 trees per game, against the figure §Read discipline instructs `schemas/README.md` to copy: \"one extraction opens at most `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` files … (544 MiB)\". The shipped budget-family ceiling is understated by up to ~512× per tree — round 1's result-not-work shape relocated from the walk leg to the content-probe leg — and nothing refuses, since `detect` returns `Option<Detection>`, so the cost `017-L1` calls untrusted input is paid silently.",
    "fix": "Gate the opens with the constant that exists: \"at most `MAX_CANDIDATE_FILES` path-grammar matches per tree are opened and content-tested, in the walk's byte order; a further match is skipped and counted the way an entry past `MAX_LOCATE_ENTRIES_PER_TREE` is\". The located set, byte order and §Determinism are untouched and the aggregate product becomes the actual open ceiling (ADR-017 Rollout 5: \"budgets enforced at read and locate time\")."
  },
  {
    "severity": "minor",
    "location": "§Config-path resolvers / §Read discipline and budgets — \"`SteamUserdata`: `<root>/userdata/<dir>/<ctx.appid>` for every `root` in `ctx.roots`, one per digit-named `<dir>`, byte order, at most `MAX_USER_DIRS_PER_GAME`\"",
    "problem": "This revision moved `userdata/` off a single anchor onto a `Vec` no constant bounds (`crates/discovery/src/steam_linux.rs:92-128` passes every candidate to `SteamCore::with_roots`, so a native plus Flatpak plus Snap install resolves about ten roots), and the sentence does not say whether `MAX_USER_DIRS_PER_GAME` caps the union over all roots or the directories found under each root. Test strategy 3's \"byte-order union over `ctx.roots`\" and \"a 17th digit-named directory is past `MAX_USER_DIRS_PER_GAME`\" imply the union — the only reading under which the aggregate's 17-tree term holds; the per-root reading makes the ceiling copied into `schemas/README.md` wrong by the roots factor.",
    "fix": "Make the scope explicit in the same clause — \"at most `MAX_USER_DIRS_PER_GAME` `SteamUserdata` trees in total across all `ctx.roots`\" — and mirror it in that constant's Bounds cell, so the shipped number is derivable from the budget table alone."
  },
  {
    "severity": "minor",
    "location": "§Read discipline and budgets / §`bindings/v1` — \"`SnapshotEntryBudget` on the pair total\" and \"the (`action`, input-token) pair total ≤ `MAX_SNAPSHOT_ENTRIES`, which bounds `entries` and every `inputs` array\"",
    "problem": "The count is now the right one but no check site is named, and `extract` folds every located file of a tree (`files` ≤ `MAX_CANDIDATE_FILES`, each ≤ `MAX_BINDING_FILE_OCTETS`) into a working set before building the document the bound is stated on. Checked at document construction, the bound prices the result while the transient peak is bounded only by bytes read: up to ~32 MiB of `+Bindings=` lines per tree folded into strings before the refusal fires, across the 17 trees `extract` returns as one `Vec`. Round 1 asked for \"the same `SnapshotEntryBudget` check site evaluated on the pair total\"; the pair total arrived, the site did not.",
    "fix": "One clause in §Read discipline: the check runs as each pair enters the tree's working set, so the fold stops at `MAX_SNAPSHOT_ENTRIES + 1` pairs and names itself; Test strategy 4's two breach shapes then read as one mechanism. No new constant, no new refusal name."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/017-keybinding-extraction-codecs.md.
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/performance-r1c2.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"§Read discipline and budgets — \"since §The first family caps a tree's reads at `MAX_CANDIDATE_FILES` the product also ceilings the *opens*, so the worst case is `17 × 32 × MAX_BINDING_FILE_OCTETS` (544 MiB) per game\"","problem":"The aggregate counts one open pass per tree, but the family performs two: `detect` opens every path-grammar match through `read::open_capped` and `read::decode` for the content test, and `extract` then \"reads only located files, through `read.rs`\" — necessarily a second open pass, since the claim payload is `UserCandidates { user: String, files: Vec<PathBuf> }` (paths, not bytes). Worst case per game is 2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES opens (1088) and ≈1.1 GiB read, against the figure this section instructs `schemas/README.md` to copy into its budget family (544 opens / 544 MiB) — the shipped aggregate is 2× low, in the repo's only place where a read cost is stated as a number, and the precedent shape it cites (`MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES`, schemas/README.md:132-133) is exact rather than approximate. Residue of round 1's finding 2, narrowed from 512× to 2× by the new cap.","fix":"One clause naming both legs: \"detect's content test and extract's fold are two open passes over the same ≤ `MAX_CANDIDATE_FILES` set, so the face reads at most `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` (1.1 GiB) per game\" — or replace §The first family's \"`extract` reads only located files, through `read.rs`\" with `UserCandidates` carrying each located file's decoded bytes so extract never re-opens, making the single-pass figure true at the cost of retaining up to `MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` per tree. Either keeps the budget table, constants and refusal names unchanged."},{"severity":"minor","location":"§`bindings/v1` Bounds with the §Read discipline table row — \"`files` ≤ `MAX_CANDIDATE_FILES`, each entry at most `MAX_LOCATE_DEPTH` path components — the locate walk's own ceiling\" vs \"`MAX_LOCATE_DEPTH = 8` | directory levels the locate walk descends below a `UserTree` root\"","problem":"The two bounds are stated in different units and differ by one. A file in a directory at the deepest legal descent has 8 directory components plus its own name = 9 path components, so the refusal Test strategy 10 pins (\"a `files` entry over `MAX_LOCATE_DEPTH` components\") fires on a located file Test strategy 6 says the walk finds (\"the `Saved/Config/<dir>/` grammar is found at depth ≤ 8\"), and Test strategy 11's \"documents produced by the family all pass the loader\" cannot hold at that boundary — in the one clause whose stated purpose (\"the locate walk's own ceiling\") is to make the contract bound derivable from the walk. Minor because the layouts this family serves are 6–7 components deep, so the gap sits at the cap's edge, not on a shipped path.","fix":"State the unit once in the table row: \"directory levels the locate walk descends below a `UserTree` root, so a located file's tree-relative path has at most `MAX_LOCATE_DEPTH` components counting the file itself (a file in the root is 1)\" — or restate the contract bound as `MAX_LOCATE_DEPTH + 1` components. No new constant, no change to the walk."}]}
```
