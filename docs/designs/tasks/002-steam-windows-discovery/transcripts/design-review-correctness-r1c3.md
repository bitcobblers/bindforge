# Transcript: design-review:correctness r1c3

## Provenance
- stage: design-review:correctness (round r1c3)
- agent: review/design-correctness
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T06:04:08.564Z
- ended: 2026-11-18T06:07:45.188Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":23142,"output":11843,"reasoning":0,"cache.read":152542,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/tasks/002-steam-windows-discovery/02-proposal.md.
This is the ratification pass c3: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff b12a4dd76acc0cfadddc1198d7e0c3e0e3747419 bc928d9c11700e2c863c4b86421761f1b4d0710a` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/tasks/T-002-steam-windows-discovery.md,
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
verify what the design claims about the code, and the report at docs/designs/tasks/002-steam-windows-discovery/reviews/design/correctness-r1c3.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "§5 Root candidates — \"A relative value is ignored by the same `absolute_env` helper Linux uses, shared per §1\" read with Test strategy #3 (re-raised from r0; the sentence was changed but the gap remains)",
    "problem": "The env-candidate gate's predicate is `PathBuf::is_absolute` (crates/discovery/src/steam_linux.rs:341), host-dependent as §6 itself states. `host_candidates` is compiled and unit-tested on Linux, so the fake can only pass POSIX-absolute values, and the Windows branch of the rule — a `C:\\Program Files (x86)`-shaped value kept, a Windows-relative one dropped — is exercised on no host. §5 still does not state what values the fake supplies. Runtime behavior is correct on each host; this is a verification gap for a rule the design depends on, unchanged by the revision.",
    "fix": "In §5, add one sentence stating that the #3 fake supplies POSIX-absolute values and that the Windows-absolute case is covered only by gates #12/#13; or generalize the absoluteness predicate over `SteamTarget` next to `install_dir_rejects_backslash` (§1's existing mechanism) and add one Test strategy #3 case pinning what a Windows-shaped value does on each host."
  },
  {
    "severity": "minor",
    "location": "Test strategy #2 — \"`foo\\bar`, the doubled form `foo\\\\bar` as a real Windows ACF writes it (A2)\"",
    "problem": "The r1 revision split the doubled-backslash spelling out as A1b, marked `unverified` with \"the design assumes no spelling\" — yet this body sentence still asserts the doubled spelling as what real Windows ACFs write, and cites A2, which is the reader's verified `\\`/`\"` unescape behavior (keyvalues.rs:186-207), not a source on what ACFs write. The changed register makes this sentence unsupported; the test case itself stays valid either way, so it is a factual/language defect, not behavioral.",
    "fix": "Reword to \"the doubled form a Windows writer may emit (A1b, spelling unverified)\", leaving the enumerated test cases unchanged."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/tasks/T-002-steam-windows-discovery.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/tasks/002-steam-windows-discovery/reviews/design/correctness-r1c3.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"§5 Root candidates — \"a relative value is ignored by `absolute_env` (§1)\" read with Test strategy #3 (re-raised; unchanged by this revision)","problem":"The env-candidate gate's predicate is `PathBuf::is_absolute` (crates/discovery/src/steam_linux.rs:338-341), host-dependent as §6 states, and `host_candidates` is unit-tested only on Linux, so the fake can supply only POSIX-absolute values; the Windows half of the rule (a `C:\\…`-shaped value kept, a Windows-relative one dropped) is exercised on no host. The revision rewrote the sentence without closing the gap, and the Revision log records no decision on it. This is a persistent verification gap, not a break introduced by the revisions; runtime behavior is correct on each host.","fix":"Add one sentence to §5 (or #3) stating the #3 fake supplies POSIX-absolute values and Windows-absolute coverage sits with gates #12/#13; or generalize the absoluteness predicate over `SteamTarget` beside `install_dir_rejects_backslash` and add one #3 case pinning a Windows-shaped value's outcome per host."},{"severity":"minor","location":"Test strategy #2 — \"the doubled form `foo\\\\bar` as a real Windows ACF writes it (A2)\" (re-raised; text untouched by this diff)","problem":"The sentence still asserts as fact the doubled-backslash spelling that A1b marks `unverified` with \"the design assumes no spelling\", and cites A2, which is the `\\`/`\"` unescape behavior (keyvalues.rs:186-207), not a source on what ACFs write. The register-side closure in the revision log did not propagate to this body sentence, leaving register and body in contradiction. The test case itself remains valid under any spelling.","fix":"Reword to \"the doubled form a Windows writer may emit (A1b, spelling unverified)\", leaving the enumerated test cases unchanged."},{"severity":"nit","location":"Test strategy #3 (changed text) — \"both views answering (one path from both → one root after the canonical-path rule, distinct paths → two roots and rows from two installs)\"","problem":"The named sub-outcomes are produced by `from_registry_and_roots`' filesystem pass (§3), not by the pure `host_candidates` candidate list this item's header (\"pure, fake registry + fake env\") scopes; \"rows from two installs\" additionally needs two fixture trees that no test item owns.","fix":"Scope #3 to the candidate list (\"two distinct candidates, ordered\") and attach the rows-from-two-installs outcome to #4's `from_registry_and_roots` harness, or drop it."}]}
```
