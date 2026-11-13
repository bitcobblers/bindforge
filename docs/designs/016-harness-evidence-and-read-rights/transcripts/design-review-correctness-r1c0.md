# Transcript: design-review:correctness r1c0

## Provenance
- stage: design-review:correctness (round r1c0)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T17:48:35.034Z
- ended: 2026-11-13T17:50:27.667Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/016-harness-evidence-and-read-rights/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/016-harness-evidence-and-read-rights.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/016-harness-evidence-and-read-rights.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/016-harness-evidence-and-read-rights/reviews/design/correctness-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chosen design §3, \"Document families\" bullet vs \"Run-record block\" bullet — \"then trailing denies: `*transcripts*` and `*reviews*` over shell, `glob` and `grep` rules over `docs/designs/**/transcripts/**`\" vs the block \"appended last to every shell-holding agent (… the five document families)\" with \"`glob` denies over `docs/designs/**/transcripts/**` and `docs/designs/**/reviews/**`; `grep` denies `*transcripts*` and `*reviews*`\"","problem":"The document families are specified with two different run-record rule sets: the first bullet scopes their glob/grep denies to transcripts/ only, the second appends denies over reviews/** as well. A grep rule keyed to the path-shaped resource docs/designs/**/transcripts/** can never match under the proposal's own A17 semantics (grep resources are the requested regex, not the search path), so it is a dead rule; and if the transcripts-only reading governs, ADR Decision 3's clause \"`glob`/`grep` pattern rules for the same directories\" (both transcripts/ and reviews/) is under-delivered for the document families.","fix":"Make the run-record block the single authority for the doc families' run-record denies — drop the narrower glob/grep clause from the first bullet, keeping its note that the read route to handed reports stays open — so one rule set per agent is specified."},{"severity":"major","location":"Chosen design §3, \"Document families\" bullet — \"(`cd`, `ls`, `cat`, `head`, `tail`, `wc`, `grep`, `rg`, `find`, `git status *`, `git diff *`, `git log *`, `git show *`)\"","problem":"Under the permission grammar the proposal itself relies on (A16/A17; whole-value matching where only a rule ending in ` *` additionally matches the bare command), bare resources like `cat` and `wc` match only argument-less commands, so every useful inspection command (`wc <file>`, `cat <path>`) stays denied behind the kept leading `shell * → deny`. As written the widening buys nothing and defeats Decision 3's purpose (\"so a proposal can check a path and a line count\"). The git entries carry stars; the rest do not — an inconsistency in exactly the rule text an implementer copies.","fix":"Spell every entry in star form (`cat *`, `ls *`, `wc *`, `find *`, `cd *`, …) or state \"byte-identical to the inspection set already shipped in `code/coder.md`\" as the design-panel bullet does; pin with-argument commands (`wc <file>` allowed) in Test strategy #4 instead of bare names."},{"severity":"minor","location":"Chosen design §1, \"Prompts\" bullet — \"the report at `{{reportPath}}` is yours — re-reading it is allowed\" while \"seat agents' `read` deny over `docs/designs/**/reviews/**` (A10) stays\"","problem":"The seat's only reachable re-read route is `git diff`, and for a report written this attempt and not yet committed, `git diff <untracked>` prints nothing; the prompt invites the seat to try the permission-denied `read` tool first and burn a turn on a dead route — the friction class ADR-016 exists to stop.","fix":"One clause naming the route and its audit-level meaning, e.g. \"re-reading it is allowed (via your `git diff` allow — your `read` tool stays fenced off `reviews/`)\"."},{"severity":"minor","location":"Chosen design §4, \"Document ceiling\" bullet — \"above the observed shipped artifacts (this run's `01-research.md` is 440 lines; the 012 proposal ~505)\"","problem":"These run-artifact line counts are stated as fact but appear in no assumption-register row, unlike every other operator-observed number (A19/A20 register that class explicitly); the 600-line cap's headroom rests entirely on them, and a miscount above 600 turns every normal full-flow run red at a document stage.","fix":"Add one unverified register row for the current artifact sizes behind the 600 cap, or drop the specific counts and cite fixture evidence."},{"severity":"nit","location":"Chosen design §3 — \"Run-record block, appended last to every shell-holding agent\" vs next bullet \"Own-report re-open, per seat agent after the deny\"","problem":"The own-report re-open must come after the block for last-match-wins to work, so the block is not actually appended last; the two bullets state contradictory orderings.","fix":"Reword to \"appended after all other rules except the per-seat own-report re-open\"."}]}
```
