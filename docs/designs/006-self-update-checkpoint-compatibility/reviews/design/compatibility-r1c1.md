# compatibility review r1 (dissent pass c1)

verdict: approve

Scope: `git diff da725a4 305aa36` on `02-proposal.md`, then the full current
document. Inputs: the proposal (incl. its Revision log),
`docs/adrs/006-self-update-checkpoint-compatibility.md`, my r1 findings.
No other review reports or transcripts opened.

## Checklist walk (compatibility)

1. Matrix statuses — all five rows carry a decision now: Linux `yes`, macOS
   `partial`, Windows `no (unverified)`, network-FS `partial`, Node floor `yes`.
   No `unknown`, no empty cell. My r1 major is closed; the Windows note also
   records *why* (no CI, POSIX-shape rules, POSIX-only repair commands), so the
   Deferred "Windows support verification" row is consistent with ADR-006, which
   claims nothing about Windows.
2. OS input APIs — not applicable: the change touches only Node `fs` primitives
   and gate predicates (`'wx'`, `rename`, `mkdir`); no X11/Wayland/Raw
   Input/HID path is involved. The primitive-level posture is justified by A5/A6.
3. Node floor — `>= 22.23.4` (verified at `tools/orchestration/package.json:30–31`,
   matching the matrix row); every API named (`'wx'`, `renameSync`, `mkdir`,
   `randomBytes`) is far below the floor. A11 holds.
4. Native modules — none; pure JS over `node:fs`. No silent native dependency.
5. Keyboard reality — not applicable; no layout, dead-key, IME or binding-name
   handling in this design.
6. File paths — `stagePathAllowed` receives repo-relative POSIX paths
   (`gates.ts:260–273` tests `^docs/` and `(^|\/)tests\/`), so the new
   `/`-segment rule matches how the sweep feeds the predicate, and it reuses the
   moves validator's vocabulary rather than new path concatenation. The
   case-folding half of the fix is mis-anchored — finding 1.
7. Platform test per supported claim — the only `yes` platform rows (Linux, Node
   floor) are covered by the named run: "the `tools/orchestration` test suite,
   run on Linux". That artifact exists (`tools/orchestration/tests/pipeline/`,
   `npm test` → `vitest run`). macOS and network-FS are `partial` and labelled
   untested; Windows is `no (unverified)`. No `yes` row lacks a named run.

## r1 closure check

- Windows `unknown` → **closed**. Status cell now carries a decision; the note
  text survived as asked, and the Deferred row states the row's status.
- macOS `yes` without a platform test → **closed** by the `partial` demotion
  (the option my fix offered), and the matrix points at Test strategy (Platform),
  which confirms it is untested in this run. Nothing else in the document still
  claims macOS support, so item 7 has no orphan.
- "Linux CI exercises all of the above" vs A12 → **closed** consistently in both
  directions: the Platform paragraph now names the suite and says the repo has
  no CI workflow, and A12's basis ("repo glob finds no `.github/workflows/`") is
  accurate — there is no `.github/` in this repo.
- Fence path derivation/case (r1 minor) → **substantially closed**: segments,
  case-insensitivity, and Test #1's `.PIPELINE.JSON` case plus the
  `.pipeline.json-notes.md` allow-case all now exist; the Windows note records
  the POSIX-shape inheritance. The cited precedent is wrong — finding 1.
- POSIX-only repair commands (r1 nit) → **closed** by the Windows row's own note;
  consistent while the row says `no (unverified)`.

## Findings

- [minor][new] §1 Fence — "compared segment-by-segment and case-insensitively
  the way the moves validator already compares these names (`moves.ts:134–147`)"
  — the cited function compares these very names case-*sensitively*: line 136 is
  `segment === STATE_FILE` and line 139 `segment === LOCK_FILE`; the only folded
  comparison in that block is `segment.toLowerCase() === '.git'` (line 142). So
  the precedent the design binds its case handling to does the opposite of what
  the sentence claims, and an implementer who copies moves.ts:134–147 verbatim
  produces a case-sensitive fence that fails Test #1's `.PIPELINE.JSON` case —
  the exact gap this revision was written to close. Fix: keep the behavior
  sentence, re-anchor it — "following the `segment.toLowerCase() === '.git'`
  pattern at `moves.ts:142`, widening the existing `segment === STATE_FILE`
  comparison at `moves.ts:136` rather than copying it" — and state that the fold
  applies on every platform, so on case-sensitive Linux it deliberately
  over-matches names like `.Pipeline.json` (which Test #1 already pins).

## FYI (no severity)

- §2 May refuse — "restore the committed copy (`git checkout HEAD -- …`) or
  start a fresh run (ADR-006:55–57)" is now verbatim the ADR's interim posture,
  which is the right advice for the schema-rejection half of that bullet but
  cannot help the unknown-`version` half: the committed copy carries the same
  version as the worktree copy. Whether the refusal message needs to branch per
  refusal cause is a correctness-seat question, not a compatibility one.
- §1 Fence — the authorship test keys on "the bytes `saveState` last wrote for
  it, kept in memory"; across a resume (a new process, sweep before the first
  save) the seeded value needs to be the loaded checkpoint bytes. Behavior
  question for the correctness seat; the compatibility matrix is unaffected.
- The matrix's `moves.ts:107–131` citation is accurate — that block is the
  "repo-relative POSIX shape, judged identically on every platform" rule — so the
  Windows row's path-shape note stands on a correct anchor.
