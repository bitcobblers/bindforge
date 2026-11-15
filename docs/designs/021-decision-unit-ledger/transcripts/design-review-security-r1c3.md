# Transcript: design-review:security r1c3

## Provenance
- stage: design-review:security (round r1c3)
- agent: review/design-security
- model: strata/qwen3.8-flash-next-nvfp4-nvfp4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-14T22:42:03.646Z
- ended: 2026-11-14T22:51:05.042Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":49945,"output":29974,"reasoning":0,"cache.read":763551,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/021-decision-unit-ledger/02-proposal.md.
This is the ratification pass c3: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 67e7b5740b46b427129312d4aa93e403e2ef5d2e e66564c0d7fdfd8c438ebf4e1c66d9f3ddd1644a` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/021-decision-unit-ledger.md,
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
verify what the design claims about the code, and the report at docs/designs/021-decision-unit-ledger/reviews/design/security-r1c3.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "minor",
    "location": "Chosen design ¶1/¶7 — \"`ledger.ts` reads files with `node:fs`, never writes, and parses no path it did not list itself\"",
    "problem": "The gate's fail path is specified only for files that parse wrong (sixteen codes → deny → escalate). A corpus file that cannot be read or decoded at all (broken symlink, permission error, non-UTF-8 bytes in a .md) has no specified handling; an exception escaping checkLedgers crashes runPipeline at the entry hook with no gate-log record and no finalizeEscalate, and crashes the completion path inside finalizeDone — violating security checklist item 7's \"reject cleanly, never crash\", and the class the harness already legislated for itself (runner.ts:2430–2431 and 3590–3593 escalate-with-record posture; sibling sourceAdmissible already converts its own read failure into a clean deny at gates.ts:103–107).",
    "fix": "Add one sentence to ¶7: a corpus read or decode failure makes ledgerGate deny with `ledger corpus not readable: <file>`, the way sourceAdmissible already turns its read failure into deny(\"source document not readable: …\"), so both hooks route through the same escalate path a finding takes; no new taxonomy code needed."
  },
  {
    "severity": "minor",
    "location": "Chosen design ¶4 — \"Handles in a fenced block (lines whose trimmed start is ``` or `~~~`, toggled) … are not handles\" (the \"fence toggle, no parser\" middle path)",
    "problem": "The toggle flips on any fence-looking line and nothing validates the toggle state at end of file, so a single stray fence marker — malformed input any document stage can emit — silently suppresses handle detection for the remainder of the file: a dangling handle past the stray marker then joins green instead of red. The check applies to only part of the file (item 7's \"never … partially apply\"), i.e. fails open, and the design's own Risks table names silent under-coverage as the defect to close without mechanically detecting its cheapest instance.",
    "fix": "¶7 adds `fence-unbalanced` to the code list: a file whose fence state is still open at EOF is a finding, turning the stray marker red instead of mute; pin with one fixture in Test strategy #2."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/021-decision-unit-ledger.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/021-decision-unit-ledger/reviews/design/security-r1c3.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design ¶1/¶7 — \"`ledger.ts` reads with `node:fs`, builds every path with `node:path` …, never writes, and parses no path it did not list\"","problem":"re-raised, and widened by this revision. ¶7 still specifies a fail path only for files that parse wrong: a corpus file that cannot be read or decoded at all (broken symlink, permission error, non-UTF-8 bytes in a .md) has no handling, so the escaping exception reaches runPipeline beside sourceAdmissible with no `ledger-join` gate log and no finalizeEscalate, and crashes the completion path inside finalizeDone — the class the harness legislated for itself (runner.ts:2430–2431, 3590–3593, \"escalation with a record, not a crash\") and the one sourceAdmissible already converts into a clean deny (gates.ts:105–107). The revision added a second silent route through the same class: ¶1's `frontmatterOf(content): { front; body } | undefined` returns undefined for a file whose `---` block does not match, and revised ¶6 now states that an absent key means \"no unit, pointer, owner or kind check\" and that depends-order binds only \"every ADR that carries `depends-on`\" — so a ledger-bearing ADR with a malformed frontmatter block reads as not ledger-bearing and neither its units nor its depends-on are checked, with no finding in any code. Security checklist item 7 (reject cleanly, never crash or partially apply).","fix":"One sentence in ¶7, no new taxonomy code: a corpus read or decode failure, or a `docs/adrs/NNN-*.md` whose frontmatter block frontmatterOf returns undefined for, is denied as `ledger corpus not readable: <file>` — the way sourceAdmissible turns its own read failure into deny(\"source document not readable: …\") — so both hooks route through the escalate path a finding already takes."},{"severity":"minor","location":"Chosen design ¶4 — \"Handles in a fenced block (lines whose trimmed start is ``` ``` or `~~~`, toggled) … are not handles\"","problem":"re-raised: the toggle and its missing end-of-file validation are untouched by this diff, so a single stray fence marker suppresses handle detection to the end of the file and a dangling handle past it joins green — a check that partially applies. What the diff changed is the blast radius: revised ¶6 (\"absent means no unit, pointer, owner or kind check … while ¶4–¶5's handle rule binds every corpus file\") and the rewritten compatibility row (\"it does not exempt it from ¶4–¶5's handle and note rule\") now make the handle rule explicitly load-bearing on every file in docs/adrs/ and schemas/, including the ledger-less ADRs 001–016 and 020 that last round's wording left ambiguous. This is the silent-under-coverage defect the Risks table names, now on a wider file set.","fix":"Unchanged from r1: add `fence-unbalanced` to ¶7's code list — a file whose fence state is still open at EOF is a finding, turning the stray marker red instead of mute — and pin it with one fixture in Test strategy #2."},{"severity":"minor","location":"Chosen design ¶2 — \"The pointer is the last `→` outside a code span and must be followed by non-space text\"","problem":"new in this diff. The sentence introduces a third scanning state the design never defines: ¶4 defines exactly two non-prose regions (fence toggle, frontmatter block), and inline code-span state gets no toggling rule, no per-line reset and no end-of-input or unbalanced case — the same shape as the ¶4 toggle in my open finding. On malformed input the outcomes split: an unmatched backtick before the real pointer yields pointer-missing (fail closed, and Test #10's green-corpus leg would surface it), while an unmatched backtick before a code-span arrow — the corpus quotes glyphs in spans, 021-C1 itself quotes `→` per A2 — lets an arrow inside a span be taken as the pointer. Since targets are presence-checked and never resolved (Deferred), that unit is then green with a pointer no reader would accept, and nothing reports it. Checklist item 7: malformed input silently changes which text the check applies to.","fix":"One clause in ¶2: code-span state for the pointer is computed per line and resets at the line terminator, and a bullet with an unbalanced number of span delimiters is `unit-malformed` — reusing a code ¶7 already has; the same clause can settle whether a backticked target (`→ \\`ADR-019\\``) counts."},{"severity":"nit","location":"Chosen design ¶7 — \"`ledgerGate` denies with at most `MAX_LEDGER_FINDINGS` findings — an exported constant, the way `MAX_GATE_LEGS` bounds a gate record\"","problem":"the cap bounds the count, and this revision is where the design leans on it hardest, but each line is `file:line code detail`, so per-line size is whatever the corpus line contributes; Test strategy #9 also dropped old #10's clause \"so no reason string is unbounded\". The harness's own findings cap does both jobs — boundedFindings slices to MAX_RECORDED_FINDINGS and runs every field through boundedField/printable (runner.ts:3206–3232) — because, per gates.ts:300–307, one gate record is cloned into every later checkpoint and rendered into every later prompt. Checklist item 4's logging half.","fix":"One clause: each `file:line code detail` line passes through the existing printable/boundedField posture (which also normalizes the non-UTF-8 case), or ¶7 states that `detail` never quotes corpus text, only coordinates and code."}]}
```
