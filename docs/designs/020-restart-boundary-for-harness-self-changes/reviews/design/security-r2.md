# security review r2

verdict: approve

Pass type: editorial confirmation (round 2). Scope: the editorial diff at
`reviews/design/editorial-r2.diff`. Inputs: `02-proposal.md`,
`docs/adrs/020-restart-boundary-for-harness-self-changes.md`, the proposal's
Revision log, and my own previous findings (none carried into this pass).
`reviews/design/security-r2.md` did not exist before this write, so nothing
here is a re-raise.

## Findings

None. Every hunk in the editorial diff is language-only and leaves the
security-bearing text intact:

- `ADR 020` → `ADR-020` (Non-goals, Options, §1, Risks, Rollout, Deferred,
  Revision log): citation form only; no Decision or requirement reference moves.
- `in favor of` → `in favour of`, `behavior` → `behaviour` (Options, Deferred):
  spelling only; the Deferred deferral of `.opencode/agents/**` still matches
  ADR-020's "Not decided here" clause (adr:119–123), so the deferral stays
  consistent with the source document and is not re-opened.
- §2 "has imported and renders" → "already imports and renders the changed
  tree": repairs a broken verb pair and now matches A1's verified basis. Checked
  against code: `PROMPTS_DIR` resolves relative to the CLI module
  (`cli.ts:20` — `new URL('../../prompts/', import.meta.url)`) and
  `loadTemplate` reads from disk at stage entry (`prompts.ts:81–83`), so the
  restated claim is true, not merely grammatical. The residual "resume from an
  older checkout" exposure is unchanged and already carried in the Risks table.
- §3 bullet relabel (`Status output:`) and "the line that": no change to the
  exit-code mapping, and the status line still prints the stage and a file
  *count* rather than echoing model-authorable paths — the narrower surface is
  preserved.
- Risks row wording: unchanged mitigation content.

Security checklist re-checked against the touched sections' claims about code
(read-only inspection; every anchor the diff-adjacent text cites still holds):

1. Trust boundaries — boundary paths come from `git diff --name-only` output,
   repo-root-relative, and are model-authorable by origin; §2's writer-side
   posture stands.
2. Parsing — the window listing keeps `-z --no-renames` through `nulPaths`
   (`runner.ts:262–264`), the same NUL posture that keeps paths unquoted and
   unescaped (`runner.ts:247–259`); no evaluation, no decompression.
3. Path handling — `git` is always called as `spawnSync('git', args)` with an
   argument array (`runner.ts:186–192`, `moves.ts:54`), never a shell string;
   the ownership predicate documents that absolute, backslash and `.`/`..`
   spellings are rejected upstream (`gates.ts:554–559`).
4. Secrets/PII — `printable` strips control characters
   (`independence.ts:443–445`); boundary records carry repo-relative paths, not
   the host's absolute layout, and the gate-evidence credential/citation
   bounding (`runner.ts:3042–3075`) is untouched.
5. Dependencies — the design introduces none; the pinned `zod ^4.6.7`
   (`tools/orchestration/package.json:48`) is above the fixed version for the
   only open-in-the-wild zod advisory (GHSA-mrhx-c7xc-m3w5 / the
   oversized-constrained-collection memory exhaustion, fixed in 4.6.6), so no
   unfixed advisory blocks.
6. Privileges — unchanged by the diff; the pause adds a checkpoint field, a
   gate-log entry and a commit, all inside the runner's existing reserved-path
   rights.
7. Fail-closed — the terminal-save guard §2 borrows is real: a
   `CheckpointWriteFailure` on the terminal save restores the committed copy and
   escalates rather than pausing (`runner.ts:3401–3410`), and the exit-code
   widening keeps `2` meaning escalation exactly (`cli.ts:17`, `cli.ts:201–228`
   map `done→0`, other `→2`), so `3` cannot be confused with a stop-and-read
   condition.

## FYI (no severity)
- The editorial pass normalized two spellings toward British (`favour`,
  `behaviour`) while `behavior` stands at A6 and A13 — the document now mixes
  varieties, which is the editorial seat's call, not a security concern.
- §2 says the record "keeps the `boundedGateEvidence` bounding (first N paths
  plus the count)". The function is real (`runner.ts:3057–3075`) but it bounds
  gate *legs*, so the sentence names a posture rather than a callable — wording
  for the correctness seat; no security posture is lost.
