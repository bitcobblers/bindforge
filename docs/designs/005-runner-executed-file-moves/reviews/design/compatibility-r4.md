# Compatibility review r4

verdict: request-changes

## Findings
- [major][new] Review scope (not a proposal section) — the scope named by the caller,
  `docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r4.diff`, does
  not exist at that path: a read of the exact path returns "File not found" and a
  targeted grep of the exact file path returns "Search path does not exist". (The
  containing directory was not listed, globbed, or grepped — review records are not
  this seat's surface.) The r4 editorial diff therefore never reached this seat, and
  no diff-scoped confirmation could be performed: with zero changed hunks in hand, I
  cannot establish that the r4 revisions left the compatibility surface intact — and a
  checklist item I could not evaluate counts as a finding, not a pass. My prior
  findings for this seat are none, so this is labeled `new` (scope defect, not a text
  defect). Fix: regenerate/emit the r4 editorial diff at the named path and re-run
  this single confirmation pass over it; the checklist verdict on the surrounding text
  is already recorded below, so only the changed hunks need to be re-checked.

## Checklist evaluation against the full current 02-proposal.md text (no open findings)
1. Compatibility matrix — every row carries a status: Linux `yes`; macOS
   `yes (unverified)`; Windows `not exercised`; linked git worktree `yes`;
   mount-point-spanning repo `partial`; `Node ≥ 22.12` `yes`; git `yes`. No
   `unknown`, no empty cells.
2. OS input APIs — the design touches no input path (no X11/Wayland/Raw
   Input/HID surface); its platform surface is filesystem plus git, which the
   matrix covers. Item vacuously satisfied.
3. Node version floor — matrix cites `Node ≥ 22.12
   (tools/orchestration/package.json:30-32)`; the change adds only
   `fs.renameSync`, `existsSync`, `path.posix` comparisons, realpath resolution,
   and zod schema pieces, all long-standing below the floor. No API above it.
4. Native modules — "no new runtime dependency: `fs.renameSync` plus the
   existing git helper" (Node row). No silent native dependency.
5. Keyboard reality — no input capture or binding-name handling in this
   change. Item vacuously satisfied.
6. File paths — rule 1 compares "repo-relative POSIX paths compared with
   `path.posix` segment semantics" (a library, not string concatenation); case is
   handled by the case-folded comparisons (rule 6) and the `existsSync`
   checkout-level guard (rule 5, A10); rule 1 rejects `\`, `:`, control
   characters, and Windows reserved names identically on every platform, which
   additionally disambiguates the `git cat-file -t <headBefore>:<from>` object
   string. The design adds no per-OS config-directory behavior.
7. Test strategy platform coverage — macOS (`yes (unverified)`) has #12's
   round trip plus the `notes.md` → `readme.md` contrast; linked worktree
   (`yes`) has #13; mount points (`partial`) has #5's operator-run `EXDEV`
   case; Linux runs the whole suite. Windows is not claimed supported, so its
   absence costs no platform test — rule 1 stays pinned on Linux in #2.

## Deferred items cross-checked against docs/adrs/005-runner-executed-file-moves.md
- Read-only git deferral matches the ADR's own "Deferred — read-only git in
  stage shells … deferred to a follow-up ADR"; the checkpoint/reply trust
  boundary is routed to ADR-006 exactly as the ADR's Consequences carve it out;
  directory moves and case-only renames are nowhere required by the ADR (its
  motivating payload was 23 file pairs). No deferral contradicts the ADR.

## FYI (no severity)
- ADR item 3's phrase "through its own git plumbing" versus the chosen
  `fs.renameSync` + existing `git add -A`: an ADR-conformance phrasing question
  for another seat — the choice is logged in the Revision log (r0, OQ1) and
  satisfies the ADR's operative requirements (runner-executed, before sweep and
  `commitStage`, one commit, rename detection intact).
