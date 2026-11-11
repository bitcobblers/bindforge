# Transcript: design-review:correctness r2c2

## Provenance
- stage: design-review:correctness (round r2c2)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T18:50:38.910Z
- ended: 2026-11-10T18:55:18.614Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 2. This pass's scope: the sections changed since you last reviewed: run `git diff 44c52fa6533e711e4a34c1987afa1d6694260c86 82b346f031267b92b35257222c125107ae1b6d0c` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "Chosen design — Read-only git (item 5) — \"`git()`'s env (106-112) becomes the shared env for every runner git call — `gitOk` (143-145) and the raw `merge-base` spawn (1417-1420) included\"",
    "problem": "re-raised. My r1 env-scope finding is partly closed — the text now claims \"every runner git call\" and names gitOk and merge-base, and #13, the Risks row and the Compatibility row follow — but the enumeration still misses a fourth spawn site: `writeEditorialDiff` (called from runner.ts:1507 in the review/editorial loop) runs `spawnSync('git', ['diff','--no-index',…], {cwd: repoDir})` at runner.ts:1933-1937, outside `git()`/`gitOk`, inheriting the operator's environment and reading the worktree's config. So the absolute claim in the paragraph and in #13's gate (\"every runner git call, `gitOk` and `merge-base` included\") is false as written, and an implementer who treats the list as exhaustive leaves that call un-pinned.",
    "fix": "Name all non-`git()` sites: \"`gitOk` (143-145), the raw `merge-base` spawn (1417-1420) and `writeEditorialDiff`'s `git diff --no-index` (1933-1937) included\", and add the editorial-diff call to #13's \"every runner git call\" list; or state the rule as \"every `spawnSync('git')` site goes through the one env helper\", which keeps the claim true as sites change."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Read-only git (item 5) — \"So neither the config sources outside the repo (A6) nor a planted `.git/config` or `.git/hooks` runs at runner privilege\" (and the Risks row's \"nothing outside the repo executes at runner privilege\")",
    "problem": "new. The conclusion is wider than the mechanism named two sentences earlier. `GIT_CONFIG_NOSYSTEM`/`GIT_CONFIG_GLOBAL` suppress only the system and per-user levels (I confirmed against git(1): setting GIT_CONFIG_GLOBAL means neither $HOME/.gitconfig nor $XDG_CONFIG_HOME/git/config is read), and `-c core.fsmonitor=false -c core.hooksPath=<empty dir>` overrides exactly two keys; the repo-local `.git/config` is still read, and A6 — cited in this same paragraph — states config is also read via \"`include.path` from any of them\". A planted `.git/config` with `[include] path = /home/<operator>/.gitconfig` re-opens the outside-repo sources at runner privilege, and other exec-capable repo-local keys that fire in the escalation's own git calls (e.g. `filter.<driver>.clean` under `git add -A` at runner.ts:298, selected by a planted `.gitattributes`, which is not a `.git` segment and so is neither config-denied nor audit-matched) are untouched by the pins. #13's fsmonitor-marker fixture passes whi… [elided by the checkpoint cap]",
    "fix": "Scope the sentence to what ships: \"neither the system nor per-user config sources A6 names are read, and the two A6 mechanisms that fire in these calls — `core.fsmonitor` on an index refresh and hooks — are overridden; other exec-capable repo-local keys (filters, external diff) and an `include.path` inside a planted `.git/config` are the Deferred sandbox's\" — or add the pins that would make the blanket form true."
  },
  {
    "severity": "nit",
    "location": "Chosen design — Read-only git (item 5) — closing sentences of the paragraph (previously \"If Test strategy #13 lets a vector land a plant, or a runner git call run over one, the change does not merge.\")",
    "problem": "new. This revision dropped the merge-blocking clause. Goals still promises item 5 lands \"conditional on the controls and ship gate named under Read-only git\", and #13 still labels itself \"Item 5's ship gate asserts containment, not just detection\", but the section that names the gate no longer states its consequence, so the condition the Goals row relies on has no stated effect.",
    "fix": "Restore the one sentence at the end of the paragraph: \"If Test strategy #13 lets a vector land a plant, or a runner git call run over one, the change does not merge.\""
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/005-runner-executed-file-moves.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/correctness-r2c2.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design — Read-only git (item 5) — \"Nothing clears it in-process; removing it is the operator's remediation step before the worktree sees another git call\"","problem":"new. The closure of my r2 over-breadth finding replaced \"plant pinned inert\" with the halt, but nothing in the design makes the halt unresumable, and this revision deleted the sentence that admitted that (\"nothing clears it in-process, so a `pipeline resume` (runner.ts:1376) runs over it until the operator removes it\"). Test strategy #6 still files `git-plant-detected` alongside `moves-rollback-failed` as an operator escalation and the Apply section keeps `finalizeEscalate` as the mechanism, so an operator `pipeline resume` over an unresolved plant runs the next finalize's sweep and `git add -A` (runner.ts:298) over a config this same paragraph now says \"survives every pin and is read by every runner git call\" — executing the planted `filter.evil.clean` at runner privilege; the `.gitattributes` selecting that filter is not a `.git` segment, so no later audit re-detects it either. Risks row 1 states the same unenforced ordering as a consequence (\"no escalation commit, so no runner git touches the plant\"), and #13's containment assertion stops at the escalation, so the ship gate the Goals row makes item 5 conditional on never exercises the resume path.","fix":"Name the enforcement in one sentence — \"the escalation leaves the run unresolved: `pipeline resume` runs no stage until the operator records the plant removed\" — and add a #13 case asserting the marker stays absent across a resume over an unresolved plant; or restore the deleted hazard sentence and mark resume-over-plant as the accepted residual in the same row. Either way narrow Risks row 1 to \"so the escalation path itself runs no git over the plant\"."},{"severity":"minor","location":"Chosen design — Read-only git (item 5) — \"`git()`'s env (106-112) is the shared env for every runner git call — `gitOk` (143-145) and the raw `merge-base` spawn (1417-1420) included\" (and Risks row 1, \"the shared env over every runner git call reads no config outside the repo\")","problem":"re-raised. My r2 enumeration finding is untouched by this revision and the changed text re-asserts the universal claim. `writeEditorialDiff` (called from runner.ts:1507) spawns `git diff --no-index` with only `cwd: repoDir` at runner.ts:1933-1937, outside `git()`/`gitOk`, so it inherits the operator environment and does read ambient system/global config at runner privilege — the Risks row's sentence is false as written. It also makes the new sentence \"a planted repo-local `.git/config` survives every pin and is read by every runner git call\" inexact for that same site, which discovers no repository. #13 still enumerates only `gitOk` and `merge-base`.","fix":"State the rule as \"every `spawnSync('git')` site goes through the one env helper\", name `writeEditorialDiff`'s call in the paragraph, and add it to #13's list."},{"severity":"nit","location":"Chosen design — Read-only git (item 5) — closing line \"a planted repo-local `.git/config` survives every pin and is read by every runner git call — hence the halt\"","problem":"re-raised. My r2 nit about the dropped merge-blocking clause is still open, and this revision strengthens the gate's load: Goals still promises item 5 \"conditional on the controls and ship gate named under Read-only git\" and #13 still labels itself \"Item 5's ship gate asserts containment, not just detection\", but the section that names the gate states runtime behavior only, never the consequence of failing it.","fix":"Restore the one sentence at the end of the paragraph: \"If Test strategy #13 lets a vector land a plant, or a runner git call run over one, the change does not merge.\""},{"severity":"nit","location":"Revision log — \"rollback failures escalate; `.git` audit hits halt\" and \"r2/r3: Read-only git — a `.git`-plant hit halts the run with no attempt 2 and no escalation commit\"","problem":"new. `halt` is a new name the body never defines or uses: the audit paragraph still routes the hit through `finalizeEscalate` \"for this class\", and #6 still says a `'git-plant-detected: …'` audit hit \"escalates to the operator and starts no attempt 2\". One failure class now carries two names, and the log's \"halts the run\" reads as terminal while #6 reads as an ordinary operator escalation — the same ambiguity the resume finding turns on.","fix":"Use one term everywhere (\"escalates with no attempt 2 and no commit\"), or define `halt` once in the audit paragraph and use it in #6 and Risks row 1."}]}
```
