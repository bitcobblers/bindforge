# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-13T20:15:11.237Z
- ended: 2026-11-13T20:16:55.845Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":36526,"output":18251,"reasoning":0,"cache.read":495316,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/020-restart-boundary-for-harness-self-changes/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 15ba04f5008f2e18b9958fbe24a7727b9f01c441 0b1355604ef1e9dec4bbabc9f7ddd54dd27be710` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/020-restart-boundary-for-harness-self-changes.md,
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
verify what the design claims about the code, and the report at docs/designs/020-restart-boundary-for-harness-self-changes/reviews/design/security-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Chosen design §1 — \"runs `git diff --name-only <headBefore>..HEAD`, filters it through the predicate\" (also Options considered: \"`git diff --name-only <window start>..HEAD` taken once in each finalize window\")",
    "problem": "The canonical detection command omits the flags this repo established as mandatory for path lists: `porcelainPaths`/`nulPaths` (runner.ts:247-264) document that without `-z` paths are quoted/escaped and without `--no-renames` a rename collapses into one pseudo-path. Implemented as written, the prefix predicate silently misses a quoted non-ASCII path (default core.quotePath prints `\"...pr\\303\\266metheus.ts\"`) or a rename-out of the two prefixes, so the boundary fails open — the run drives on with the stale renderer, the exact run-016 class the design exists to close — and Test #7 does not catch it because the missed path is under a named prefix. Every range path is model-authorable (implement/fix own `tools/orchestration/**`, gates.ts:575-582), making crafted evasion in threat model.",
    "fix": "Replace the call in §1 and Options with `git diff -z --no-renames --name-only <headBefore>..HEAD` parsed with the existing `nulPaths` helper — the same posture the sweep's `diff-tree -z --name-only` already uses (runner.ts:308)."
  },
  {
    "severity": "minor",
    "location": "Chosen design §2/§3 — \"gate-logs a `restart-boundary` entry naming the stage and hit paths … mints a terminal … commit whose body names the changed paths … It prints the one-line summary §3 names\"",
    "problem": "The design never states that the model-authorable path strings are sanitized or bounded where written. The established posture is writer-side: `gateLog` prints every reason through `printable` (runner.ts:2006-2023; independence.ts:434-445 names paths as attacker-chosen terminal-spoofing data), and checkpoint-cloned collections are writer-capped (`boundedFindings`, `boundedGateEvidence`; state-store.ts:43). The gate-log entry and the §3 status line inherit those printables, but the printed summary and the boundary-commit body name the paths raw, and `paths: string[]` is unbounded in the record — a crafted control byte in a path spoofs the operator's terminal and the `git log` the operator reads.",
    "fix": "One §2 clause: the record, printed summary, and commit body take each path through `printable`, and the record follows the `boundedGateEvidence` posture (first N paths plus the count §3's status line already prints) — naming those two existing helpers, adding no new mechanism."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/020-restart-boundary-for-harness-self-changes.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/020-restart-boundary-for-harness-self-changes/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design §2 — \"the record keeps the `boundedGateEvidence` bounding (first N paths plus the count)\"","problem":"The bounding posture named in the new clause carries its elision note as an element of the bounded array: `boundedGateEvidence` pushes a synthetic leg `{ command: '(gate legs elided)', … }` (runner.ts:3066-3073) and `boundedFindings` a synthetic finding (runner.ts:3031-3038). Those arrays hold typed objects with a field to carry the label; `paths: string[]` has none, so read literally the clause puts a runner-authored pseudo-path into the same untagged array as the model-authorable untrusted path strings — and that array is exactly what the boundary-commit body and §4's cross-reference read as \"the changed paths\". The same sentence also leaves §3's status line (\"how many harness files it changed\") with no number to print once the array is truncated: the total survives only inside a display string.","fix":"Keep `paths` as paths only — first N printable-copied entries — and carry the pre-bound total as a number on the `restartRequired` object (a field inside a field this change is already adding, so ADR-006 §2 is untouched); the gate-log entry and §3's status line print the count from that number instead of from a pseudo-path entry."},{"severity":"nit","location":"Chosen design §2 — \"every copy of a path takes `printable` into the record, gate-log entry, commit body and summary\"","problem":"`printable` is `value.replace(/\\p{Cc}/gu, ' ')` (independence.ts:443-445): it neutralizes the C0/C1 controls, ESC sequences, newlines and BEL that r1 named, but leaves format characters (`\\p{Cf}` — the U+202A–U+202E directional isolates and overrides, zero-width joiners) untouched, and those are the other way a crafted model-authored filename reorders or hides characters on the operator's terminal and in the `git log` the operator reads for the boundary commit. This is the repo-wide posture rather than something this design introduces, so it should not be fixed here — but the design should not claim terminal-safety beyond what the helper gives.","fix":"No new mechanism: extend the same §2 sentence to say the path copies take printable's Cc-only coverage, with BiDi/format characters out of scope repo-wide, so any later hardening belongs to `printable` itself, not to this boundary."}]}
```
