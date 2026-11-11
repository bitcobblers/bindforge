# Code review — compatibility — r1c0

Seat: compatibility. Scope: `git diff f523ce7e...HEAD -- ':(exclude)docs/designs'`.
Inputs: that diff, `02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`,
`03-build-notes.md` (per skill step 1). No prior findings on this seat.

## Verdict: request-changes

1 major finding open. Checklist items otherwise pass; each was evaluated.

## Checklist results

1. **Platform code vs the Compatibility matrix — pass.** The change adds no
   OS-specific code, which is what the design wants ("rule 1 rejects ...
   identically on every platform, pinned on Linux in #2" — pinned at
   `moves.test.ts:236-276`). Every "supported" row has a real code path and a
   matching test: linked worktree (`moves.ts` reserved-`.git` segment rule +
   realpath containment; `moves.test.ts:567` round-trips a move in a
   `git worktree add` checkout and pins the `.git` pointer file byte-identical,
   running unguarded on Linux); mount points (`applyMoves` catch → reverse
   rollback → `moves-apply-failed` at `moves.ts:331-348` +
   `runner.ts:1119-1141`, exactly the "partial" row); Node row — see item 2;
   git row — see finding 1 (the row stays true only after the fix). The macOS
   row's `yes (unverified)` is honest: no macOS-specific branch exists, the
   guard is FS semantics (`lstatSync`), and the contrast test pair
   (`moves.test.ts:530-562`) is platform-guarded.

2. **Node floor — pass.** `package.json:30-32` engines is raised to
   `>=22.23.4`, exactly the matrix row and `03-build-notes.md:21`; that is the
   repo's only Node-floor declaration. Every API the diff touches predates the
   floor: `spawnSync`, `lstatSync`, `realpathSync`, `renameSync`,
   `posix.dirname`, `join`/`relative`/`isAbsolute`, `codePointAt`.

3. **Paths — pass on construction, broken on literal-name assumption.** All FS
   endpoints are `path.join` joins with argv-array spawns (no shell, `--`
   separates revs from paths); prefix tests compare folded POSIX strings that
   rule 1 has already normalized (no `\`, no empty/`.`/`..` segments), so the
   `${shorter}/` concat in `nests` is a string-domain comparison, not path
   building. Case-sensitivity assumptions are the documented ones (rule 6 fold
   every platform; rule 5 disk check as the case-insensitive-checkout guard,
   pinned by the #12 contrast). What breaks is the *literal* lookup assumption
   behind `git ls-tree <headBefore> -- <from>` — finding 1.

4. **Guarded platform imports — n/a.** No platform-only module is imported;
   nothing can break load on another platform.

5. **Locale/layout reality — pass for non-ASCII, fails for glob metacharacters.**
   Non-ASCII endpoints survive intact end to end: zod/JSON checkpoint, argv
   (UTF-8), byte-wise git tree matching, `path.join`, `rename(2)`; `fold()` is
   `toLowerCase()` — locale-independent (no `toLocaleLowerCase` Turkish-I
   divergence between call sites) and Unicode-aware. Windows-illegal names the
   matrix does claim to reject (`\`, `:`, reserved names, trailing dot/space)
   are rejected identically on every platform. Filenames containing `*?[ ]` do
   not survive — finding 1.

6. **Native dependencies — pass.** None introduced; runtime deps stay
   `@opencode/client`, `yargs`, `zod`.

7. **Plan-named tests vs platforms — pass.** #12 macOS half is
   `runIf(process.platform === 'darwin')` and its Linux contrast half runs on
   the reference platform (guarded, not skipped-everywhere — and the matrix
   already declares macOS unverified/operator-run). #13 worktree test runs on
   Linux unguarded, matching its "yes" row. #5's EXDEV case is capability-guarded
   (`linux && getuid()===0` plus a try/catch early-return,
   `moves.test.ts:496-515`) — precisely the matrix's "operator-run since
   mounting a tmpfs needs capability the vitest suite lacks". No
   platform-claimed plan test is silently skipped on the platform it claims.

## Findings

### [major] `git ls-tree` pathspecs glob-match; metacharacter filenames break rules 4–5 on every platform

- **Location:** `tools/orchestration/src/pipeline/moves.ts:57-64` (`treeMode`,
  called by rule 4 at :181 and rule 5 at :194), in combination with rule 1
  (`shapeIssue`, :99-122), which does not reject `*`, `?`, `[`, `]`.
- **Problem:** Git pathspecs apply fnmatch/wildmatch **by default** (that is
  why `git ls-tree HEAD 'src/*.ts'` lists a directory); `--` does not disable
  it. The validator needs an exact-path lookup and gets a glob match on the
  reference platform and every git version the matrix's "git, any version" row
  covers. Consequences, all platform-invariant:
  (a) *fail-safe but costly:* a tracked file legitimately named e.g.
  `bindings[en].json` (legal on Linux and macOS) fails rule 4 as "not a regular
  file tracked at the stage's base commit" — the design's file-only rule rejects
  its own legitimate name, and per A3 each such rejection burns code budget;
  (b) *fail-open:* rule 4 checks the mode of whatever the glob matches, not the
  literal `from`. An untracked `from` whose name glob-matches a tracked sibling
  (e.g. on-disk `aX.ts` untracked, `a.ts` tracked, `from: 'a?.ts'`) passes the
  tracked regular-file gate, and an untracked **directory** with such a name
  passes too — `renameSync` then moves the whole directory, silently defeating
  the proposal's file-only rule and its directory-move deferral; rule 5's
  "not tracked at `headBefore`" check has the same mis-match in the reject
  direction. The proposal's rule 4 text ("`git ls-tree <headBefore> -- <from>`
  mode 100644/100755 — this is the file-only rule") assumes the lookup is exact.
- **Fix:** make both pathspecs literal at the single choke point in `treeMode`:
  prefix the pathspec with `:(literal)` (unambiguous here because rule 1
  already rejects any `:` in an endpoint; magic signatures date to git 1.9, so
  the "no new git floor" matrix row stays true), or set
  `GIT_LITERAL_PATHSPECS=1` in the `spawnSync` env. Add `moves.test.ts` pins:
  a tracked `[`-bearing filename accepted by rule 4, and an untracked glob-
  ambiguous `from` (and a directory of that shape) rejected — the file-only
  gate must evaluate the literal name.

## FYI (no severity, outside this seat)

- `moves.test.ts:524` keeps the plan-named rollback-rename-failure case as an
  unconditional `it.skip` with a documented non-constructibility argument; its
  outcome class is pinned at `reply-contract.test.ts:194-196`. It is not a
  platform claim, so checklist item 7 is not violated — the testing seat may
  want to weigh the standing skip.
- `fold()` case-folds but does not normalize Unicode: NFC/NFD variant names
  are not folded as duplicates (rule 6) — on case-insensitive checkouts rule 5's
  disk check still catches the collision, and the proposal's rule 6 wording
  promises only case folding, with the deferral list covering the case-variant
  family. Relevant only to the unverified macOS row if an operator exercises it.

## Notes on evaluation

Every checklist item was evaluable from the diff; none counts as un-evaluable.
The diff was reviewed at HEAD across all source, prompt, and test paths named
by the range (`moves.ts`, `runner.ts` wiring at :681-696, :950-1004,
:1107-1158, `types.ts:32-67`, engines bump, the four prompt sentences, and the
new/extended test files).
