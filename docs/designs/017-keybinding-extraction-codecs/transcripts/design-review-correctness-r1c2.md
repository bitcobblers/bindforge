# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T04:38:44.103Z
- ended: 2026-11-24T04:45:41.504Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":45304,"output":31811,"reasoning":0,"cache.read":535639,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/correctness-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Determinism (`017-L4`) — canonical — \"across documents, deduplicated by (`user`, `provenance.files`) — two records that resolve the same tree state it once — then byte order by `user`\" (with §The CLI face step 3 and Test strategy #12)",
    "problem": "new (created by the round-1 duplicate-document closure): the dedup key is weaker than tree identity, so it merges documents from *different* trees whenever they share a `user` label and tree-relative paths — the normal shape of the multi-library case §The CLI face step 3 explicitly supports: one game in two libraries yields two `ProtonPrefix` trees whose `user` is the constant `steamuser` with the same tree-relative Windows path, and two resolved roots each holding `<same steamid>/<same appid>` yield two `SteamUserdata` trees with the same `user` and the same relative files. The dropped tree's located files then appear in no document's `provenance.files` and its bindings are silently lost — against §`bindings/v1`'s own \"One JSON document per game per located user tree\", against `017-C3`'s per-user document rule, and against `017-L3`, while `--probe` still prints those files as located. The key also fails to finish what it was added for: surviving same-`user` documents from distinct tre… [elided by the checkpoint cap]",
    "fix": "Key the dedup on the resolved `UserTree` anchor path (identity, never rendered) together with (`user`, `provenance.files`), so a tree is stated once per tree and two distinct trees under one label stay two documents; break the surviving `user` tie in the law by byte order over the serialized document; add one fixture (one slug in two libraries, two prefixes with different `Bindings`) asserting two documents and two `--probe` groups."
  },
  {
    "severity": "minor",
    "location": "Read discipline and budgets — `src/read.rs` — \"one extraction opens at most `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` files — the `+1` is the `ProtonPrefix` tree\" (with §Config-path resolvers' \"for every `root` in `ctx.roots` … at most `MAX_USER_DIRS_PER_GAME`\" and the table's \"userdata/ directories considered per game\")",
    "problem": "new: the §GameContext closure made `SteamUserdata` a union over `ctx.roots`, but no constant bounds `ctx.roots` — verified at `crates/discovery/src/steam_common.rs:77-82,369-381`: `with_roots` keeps the whole `host_candidates` list (10 candidates on Linux, `steam_linux.rs:92-128`) filtered only to installs that exist — so the tree count is `≤ MAX_USER_DIRS_PER_GAME × |ctx.roots| + 1` and the aggregate that `schemas/README.md` is told to publish understates the worst case by the root count. The resolver sentence also reads as a per-root cap while the constant's Bounds column says per game, leaving Test #3's \"a 17th digit-named directory is past `MAX_USER_DIRS_PER_GAME` and unvisited\" ambiguous as to 17th within one root or across the union.",
    "fix": "State that `MAX_USER_DIRS_PER_GAME` caps the *union*: the `user_trees` pass takes the distinct digit-named `<dir>` names across all of `ctx.roots` and stops at the 16th — which makes the `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` product true as written and makes #3 unambiguous."
  },
  {
    "severity": "minor",
    "location": "Discovery plumbing — `crates/discovery` — \"`SteamLinuxPlugin`/`SteamWindowsPlugin` expose `scan_records()` and `roots()` (the selected root candidates, in candidate order) as thin inherent wrappers\" (with §`GameContext`'s \"`pub roots: Vec<PathBuf>,  // every root the scan resolved, candidate order`\" and §Discovery plumbing's \"libraries are enumerated per root in root-candidate order\")",
    "problem": "new: \"candidate order\" is false of what a thin wrapper reads. `SteamCore::with_roots` replaces the candidate list with `canonical_paths(roots)` (`steam_common.rs:77-82`), which drops entries that fail to canonicalize or lack a `steamapps` directory and returns a `BTreeSet` — canonicalized, existence-filtered, byte-sorted (`:369-381`). The seam the design hands `GameContext.roots` therefore cannot deliver the order the comment and the enumeration sentence state, and Test #2 (\"yields … every resolved root\") pins a field whose stated order does not exist.",
    "fix": "Say what the seam holds: \"`roots()` returns the selected roots as the core holds them — canonicalized, existence-filtered, byte order — and `GameContext.roots` carries that order\", replacing both \"candidate order\" claims."
  },
  {
    "severity": "minor",
    "location": "The first family — `src/families/unreal_ini.rs` — \"`!` removes by name\" beside \"`-` removes on an exact match\", with \"blank lines and out-of-grammar lines in a section are skipped\"",
    "problem": "new: the newly accepted operator's operand grammar is unpinned, and the spelling the cited UE doc actually documents is the bare property form (verified live at A15's URL: \"`!` — Removes a property; but you do not have to have an exact match, just the name of the property\"). `!Bindings=` carries no tuple, so under this section's rules it is neither a `Bindings=(…)` tuple (the `UnparsableEntry` clause) nor an accepted pair line and falls to \"out-of-grammar lines … are skipped\": a binding set the file cleared keeps being reported — the same silent drift round 1 raised `!` to close. `!Bindings=` and `!Bindings=(Name=\"Q\")` have no stated difference.",
    "fix": "One clause replacing the bare \"`!` removes by name\": \"`!Key=` with no value clears every pair accumulated for that key in the working set; `!Key=<tuple>` removes the pairs the tuple names\" — and name both spellings in Test #7."
  },
  {
    "severity": "minor",
    "location": "Assumption register A15 — \"its `Input`/`PlayerInput` category carries `Bindings=(Name=…,Command=…)` lines with documented `+`/`-`/`.` merge operators\"",
    "problem": "new in changed text: the register still enumerates three operators while §The first family now decides four and the Revision log records \"the fold keeps UE's four merge operators (`+`, `.`, `-`, `!`)\". The A15 basis URL lists exactly four (`+`, `-`, `.`, `!`), so the row understates the basis its own citation supports and the register and the body disagree about the grammar's extent.",
    "fix": "Add `!` to A15's enumeration in place — one character and its wording, no new row."
  },
  {
    "severity": "minor",
    "location": "`bindings/v1` — the snapshot contract — \"Bounds, stated once here and cited elsewhere: … `files` ≤ `MAX_CANDIDATE_FILES`; the document ≤ `MAX_SNAPSHOT_OCTETS`\"",
    "problem": "new: the rewritten bounds list gives every other string in the closed key set a length bound (`codec`, `game`, `action`, every input token, `user`) but gives `provenance.files` entries only a count, so a contract `017-C3` requires to keep \"every string … inside the shared bounds\" ships one unbounded string field guarded only by the whole-document read cap; Test #10's refusal-by-name list consequently has no over-bound `files` case.",
    "fix": "Bound each `files` entry with one named constant stated once here (the locate walk's own ceiling — `MAX_LOCATE_DEPTH` components — is its honest home), and add the over-bound path to #10's refusal list."
  },
  {
    "severity": "minor",
    "location": "Rollout item 8 **Ledger** — \"the units whose shipped reading differs from their wording (`017-C1`'s bound and `extract` return shape, `017-C3`'s `inputs`, `017-C4`'s \"same scan\", `017-S1`'s launcher contexts)\", with §Context's \"Five clauses of the source are read differently than worded\"",
    "problem": "new: the Risks row states the run's own rule — \"Each divergence is a Revision-log entry with its basis, the unit ids are listed for the operator's ledger annotation (§Rollout item 8)\" — but two Revision-log decisions reinterpret source units and are in neither the five-clause count nor this hand-off: §GameContext's \"`roots` is `017-C2`'s whole resolved set, so `userdata/` is found under the install that owns the game, not a 'main' one\", which contradicts `017-C2`'s \"every steam root the scan resolved — main root and library roots separately, because `userdata/` lives only under the main root\"; and §Read discipline's \"`017-B2` counts (`action`, input-token) pairs\" against `017-B2`'s \"the cap on one snapshot's entries\". On a `ledger: yes` ADR the operator's `amended-by` lines are the only record of what shipped, and §Context's count is what the operator reconciles against. The §GameContext entry is also labelled \"Initial:\" though it is a round-1 change.",
    "fix": "Add `017-C2`'s root labelling and `017-B2`'s pair reading to Rollout 8's hand-off list and to §Context's count (seven), and relabel the §GameContext log entry `R1:`."
  },
  {
    "severity": "nit",
    "location": "The first family — `detect` — \"never descending a directory whose remaining depth is under the grammar's four trailing components\"",
    "problem": "\"remaining depth\" is undefined against `MAX_LOCATE_DEPTH`'s own definition (\"directory levels the locate walk descends below a `UserTree` root\"), and the natural reading (prune when `MAX_LOCATE_DEPTH − current depth < 4`) refuses to descend past level 4 and so cannot reach the depths 6–8 that Test strategy #6 pins as found (\"found at depth ≤ 8 and not at depth 9\").",
    "fix": "State the prune against the candidate rather than the walk: \"a directory at level *d* below the tree root is descended only while a `Saved/Config/<dir>/<name>.ini` can still complete by level `MAX_LOCATE_DEPTH`\"."
  },
  {
    "severity": "nit",
    "location": "The first family — `detect` — \"opened through `read::open_capped` and `read::decode`, so an over-budget or undecodable match claims and refuses by name rather than going unseen\"",
    "problem": "`detect` returns `Option<Detection>` and has no refusal channel, so the promise needs the locating rule spelled out: a file that fails `open_capped`/`decode` cannot also satisfy \"holding a `[/Script/Engine.PlayerInput]` section with a `Bindings=` line\", so as written it is unlocated and the refusal the sentence promises never happens.",
    "fix": "One clause: a path-grammar match that fails `open_capped`/`decode` is located on path grammar alone and refused later by `extract` by name, while a decodable file without the section is not located."
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix — Steam Deck row — \"A Linux Steam install with the `$HOME` and Proton prefix layout the two rows above claim\"",
    "problem": "no shipped resolver builds anything from `$HOME` (§Config-path resolvers: \"No host-config tree ships this run\"; §Deferred carries it), and the two rows above claim exactly the opposite for the host config home (\"a host-config title answers `claim-refused`\"); the row is right that the Linux row covers the Deck, but the layout it names is not one this run resolves.",
    "fix": "\"the root-relative `userdata/` and Proton prefix layout the two rows above claim, with the host config home deferred as they state\"."
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
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Determinism (`017-L4`) — canonical + The codec face — \"across documents, deduplicated by the `UserTree.root` path each was built from — tree identity, never rendered\" (with §The CLI face step 3's \"§Determinism's dedup by tree identity keeps one tree stated once\" and Test strategy #12's \"with the registry and the `UserTree` vector shuffled in the test harness, yields byte-identical documents\")","problem":"new (created by the round-1 dedup closure): the dedup key the law now names is carried by no type in the design. `Detection` is `{family, evidence, per_user: Vec<UserCandidates>}` and `UserCandidates` is `{user, files}`; `extract` returns `Vec<BindingSnapshot>` whose top-level keys are closed at `{format, version, codec, game, user, entries, provenance}` with tree-*relative* `files`. Neither the face that performs the cross-record dedup at step 3 nor the ordering law can see a document's `UserTree.root` — the round-1 key (`user`, `provenance.files`) was exactly what the payload exposes, and the replacement key is absent from it. Test #12 compounds it: shuffling the `UserTree` vector and demanding byte-identical documents requires re-sorting documents by tree identity, i.e. the same missing field. Failure scenario as written: two records for one slug each return their trees' documents and the face can compare nothing but `user` plus relative paths — the round-1 collision re-opened one layer down.","fix":"Add tree identity to the claim payload — `pub struct UserCandidates { pub user: String, pub tree_root: PathBuf, pub files: Vec<PathBuf> }`, `tree_root` carried for dedup and ordering only and never rendered into a document (§Determinism already says \"never rendered\") — and state that `per_user` is built in `user_trees` order and that `extract`'s `Vec` is positionally parallel to it; this replaces the implicit positional coupling and changes nothing in `bindings/v1`'s closed key set."},{"severity":"minor","location":"Determinism + `bindings/v1` + Test strategy #12 — \"two distinct trees that share a `user` label stay two documents\" (with §`bindings/v1`'s \"One JSON document per game per located user tree\" and #12's \"two `ProtonPrefix` trees sharing the `steamuser` label with different bindings — yield two documents\")","problem":"new: document identity is now the located tree, which is a reading of `017-C3`'s \"one document per game per local SteamID where user-scoped\" that the run's own divergence record does not carry — §Context's seven clauses name `017-C3`'s \"`inputs` spelling\" only, and the Revision-log entry that speaks to document count still gives the basis \"`017-C3`'s per-user rule wins\", which is now the opposite of what ships. Consequence beyond the ledger on a `ledger: yes` source: two documents can share every identity field a consumer can read (`codec`, `game`, `user`) while differing in `entries`, with identical tree-relative `provenance.files`, so `--json` shows an unlabelled pair of same-user documents.","fix":"Keep the per-tree rule and make the record true: reword that Revision-log entry's basis to \"read differently than `017-C3`'s per-local-SteamID wording\", name `017-C3`'s document-identity clause in §Context's count and §Rollout item 8's hand-off list, and add one clause to §`bindings/v1` stating that a consumer distinguishes same-`user` documents by nothing in the contract and must union their entries."},{"severity":"minor","location":"The first family — `src/families/unreal_ini.rs` `extract` — \"a bare `!Key=` (no value) clears every pair accumulated for that key, `!Key=<tuple>` removes only the pairs it names\" (with §`bindings/v1`'s \"`inputs` may be empty when the file's own merge operators removed every input for an action\" and \"the (`action`, input-token) pair total ≤ `MAX_SNAPSHOT_ENTRIES`, which bounds `entries`\", Options' \"record every distinct (native action, native input) pair … grouped by action\", and §Read discipline's \"`SnapshotEntryBudget`, checked as each pair enters a tree's working set\")","problem":"new: the fold still does not say whether an action whose pairs have all been removed survives as an entry with `inputs: []`, and the cited passages answer opposite ways — §`bindings/v1`'s allowance says it survives, Options(b) derives entries from the surviving pair set so an action with no pairs contributes nothing. The newly added bare-clear is the operation that makes this the common case, and it decides the budget: if emptied actions survive and the mid-fold check counts the current working-set size, a churn file (add 4096, remove 4096, add 4096 others) passes the check and ships more than `MAX_SNAPSHOT_ENTRIES` entries, so \"which bounds `entries`\" is false.","fix":"One clause in §The first family — \"a `Command` whose pairs are all removed leaves no entry\" (then delete §`bindings/v1`'s empty-`inputs` allowance and its \"never for any other reason\" tail), or the mirror of it — plus three words in §Read discipline fixing the check to distinct pairs admitted; and name the bare operand for the other operators in the same sentence and in Test #7: \"`-Bindings=`, `+Bindings=` and `.Bindings=` with no value are out-of-grammar and skipped\", which the new bare-`!` form otherwise leaves open."},{"severity":"minor","location":"`bindings/v1` — the snapshot contract — \"`files` ≤ `MAX_CANDIDATE_FILES`, each entry at most `MAX_LOCATE_DEPTH` path components — the locate walk's own ceiling\" (with §Read discipline's table row \"`MAX_LOCATE_DEPTH = 8` | directory levels the locate walk descends below a `UserTree` root\" and Test strategy #6's \"found at depth ≤ 8 and not at depth 9\")","problem":"new: the bound borrowed a constant whose only definition counts directory levels, while the bound itself counts path components, and nothing in the document defines the depth of a file. Under the table's definition a file eight directory levels below the tree root has nine tree-relative components, so `detect` legitimately locates a file whose entry the loader then refuses — Test #10 (\"a `files` entry over `MAX_LOCATE_DEPTH` components\") and Test #11 (\"documents produced by the family all pass the loader\") cannot both hold. The fixture layouts sit at that boundary: `Documents/My Games/<Project>/Saved/Config/<Platform>/<name>.ini` is seven components under a Proton prefix root.","fix":"State the units once, in the candidate's terms — \"a located file's tree-relative path is at most `MAX_LOCATE_DEPTH` components, its last four being the grammar's `Saved/Config/<dir>/<name>.ini`\" — and make #6 say \"found at ≤ `MAX_LOCATE_DEPTH` components and not beyond\" instead of an undefined \"depth\"."},{"severity":"nit","location":"Config-path resolvers — `src/paths.rs` — \"one per digit-named `<dir>` in byte order, at most `MAX_USER_DIRS_PER_GAME` such trees in total across all roots\" (with the section's return order \"in `TreeKind` declaration order then byte order of the tree's `root`\" and Test strategy #3's \"a 17th tree over all of `ctx.roots`\")","problem":"new: the cap is now stated over the union, but the order in which the 16th tree is reached is not, and the bullet runs two orders at once — the enumeration reads as roots-then-directory, the return order is byte order of the full tree path, and those disagree for prefix-adjacent roots (byte order puts `/home/a-b/userdata/…` before `/home/a/userdata/…` because `-` < `/`, while `roots()` byte order puts `/home/a` first). Which users the silent bound keeps is therefore open, and #3 does not pin it.","fix":"Name the cut order in that bullet — \"the cap is applied while enumerating `ctx.roots` in `roots()` order and `<dir>` in byte order; the surviving trees are then returned in `root` byte order\" — and state which two roots and directories #3's fixture uses."}]}
```
