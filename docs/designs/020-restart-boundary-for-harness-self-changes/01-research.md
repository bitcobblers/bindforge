# Research: restart boundary for harness self-changes

Research input for ADR 020 (`docs/adrs/020-restart-boundary-for-harness-self-changes.md`,
status `proposed`, 2026-11-13). The ADR asks one question: how should the runner
detect that a stage commit changed the code it itself executes or renders from,
pause cleanly at a `restart-required` boundary, and let a fresh process resume —
and what does that cost the checkpoint grammar, the exit codes, the lock, and the
run records. This run's design directory held no prior `01-research.md` or
`02-proposal.md`; this is the first design attempt for ADR 020.

Lenses applied: **prior art** (how comparable systems handle "the running thing
changed underneath itself") and **platform notes** (Node process/module
mechanics and exit-code conventions relevant to the TypeScript harness).

## Summary

The staleness ADR 020 describes is mechanically visible in the tree: stage
prompt templates are read from disk at stage time (`loadTemplate`), while the
runner's own modules are fixed at process start, so a stage commit that edits
`prompts/*.md` or `src/pipeline/*.ts` leaves the live process rendering new
templates with old code (source: tools/orchestration/src/pipeline/prompts.ts:81-83;
tools/orchestration/src/pipeline/runner.ts:2453-2467). The primitives the ADR
needs mostly exist: the ownership sweep already computes the exact path list of
every runner-authored commit (`git diff-tree --name-only` over
`headBefore..HEAD`), stage commits are runner-minted at known sites, and the
checkpoint + `resume` + lock machinery is already designed to survive a process
death between stages (source: tools/orchestration/src/pipeline/runner.ts:301-318,
422-443; docs/workflows.md:226-233). The costs the ADR names are equally
concrete: `RunStatus` and the checkpoint's zod schema hard-code exactly
`running|done|escalated`, and the CLI collapses every non-`done` status to exit
2, so a distinct pause state and exit status are new, load-bearing values in
both (source: tools/orchestration/src/pipeline/types.ts:202;
tools/orchestration/src/pipeline/state-store.ts:97;
tools/orchestration/src/pipeline/cli.ts:17,214,227). External practice is one-
sided: watch-and-restart (nodemon), supervisor-driven replacement (systemd,
Kubernetes), and in-process hot swap with version limits (Erlang) all exist, but
production systems overwhelmingly choose clean process restart over partial
reload — which is the ADR's posture (source: https://nodemon.io/;
https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html;
https://kubernetes.io/docs/concepts/workloads/controllers/deployment/;
https://www.erlang.org/doc/system/code_loading.html). The sharpest open issues
found are forward-compatibility of a new status value against the ADR-006
grammar, pass-level (not just stage-level) commits, and the exit-code contract a
driver can actually rely on.

## Findings

Mechanism — what actually goes stale, where the code is:

- The runner assembles each stage prompt by reading the template from disk at
  stage entry (`loadTemplate(promptsDir, name)`), while `promptsDir` itself is
  a package-relative path (`../../prompts/` from the CLI module), so a resumed
  process in the same worktree reads the branch's current templates. (source:
  tools/orchestration/src/pipeline/prompts.ts:81-83;
  tools/orchestration/src/pipeline/cli.ts:20)
- The drive loop already learned one lesson in this hazard class: a stage's
  template is loaded once at stage entry and pinned for every attempt, because
  a stage that edits its own template would otherwise be re-rendered with a
  half-updated template on the timeout retry — the comment names run 010's r8
  escalation as the cause. The pin covers retries inside one stage; it does not
  cover the next stage of the same process, which is exactly ADR 020's gap.
  (source: tools/orchestration/src/pipeline/runner.ts:2453-2467)
- Strict render scans the template text and throws `unresolved template
  placeholder: {{key}}` for any placeholder missing from the vars — the rule
  that turned run 016's stale renderer into an escalation, and the rule ADR 020
  explicitly refuses to loosen. `TEMPLATE_DEFAULTS` already carries fallbacks
  for `testOwnership` and `gateEvidence`, added for exactly the fixture/render
  staleness reason ADR 020 quotes. (source:
  tools/orchestration/src/pipeline/prompts.ts:42-56,58-79;
  docs/adrs/020-restart-boundary-for-harness-self-changes.md:34-38,89-92)
- The templates carrying `{{gateEvidence}}` today are `implement.md`, `fix.md`,
  `fix-tests.md`, `code-review.md` — the four ADR 016 stage commits that
  triggered the incident are merged, so the current tree renders them only
  because the merged renderer supplies the var. (source:
  tools/orchestration/prompts/implement.md:15, fix.md:12, fix-tests.md:12,
  code-review.md:20)
- Template provenance for decision 4 already exists per attempt: the runner
  computes `templateHash` (first 12 hex of sha256) of the exact text rendered
  and writes `- template: <file> @ <sha>` into the transcript's provenance
  header. (source: tools/orchestration/src/pipeline/prompts.ts:85-87,104,208;
  tools/orchestration/src/pipeline/runner.ts:1343-1345)

The detection hook the ADR wants:

- Stage commits are runner-minted, never model-made (models have no git
  access), at three sites: `finalizeStage` (single stages),
  `finalizeReviewPass` (one commit per review pass — not per stage), and the
  revert commits `enforceOwnership` can mint. The boundary check therefore has
  to hook all three, not just the single-stage path. (source:
  tools/orchestration/src/pipeline/runner.ts:1848-1892,1940-1961,363-377;
  docs/workflows.md:109-123)
- The path list of a runner-authored commit is already computed every stage:
  `enforceOwnership` walks `git rev-list headBefore..HEAD`, splits own vs
  foreign commits via `state.stageCommits`, and lists own commits' paths with
  `git diff-tree -r --no-commit-id -z --name-only`. The executable-surface
  comparison decision 1 asks for is that same primitive against a path set.
  (source: tools/orchestration/src/pipeline/runner.ts:292-318,445-460)
- The harness's executable surface is small and locatable: 15 TypeScript files
  under `tools/orchestration/src/` (the runner imports them at process start)
  and 13 stage templates under `tools/orchestration/prompts/`. Agent permission
  files (`tools` of the stage agents) live separately under
  `.opencode/agents/{research,design,code,codereview,review,ops}/*.md` — the
  ADR explicitly does not decide whether they belong to the boundary. (source:
  docs/adrs/020-restart-boundary-for-harness-self-changes.md:119-123)

Clean pause vs escalation — what already distinguishes a terminal stop:

- `finalizeEscalate` sets `status='escalated'`, gate-logs `escalate`, writes
  `ESCALATION.md` through a tmp+rename pair, prints repair steps, and mints a
  terminal `docs: <nnn>-<slug> pipeline escalated` commit. A boundary pause by
  contrast must produce no marker and no escalation exit but must still write
  the checkpoint and commit the stage normally. (source:
  tools/orchestration/src/pipeline/runner.ts:3454-3566)
- The drive loop already has a control-flow signal for "stop driving without
  escalating": `StopDriving`, thrown after gate refusals and caught in the loop
  after the checkpoint save. A `restart-required` return is its natural sibling.
  (source: tools/orchestration/src/pipeline/runner.ts:2200,2471-2477,2899)
- Precedent for a sticky, record-driven stop that a plain resume honours: the
  light flow's `planStop` field — checkpoint-recorded, `decideNext` re-escalates
  while it stands, cleared only by an in-flow repair. A restart record can
  follow that shape rather than inventing one. (source:
  tools/orchestration/src/pipeline/types.ts:290-301;
  tools/orchestration/src/pipeline/states.ts:136-138;
  tools/orchestration/src/pipeline/runner.ts:2406-2429)
- Resume already deletes a stale `ESCALATION.md`, flips `status` to `running`,
  and gate-logs split records (marker present, checkpoint says running). A
  restart pause that lives only in a file and not in the checkpoint would
  recreate that split-record class ADR-006 §3 made a rule about. (source:
  tools/orchestration/src/pipeline/runner.ts:2327,2345-2378;
  docs/adrs/006-self-update-checkpoint-compatibility.md:44-46;
  docs/workflows.md:235-240)
- `pipeline status` is one `renderStatus` from the gate log plus state; decision
  3's "named in the summary a driver logs" and the status line for a paused run
  both land in that one renderer and the CLI's exit mapping. (source:
  tools/orchestration/src/pipeline/runner.ts:3598-3648;
  tools/orchestration/src/pipeline/cli.ts:188-200)

Resume is already a designed restart primitive:

- A run's checkpoint is committed at every stage boundary; the lock is held by
  atomic `wx` create at the repo root with pid-liveness recovery, and is
  re-asserted at every stage boundary before the next stage window opens — the
  existing single-flight posture is what makes a fresh `resume` process safe
  immediately after a pause. (source: docs/workflows.md:206-212,232-233;
  tools/orchestration/src/pipeline/runner.ts:499-515,2548-2554)
- `runPipeline`/`resumePipeline` acquire the lock, load or create state, then
  `drive`; budgets and flow are read from the checkpoint, not flags, so a new
  process genuinely picks the run up at the same stage with zero operator state
  transfer — the "clean restart primitive" ADR 020 leans on is real code.
  (source: tools/orchestration/src/pipeline/runner.ts:2263-2300,2302-2394;
  docs/adrs/020-restart-boundary-for-harness-self-changes.md:98-100)

Costs named by the ADR, as they appear in today's code:

- `RunStatus = 'running' | 'done' | 'escalated'` (types) and the checkpoint's
  `status: z.enum(['running','done','escalated'])` (StateSchema) are the two
  places the new state has to enter; the CLI today maps `done→0`,
  everything-else→`2`, errors→`1`. (source:
  tools/orchestration/src/pipeline/types.ts:202;
  tools/orchestration/src/pipeline/state-store.ts:97;
  tools/orchestration/src/pipeline/cli.ts:17,214,227)
- The set of "files the runner executes" needing an honesty test has an
  existing sibling pattern: `tests/pipeline/agent-shell.test.ts` pins the gate
  commands against the agent rule sets "so a future tightening cannot silently
  re-blind a stage". (source: docs/workflows.md:345-347;
  docs/adrs/020-restart-boundary-for-harness-self-changes.md:113-115)
- ADR-006's checkpoint grammar already ships with per-era golden fixtures
  (`tests/pipeline/fixtures/checkpoints/`: run-1-shape, run-2-era, run-3-live,
  light-era) and an explicit rule set: new fields optional with behaviour-safe
  defaults, nothing removed/retyped/renamed/re-interpreted, `version` stays 1.
  (source: tools/orchestration/src/pipeline/state-store.ts:58-79; fixture
  directory listing)

## Prior art

The problem class — a long-lived process whose own code changed on disk — is
attacked three ways in comparable systems, and the industry's weight falls on
restart, not reload:

- **Watch-and-restart (the driver model).** nodemon "will monitor for any
  changes in your source and automatically restart your server" — the restart
  is external to the watched process, exactly the ADR's driver-resumes posture.
  (source: https://nodemon.io/)
- **Supervisor stop/start and replacement.** systemd distinguishes *reload*
  (ExecReload, the `kill -HUP $MAINPID` idiom) from *restart*, and its own docs
  call the async signal reload "usually not a good choice", recommending a
  synchronous protocol (`Type=notify-reload`: `RELOADING=1` then `READY=1`) or
  a reload command that waits; a timed-out reload leaves "the service running
  with the old configuration". Kubernetes goes further: a Deployment update
  replaces whole Pods ("gradually scale down the old ReplicaSets and scale up
  the new one") rather than mutating a running one. Both support the ADR's
  rejection of partial in-place update. (source:
  https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html;
  https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)
- **In-process code replacement, with hard limits.** Erlang/OTP is the
  best-engineered hot-swap system: a module exists as at most two variants
  (`current` and `old`); loading a third purges the old and terminates
  processes lingering in it, and a process only moves to new code at a fully
  qualified call. It works because the language and VM were built for it;
  Node is not. Webpack's HMR carries the same verdict for the JS world: "HMR is
  not intended for use in production, meaning it should only be used in
  development", with a page of gotchas (stale bindings, double side effects,
  dispose handlers), and for Node-server HMR the doc's own failure advice is a
  process restart ("there is no page to reload… your cue to restart the
  process"). (source: https://www.erlang.org/doc/system/code_loading.html;
  https://webpack.js.org/guides/hot-module-replacement/)
- **Naming/format.** The settled vocabulary split is *reload* (config,
  in-process, systemd) vs *restart*/*rollout* (new process, systemd/Kubernetes)
  vs *hot swap* (in-process code, Erlang/webpack). BindForge already reserves
  `resume` for the checkpoint-driven continuation, so "restart-required" as a
  pause marker reads consistently with that vocabulary; note that the harness's
  own code already uses "stage boundary" for the checkpoint-commit point
  ("checkpoint is committed at every stage boundary"; "boundary commits advance
  the committed checkpoint"), which the design should not silently overload.
  (source:
  https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html;
  docs/workflows.md:232;
  tools/orchestration/src/pipeline/runner.ts:325-330)
- **What they regret / where they retreated.** systemd deprecates the
  fire-and-forget signal reload in favour of a completion protocol; FreeBSD
  has formally deprecated the `sysexits` exit-code set (EX_TEMPFAIL 75 was the
  "temporary failure, reattempt later" code) with the honest BUGS note "the
  choice of an appropriate exit value is often ambiguous". Lesson for decision
  3: a distinct exit code is workable, but the *record* (checkpoint + named
  reason) is the durable contract; keep the code simple and documented.
  (source:
  https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html;
  https://www.freebsd.org/cgi/man.cgi?query=sysexits)

## Platform notes

- Node caches modules after first load — "every call to `require('foo')` will
  get exactly the same object returned" — so the "live process renders old
  code" premise of ADR 020 is Node's documented default, not a quirk. The
  documented eviction lever (`require.cache` key deletion) is CommonJS-specific;
  the harness is authored as ES modules (`import`, `import.meta.url`,
  `.js` specifiers throughout `src/pipeline/`), so no documented reload lever
  applies to it at all — consistent with the ADR's rejection of hot-reload.
  (source: https://nodejs.org/api/modules.html#caching,
  https://nodejs.org/api/modules.html#requirecache;
  tools/orchestration/src/pipeline/cli.ts:1-13)
- The lock's single-flight guarantee is POSIX-flavoured: atomic `'wx'` create
  plus `process.kill(pid, 0)` liveness, with a documented stale-clear race
  (lock-ID nonce deferred). A restart pause must release the lock on exit like
  every other terminal path, or the fresh process pays the dead-holder
  posture. (source: tools/orchestration/src/pipeline/runner.ts:480-515,2298,2392;
  tools/orchestration/src/pipeline/types.ts:236-246)
- Worktree mechanics make "the new code is already in force" true: runs execute
  on the `pipeline/NNN-<slug>` branch checked out in the worktree the runner
  started from, and the prompts directory is resolved relative to the runner's
  own module path — so after the stage commit, the on-disk tree a fresh
  `resume` process imports is already the changed tree. (source:
  docs/adrs/006-self-update-checkpoint-compatibility.md:18-21;
  tools/orchestration/src/pipeline/cli.ts:20;
  tools/orchestration/src/pipeline/runner.ts:2277,2388)
- Cost budget for the pause: the process being replaced is a minutes-long
  inference driver (stage timeouts default to 30 min full / 12 min light;
  run 1's escalated resume was 47 minutes wall clock), against a process start
  that does a lock create, one checkpoint read, and one `git rev-parse`/branch
  ensure — the ADR's "seconds, far less than the escalation" claim is
  consistent with the code's shape. (source: docs/adrs/010-lightweight-plan-flow.md:97;
  tools/orchestration/src/pipeline/stages.ts:98-127;
  tools/orchestration/src/pipeline/runner.ts:3534; docs/workflows.md:267)

## Risks & unknowns

- **Forward compatibility of the new status value.** ADR-006 §2 guarantees old
  checkpoints load in new runners, but StateSchema is a zod enum
  `['running','done','escalated']` and the grammar says a runner "may refuse…
  a shape this schema rejects": a checkpoint carrying a new `restart-required`
  status value is unreadable by any pre-020 runner. Whether that matters in
  practice (old-runner resume after a pause) is an open design question, and
  the `planStop` precedent (companion optional field, status untouched) shows a
  way to add the pause record without touching the enum. (source:
  tools/orchestration/src/pipeline/state-store.ts:58-97;
  tools/orchestration/src/pipeline/types.ts:290-301)
- **Exit code 2 is currently "not done".** Both `run` and `resume` return
  `status === 'done' ? 0 : 2`; without a CLI change a boundary pause is
  indistinguishable from an escalation to any driver, and `2` is today's
  "read `ESCALATION.md`" signal — the risk of automating past an escalation by
  code collision is real and one line wide. (source:
  tools/orchestration/src/pipeline/cli.ts:214,227)
- **Commit-site coverage.** A boundary check hooked only at `finalizeStage`
  misses review-pass commits, ownership-revert commits, and the `done` commit;
  the executable-surface test the ADR promises ("a test that fails when a new
  executable path is added without it") has to enumerate both the file set and
  the commit sites, or drift silently as the harness grows. (source:
  tools/orchestration/src/pipeline/runner.ts:363-377,1940-1961,3419-3424;
  docs/adrs/020-restart-boundary-for-harness-self-changes.md:112-115)
- **Sweep ordering.** `enforceOwnership` can revert files a stage wrote outside
  its ownership before the boundary check runs; whether `prompts/**` and
  `tools/orchestration/**` are writable by which stages per `stagePathAllowed`
  determines whether a self-change can even land a surface-touching commit from
  an arbitrary stage — the design should state the interaction rather than
  assume every stage can change the harness. (source:
  tools/orchestration/src/pipeline/runner.ts:292-380)
- **Marker/checkpoint split.** If the `restart-required` marker is a file and
  the checkpoint save fails, the run reproduces the split-record posture ADR-006
  §3 exists for (terminal truth vs checkpoint truth); the design must say which
  record is authoritative. (source:
  docs/adrs/006-self-update-checkpoint-compatibility.md:44-46;
  tools/orchestration/src/pipeline/runner.ts:2345-2378)
- **Unmeasured:** the ADR's incident timeline (run 016 escalated 13:57; runner
  imported 12:38) is taken from ADR 020's own text; this run did not open
  run 016's records (design dirs of other runs are not in the read set), so the
  timeline is cited to the ADR, not re-derived. (source:
  docs/adrs/020-restart-boundary-for-harness-self-changes.md:16-32)

## Open questions

For the designer:

1. Representation: a new `RunStatus` enum member, or an optional companion
   field beside `status` in the `planStop` style (types.ts:290-301)? Which one
   keeps a pre-020 runner from refusing the checkpoint, and how is the choice
   pinned by a golden-fixture resume test in the existing era set?
2. The exact executable-surface rule: a static allowlist, a directory-scoped
   match (`tools/orchestration/src/**`, `tools/orchestration/prompts/**`), or a
   derived import graph — and how does the honesty test fail when a new
   importable dir (e.g. `tools/orchestration/scripts/`) appears?
3. Exit contract: what integer does the restart status exit with, what does
   `pipeline status` print for a paused run, and does `resume` require or reject
   anything about the marker (compare the `--stage plan` repair-command posture
   for plan stops)?
4. Loop guard: may one run cross multiple boundaries; if two consecutive stage
   commits both touch the surface, is that routine or a signal the run is
   thrashing the harness (ADR 020 defers "too large for a single pass")?
5. Who restarts: ADR 020 leaves the runner self-restart undecided. Should the
   v1 rule be pause-and-exit only (driver automation), given the lock and
   pid-liveness design make a same-tree relaunch cheap?
6. Scope of "what the runner renders from": does the surface include the agent
   permission files under `.opencode/agents/` (the runner passes agent names to
   the server per stage; whether those files are re-read per session was not
   established in this read set) and the flow configuration in `flows.ts`
   (already covered if `src/**` is, since it is imported at start)?
7. Does decision 4 need any new record beyond the per-transcript
   `template @ sha` line that already exists, or is "the run's records make the
   revision visible" satisfied by pinning that line in a test?
8. Should a boundary pause gate-log an entry (e.g. gate `restart-boundary`) so
   the existing `status` renderer shows it in "recent gates" without a new
   status-line mechanism?
