# Transcript: design-review:correctness r2c3

## Provenance
- stage: design-review:correctness (round r2c3)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T19:03:15.750Z
- ended: 2026-11-10T19:06:48.912Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 2. This pass's scope: the sections changed since you last reviewed: run `git diff 82b346f031267b92b35257222c125107ae1b6d0c 11f23cea3168a4ce73e9917b8dfeeb34e0b50c9c` (both are blob ids of 02-proposal.md), then read the full current document.
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
    "severity": "major",
    "location": "Chosen design — Read-only git (item 5) — \"Nothing clears it in-process; removing it is the operator's remediation step before the worktree sees another git call\"",
    "problem": "new. The closure of my r2 over-breadth finding replaced \"plant pinned inert\" with the halt, but nothing in the design makes the halt unresumable, and this revision deleted the sentence that admitted that (\"nothing clears it in-process, so a `pipeline resume` (runner.ts:1376) runs over it until the operator removes it\"). Test strategy #6 still files `git-plant-detected` alongside `moves-rollback-failed` as an operator escalation and the Apply section keeps `finalizeEscalate` as the mechanism, so an operator `pipeline resume` over an unresolved plant runs the next finalize's sweep and `git add -A` (runner.ts:298) over a config this same paragraph now says \"survives every pin and is read by every runner git call\" — executing the planted `filter.evil.clean` at runner privilege; the `.gitattributes` selecting that filter is not a `.git` segment, so no later audit re-detects it either. Risks row 1 states the same unenforced ordering as a consequence (\"no escalation commit, so no runner git t… [elided by the checkpoint cap]",
    "fix": "Name the enforcement in one sentence — \"the escalation leaves the run unresolved: `pipeline resume` runs no stage until the operator records the plant removed\" — and add a #13 case asserting the marker stays absent across a resume over an unresolved plant; or restore the deleted hazard sentence and mark resume-over-plant as the accepted residual in the same row. Either way narrow Risks row 1 to \"so the escalation path itself runs no git over the plant\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design — Read-only git (item 5) — \"`git()`'s env (106-112) is the shared env for every runner git call — `gitOk` (143-145) and the raw `merge-base` spawn (1417-1420) included\" (and Risks row 1, \"the shared env over every runner git call reads no config outside the repo\")",
    "problem": "re-raised. My r2 enumeration finding is untouched by this revision and the changed text re-asserts the universal claim. `writeEditorialDiff` (called from runner.ts:1507) spawns `git diff --no-index` with only `cwd: repoDir` at runner.ts:1933-1937, outside `git()`/`gitOk`, so it inherits the operator environment and does read ambient system/global config at runner privilege — the Risks row's sentence is false as written. It also makes the new sentence \"a planted repo-local `.git/config` survives every pin and is read by every runner git call\" inexact for that same site, which discovers no repository. #13 still enumerates only `gitOk` and `merge-base`.",
    "fix": "State the rule as \"every `spawnSync('git')` site goes through the one env helper\", name `writeEditorialDiff`'s call in the paragraph, and add it to #13's list."
  },
  {
    "severity": "nit",
    "location": "Chosen design — Read-only git (item 5) — closing line \"a planted repo-local `.git/config` survives every pin and is read by every runner git call — hence the halt\"",
    "problem": "re-raised. My r2 nit about the dropped merge-blocking clause is still open, and this revision strengthens the gate's load: Goals still promises item 5 \"conditional on the controls and ship gate named under Read-only git\" and #13 still labels itself \"Item 5's ship gate asserts containment, not just detection\", but the section that names the gate states runtime behavior only, never the consequence of failing it.",
    "fix": "Restore the one sentence at the end of the paragraph: \"If Test strategy #13 lets a vector land a plant, or a runner git call run over one, the change does not merge.\""
  },
  {
    "severity": "nit",
    "location": "Revision log — \"rollback failures escalate; `.git` audit hits halt\" and \"r2/r3: Read-only git — a `.git`-plant hit halts the run with no attempt 2 and no escalation commit\"",
    "problem": "new. `halt` is a new name the body never defines or uses: the audit paragraph still routes the hit through `finalizeEscalate` \"for this class\", and #6 still says a `'git-plant-detected: …'` audit hit \"escalates to the operator and starts no attempt 2\". One failure class now carries two names, and the log's \"halts the run\" reads as terminal while #6 reads as an ordinary operator escalation — the same ambiguity the resume finding turns on.",
    "fix": "Use one term everywhere (\"escalates with no attempt 2 and no commit\"), or define `halt` once in the audit paragraph and use it in #6 and Risks row 1."
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
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/correctness-r2c3.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Context — \"This proposal also lands item 5, read-only git in the coder and test-engineer shells\" (and Options 6 — \"**Land `moves` without item 5.** Rejected: the ADR decided item 5\")","problem":"new. The source document's Decision now contradicts this: docs/adrs/005-runner-executed-file-moves.md reads \"**Deferred — read-only git in stage shells.** ... It is deferred to a follow-up ADR. Items 1-4 stand without it\", with the stated reason that the plant-then-runner-git attack surface \"is a separable, non-terminating problem\". No such follow-up ADR exists under docs/adrs/. The changed text keeps the contradiction live: Risks row 1's new accepted residual — \"an operator `pipeline resume` (runner.ts:1376) runs the next stage's sweep and `git add -A` (298) over it until the operator removes it\" — is exactly the sink the ADR names for deferral (\"having the runner's own next git calls execute them, through sinks the ownership sweep can neither see nor undo\"). The proposal asserts the ADR is unconditional (\"the ADR decided item 5\"), and Rollout 1 flips this ADR to accepted while it defers what the merged branch ships, leaving the repo record self-contradictory.","fix":"Choose one in the next revision: amend ADR-005's Decision to re-instate item 5 conditional on the named controls and ship gate (with a revision-log entry recording that decision), or move item 5 into Deferred / out of scope in the ADR's own words and drop the Read-only-git section, the Goals bullet, #13's item-5 half, and item 5's Risks row from this change."},{"severity":"minor","location":"Read-only git (item 5) — \"`git()`'s env (106-112) is the one env helper behind every `spawnSync('git')` site — `gitOk` (143-145), `merge-base` (1417-1420) and `writeEditorialDiff`'s `git diff --no-index` (1933-1937, only `cwd` today)\"","problem":"re-raised. The closure adopts my r2 rule verbatim and adds writeEditorialDiff as asked, but the enumeration is still not exhaustive: the same package has three more `spawnSync('git')` sites — identity.ts:96 (`rev-parse --abbrev-ref HEAD`), identity.ts:104 (`show-ref`), and cli.ts:46 (`rev-parse --show-toplevel`, which passes neither `cwd` nor `env`, so it reads operator system/global config today and runs on every CLI command including the `pipeline resume` the new accepted residual names). Therefore the paragraph's and Risks row 1's claim that this one helper means no `spawnSync('git')` site \"reads no config outside the repo\" is false as written, and #13's gate wording \"every `spawnSync('git')` site ... ignores a hostile `~/.gitconfig`\" cannot be pinned as enumerated.","fix":"Either add the three sites and name the mechanism (export the env helper and route identity.ts's and cli.ts's spawns through it), or scope the claim to \"every `spawnSync('git')` site that runs during a run\" and add the CLI-startup sites to the accepted residual's sentence in Risks row 1, since detectRepo runs unpinned git over an unresolved plant on resume."},{"severity":"nit","location":"Context / Non-goals / Rollout — \"(ADR-005:88)\", \"(ADR-005:84-85)\", \"(ADR-005:87-88)\"","problem":"new. The ADR line cites no longer land on the cited content: the run-004 ordering is now at :93, \"Untouched: review protocol, gates, ownership semantics\" at :91, and dogfood acceptance at :93-94. This is the fingerprint of the deferral paragraph (lines 55-65) being inserted into the ADR after the proposal last cited it — see the major finding.","fix":"Refresh or drop the line numbers when the item-5 scope decision is made."}]}
```
