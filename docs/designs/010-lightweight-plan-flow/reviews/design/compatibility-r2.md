# compatibility review r2

verdict: request-changes

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial pass over
`02-proposal.md`). Inputs read: the diff, `02-proposal.md`,
`docs/adrs/010-lightweight-plan-flow.md`, the proposal's Revision log, and the
one repo file the changed text cites as its shape (`design/designer`).

## Findings

- [major][new] The `plan` stage — "denies `.pipeline.json`, reads and writes
  `transcripts/**`, and allows shell." — the editorial rewrite inverted a deny
  list into two grants. The pre-edit sentence read "denies `.pipeline.json`,
  reading/writing `transcripts/**`, and shell", which is exactly the
  `design/designer` shape the same bullet claims to copy (`.opencode/agents/design/designer.md`:
  `edit`/`read` on `docs/designs/**/transcripts/**` → `deny`, `shell` `*` →
  `deny`). After the edit, the new `design/planner` agent reads and writes the
  run's transcripts and has shell. On my checklist this is a compatibility
  defect, not only a wording one: it contradicts A20 and the Compatibility
  matrix row "No new platform surface: a stage, a seat list, a config record",
  and it undercuts the Windows/macOS row's claim that "nothing here adds a Unix
  dependency" — an agent with shell runs platform binaries at a stage the
  design says adds none, and the row's "not a shell-out" argument was written
  against the pre-edit design. Fix: restore the deny reading in one clause —
  "`mode: primary`, edits `docs/designs/**`, and denies `.pipeline.json`,
  reading and writing `transcripts/**`, and shell" — keeping the copy's shape
  and leaving the matrix's no-new-surface claims true. (If shell is genuinely
  wanted for `plan`, that is a new capability: it needs to be named in A20 and
  in the Windows/macOS row, not smuggled into a comma list.)

## Checklist coverage

- 1 Compatibility matrix — every row carries a status (`yes` / `no` /
  `yes (no-op)`); no `unknown`, no empty cell. The diff touches no matrix row.
- 2 OS input APIs — not applicable: the design declares no input surface; the
  matrix states the linux-only tested surface explicitly.
- 3 Node floor — named in A20 and in the matrix (`>= 22.23.4`, the package
  `engines`), with an explicit "older than the floor → no" row; the diff does
  not add an API above it.
- 4 Native modules — none claimed; the design adds a stage, a seat list and a
  config record. The one silent platform dependency the diff introduces is the
  planner's shell grant above.
- 5 Keyboard reality — not applicable to this design.
- 6 File paths — Ownership + A28: paths arrive from git plumbing,
  repo-root-relative with POSIX separators, no string concatenation, absolute
  and `.`/`..` shapes rejected by `moves.ts`; case is deliberately
  case-sensitive "as today". Unchanged by the diff.
- 7 Platform test per supported row — Test strategy #21 runs the suite on
  linux at the `engines` floor and states that no macOS/Windows job exists, so
  those rows read `no`; the only supported platform row has a named test.

Terminology pass: the "heavier flow" → "full flow" replacements (The `plan`
stage, Risks row 6) are consistent — no dangling "heavier/heavy" reference
remains, and both escalation strings quote `flow: full`, so the Risks row's
"The escalation strings name the full flow" is now accurate. The Rollout 6 and
Budget-resolution reflows preserve the 4–6 band and the spent-4 round-entry
arithmetic.

## FYI (no severity)

- The shell/transcripts inversion is primarily a security-spine finding (the
  ADR names `stagePathAllowed` as the spine and the transcripts are the run's
  audit record) — the security seat owns it; I report it only for the matrix
  claims it falsifies.
