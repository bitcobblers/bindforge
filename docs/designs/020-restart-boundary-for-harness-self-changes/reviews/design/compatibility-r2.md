# Compatibility review r2

verdict: approve

Scope: the editorial-r2 diff (`reviews/design/editorial-r2.diff`) over `02-proposal.md`.
Inputs: the proposal, the ADR, the Revision log, the diff. No prior compatibility
findings in this editorial phase, so this is a confirmation pass: does any edited
pass break a compatibility-relevant section?

## Findings

None.

## Checks run

- Diff is language-only: `ADR 020` → `ADR-020` (reference normalization, now
  matching the `ADR-006`/`ADR-003` style used throughout), `favor` → `favour`,
  `behavior` → `behaviour`, and two sentence reworks in §2 (resume paragraph)
  and §3 (summary/status bullets). The Compatibility matrix, the Node floor
  row, the native-modules posture, and the Test strategy's Platform basis
  paragraph appear in the diff only as context lines — unchanged.
- Matrix (checklist 1): every row still carries a status (`yes` / `partial` /
  `no (unverified)` / `yes` / `partial`); no empty cell, no `unknown`.
- Node floor (checklist 3): matrix row "Node `>= 22.23.5` (engine floor) ...
  (package.json:30–31)" — verified: `package.json:30–31` is exactly
  `"engines": { "node": ">=22.23.5" }`; the diff adds no primitive claim.
- File paths (checklist 6): the only semantically repaired sentence, §2 "any
  fresh process started from the run's worktree already imports and renders
  the changed tree (A1)", now matches A1 and checks out in code:
  `cli.ts:20` resolves `PROMPTS_DIR` relative to the CLI module
  (`new URL('../../prompts/', import.meta.url)` + `join`), and
  `prompts.ts:81–83` reads templates from disk at stage time via `join` —
  module-relative resolution and library path handling, not string
  concatenation. The reworked §3 status bullet leaves the `planStopStatusLines`
  citation (runner.ts:3580–3589) byte-identical.
- Platform test naming (checklist 7): unchanged — Linux-yes row is matched by
  the named `tools/orchestration` suite; macOS partial and Windows
  no (unverified) rows are explicitly recorded as not exercised.
- Deferred section: the edited `.opencode/agents/**` bullet still matches
  ADR-020's "Not decided here" clause (agent permission files); no deferral
  contradicts the ADR.
- Revision log: the only log change is the `ADR-020` normalization; no closure
  or rejection is altered by this diff.

## FYI (no severity)

- The r2 editorial pass normalized British spelling in the hunks it touched
  ("favour", "behaviour", matching "honour" elsewhere), but the assumption
  register keeps American "behavior" at A6 ("behavior-safe defaults") and A13
  ("staleness behavior"). Language-consistency concern for the editorial seat;
  no compatibility impact.
