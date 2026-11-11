# Transcript: code-review:compatibility r1c2

## Provenance
- stage: code-review:compatibility (round r1c2)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T03:19:09.804Z
- ended: 2026-11-11T03:25:33.131Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/006-self-update-checkpoint-compatibility/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 4fc97b316349ed487b5ef49430175c04aabc1e61..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/006-self-update-checkpoint-compatibility.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/types.ts:181 (with tools/orchestration/src/pipeline/state-store.ts:277)",
    "problem": "The fence's tmp-shape matcher STATE_TMP_RE hard-codes the checkpoint literal, the 8-hex suffix width and '.tmp' as a second, independent copy of the naming rule that lives in saveState (`${target}.${randomBytes(4).toString('hex')}.tmp`). Nothing links them: if the writer's tmp name ever changes, saveState keeps working while the fence silently stops matching the runner's own tmp files, so stray checkpoint tmps reach `git add -A` instead of being swept. Tests cannot catch the drift because ownership.test.ts:245 plants a hand-written 8-hex name that matches the literal regardless of what saveState writes.",
    "fix": "Define the shape once: export a stateTmpName(target) helper from state-store.ts, or build STATE_TMP_RE from STATE_FILE, and add a case that takes a tmp name actually produced by saveState and asserts namesReservedSegment denies it."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/state-store.ts:149 and state-store.ts:293 (consumed at runner.ts:2445 and runner.ts:2743)",
    "problem": "The loadState and CheckpointWriteFailure repair messages interpolate `file`/`target`, which are absolute whenever the runner passes an absolute design dir (it always does). That text is not only printed: it becomes the escalation reason persisted in the gate log and re-saved by the next resume, and it is appended verbatim to ESCALATION.md. The run's terminal record therefore carries one machine's worktree path — the run-3-live fixture (line 408) shows exactly this shape — while ADR-006's own scenario is a resume after a self-update, possibly in a different worktree. finalizeEscalate already formats the same repair repo-relative (runner.ts:2758-2763), so the store-side messages are the inconsistent half.",
    "fix": "Keep the path structured rather than pre-formatted: put `path` on CheckpointWriteFailure, render loadState's message from the design-dir-relative form, and let the printers emit repo-relative commands as runner.ts:2761 already does."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/tests/pipeline/runner.test.ts:392-403 (claim at runner.ts:423 and the proposal's Compatibility matrix network-FS row)",
    "problem": "Test #10's concurrency case does not exercise the platform claim §4 makes. acquireLock is fully synchronous, so Promise.all of two in-process calls simply serializes; the test passes unchanged against the deleted existsSync+writeFileSync code and pins nothing about 'wx'. The real local-FS window is still open and untested: a contender that reads the lock between the exclusive create and the write gets an empty file, holderOf() returns null, the code warns and unlinkSyncs the live holder's lock, then re-creates — two winners on a local filesystem, while the matrix attributes all residual risk to network/shared filesystems.",
    "fix": "Make #10's third case contend two real processes (spawn/execFileSync of process.execPath running acquireLock against the same fixture repo) and assert exactly one succeeds; and either narrow the docstring/matrix claim to the create itself or state the empty/unparseable-lock window alongside the deferred lock-ID nonce."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/types.ts:183-189 (precedent cited at moves.ts:136-142; matrix macOS row)",
    "problem": "The code comment (and the proposal's §1 citation) claims the reserved-name comparison is case-insensitive 'the way the moves validator already compares these names'. moves.ts compares STATE_FILE and LOCK_FILE case-sensitively — only .git is lower-cased. Two reserved-name sets now diverge in both directions (namesReservedSegment adds ESCALATION.md and the tmp shape, moves.ts keeps LOCK_FILE which the fence omits) while a comment asserts parity. Separately, the Compatibility matrix justifies macOS 'partial' purely on rename/'wx' POSIX semantics and never flags that its default filesystem is case-insensitive — the assumption this case-based anti-evasion rule rests on, and the one the repo already models at moves.test.ts:675-687 — even though the revert branch resolves case through git's case-sensitive tree lookup (runner.ts:294-299).",
    "fix": "Correct the comment; decide the reserved-name set once (adding LOCK_FILE to namesReservedSegment is free for the sweep, since .pipeline.lock is gitignored) or document the divergence; and add a case-insensitive-checkout row to the matrix, with a runIf(process.platform === 'darwin') companion case following the moves.test.ts pattern."
  },
  {
    "severity": "nit",
    "location": "docs/workflows.md (new 'A state file from another pipeline version' section) vs tools/orchestration/src/pipeline/state-store.ts:176-186",
    "problem": "The operator doc says the refusal message 'says to run the version that wrote it, or to restore the checkpoint or start a fresh run'. The message actually printed names only the supported version, the committed-copy restore and a fresh run; the 'run the version that wrote it' posture is absent, and it also sits oddly against §2's guarantee that any runner resumes any version 1 checkpoint.",
    "fix": "Reword the doc to the two postures §2 names, or print the third posture in the refusal message."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/006-self-update-checkpoint-compatibility/reviews/code/compatibility-r1c2.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict":"approve","findings":[{"severity":"minor","location":"tools/orchestration/src/pipeline/runner.ts:2880 (with tools/orchestration/src/pipeline/types.ts:181-208 and tools/orchestration/tests/pipeline/gates.test.ts:352-366)","problem":"The revised finalizeEscalate writes ESCALATION.md through a new runner tmp shape, ESCALATION.md.<8hex>.tmp, which is not covered by the reserved-name fence: namesReservedSegment matches ESCALATION.md exactly and STATE_TMP_RE only the .pipeline.json.<8hex>.tmp prefix. Residue from a kill between the tmp writeFileSync and the renameSync — exactly ADR-006's kill-mid-escalation scenario resumed later on the new branch — survives every boundary sweep and is committed by the next git add -A, and a stage can plant an ESCALATION.md.<hex>.tmp copycat under docs/** that likewise reaches a run commit. This breaks the §1 'keeping git add -A clean' promise the fence was approved on, for the escalation marker's own tmp, and is a third independent copy of the tmp-suffix naming rule (after saveState and STATE_TMP_RE).","fix":"Define the tmp shape once: make a segment reserved when it matches `<reserved-name>.<8hex>.tmp` for any reserved name (STATE_FILE or ESCALATION_FILE), built from the same constants the writers use, so STATE_TMP_RE and the marker tmp become instances of one grammar; add the ESCALATION.md.<hex>.tmp shape to gates.test.ts RESERVED_PATHS plus one sweep case."}]}
```
