# compatibility review r3

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r3.diff` (single editorial
confirmation pass, round 3). Inputs: `02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`,
the Revision log, and this seat's previous findings (none). I have no prior
findings, so per the ratification rule I reported only if a revision broke
something approved or made a cited section false; I additionally walked each
diff hunk against the compatibility checklist to confirm the edit is
language-only.

## Findings

None.

Hunk-by-hunk compatibility check (all edits are punctuation, word-order, or
line-wrap changes; no technical content moved):

- Context (`budget limit`, because`): comma insertion; run 004 escalation claim
  unchanged, matches ADR-005 Context.
- Non-goals (`at any scope`): identical to ADR-005 Alternatives heading
  ("Agent git access (any scope)"); no deferral changed.
- Reply field (`are added to` / `a list over either cap`): "either cap" is
  strictly more precise — it names both the 200-char endpoint cap and the
  64-pair cap; both caps remain pinned by Test strategy #1. Node floor
  (checklist 3) untouched: matrix row `Node ≥ 22.12` still covers
  `fs.renameSync`/`existsSync`/`path.posix` and `git cat-file -t`; no new
  native dependency introduced (checklist 4).
- Rule 5 (`to does not exist anywhere`): the `existsSync` case-insensitive-
  checkout guard (A10) and the parent-directory requirement are verbatim
  preserved; macOS row and Test strategy #12 still back it (checklist 1, 7).
- Rule 6 (`What remains is a matching`): case-folded disjointness and the
  case-only-move rejection are unchanged; `path.posix` segment semantics and
  the `\` rejection of rule 1 (checklist 6) are untouched by the diff.
- Wiring into retry (em-dashes → commas): failure classification
  (`invalid-reply: moves`, `moves-apply-failed`, `moves-rollback-failed`) and
  its Test strategy #6 pins unchanged.
- Apply (re-wrap, `as that of a rename error`, `leaving the preserved-worktree
  state`): rules 4–7 guard, reverse rollback, and EXDEV/mount-point
  behavior (matrix row "partial", Test strategy #5/#11) unchanged.

Matrix completeness (checklist 1): every row carries a status — Linux `yes`,
macOS `yes (unverified)`, Windows `not exercised`, linked worktree `yes`,
mount points `partial`, Node `yes`, git `yes`. No `unknown`, no empty cell.
The macOS `(unverified)` qualifier is a recorded r3 Revision-log decision with
evidence behind it (#12 contrast), not a new finding, and the diff does not
touch it. Platform test coverage (checklist 7): macOS → #12, linked worktree
→ #13, mount-point partial case → #5 (declared operator-run); Windows is not
claimed supported, so no platform test is owed. Deferred items remain
consistent with ADR-005's Decision and its read-only-git deferral; the diff
touches none of them. Checklist items 2 (OS input APIs) and 5 (keyboard
reality) do not apply to a file-move design and were unchanged by the diff.

## FYI (no severity)

- The diff is confirmed language-only: no compatibility-relevant clause
  (caps, case-folding, platform statuses, failure classes, Node/git floors)
  moved meaning. Any residual compatibility question belongs to the
  substantive r3 matrix entry already recorded in the Revision log, not to
  this editorial pass.
