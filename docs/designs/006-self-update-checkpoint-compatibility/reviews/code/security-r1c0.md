# Code review — security seat, round 1, pass c0

Scope: the pipeline-branch change (range `846ca090053b…HEAD`, `docs/designs/` excluded)
judged against `02-proposal.md` and ADR-006, security checklist only.

Method note: the shell tool is not available in this seat, so the literal
`git diff` could not be produced. The review read the changed paths directly
(gates.ts, types.ts, state-store.ts, runner.ts, cli.ts, moves.ts, and the new
tests/fixtures), treating the §1–§4 mechanisms the proposal enumerates as the
change surface. Findings are anchored to code this design introduces or
rewrites.

## Checklist evaluation

1. **External input validated at the boundary** — evaluated. `loadState`
   schema-validates every checkpoint (`StateSchema.safeParse`), refuses
   unknown `version` before parsing deep, never evaluates content, and
   refusal is read-only (state-store.ts:133–194; test #4 pins byte-identical
   refusal). No size bound on the raw read — minor finding below.
2. **Path handling** — evaluated, clean. `namesReservedSegment`
   (types.ts:190–202) matches whole `/`-separated segments, case-insensitive,
   at any depth; `.pipeline.json-notes.md` neighbors stay allowed (gates.test
   pins both directions). Repair strings are *printed* for the operator,
   never executed, and the paths embedded in them derive from schema-validated
   `nnn`/`slug`. All git access is array-arg `spawnSync` — no interpolated
   shell.
3. **Injection** — evaluated, clean. `STATE_TMP_RE` is anchored with a bounded
   character class (no catastrophic backtracking); no user string enters a
   regex constructor in the diff. `printable()` sanitizes model-authored text
   into the gate log, terminal, and `ESCALATION.md`.
4. **Secrets/PII** — evaluated, clean. The three golden fixtures
   (`run-1-shape.json`, `run-2-era.json`, `run-3-live.json`) are synthetic
   ids/summaries; test repos use placeholder identity (`f@f`).
5. **New dependencies** — evaluated, none. The reviewed code uses only
   built-ins (`node:fs`, `node:crypto` `randomBytes`, the `'wx'` flag) plus
   the pre-existing zod/yargs/@opencode/client trio; engine floor unchanged
   (A11).
6. **Filesystem writes atomic/least-privilege** — `saveState` tmp+rename and
   `'wx'` lock create are right (state-store.ts:266–299, runner.ts:423–467).
   The `ESCALATION.md` marker write breaks the rule — findings below.
7. **Clean rejection of hostile input** — mostly strong (errno-named repairs
   for EISDIR/ENOTDIR/ENOTEMPTY, invalid-JSON recovery hint, version posture);
   one reachable crash defeats it — major finding below.

## Findings

### 1. [major] Planted empty directory at `ESCALATION.md` crashes escalation with no terminal record

Location: `tools/orchestration/src/pipeline/runner.ts:2738–2745` (with
`enforceOwnership` residue at runner.ts:290–303)

`finalizeEscalate` writes the marker with a bare `writeFileSync` that is not
wrapped and not preceded by a directory-clear. A stage that owns `docs/**`
creates `docs/designs/<dir>/ESCALATION.md/probe`:

- the boundary sweep treats `ESCALATION.md` as reserved and reverts
  `probe` — but reverting a *file* path never removes the directory it sat
  in, and git never lists the (then-empty, untracked) directory itself, so it
  survives the sweep invisibly (dirtyFiles is file-only with
  `--untracked-files=all`, runner.ts:204–225);
- at the next escalation, `writeFileSync(join(repoDir, reportRel))` hits
  `EISDIR`; that error is not a `CheckpointWriteFailure`, so the drive-loop
  branch (runner.ts:2445) rethrows, and escalations raised outside the try
  (decideNext, runner.ts:2129) escape outright.

Result: the run crashes with no `ESCALATION.md`, no terminal commit, status
stuck `running` — precisely the run-2 blocker §3 exists to close, now
reachable through the marker path instead of the checkpoint path. The
checkpoint-path twin of this attack is covered (sweep restores a *tracked*
path's directory replacement, ownership.test #3; `restoreCommittedCheckpoint`
clears it) — the marker path has no equivalent because `ESCALATION.md` is
untracked at first escalation.

Fix: before writing the marker, clear a directory occupying `reportRel`
(exactly the `existsSync(abs) && statSync(abs).isDirectory() → rmSync`
pattern already in `restoreCommittedCheckpoint`, runner.ts:2693–2696), and
wrap the write in try/catch that falls back to printing the full report to
stderr so a record survives either way. Add the regression test (plant
`ESCALATION.md/<file>`, run a stage boundary, then force escalation; assert
`ESCALATION.md` exists and no error escapes).

### 2. [minor] Marker write is not atomic, and the comment claims it is

Location: `tools/orchestration/src/pipeline/runner.ts:2722–2724, 2744`

The comment says "Atomic tmp+rename, like every other checkpoint write: this
is the one record the operator must be able to read afterwards" — but line
2744 is a direct `writeFileSync`. A crash mid-write truncates the one record
§3 designates as the terminal marker (the split-record gate-log extraction at
runner.ts:2070–2074 then reads a partial failure line). Fix: write the marker
through a tmp+rename pair (the `saveState` shape), and align the comment.

### 3. [minor] The `done` terminal save has no `CheckpointWriteFailure` guard

Location: `tools/orchestration/src/pipeline/runner.ts:2124–2127, 2669`

`finalizeDone`'s `checkpoint()` sits outside the drive loop's try (which
starts at 2195), so the directory-at-checkpoint-path class that §3 converts
to an escalation elsewhere still crashes the completion path: the `done`
record and its commit never land and the raw failure escapes to the CLI.
Same hostile-input class, one unguarded terminal save. Fix: route
`finalizeDone`'s save through the same guard — on `CheckpointWriteFailure`
restore the committed checkpoint and escalate (or at minimum print the §3
repair), instead of throwing.

### 4. [minor] Single-flight premise: a coder-class stage may overwrite or delete `.pipeline.lock`

Location: `tools/orchestration/src/pipeline/types.ts:190–202` (fence excludes
`LOCK_FILE`), `.gitignore:3`

§4's atomic create closes the process-vs-process race, and §1's trust posture
explicitly leans on it ("trustworthy inside a run by deterministic fencing
plus single-flight (§4)"). But `namesReservedSegment` omits `LOCK_FILE`, so
`stagePathAllowed('implement'|'fix', '.pipeline.lock')` is true, and the lock
is gitignored — so even if the matcher covered it, `dirtyFiles` (the sweep's
only view) structurally never lists it. A stage that deletes or rewrites the
lock breaks the single-flight story the checkpoint fence depends on: the next
`acquireLock` then admits a second writer, or a planted live-pid holder blocks
`resume` (DoS). The moves validator already names `LOCK_FILE` reserved
(moves.ts:139–141), which makes the ownership fence's exclusion look like an
oversight rather than a decision. Fix: add `LOCK_FILE` to the reserved-segment
set, and re-assert the lock at stage boundaries (if the lock file is absent or
no longer names this pid/stage, recreate it with `'wx'` before continuing).
This is distinct from the Deferred lock-ID nonce.

### 5. [minor] No size bound on the checkpoint read

Location: `tools/orchestration/src/pipeline/state-store.ts:133–137`

`loadState` reads the raw file without a cap before `JSON.parse`. The
checklist's untrusted-file rule is "schema-validated, size-limited"; the
committed copy remains a recovery anchor for pre-fence history the design's
own Risks row admits can carry model-authored bytes, so a multi-MB hostile
checkpoint at `resume`/`status` spends unbounded memory before the schema
rejects it. Cheap fix: `stat` the file (or read a bounded prefix + reject)
and fold an "checkpoint file too large" refusal into the existing message
posture.

## Positives worth pinning

- Fence correctness is genuinely strong: segment-wise, case-folded, any-depth,
  with neighbor paths (`.pipeline.json-notes.md`, `pipeline.json`) pinned
  allowed; planted `saveState`-shaped tmp files are reverted (they match
  nothing `saveState` recorded); the author-skip test compares against the
  runner's own in-memory bytes, so "dirty ⇒ runner" holds as the proposal
  claims.
- Version refusal is provably inert on disk (byte-identical test), and the
  refusal/recovery messages name operator postures without leaking stack
  traces.
- `'wx'` exclusive create plus holder re-read on retry is the right shape;
  dead-holder clear is unchanged, narrow race honestly documented.

## Verdict

request-changes — one open major (finding 1), four minors.
