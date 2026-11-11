# Code review — compatibility seat, round 1, pass c0

Scope: `git diff 846ca090053b43ec99295a7cedafb47910043df9...HEAD -- ':(exclude)docs/designs'`
(17 files: `tools/orchestration/src/pipeline/{cli,gates,runner,state-store,types}.ts`,
`docs/workflows.md`, 7 test files, 3 checkpoint fixtures).
Judged against `02-proposal.md` (Chosen design §1–§4, Compatibility matrix, Test strategy)
and `docs/adrs/006-self-update-checkpoint-compatibility.md`.

Verdict: **approve** — no blocker or major finding. Four minor findings and one nit, all in the
platform/record-portability half of the checklist.

## Checklist evaluation

**1. Platform code matches the Compatibility matrix — pass.**
No new OS branch exists anywhere in the diff (`grep process.platform src/` → nothing): `'wx'`,
`rename`, `statSync`, `rmSync(recursive)`, `git restore` are all cross-platform calls. The only
platform-specific artefact is the POSIX-shell repair text (`rm -r … && git checkout HEAD -- …`),
which the matrix's Windows row already declares POSIX-only, and the macOS row's "partial" is
consistent with "same POSIX code, not exercised here". Every row claimed `yes` (Linux, Node
floor) has a real code path. Residual-risk attribution is slightly too narrow — finding 3.

**2. Node floor — pass.** `tools/orchestration/package.json:30–31` is `>=22.23.4` and unchanged.
New APIs used: `writeFile(..., {flag:'wx'})`, `fs/promises.unlink`, `statSync`,
`rmSync({recursive,force})`, `new Error(msg, {cause})`, `String.includes` — all far below the
floor (A11 holds). `git restore` needs git ≥ 2.23 but was already used by the pre-existing sweep
(`runner.ts:295`), so no new toolchain floor is introduced.

**3. Paths — pass with one drift risk.** Every new path is a `join`/`pathToFileURL` join
(`cli.ts:197`, `runner.ts:2066`, `runner.ts:2742`, `runner.ts:2758`, `state-store.ts:149`); no new
string-concatenated path. The one concatenation is the pre-existing tmp name at
`state-store.ts:277`, whose *shape* is now duplicated as a hard-coded regex — finding 1. The
repair commands stay unquoted safely: design dirs are `NNN_RE`/`SLUG_RE` (`types.ts:210–211`),
ASCII with no spaces. Case-sensitivity assumptions are not fully flagged — finding 4.

**4. Guarded platform imports — pass.** No platform-only module is imported; `node:fs`,
`node:fs/promises`, `node:path`, `node:child_process` only. Nothing can break module load on
another platform.

**5. Locale/layout reality — pass.** Design-dir identity is ASCII by grammar; the checkpoint stays
UTF-8 JSON; `printable()` (`independence.ts:311`) strips only `\p{Cc}`, so non-ASCII finding text
survives into ESCALATION.md and the gate log unchanged; the matcher uses locale-independent
`toLowerCase()`. Non-QWERTY/keybinding surfaces are untouched by this diff. What does *not*
survive a different machine is one machine's absolute worktree path, baked into record text —
finding 2.

**6. Native dependencies — pass.** No dependency added; `package.json` untouched. No CI matrix
exists in this repo (A12) and the diff adds none, which the matrix states honestly.

**7. Tests run where they claim — pass statically, with one gap.** All 13 planned cases exist with
`proves: Test strategy #N` markers: #1 `gates.test.ts:329–378`, #2 `ownership.test.ts:229–296`,
#3 `checkpoint-compat.test.ts`, #4/#5 `state-store.test.ts:228–276`, #6
`checkpoint-writer.test.ts:125–178`, #7 `escalation.test.ts:207–267`, #8/#9/#10
`runner.test.ts:296–409`, #11–#13 `checkpoint-integration.test.ts`. No `.skip`/`runIf` in any new
test, so none is silently skipped everywhere; the only platform-gated suites are the pre-existing
`moves.test.ts` pairs. The single claim that is not actually exercised is #10's concurrency claim —
finding 3. Note the evaluation limit below: I could not execute the suite from this seat.

## Findings

### Minor 1 — the fence's tmp-shape matcher is a second, unlinked copy of the writer's naming rule
`tools/orchestration/src/pipeline/types.ts:181` hard-codes
`/^\.pipeline\.json\.[0-9a-f]{8}\.tmp$/i` — the checkpoint literal, the 8-hex suffix length and
`.tmp` again — while the name it must recognise is built independently at
`state-store.ts:277` (`${target}.${randomBytes(4).toString('hex')}.tmp`). Nothing links them: a
future change to the writer's suffix width or entropy (hex→base36, 8→12 chars) keeps
`saveState` working and silently stops the fence matching the tmp files the runner itself
produces, so a stray tmp becomes a committed violation rather than a swept file. The tests cannot
catch the drift either — `ownership.test.ts:245` plants a *hand-written*
`.pipeline.json.a1b2c3d4.tmp`, which matches the literal regardless of what `saveState` writes.
*Fix:* make the shape one definition — export the literal from `state-store.ts` (e.g.
`stateTmpName(target)`) or build `STATE_TMP_RE` from `STATE_FILE` — and add a case that takes a tmp
name actually produced by `saveState` and asserts `namesReservedSegment` denies it.

### Minor 2 — repair text bakes one machine's absolute path into the printed *and persisted* record
`state-store.ts:149` and `state-store.ts:293` interpolate `file`/`target`, which are absolute
whenever the runner calls them with an absolute design dir — and it always does
(`runner.ts:2001`, `runner.ts:2712` pass `join(repoDir, designDirOf(state))`). That message is not
only printed: `runner.ts:2445` makes it the escalation reason (persisted into the gate log, and
into the next saved checkpoint), and `runner.ts:2743` appends it verbatim to `ESCALATION.md`. So an
escalated run's terminal record carries `/home/<user>/.local/share/opencode/worktree/<id>/…` —
exactly the string that is false on the next machine. ADR-006's own scenario is a resume after a
self-update, where the operator may be in a different worktree; the fixture set proves the shape
(`run-3-live.json:408` is full of such stale worktree paths). The design's §3 wording is
relative-shaped (`git checkout HEAD -- <designDir>/.pipeline.json`) and `finalizeEscalate` already
formats it relatively (`runner.ts:2758–2763`, `cpRel`), so the store-side messages are the
inconsistent half.
*Fix:* keep the path structured rather than pre-formatted (put `path` on `CheckpointWriteFailure`,
and pass the design-dir-relative form into the `loadState` message), and let the printers render
repo-relative commands the way `runner.ts:2761` does.

### Minor 3 — §4's local-filesystem single-flight claim is asserted by the matrix and the docstring but exercised by no cross-process test
`runner.ts:423` documents "on local filesystems exactly one pipeline process takes the lock", and
the Compatibility matrix attributes the residual race to network/shared filesystems only ("stale
lock-ID nonce deferred"). The test that names the claim, `runner.test.ts:392–403`, calls
`acquireLock` twice from one process via `Promise.all`; `acquireLock` is fully synchronous, so the
two calls serialize and the second simply sees a complete lock file. That test passes unchanged
against the deleted `existsSync`+`writeFileSync` code — it pins nothing about `'wx'`. The real
local window is still open and untested: between the exclusive create and the write, a contender
reads an empty file, `holderOf()` returns `null` (`runner.ts:434–441`), the code warns "clearing
stale pipeline lock held by unknown", `unlinkSync`s the live holder's lock and re-creates it — two
winners on a local FS. The proposal does accept "the stale-clear keeps a narrow race", so this is
not a defect against the design; it is the matrix's local-FS guarantee being unverified on every
platform, including the one it claims `yes` for.
*Fix:* make plan #10's third case contend two real processes (`spawn`/`execFileSync` of
`process.execPath` running `acquireLock` against the same fixture repo) and assert exactly one exit
0; and either narrow the docstring/matrix wording to "no race from the create itself" or state the
empty/unparseable-lock window in the matrix's network-FS row alongside the deferred nonce.

### Minor 4 — the case-insensitivity precedent cited in code and matrix does not exist, and the case-insensitive-checkout assumption is not in the matrix
`types.ts:183–189` says the matcher compares "case-insensitively — the way the moves validator
already compares these names", and §1 of the proposal repeats that citation. `moves.ts:136–141`
compares `segment === STATE_FILE` and `segment === LOCK_FILE` case-*sensitively*; only `.git` is
lower-cased (`moves.ts:142`). So the two reserved-name sets now diverge in both directions —
`namesReservedSegment` adds `ESCALATION.md` and the tmp shape, `moves.ts` keeps `LOCK_FILE`, which
the fence omits — and the divergence is papered over by a comment claiming parity. (Coverage is
still sound: a move into a reserved name is refused by rule 3 through the shared
`stagePathAllowed`, `moves.ts:155–161`.) Separately, the Compatibility matrix justifies macOS
"partial" purely from rename/`'wx'` POSIX semantics (A5, A6) and never mentions that its default
filesystem is case-insensitive — the assumption class this whole case-based anti-evasion rule sits
on, and the one this repo already models explicitly at `moves.test.ts:675–687` ("case-sensitive
checkout (Linux)" vs "case-insensitive checkout (macOS)"). The matcher's direction of error is safe
(it denies more), but the *revert* branch resolves case through git's case-sensitive tree lookup
(`runner.ts:294–299`), and that combination is unflagged and untested on the platform the matrix
calls partial.
*Fix:* correct the comment (and note the follow-up for the proposal's §1 citation); decide the
reserved-name question once — folding `LOCK_FILE` into `namesReservedSegment` is free for the sweep
because `.pipeline.lock` is gitignored (`.gitignore:3`) — and add a case-insensitive-checkout row
or a `runIf(process.platform === 'darwin')` companion case per the `moves.test.ts` pattern.

### Nit 5 — `docs/workflows.md` promises a refusal message that is not the message the code prints
The new operator section states the refusal "says to run the version that wrote it, or to restore
the checkpoint or start a fresh run". `state-store.ts:176–186` prints "This runner reads version 1
checkpoints. Until a compatible reader exists, restore the committed copy … or start a fresh run" —
there is no "run the version that wrote it" line, and that posture sits oddly next to §2's
guarantee that any runner resumes any `version: 1` checkpoint.
*Fix:* reword the doc to the two postures §2 actually names, or print the third.

## FYI (out of seat, no severity)

`runner.ts:2066–2083` reads `ESCALATION.md` and splices the matched failure line into the
`terminalSaveFailed` gate-log reason without `printable()`, while `finalizeEscalate` sanitizes its
own reason a few dozen lines above for exactly the stated one-line/no-control-byte reason; the
marker sits inside the model-writable worktree, so a `\r` or an ANSI sequence in that file reaches
the checkpoint.

## Evaluation limits for this seat

- The shell was not available to this seat, so item 7 was verified statically (test existence,
  plan-to-case mapping, absence of skips/platform-gating, no new dependency or CI claim), not by
  running `npm test`. Nothing in the diff looks platform-fragile on Linux: temp-dir fixtures,
  `git` on PATH, and the `npm run check` green-gate stub follow the pattern of the ~15 existing
  integration suites, including the pre-existing reliance on an ambient git identity for the
  runner's own `commitStage` (the `-c user.name=f` overrides cover only fixture setup).
- The run-003 fixture's provenance could not be checked against its cited source, because
  `docs/designs/003-parallel-inference-lane/.pipeline.json` no longer exists in the tree (the
  design dirs were cleared on 9 Nov per ADR-006) and git history was not readable from this seat.
  The fixture reads as a genuine copy rather than a normalized reconstruction — it carries
  machine-specific absolute worktree paths and the reply-less independence-violation records the
  proposal cites — so §2's "golden fixture per era" pin looks real, but that is inference, not
  verification.
