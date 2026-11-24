# Transcript: design-review:correctness r1c3

## Provenance
- stage: design-review:correctness (round r1c3)
- agent: review/design-correctness
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T04:52:16.527Z
- ended: 2026-11-24T04:55:55.771Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":40642,"output":15720,"reasoning":0,"cache.read":419781,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff e068cf6132a6fc1e4d93c99cfa7a7f303e66939f 316ef46c5517497b633f76f56fa0ffa689babebf` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/correctness-r1c3.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Determinism (`017-L4`) — canonical + The codec face — \"across documents, deduplicated by the `UserTree.root` path each was built from — tree identity, never rendered\" (with §The CLI face step 3's \"§Determinism's dedup by tree identity keeps one tree stated once\" and Test strategy #12's \"with the registry and the `UserTree` vector shuffled in the test harness, yields byte-identical documents\")",
    "problem": "new (created by the round-1 dedup closure): the dedup key the law now names is carried by no type in the design. `Detection` is `{family, evidence, per_user: Vec<UserCandidates>}` and `UserCandidates` is `{user, files}`; `extract` returns `Vec<BindingSnapshot>` whose top-level keys are closed at `{format, version, codec, game, user, entries, provenance}` with tree-*relative* `files`. Neither the face that performs the cross-record dedup at step 3 nor the ordering law can see a document's `UserTree.root` — the round-1 key (`user`, `provenance.files`) was exactly what the payload exposes, and the replacement key is absent from it. Test #12 compounds it: shuffling the `UserTree` vector and demanding byte-identical documents requires re-sorting documents by tree identity, i.e. the same missing field. Failure scenario as written: two records for one slug each return their trees' documents and the face can compare nothing but `user` plus relative paths — the round-1 collision re-opened one l… [elided by the checkpoint cap]",
    "fix": "Add tree identity to the claim payload — `pub struct UserCandidates { pub user: String, pub tree_root: PathBuf, pub files: Vec<PathBuf> }`, `tree_root` carried for dedup and ordering only and never rendered into a document (§Determinism already says \"never rendered\") — and state that `per_user` is built in `user_trees` order and that `extract`'s `Vec` is positionally parallel to it; this replaces the implicit positional coupling and changes nothing in `bindings/v1`'s closed key set."
  },
  {
    "severity": "minor",
    "location": "Determinism + `bindings/v1` + Test strategy #12 — \"two distinct trees that share a `user` label stay two documents\" (with §`bindings/v1`'s \"One JSON document per game per located user tree\" and #12's \"two `ProtonPrefix` trees sharing the `steamuser` label with different bindings — yield two documents\")",
    "problem": "new: document identity is now the located tree, which is a reading of `017-C3`'s \"one document per game per local SteamID where user-scoped\" that the run's own divergence record does not carry — §Context's seven clauses name `017-C3`'s \"`inputs` spelling\" only, and the Revision-log entry that speaks to document count still gives the basis \"`017-C3`'s per-user rule wins\", which is now the opposite of what ships. Consequence beyond the ledger on a `ledger: yes` source: two documents can share every identity field a consumer can read (`codec`, `game`, `user`) while differing in `entries`, with identical tree-relative `provenance.files`, so `--json` shows an unlabelled pair of same-user documents.",
    "fix": "Keep the per-tree rule and make the record true: reword that Revision-log entry's basis to \"read differently than `017-C3`'s per-local-SteamID wording\", name `017-C3`'s document-identity clause in §Context's count and §Rollout item 8's hand-off list, and add one clause to §`bindings/v1` stating that a consumer distinguishes same-`user` documents by nothing in the contract and must union their entries."
  },
  {
    "severity": "minor",
    "location": "The first family — `src/families/unreal_ini.rs` `extract` — \"a bare `!Key=` (no value) clears every pair accumulated for that key, `!Key=<tuple>` removes only the pairs it names\" (with §`bindings/v1`'s \"`inputs` may be empty when the file's own merge operators removed every input for an action\" and \"the (`action`, input-token) pair total ≤ `MAX_SNAPSHOT_ENTRIES`, which bounds `entries`\", Options' \"record every distinct (native action, native input) pair … grouped by action\", and §Read discipline's \"`SnapshotEntryBudget`, checked as each pair enters a tree's working set\")",
    "problem": "new: the fold still does not say whether an action whose pairs have all been removed survives as an entry with `inputs: []`, and the cited passages answer opposite ways — §`bindings/v1`'s allowance says it survives, Options(b) derives entries from the surviving pair set so an action with no pairs contributes nothing. The newly added bare-clear is the operation that makes this the common case, and it decides the budget: if emptied actions survive and the mid-fold check counts the current working-set size, a churn file (add 4096, remove 4096, add 4096 others) passes the check and ships more than `MAX_SNAPSHOT_ENTRIES` entries, so \"which bounds `entries`\" is false.",
    "fix": "One clause in §The first family — \"a `Command` whose pairs are all removed leaves no entry\" (then delete §`bindings/v1`'s empty-`inputs` allowance and its \"never for any other reason\" tail), or the mirror of it — plus three words in §Read discipline fixing the check to distinct pairs admitted; and name the bare operand for the other operators in the same sentence and in Test #7: \"`-Bindings=`, `+Bindings=` and `.Bindings=` with no value are out-of-grammar and skipped\", which the new bare-`!` form otherwise leaves open."
  },
  {
    "severity": "minor",
    "location": "`bindings/v1` — the snapshot contract — \"`files` ≤ `MAX_CANDIDATE_FILES`, each entry at most `MAX_LOCATE_DEPTH` path components — the locate walk's own ceiling\" (with §Read discipline's table row \"`MAX_LOCATE_DEPTH = 8` | directory levels the locate walk descends below a `UserTree` root\" and Test strategy #6's \"found at depth ≤ 8 and not at depth 9\")",
    "problem": "new: the bound borrowed a constant whose only definition counts directory levels, while the bound itself counts path components, and nothing in the document defines the depth of a file. Under the table's definition a file eight directory levels below the tree root has nine tree-relative components, so `detect` legitimately locates a file whose entry the loader then refuses — Test #10 (\"a `files` entry over `MAX_LOCATE_DEPTH` components\") and Test #11 (\"documents produced by the family all pass the loader\") cannot both hold. The fixture layouts sit at that boundary: `Documents/My Games/<Project>/Saved/Config/<Platform>/<name>.ini` is seven components under a Proton prefix root.",
    "fix": "State the units once, in the candidate's terms — \"a located file's tree-relative path is at most `MAX_LOCATE_DEPTH` components, its last four being the grammar's `Saved/Config/<dir>/<name>.ini`\" — and make #6 say \"found at ≤ `MAX_LOCATE_DEPTH` components and not beyond\" instead of an undefined \"depth\"."
  },
  {
    "severity": "nit",
    "location": "Config-path resolvers — `src/paths.rs` — \"one per digit-named `<dir>` in byte order, at most `MAX_USER_DIRS_PER_GAME` such trees in total across all roots\" (with the section's return order \"in `TreeKind` declaration order then byte order of the tree's `root`\" and Test strategy #3's \"a 17th tree over all of `ctx.roots`\")",
    "problem": "new: the cap is now stated over the union, but the order in which the 16th tree is reached is not, and the bullet runs two orders at once — the enumeration reads as roots-then-directory, the return order is byte order of the full tree path, and those disagree for prefix-adjacent roots (byte order puts `/home/a-b/userdata/…` before `/home/a/userdata/…` because `-` < `/`, while `roots()` byte order puts `/home/a` first). Which users the silent bound keeps is therefore open, and #3 does not pin it.",
    "fix": "Name the cut order in that bullet — \"the cap is applied while enumerating `ctx.roots` in `roots()` order and `<dir>` in byte order; the surviving trees are then returned in `root` byte order\" — and state which two roots and directories #3's fixture uses."
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
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/correctness-r1c3.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"nit","location":"`bindings/v1` — the snapshot contract, Input spelling (canonical) — \"so both halves of a row are reported in the game's words and nothing in this format validates against the closed member sets (A5, A20; Options)\" (the paragraph from which \"`inputs` may be empty when the file's own merge operators removed every input for an action, and never for any other reason\" was deleted)","problem":"new (residual of the round-1 closure of this seat's emptied-`Command` finding): deleting that sentence settled what the family emits (\"a `Command` whose pairs the fold has all removed leaves no entry\"), but the contract never gained the array bound that replaces it. `entries` is `[{action, inputs}]`, entry keys are closed to `{action, inputs}`, and the Bounds sentence bounds only `action` and \"every input token\" lengths — nothing states that `inputs` is non-empty, `spec.md` is said to carry \"the key set, the bounds, … the refusal names\", and Test #10's invalid-corpus list does not name it. Failure scenario: a hand-authored or future-family `bindings/v1` document with `{\"action\":\"MoveForward\",\"inputs\":[]}` loads clean, prints an empty `Inputs` column, and states a binding no located file states (`017-L3`) — the fact the deleted sentence was carrying, now unstated.","fix":"Three words in the Bounds sentence — \"`inputs` is a non-empty array\" — plus one item in Test #10's invalid list (\"an empty `inputs` array\"). This replaces the deleted allowance as the contract's statement of that fact; no new mechanism, and it rides the loader's existing \"value outside its bound\" refusal name."},{"severity":"nit","location":"Test strategy #4 — \"`UserCandidates::push` refuses a 33rd file (`017-B3`'s type invariant, behind the walk's skip-and-count)\"","problem":"new: this revision names two owners for one bound on one type. §The codec face publishes the field as `pub files: Vec<PathBuf>` and never lists a `push`, so a caller can build or extend a `UserCandidates` past 32 without calling it and the bound is not a \"type invariant\"; §The first family puts enforcement in the walk (\"At most `MAX_CANDIDATE_FILES` path-grammar matches per tree, in the walk's byte order … a further match is skipped and counted\"), and §Read discipline's row reads \"one user tree's located candidate set\". #4 asserts the enforcement point the published surface cannot guarantee.","fix":"Name one owner: either make the walk the bound and have #4 assert \"a 33rd path-grammar match in one tree is skipped and counted\", or keep `push` and change the listing to a private field built only by `UserCandidates::new`/`push` — in which case §The first family's \"skipped and counted\" clause stops claiming to enforce `MAX_CANDIDATE_FILES`."}]}
```
