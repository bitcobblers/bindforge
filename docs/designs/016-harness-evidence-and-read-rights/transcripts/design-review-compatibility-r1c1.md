# Transcript: design-review:compatibility r1c1

## Provenance
- stage: design-review:compatibility (round r1c1)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T18:02:30.186Z
- ended: 2026-11-13T18:04:00.665Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/016-harness-evidence-and-read-rights/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 216021595de4065afe3b38fbe4ce01559741000c 9b6b5edfbd2d3eec658a6013881dbe38d5c28a84` (both are blob ids of 02-proposal.md), then read the full current document.
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
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "Compatibility matrix — \"| macOS | unknown | Nothing platform-specific is added (pipe capture and `spawn` are portable Node), but no environment here exercises it |\" and \"| Windows / WSL2 | unknown | Same posture as macOS |\"",
    "problem": "The checklist treats an `unknown` matrix status as a major finding, and the rows' justification is contradicted by §3: doc families gain a POSIX-only inspection set (\"append the inspection set (`cd`, `ls`, `cat`, `head`, `tail`, `wc`, `grep`, `rg`, `find`, …)\") and the design panel gains spawned cargo/rustfmt/npm legs. Both are platform-dependent tool surfaces, so \"nothing platform-specific is added\" overstates portability instead of justifying the abstraction.",
    "fix": "Replace `unknown` in both rows with a declared posture (\"not exercised — harness change developed and tested on Linux only\") and add one sentence naming the two platform-dependent additions (POSIX inspection set, cargo/npm check legs) as the reason no claim is made. Document-only."
  },
  {
    "severity": "minor",
    "location": "§1 Own-report identity — \"a `reviews/` access value is the seat's own report iff `reportKey(value)` (everything from the `reviews` segment on) equals `reportKey(opts.ownReportPath)`\"",
    "problem": "The identity rule, §3's `glob` denies over `docs/designs/**/reviews/**`, and the per-seat shell allow all key on `/`-separated, case-exact path spelling, and no assumption row (A2, A16, A17) states that access values or shell command text reach the audit separator-normalized. Where paths are spelled with `\\` or differ in case, the deny side fails safe but the own-report exemption silently never fires, so Decision 1's promise is unmet on that host while the matrix claims nothing platform-specific changes.",
    "fix": "Add one clause to the identity rule: `reportKey` compares after normalizing separators to `/` and case — or, if the SDK already normalizes, add an assumption row stating that with where it was verified."
  },
  {
    "severity": "minor",
    "location": "§3 Read-only reach — \"`shell git diff *reviews/<phase>/<specialty>-*` → allow, naming the agent's own specialty in the pattern (A11)\"",
    "problem": "The deny-then-reallow order depends entirely on OpenCode's matcher grammar (`*` spans `/`, a rule ending ` *` matches the bare command, last match wins), and A16 says the pin is \"a repo-local re-implementation of OpenCode's wildcard matching\". The matrix row \"Pre-ADR-016 OpenCode server config | yes | Only standard rule types … are used\" vouches for rule types, not for that grammar, so drift in the engine silently inverts the whole §3 deny stack.",
    "fix": "Name the OpenCode version whose documented grammar the rules rely on in one assumption row and in that matrix row's note, and state in Test strategy #4 that the repo-local matcher stays pinned to that version."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix — \"| A run in flight across the merge | yes | Pre-016 checkpoint ⇒ `gateEvidence` absent ⇒ prompts render \\\"(no gate record for this run yet)\\\" |\"",
    "problem": "The row claims a flat `yes` but accounts only for the checkpoint and transcript effects. The same merge applies `docMaxLines: 600` and `research` in `docStages` to a resumed run, and the Risks table itself says \"A resumed or in-flight run meets the new `docMaxLines`: a document already over 600 fails its next document stage\". The matrix understates a behavior change across the merge.",
    "fix": "Extend that row with the condition: \"…; a document already over `docMaxLines` fails its next document stage — raise the cap or force the stage (Risks)\", keeping the status."
  },
  {
    "severity": "minor",
    "location": "§2 The gate record — \"`tail` the last `MAX_GATE_TAIL_BYTES = 2_000` bytes of the leg's merged stdout+stderr, passed through `printable` (independence.ts:311) before storage\"",
    "problem": "The bound is in bytes over raw child output, so the slice can cut a multi-byte code point, and cargo/npm output under a non-UTF-8 console codepage (the Windows row) is not UTF-8 at all; that string is persisted in the checkpoint and rendered into four prompts and the build note. The Risks row about the rolling tail covers only truncation of the `test result:` line, so \"tails are evidence, not parsed contracts\" does not cover mojibake reaching five seats.",
    "fix": "One clause: capture bytes, decode once as UTF-8 with replacement, then bound the tail in characters against the same `2_000` budget — or state explicitly that a tail's head may drop a partial code point and who strips it."
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix, Linux row — \"all changes are Node ≥ 22.23.4 in `tools/orchestration`, agent-config YAML text, and prompt text\"",
    "problem": "The Node floor is asserted with no anchor (no engines reference, no assumption row), so the checklist's Node-floor check cannot be verified from the document; and §3 edits `.md` agent rule files (\"Config edits in `.opencode/agents/`, using the rule text already shipped in `code/coder.md`\"), not \"YAML\".",
    "fix": "Cite the declared floor's source (the package's engines field) and write \"agent rule text in `.opencode/agents/`\" instead of \"agent-config YAML text\"."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/016-harness-evidence-and-read-rights.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/016-harness-evidence-and-read-rights/reviews/design/compatibility-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design §3 Read-only reach — \"then the inspection set byte-for-byte as `code/coder.md` ships it — star form throughout\"","problem":"new: The inherited read-only set was enumerated inside the document in r0 (matching ADR-016 Decision 3's named commands); the revision defines it only by reference to code/coder.md. ADR Decision 3 names four read commands (git status, git diff, git log, git show) besides the file readers, and Test strategy #4 simultaneously narrowed its pin from r0's \"doc families allow `cat`/`wc`/`git status`\" to \"doc families allow `wc <file>` and `cat <path>`\" — so neither the document nor its tests pin that the referenced coder.md set contains the ADR's git read commands, and drift in coder.md would silently shrink the doc families' ADR-mandated reach; the matrix's own macOS/Windows rows cite \"§3's inspection set\" as the host-dependent surface, which is now un-enumerable from the document. Fix replaces the dropped r0 enumeration rather than adding a mechanism.","fix":"One clause (or one assumption row with coder.md line refs, as A8 provides) stating coder.md's inspection set is exactly ADR-016 Decision 3's read-only set in star form, and extend Test #4's pin to the `git status *`/`git diff *`/`git log *`/`git show *` allows for doc families."}]}
```
