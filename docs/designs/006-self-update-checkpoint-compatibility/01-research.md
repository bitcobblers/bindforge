# Research: Resumability of the pipeline checkpoint across a self-update

## Summary

ADR-006 owns exactly three questions — cross-version resume guarantees, checkpoint
trust, and behavior when the checkpoint cannot be written (source:
docs/adrs/006-self-update-checkpoint-compatibility.md:37-46) — and the current
code answers none of them with a mechanism: the schema version is a hard-rejected
literal 1 and all cross-version compatibility is the accidental discipline of
"optional fields plus backfill at resume" (source:
tools/orchestration/src/pipeline/state-store.ts:42,145-148;
tools/orchestration/src/pipeline/runner.ts:1990-1998). The worktree copy of the
checkpoint is writable by every document and review stage, because the ownership
matrix admits any `docs/**` path for them (source:
tools/orchestration/src/pipeline/gates.ts:260-273), while the runner's own stated
doctrine refuses to trust anything by path convention in a model-writable worktree
(source: tools/orchestration/src/pipeline/runner.ts:296-306). The committed copy
is already the de facto trust anchor: both the loader's recovery hint and the
ADR's interim posture are `git checkout HEAD -- …` (source:
tools/orchestration/src/pipeline/state-store.ts:130-140;
docs/adrs/006-self-update-checkpoint-compatibility.md:55-57). When the checkpoint
cannot be written, today's escalation dies with it — the terminal record is
written through the same failing save path (source:
tools/orchestration/src/pipeline/runner.ts:2576-2605), and the run-2 postmortem
records this cluster as that run's blocker (source:
docs/adrs/006-self-update-checkpoint-compatibility.md:24-33; git show
3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md). External prior
art offers three distinct postures to choose among: additive backward-compatible
snapshots (Terraform), immutable version-pinned executions (Step Functions), and
hard-stop on state-subsystem failure (Terraform locking, SQLite journal markers).

## Findings

### Q1 — What a new runner must guarantee, and what it may refuse

- The schema pins `version: z.literal(1)`, and `loadState` rejects any other
  value before parsing with `unsupported pipeline state version`; a test pins
  the rejection of version 99. So "refuse" is today the ONLY cross-version
  behavior, and it is total (source:
  tools/orchestration/src/pipeline/state-store.ts:42,145-148;
  tools/orchestration/tests/pipeline/state-store.test.ts:122-125).
- Cross-version compatibility so far is a field-discipline, not a mechanism: the
  schema header states "every field added after run 1 is optional", and ADR-003
  requires schema changes introduced by that change to be "additive with
  defaults; `version` stays 1" (source:
  tools/orchestration/src/pipeline/state-store.ts:21-27,32-38,58-61;
  docs/adrs/003-parallel-inference-lane.md:115-116).
- The discipline has a scar: making `results[].reply` required "made run 003
  unresumable", so it is optional and failed attempts persist without it — the
  live 003 checkpoint shows exactly such reply-less failure records (source:
  tools/orchestration/src/pipeline/state-store.ts:88-91;
  docs/designs/003-parallel-inference-lane/.pipeline.json:399-409,475,486).
- There is no migration facility: the only backfill on resume is `baseRef`,
  recomputed via `git merge-base main <branch>` for checkpoints written before
  the field existed (source:
  tools/orchestration/src/pipeline/runner.ts:1990-1998).
- The generator and the checkpoint validator must share grammar, or "a validator
  that rejects it makes runs unresumable by their own checkpoint" — the run
  identity `NNN_RE`/`SLUG_RE` is deliberately the same regex pair in both places
  (source: tools/orchestration/src/pipeline/types.ts:170-177;
  tools/orchestration/src/pipeline/state-store.ts:21-27).
- `saveState` refuses to persist anything its own loader would reject: "a
  checkpoint the run cannot resume is worse than a crash" — a self-update design
  inherits this generator-side guard (source:
  tools/orchestration/src/pipeline/state-store.ts:175-184).
- `updatedAt` means "when the snapshot was taken", stamped by the caller, not at
  rename — a documented field SEMANTICS that a later version must preserve when
  resuming an old checkpoint, independent of shape (source:
  tools/orchestration/src/pipeline/state-store.ts:159-174).
- `stageCommits` is a bounded 200-entry ownership window of runner-authored SHAs;
  a post-merge resume inherits the pre-merge list, so the window's meaning is
  part of the cross-version contract too (source:
  tools/orchestration/src/pipeline/runner.ts:345-355;
  tools/orchestration/src/pipeline/types.ts:207-214).

### Q2 — Worktree copy vs committed copy vs moving the checkpoint

- Every document/review stage may write any `docs/**` path, and the coder stages
  may write anything except tests — so the checkpoint is inside the write-set of
  the very stages the ADR names; the ownership sweep reverts only paths OUTSIDE
  that matrix, so it cannot distinguish tampering from a sanctioned write
  (source: tools/orchestration/src/pipeline/gates.ts:255-273;
  tools/orchestration/src/pipeline/runner.ts:233-294).
- The runner's own doctrine: "Nothing in a model-writable worktree can be made
  trustworthy from a path convention", recorded when the runner stopped executing
  formatters out of the worktree (source:
  tools/orchestration/src/pipeline/runner.ts:296-306).
- A tampered checkpoint is not inert data: seat `findings` are cloned through
  every checkpoint and re-injected into that seat's next prompt, bounded only by
  a 1,000-character field cap on the way out and again on the way in — size is
  capped, truth is not (source:
  tools/orchestration/src/pipeline/state-store.ts:31-38;
  tools/orchestration/src/pipeline/runner.ts:2414-2423,2493-2502).
- The committed copy is already the designated recovery anchor: `loadState`'s
  parse-failure message names `git checkout HEAD -- <designDir>/.pipeline.json`,
  a test pins that hint, and the ADR's interim operator posture is the same
  command (source: tools/orchestration/src/pipeline/state-store.ts:126-141;
  tools/orchestration/tests/pipeline/state-store.test.ts:147-151;
  docs/adrs/006-self-update-checkpoint-compatibility.md:55-57).
- The checkpoint is committed by construction: `commitStage` stages everything
  with `git add -A`, and `finalizeDone`/`finalizeEscalate` save BEFORE committing
  so the terminal status lands in the commit — the committed copy is
  deliberately the newest good state at every stage boundary (source:
  tools/orchestration/src/pipeline/runner.ts:316-338,2554-2562,2585-2605).
- Precedent for moving a runner-owned file out of the writable area exists:
  `.pipeline.lock` lives at the repository root, explicitly "named … so the
  moves validator can reject it as a path segment", and the moves validator
  reserves `STATE_FILE`, `LOCK_FILE` and `.git` as never being move endpoints
  (source: tools/orchestration/src/pipeline/types.ts:163-168;
  tools/orchestration/src/pipeline/moves.ts:134-147).
- A run that would relocate a checkpoint is guarded at entry: starting refuses
  when a checkpoint already exists at the identity's design dir ("resume it, or
  remove …") (source: tools/orchestration/src/pipeline/identity.ts:113-121).
- ADR-003 already fixed the in-process write discipline the relocation must keep:
  one write path inside a concurrent pass, drained before the pass finalize —
  "Checkpoint trust, tamper recovery, and recovery from a corrupted or replaced
  checkpoint are out of scope here and belong to ADR-006" (source:
  docs/adrs/003-parallel-inference-lane.md:117-122;
  tools/orchestration/src/pipeline/state-store.ts:192-229).
- Counterpoint from the largest production state-file design: Terraform tells
  users "Do not directly edit this file" and to avoid storing state "in a
  version control system … that does not support Terraform state locking and
  secure access control" — BindForge's checkpoint is precisely such a file, in a
  tree its writers can edit (source:
  https://developer.hashicorp.com/terraform/language/state).

### Q3 — When the checkpoint cannot be written

- The terminal record is written through the failing path itself:
  `finalizeEscalate` sets `status: 'escalated'`, appends the gate-log reason,
  and calls the checkpoint save BEFORE writing `ESCALATION.md` and committing —
  the gate-log reason exists only inside the checkpoint (source:
  tools/orchestration/src/pipeline/runner.ts:2576-2605).
- Today's failure behavior on that path: a checkpoint error from a seat rejects
  the seat and ends the run (source:
  tools/orchestration/src/pipeline/runner.ts:1868-1876); the ownership sweep on
  the escalation path was made best-effort precisely because "a throw from the
  escalation-path sweep would escape `drive` before `finalizeEscalate`, leaving
  status `running`, no ESCALATION.md, and the run's only terminal record never
  minted" (source: tools/orchestration/src/pipeline/runner.ts:1877-1896).
- A refused (invalid) snapshot rejects only its own caller; the writer chain
  keeps landing later writes — partial-failure semantics already exist in the
  write path (source: tools/orchestration/src/pipeline/state-store.ts:215-222).
- Replacing the checkpoint path with a directory makes every future write throw:
  POSIX `rename` fails `EISDIR`/`ENOTEMPTY` when `new` names a (non-empty)
  directory and on failure "neither the file named by old nor the file named by
  new shall be changed"; Node's `fs.rename` likewise "an error will be raised"
  when a directory sits at newPath (source:
  https://pubs.opengroup.org/onlinepubs/9699919799/functions/rename.html;
  https://nodejs.org/api/fs.html). With no fallback around `saveState`, that is
  a run-ender on both paths today.
- The run-2 postmortem records exactly this cluster as the convergence blocker:
  concurrent-write ordering, trusting an in-worktree file every review stage may
  write, the checkpoint replaced by a directory, and what to commit when the
  checkpoint cannot be written at all — one escalation chain ends with "escalate
  writes its own record through the failing write" (source:
  docs/adrs/006-self-update-checkpoint-compatibility.md:24-33; git show
  3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md).
- Resume cannot currently observe an escalation whose checkpoint write failed:
  `resumePipeline` reads only the checkpoint, sets `status = 'running'`
  unconditionally, and merely deletes any `ESCALATION.md` marker — the marker is
  never read back as state (source:
  tools/orchestration/src/pipeline/runner.ts:1952-2005).

### Security landscape lens

- The run lock is check-then-write, not atomic: `acquireLock` does
  `existsSync` + pid-liveness, then `writeFileSync` — two invocations can both
  pass the check. The single-flight guarantee the checkpoint relies on across
  processes rests on this racy file (source:
  tools/orchestration/src/pipeline/runner.ts:383-404).
- External contrast for both points: Terraform treats lock failure as fatal —
  "If state locking fails, Terraform does not continue" — and its `force-unlock`
  requires a lock-ID nonce so unlocks target the right lock (source:
  https://developer.hashicorp.com/terraform/language/state/locking); Node offers
  the exclusive-create primitive that would close the TOCTOU: the `'wx'` flag
  fails if the path exists, with an explicit caveat that exclusivity is not
  guaranteed on network file systems (source: https://nodejs.org/api/fs.html).
- The run-2 postmortem records that run's single blocker as a path-traversal
  advisory (GHSA-fx2r-fm89-x94v / CVE-2026-88624) in the `Worktree.remove`
  helper — cited here as recorded in the postmortem; the advisory pages could
  not be independently fetched (source: git show
  3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md).
- Move validation already hardens the path space the checkpoint relocation must
  survive: repo-relative POSIX shape, no `.`/`..` segments, Windows reserved
  names, trailing dot/space segments, and the reserved-segment rule — judged
  "identically on every platform" (source:
  tools/orchestration/src/pipeline/moves.ts:107-147).

## Prior art

- Terraform state: local `terraform.tfstate` plus `terraform.tfstate.backup`;
  "new Terraform versions are generally backward compatible with state snapshots
  produced by earlier versions", while warning that direct consumers of the
  format should "expect to perform ongoing maintenance" (source:
  https://developer.hashicorp.com/terraform/language/state). That is the
  additive-forward posture ADR-003 already follows (source:
  docs/adrs/003-parallel-inference-lane.md:115-116).
- AWS Step Functions: a version is a "numbered, immutable snapshot of a state
  machine"; executions started with a version ARN run that snapshot, and version
  numbers are never reused — the pin-to-the-writer posture, which the runner's
  startup code-freeze already mirrors informally (source:
  https://docs.aws.amazon.com/step-functions/latest/dg/concepts-state-machine-version.html;
  docs/adrs/003-parallel-inference-lane.md:108-110).
- Temporal: long-running executions resume under changed code only under replay
  determinism; `patched()`/`deprecatePatch()` markers branch on the revision
  that produced the history, Worker Versioning pins executions to code
  revisions, and a whole-workflow cutover (new name) is the blunt alternative —
  the design space for "resume with new code" in three tiers (source:
  https://docs.temporal.io/develop/typescript/workflows/versioning).
- SQLite: atomic commit via a rollback journal whose mere existence is the
  commit marker, fsync ordering around it, and automatic "hot journal" rollback
  on next access — recovery that needs no operator, at the cost of a side file
  in the same directory (source: https://www.sqlite.org/atomiccommit.html).
- CRIU: a checkpoint image is restored only after validating the environment —
  `--file-validation` (build-ID by default) ensures the restored file "is the
  same version when it was dumped", default CPU caps refuse to proceed on
  mismatch, and disabling the check is called the "Most dangerous mode" — the
  match-what-created-it posture for cross-version restore (source:
  https://man.archlinux.org/man/criu.8.en).
- In-repo precedent for each ADR-006 question: the CheckpointWriter single-path
  + drain invariant (source:
  tools/orchestration/src/pipeline/state-store.ts:192-229;
  docs/adrs/003-parallel-inference-lane.md:117-121); the git-anchored recovery
  hint (source: tools/orchestration/src/pipeline/state-store.ts:126-141); the
  reserved-segment guard protecting the checkpoint from model-initiated moves
  (source: tools/orchestration/src/pipeline/moves.ts:134-147); and the run-2
  escalation chain that named digest-anchored trust, all in the postmortem read
  from history (source: git show
  3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md).

## Platform notes

- Toolchain floor: Node `>=22.23.4` and `zod ^4.6.7`; the version check today is
  a hand-written literal compare before the zod parse, so any versioned-migration
  design changes that seam (source: tools/orchestration/package.json:30-31,48;
  tools/orchestration/src/pipeline/state-store.ts:145-149).
- Atomic write primitive in use: unique `${target}.${hex}.tmp` in the same
  directory, then `rename` (source:
  tools/orchestration/src/pipeline/state-store.ts:185-189). POSIX guarantees the
  rename is atomic and failure-free-of-side-effects; a stray `.tmp` left beside
  the checkpoint is visible to `git add -A`, which the drain barrier exists to
  prevent (source:
  https://pubs.opengroup.org/onlinepubs/9699919799/functions/rename.html;
  tools/orchestration/src/pipeline/state-store.ts:192-201;
  tools/orchestration/tests/pipeline/state-store.test.ts:129-132).
- `rename` cannot land a file over a directory (POSIX `EISDIR`/`ENOTEMPTY`;
  Node raises) — the "checkpoint path replaced by a directory" obstacle is
  therefore not repairable inside the write call, only by detection and an
  explicit repair step (source:
  https://pubs.opengroup.org/onlinepubs/9699919799/functions/rename.html;
  https://nodejs.org/api/fs.html).
- Node's fs has no advisory-lock primitive; exclusive create via `'wx'`
  (O_EXCL) is the available building block for an atomic lockfile, with the
  documented network-filesystem caveat (source: https://nodejs.org/api/fs.html).
- Runs execute in runner-managed git worktrees: the 003 checkpoint's failure
  records name absolute paths under
  `/home/mike/.local/share/opencode/worktree/…` — a self-update resume happens
  on the merged branch, where both a worktree copy and the branch's committed
  copy of the checkpoint exist and can diverge (source:
  docs/designs/003-parallel-inference-lane/.pipeline.json:408,475,486;
  docs/adrs/006-self-update-checkpoint-compatibility.md:52-53).
- Path-shape portability is already a solved in-repo problem for move
  validation (Windows reserved names, dot/space segments, backslash/colon
  rejection) — any relocated checkpoint path inherits that validator (source:
  tools/orchestration/src/pipeline/moves.ts:107-131).

## Risks & unknowns

- Version bump is a cliff, not a step: `version: z.literal(1)` rejects anything
  else wholesale, and no read-side migration exists to write — introducing v2
  without a migration orphans every in-flight checkpoint by design (source:
  tools/orchestration/src/pipeline/state-store.ts:42,145-148).
- The drain barrier guarantees one-writer only within one process; cross-process
  single-flight rests on the check-then-write lockfile (source:
  tools/orchestration/src/pipeline/state-store.ts:192-229;
  tools/orchestration/src/pipeline/runner.ts:383-404). Whether self-update
  resume can race a still-running old-code process is undetermined here.
- The two checkpoint write paths (sequential `checkpoint()` and
  `CheckpointWriter`) must both keep working across a version boundary; their
  shared field semantics (`updatedAt` meaning) are documented in comments only
  (source: tools/orchestration/src/pipeline/state-store.ts:159-174).
- Anchoring trust in the committed copy shifts trust to the branch/merge
  process: the committed checkpoint was authored by the old runner and reaches
  `HEAD` via human merge (the ADR's scenario is a human resuming "on the new
  branch"), but no code enforces what review of that branch covers (source:
  docs/adrs/006-self-update-checkpoint-compatibility.md:52-53).
- The postmortem file is not in the working tree — the run-2 receipts (the 27/33
  split, the escalation chains, the GHSA/CVE blocker) are readable only via git
  history and were not independently re-verified against public advisories
  (source: docs/adrs/006-self-update-checkpoint-compatibility.md:30-33; git
  show 3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md).
- Digest-anchored trust appears in the postmortem only as an escalation step in
  a chain that never completed; no accepted design for it exists in this tree
  ("no design is recorded here") (source: git show
  3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md;
  docs/adrs/006-self-update-checkpoint-compatibility.md:37).

## Open questions

1. Which compatibility posture: the Terraform promise (new versions read old
   snapshots, additive-optional forever), the Step Functions pin (executions are
   bound to the code revision that started them — matching the startup
   code-freeze the self-update run already relies on), or an explicit
   migration-on-read? What may the new runner "simply refuse", and with what
   operator message beyond `unsupported pipeline state version` (source:
   https://developer.hashicorp.com/terraform/language/state;
   https://docs.aws.amazon.com/step-functions/latest/dg/concepts-state-machine-version.html;
   tools/orchestration/src/pipeline/state-store.ts:145-148)?
2. Is the trust anchor the worktree copy, the committed copy (already the de
   facto recovery anchor via the `git checkout HEAD --` hint), or a relocated
   file outside `docs/**` with the `LOCK_FILE` as precedent — and if relocated,
   what happens to `finalizeDone`'s invariant that the terminal status is saved
   BEFORE the commit so it lands inside it (source:
   tools/orchestration/src/pipeline/runner.ts:2554-2562;
   tools/orchestration/src/pipeline/types.ts:163-168)?
3. Should resume verify the worktree checkpoint against the committed copy (a
   digest comparison, the run-2 "digest-anchored trust" step), or is
   deterministic write-fencing plus a human-initiated resume enough — remembering
   that seat findings from the checkpoint flow into later prompts verbatim up to
   the field cap (source: git show
   3cc0c2b:docs/designs/002-run2/postmortem/convergence-blockers.md;
   tools/orchestration/src/pipeline/runner.ts:2493-2502)?
4. When the checkpoint write fails: hard-stop like Terraform's lock failure, or
   a degraded terminal where `ESCALATION.md` is authoritative and `resume`
   learns the run's state from it or from git — today `resumePipeline` never
   reads the marker and only the checkpoint can say `escalated` (source:
   https://developer.hashicorp.com/terraform/language/state/locking;
   tools/orchestration/src/pipeline/runner.ts:1952-2005)?
5. Is a marker-vs-checkpoint split-record acceptable (SQLite's journal-existence
   as commit marker), or must the run's terminal state always have exactly one
   authoritative location (source: https://www.sqlite.org/atomiccommit.html)?
6. Should `acquireLock` become an atomic exclusive create (`'wx'`), and should
   lock acquisition failure be fatal in the Terraform sense (source:
   https://nodejs.org/api/fs.html;
   https://developer.hashicorp.com/terraform/language/state/locking;
   tools/orchestration/src/pipeline/runner.ts:383-404)?
7. Does the cross-version guarantee extend to semantics, not just shape —
   e.g. a post-merge runner re-interpreting `stageCommits`, `designCycle`, or
   gate-log entries produced by the old code with the same numbers but a
   different control flow (source:
   tools/orchestration/src/pipeline/runner.ts:345-355;
   tools/orchestration/src/pipeline/state-store.ts:68-80)?
