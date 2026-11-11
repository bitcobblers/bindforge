# Transcript: design-review:performance r1c0

## Provenance
- stage: design-review:performance (round r1c0)
- agent: review/design-performance
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:09:06.015Z
- ended: 2026-11-10T17:12:22.791Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
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
re-open one only if it contradicts docs/adrs/005-runner-executed-file-moves.md.
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/performance-r1c0.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Validator, rule 4 — \"`from` is a tracked **file** at `headBefore` (`git cat-file -t <headBefore>:<from>` equals `blob` — this is the file-only rule)\" [new]","problem":"The validator is specified as one git subprocess per endpoint: `git cat-file` per `from` (rule 4) plus a second headBefore membership query per `to` (rule 5), plus `existsSync`/`realpath` per pair. Run 004's motivating payload (23 pairs, A12) is ~46 subprocess spawns per validation pass, and the pass re-runs on every nudge cycle and again in attempt 2 (Wiring into retry), so a list needing three nudges across two attempts costs several hundred spawns. It sits in finalize/reply validation, not on a keystroke path, so this is minor rather than major, but the design never acknowledges the per-pair spawn cost and leaves the obvious batching unspecified.","fix":"Read headBefore once per validation pass instead of per pair: `git cat-file --batch` fed all `<headBefore>:<path>` endpoint queries through one process, or one `git ls-tree -r --name-only <headBefore>` snapshot held in a Set. This replaces the per-pair `git cat-file -t` in rule 4 and the per-`to` tracked lookup in rule 5 — same answers, one spawn per pass."},{"severity":"minor","location":"Chosen design → Reply field + Validator rule 6 — \"`moves: z.array(MovePairSchema).default([])`\" and \"no listed path is a path-prefix of another\" [new]","problem":"The list length is unbounded — nothing caps the array, so n is model-controlled — and the prefix rule as written is naturally a pairwise scan, i.e. O(n²) in n for the disjointness/containment bookkeeping. Nothing bounds the blast radius of a pathological list (thousands of pairs) beyond the reply's own token size.","fix":"Bound it at the schema: `z.array(MovePairSchema).max(N).default([])` with N stated (a few hundred comfortably covers run 004's 23), and make rule 6 non-quadratic by sorting the normalized paths once and comparing each path only against its sorted neighbours, replacing the pairwise scan rather than adding to it."},{"severity":"minor","location":"Compatibility matrix → \"Repository spanning mount points\" row — \"`rename(2)` fails `EXDEV` (A8): the pair fails at apply, the list rolls back, the stage fails with `moves-apply-failed`\" [new]","problem":"A statically-detectable condition is routed through the most expensive failure path in the design. `moves-apply-failed` deliberately drops the `invalid-reply` prefix (Apply: \"so attempt 2 starts a fresh session\"), so a cross-mount pair costs a full rollback of up to n−1 renames, then a fresh no-context session that re-derives the stage's whole reply, then escalation — and a fresh inference session is the dominant cost in this system. Rule 7 already realpaths `from` and `to`'s nearest existing ancestor, so the condition is knowable at validation for free.","fix":"Extend rule 7 to compare `statSync(from).dev` against the `dev` of `to`'s nearest existing ancestor and fail as `invalid-reply: moves` (the cheap nudge path); keep the apply-time `EXDEV` handling as the race guard rather than the first line of detection."},{"severity":"minor","location":"Wiring into retry / Rollout 4 — \"The in-attempt nudge loop is the cheap correction path\" [new]","problem":"The document's cost claims are asserted, never quantified: \"cheap\" has no bound attached (the nudge cap it leans on is existing behavior cited as A2 but never numbered), and no section states the expected added finalize cost at the run-004 baseline size. Rollout 4's dogfood criteria are outcome-only (\"no `operatorForce`/…\", \"land as `R` entries\"), so nothing would notice if the new path made finalize slower — e.g. if the per-pair spawn cost scaled with a larger move set. There is no latency/cost budget line in the document, so this checklist item could not be evaluated.","fix":"One sentence in Apply or Rollout 1 stating the per-stage cost model after the batching fix — one headBefore read + n `renameSync` calls + n stat checks, target under a second for a run-004-sized 23-pair list — plus the existing max-nudges-per-attempt number that bounds \"cheap\". This does not re-open the budget-burn decision closed at r0 OQ6: one code-budget unit per full failed attempt stands; only its bound is unstated."},{"severity":"nit","location":"Chosen design → Read-only git — \"`git status --no-optional-locks*` … `git log*`, `git diff*`, `git show*`\" [new]","problem":"The only cost-pinned form is `status`; the other three are granted in unbounded spellings, and `git log -p` / `git log --follow` over full history is the most output- and CPU-hungry invocation available inside a stage turn, landing straight in agent context. Precedent exists (review seats already run the same three of four), so this is preference, not a defect.","fix":"Optionally mirror the `status` treatment with a one-line prompt/config note that log/diff calls be bounded (`-n`, path-scoped), the same way `--no-optional-locks` was pinned for cost reasons."}]}
```
