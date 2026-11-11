# Code review — compatibility, round 1 dissent pass c1

Diff range: `29a6bc14..HEAD -- ':(exclude)docs/designs'`, read against the full
current `02-proposal.md` and `docs/adrs/005-runner-executed-file-moves.md`.
Scope: verify the last revision answers my round-1 findings; report anything
else only if the changed code makes it true.

## Round-1 finding re-check

**Major (r1): `git ls-tree` pathspec glob-matching at the `treeMode` choke
point — RESOLVED.**

- The fix lands exactly at the single choke point: `treeMode`
  (`tools/orchestration/src/pipeline/moves.ts:66-73`) now passes
  `:(literal)${relPath}` as the pathspec. Both consumers — rule 4's `from`
  regular-file check (`:190`) and rule 5's `to` not-tracked check (`:203`) —
  go through it, so neither can borrow a glob-matched sibling's mode. A bare
  pathspec beginning with `!` (exclusion magic, another hole I did not name in
  r1) is equally neutralized: everything after `:(literal)` is literal.
- The fix's safety preconditions hold: rule 1 (`shapeIssue`,
  `moves.ts:110`) rejects any `:` in an endpoint before `endpointsIssue` runs,
  so no reply-supplied path can carry or interfere with pathspec magic, and
  the `:`-endpoint rejection is itself pinned (`moves.test.ts:301-307`).
- No new git floor, as the matrix row requires: `:(literal)` magic dates to
  git 1.9; the repo's runner already shells `git restore` (git 2.23+) at
  `runner.ts:265`, so the "git, any version this repo runs today" row stays
  true. The comment at `moves.ts:56-64` states this correctly.
- The requested pins exist, one per sub-case:
  - tracked `[`-bearing filename accepted by rule 4:
    `moves.test.ts:210-217` (`src/bindings[en].json`, paired with its glob
    siblings at `:92-94`);
  - untracked glob-ambiguous `from` rejected: `moves.test.ts:222-229`
    (`src/a?.ts` glob-matching tracked `ab.ts`);
  - untracked glob-ambiguous *directory* rejected: `moves.test.ts:234-242`
    (`src/b?.ts` glob-matching tracked `bb.ts`) — the case that previously
    defeated rule 4's file-only guarantee and the proposal's directory-move
    deferral.
  I verified each pin fails against the pre-fix bare pathspec: `a?.ts` matched
  `ab.ts` (mode 100644) and `bindings[en].json` matched nothing, so the tests
  genuinely pin the fix, not a vacuity.

## Other compatibility checklist items re-checked against the changed code

1. **Platform matrix**: no new OS-specific code; rule 1 stays platform-neutral
   and Windows-illegal names are pinned on Linux (`moves.test.ts:313-337`), as
   the Windows row promises. macOS path stays `yes (unverified)` behind the
   guarded split (`moves.test.ts:675-700`).
2. **Node floor**: `package.json:30-32` engines `>=22.23.4` matches the matrix;
   `moves.ts` uses only long-stable `spawnSync`/`lstatSync`/`realpathSync`/
   `renameSync` — nothing above 22.x.
3. **Paths**: endpoints joined with `join`/`posix.dirname`; the
   `:(literal)` prefix is git pathspec syntax, not filesystem concatenation.
4. **Guarded imports**: none added.
5. **Locale/layout**: non-ASCII endpoints pass through argv as bytes and the
   case fold is symmetric; nothing in the changed code breaks them.
6. **Native deps**: none introduced (`package.json` dependencies unchanged).
7. **Platform tests**: darwin half guarded by `runIf(process.platform ===
   'darwin')` with the Linux contrast running here; EXDEV guarded by the
   one-shot mount capability probe (`moves.test.ts:545-561, 626`) exactly as
   the matrix's "operator-run" row states; the linked-worktree round trip
   (#13, `moves.test.ts:705-722`) runs on this platform. The
   `reply-contract.test.ts` fault-injection matcher (`:46`) still hits the
   armed spawn after the `:(literal)` prefix because it matches with
   `includes('boom.md')` — the fix did not blind it.

Nothing in the revised code introduces a new compatibility finding; nothing I
approved in round 1 was broken by the fix.

## Verdict

approve — 0 blocker, 0 major, 0 minor, 0 nit open.
