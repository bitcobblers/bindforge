# compatibility review r2

verdict: approve

Scope: ratification pass over `reviews/design/editorial-r2.diff` (9 hunks, language-only).
Inputs: `02-proposal.md`, `docs/prompts/audit-by-deeds.md`, the Revision log, and my r1
findings (none — editorial phase is a single language pass per round).

## Findings

None. No hunk in the editorial diff touches compatibility-bearing text, and none makes a
section the compatibility seat cited at r1 false. Walk of each checklist item against the
changed text and the proposal as it now stands:

1. **Compatibility matrix** — untouched by the diff. Every row carries a status; no
   `unknown`, no empty cell. The one diff hunk inside the document near the matrix (Risks
   row "the attempt is audited parent-only") is grammar-only and preserves the A10
   partial-audit posture the matrix and §2 state.
2. **OS input APIs** — not applicable to this design (no input-capture surface); the
   matrix's justification ("the audit is pure string matching over server records, so no
   OS dependency is added") is unchanged by the diff.
3. **Node version floor** — matrix row "Linux, Node ≥ 22.12.0" unchanged; the diff adds
   no API surface, and the constructs in the cited design text (async generators, `Set`
   allowlist) are long below the floor.
4. **Native modules** — untouched; the only dependency named anywhere in the diff's
   context is the already-pinned `@opencode/client`, no new native dep.
5. **Keyboard reality** — not applicable; unchanged.
6. **File paths** — the diff's path-shaped text (`gateLog`, `brief(printable(...))`,
   runner.ts citations) is unchanged in substance; no path construction introduced.
7. **Test strategy vs matrix** — unchanged. The matrix claims only Linux as supported,
   and Test strategy #9 names the Linux CI platform check; no supported row lost its
   test.

Hunk-by-hunk semantic check (editorial rewrites can still flip meaning):

- Goals "never evidence from their input text" → "never judged on their input text":
  preserves the source §1 rule and the subagent/`subagent: []` design; no drift.
- Non-goals "re-affirmed rejected" → "rejection re-affirmed": same decision, matches
  Revision log A11 entry.
- Depth-1 rejection "for the price of zero extra machinery" → "closing it costs no extra
  machinery": this removes an ambiguity that could have read as the *defect* being free;
  the new wording matches the cost claim in §2 ("one children call per session"), keeping
  the rejected-option rationale consistent with the chosen design.
- Fetch-error bullet "is caught, logged" → "is caught and logged": no change to the
  caught-and-continue posture.
- Retry bullet "and `handle.resume` is absent and `failureNote` is set" → "if ...":
  fixes a run-on conditional; condition unchanged.
- Validation posture and Rollout "stages" plural: language only.

## FYI (no severity)

- The Depth-1 rejection rewrite quietly strengthens internal consistency: the cost figure
  in Options now reads in the same direction as §2's per-session cost line. No action
  needed; noted for the correctness/clarity seats only as confirmation, not a change.
