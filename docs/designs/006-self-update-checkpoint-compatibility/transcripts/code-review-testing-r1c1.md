# Transcript: code-review:testing r1c1

## Provenance
- stage: code-review:testing (round r1c1)
- agent: codereview/code-testing
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T03:07:52.680Z
- ended: 2026-11-11T03:19:09.712Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/006-self-update-checkpoint-compatibility/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 4fc97b316349ed487b5ef49430175c04aabc1e61..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/006-self-update-checkpoint-compatibility.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (testing seat):
[
  {
    "severity": "major",
    "location": "tools/orchestration/tests/pipeline/runner.test.ts:392-401",
    "problem": "The test that is meant to pin ADR-006 §4 (\"two concurrent acquireLock calls admit exactly one winner\") cannot fail under the pre-fix code. acquireLock is synchronous (runner.ts:423, writeFileSync with 'wx'), so Promise.all([tryAcquire(dir), tryAcquire(dir)]) runs the two acquisitions strictly sequentially: the first creates the lock file before the second call starts, and the second is refused by the pid-liveness branch against the winner's own pid. Reverting to the old existsSync+writeFileSync pair keeps the test green, so the atomic exclusive create — §1's single-flight premise — has no coverage, and the test name promises more than the body proves.",
    "fix": "Race two real processes on one temp dir (Promise.all of two execFileAsync children that import acquireLock and exit 0/1; assert exactly one exit 0 and that .pipeline.lock holds that child's pid), or pin O_EXCL in-process: pre-write .pipeline.lock with {slug:'other',pid:process.pid,stage:'start',at:0}, call acquireLock, expect the throw AND readFileSync still byte-identical to the pre-write (a plain writeFileSync would have clobbered it)."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/tests/pipeline/runner.test.ts:370-381",
    "problem": "§4 promises \"a live holder throws with today's message\", but the case pins only not.toMatch(/is not a function/) and /lock/i — an assertion that the seam exists, not that the operator-facing contract holds. Any rewrite of the refusal, including the new race-loser message at runner.ts:462-465, passes it.",
    "fix": "Write the fixture lock as a full LockInfo instead of {pid} and assert the shape: expect(message).toMatch(/^another pipeline run holds the lock: slug=\\S+ stage=\\S+ pid=\\d+$/); reuse the same assertion for the race-loser branch."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/tests/pipeline/ownership.test.ts:239-296",
    "problem": "The §1 core case is untested: a stage editing the checkpoint after the runner saved it in the same process. The suite covers \"no recorded bytes → revert\" (:239) and \"recorded bytes match → skip\" (:286), but never \"recorded bytes exist and differ → revert\". An implementation that skipped on runnerBytes.has(path) rather than comparing content — trusting the map's presence instead of the bytes — would keep every test green.",
    "fix": "Add a case: saveState(...) the advanced state, then writeFileSync a tampered body over the same path, then enforceOwnership(dir, base, 'design-review', advanced); assert report.reverted contains the checkpoint and the file is restored to headBefore (lastStage: 'proposal')."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/tests/pipeline/runner.test.ts:332-352 (and state-store.ts:244,253)",
    "problem": "No test pins §1's promise that resume's marker deletion is not restored from headBefore — the only consumer of recordRunnerDelete/runnerDeleted. In the split-record test the ESCALATION.md marker is written after tmpRepo() commits, so it is untracked: its deletion never appears in dirtyFiles, the sweep never sees it, and holdsRunnerBytes' runnerDeleted.has(abs) && !existsSync(abs) branch is never reached with a real violation. In checkpoint-integration.test.ts:275-310 the marker is committed and the branch does run, but nothing asserts it stays deleted, so removing recordRunnerDelete(markerAbs) from runner.ts would leave the suite green while resurrecting a terminal marker at the next boundary.",
    "fix": "Commit the marker in the fixture before resuming (as an escalation commit does), then after the run assert existsSync(marker) === false and that git status --porcelain still lists ' D docs/designs/002-x/ESCALATION.md'. Alternative unit shape: committed marker → unlinkSync + recordRunnerDelete → enforceOwnership(...).reverted excludes it."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/tests/pipeline/checkpoint-integration.test.ts:275-310",
    "problem": "restoreCommittedCheckpoint (runner.ts:2687-2704, called at :2765) has two branches and only the no-op branch is asserted: escalation.test.ts:212-267 pins the \"nothing committed at that path → planted directory stays\" case, while here the restore branch runs (the checkpoint is in the base commit) but the test immediately does its own rmSync + git checkout (:298-305), so it cannot distinguish runner repair from operator repair and never checks the function's stated promise that planted bytes never reach the escalation commit's git add -A.",
    "fix": "Assert right after expect(status).toBe('escalated'): readFileSync(checkpoint) parses and equals the last runner-authored save; existsSync(join(checkpoint,'planted.md')) === false; git show --name-only HEAD names no reserved path except ESCALATION.md. Keep the operator repair steps only for the second resume leg."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/tests/pipeline/escalation.test.ts:248-250 and tools/orchestration/tests/pipeline/runner.test.ts:348",
    "problem": "The marker's failure line shape is unpinned, and it currently self-duplicates: runner.ts:2743 appends `checkpoint save failed: ${saveFailure.message}` while saveState's message already starts with \"checkpoint save failed:\", so ESCALATION.md — and the terminalSaveFailed gate-log reason that quotes that line — reads \"checkpoint save failed: checkpoint save failed: EISDIR …\". toContain('checkpoint save failed') cannot detect it.",
    "fix": "Pin the line once, e.g. expect(report.match(/checkpoint save failed/g)).toHaveLength(1) or /^checkpoint save failed: (?!checkpoint save failed)/m, and assert the resume gate-log reason equals that exact line (the code-side dedupe belongs to the correctness seat)."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/006-self-update-checkpoint-compatibility/reviews/code/testing-r1c1.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"tools/orchestration/tests/pipeline/runner.test.ts:408-421","problem":"Test strategy #10's third clause (\"two concurrent acquireLock calls admit exactly one winner\") is still pinned only by proxy. The replacement pre-writes a full live-holder LockInfo and asserts refusal plus byte-identity of the holder's bytes; the pre-design existsSync+writeFileSync pair passes this unchanged (the exists-check throws on the live holder before any write, so the bytes survive there too). The test now catches the realistic forward regression — dropping {flag:'wx'} — which closes the core of my r1 major, but cross-process single-flight itself remains untested and the proposal clause has no literal two-acquisition test.","fix":"Optional strictness: race two execFileAsync children that import acquireLock and exit 0/1; under 'wx' exactly one exits 0 deterministically (deterministic assertion for the current code) while the check-then-write pair admits both winners."},{"severity":"minor","location":"tools/orchestration/src/pipeline/runner.ts:536-574 and runner.ts:353-356","problem":"The new boundary re-assertion (ensureLockHeld, security r1 #4) has four branches — self-held return, foreign-live-pid refuse, missing/dead re-take, race-loser refuse — and no test reaches any throw, yet docs/workflows.md promises operators \"if another live process has taken it, the run refuses to continue.\" Likewise stageAll's lock-unstage runs only incidentally (escalation.test.ts/budget.test.ts fixtures do not gitignore the lock) and is never asserted; checkpoint-integration.test.ts:83-85 gitignores the lock, so its committed-file scan cannot catch removal of the reset at runner.ts:355 committing the run's lock.","fix":"Add (a) a resume-integration case that rewrites .pipeline.lock to a live foreign pid (process.ppid) before resuming and expects the run to end on the \"lost the pipeline lock at a stage boundary\" refusal rather than complete, and (b) in a non-gitignored full-run fixture, assert no commit on the branch names .pipeline.lock (scan git log --all --name-only)."},{"severity":"minor","location":"tools/orchestration/src/pipeline/runner.ts:2782-2792","problem":"The new finalizeDone guard converting a terminal CheckpointWriteFailure into restoreCommittedCheckpoint + finalizeEscalate (security r1 #3) has no test: every directory-at-path scenario in the suite sabotages mid-run (checkpoint-integration.test.ts:289-295, exercising the drive-loop guard), never the done boundary. Removing the try/catch would crash the completion path with no terminal record and leave the suite green — the exact run-2 class this design closes (checklist item 7: untested new branch).","fix":"Add a checkpoint-integration variant whose sabotage client replaces the checkpoint path at the final boundary, asserting status 'escalated', an ESCALATION.md, and a recorded terminal commit instead of an escape."},{"severity":"nit","location":"tools/orchestration/tests/pipeline/ownership.test.ts:390-405","problem":"The marker-delete unit case never recordRunnerWrite's the marker's bytes before rmSync + recordRunnerDelete, so it pins only the plain delete-skip; the delete-outranks-write ordering in holdsRunnerBytes (state-store.ts:290-303) is pinned today only through the checkpoint-integration second-resume leg.","fix":"Call recordRunnerWrite(marker, body) when planting the fixture marker so the ordering fix is pinned at unit level too."}]}
```
