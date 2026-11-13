# correctness review r3c0 (ratification)

verdict: request-changes

Scope: `git diff c1b351d 92f71e0` over `02-proposal.md`, then the full current
document, checked against `docs/adrs/010-lightweight-plan-flow.md`, the Revision
log, and this seat's round-3 findings. Only the changed text is in scope;
findings are reported when a revision broke what was approved.

## Findings

- [major][new] Assumption register A12 — "no document-stage primary agent reads,
  writes or shells — `designer.md` and `researcher.md` both deny `transcripts/**`
  (read and edit) and `shell: "*"`" — the added clause is stated as **verified**
  and is false as written, and its cited basis covers only two of the four
  document-stage primary agents. `STAGE_BASES` (`stages.ts:115-150`) puts four
  primary agents in the document-stage class that `gates.ts:314` names
  ("research, proposal*, reviews, doc-sync"): `research/researcher`,
  `design/designer` (proposal, proposal-revise), `design/editor`
  (proposal-final, `stages.ts:122-126`) and `ops/doc-sync` (doc-sync,
  `stages.ts:150`). `ops/doc-sync.md:20-25` denies `shell: "*"` and then
  **allows** `shell: "git diff*"`, mode `primary`. So a document-stage primary
  agent does shell today, and the universal claim in a `verified` row is
  contradicted by the repo while `doc-sync.md` is not in the basis column. The
  same revision restated the claim in the body (next finding) and the Revision
  log records it as the r2 closure ("A12 carries the shape"), so the register is
  the surface a reviser or implementer relies on for the planner's shape. Fix:
  scope the clause to the agents the basis names and record the one exception —
  "no document-stage primary agent reads transcripts, writes transcripts, or
  shells beyond read-only git — `designer.md` and `researcher.md` deny
  `docs/designs/**/transcripts/**` (read and edit) and `shell: "*"`
  verbatim; `design/editor.md:4-22` is identical; `ops/doc-sync.md:23-25` is the
  one exception, an allowed `git diff*`" (add `editor.md`/`doc-sync.md` to the
  basis).

- [minor][new] The `plan` stage, planner bullet — "No document-stage agent has
  either privilege today." — the sentence added with the r2 closure asserts the
  same universal over all document-stage agents, and `ops/doc-sync.md:23-25`
  gives the doc-sync primary agent a shell allow (`git diff*`), so the privilege
  pair is not uniformly absent. The bullet's operative decision is unaffected —
  "designer's permission list verbatim" is the right shape and needs no change —
  so this is the comparative sentence, not the design. Same datum as the A12
  finding. Fix: "Designer, editor and researcher deny transcripts (read and
  edit) and shell; `ops/doc-sync` is the one document-stage agent with a shell
  allow (`git diff*`), and `planner.md` copies the designer list, not that one."

## FYI (no severity)

- Round 3's open minors/nits sit in text this revision left untouched, so this
  pass does not break them and does not re-open them as verdict findings: the
  `so`-clause basis in "Which knob binds" (pass backstop decided upstream at
  `states.ts:181-183`); "case is **not** normalized — as today" in Ownership
  (reserved-name matching is case-insensitive, Test strategy #7 scopes it);
  "ADR-010's Consequences records the widened surfaces" in Risks row 3 (the ADR's
  Consequences at `docs/adrs/010-lightweight-plan-flow.md:222-233` records the
  coder's widened surface, the shared `NNN` space, promotion overwriting the
  plan, and the plan's small-model hazards — not the plan stage's `docs/**`
  write surface); Test strategy #7's "**every** path under `docs/`" vs. the
  checkpoint denial; "no call site keeps a `*_SPECIALTIES` import" next to
  `states.ts:152`; Rollout 6's "≈ 9 sessions" component list that sums to 10
  with both escalation shapes saturating at 8; the deadlock stop vs.
  "every shape of the round it starts"; Options' "ownership-matrix and
  session-count grounds" summary of the ADR's five rejections.
- Checked and clean in the changed text: the two Goals/planning comma edits, the
  `light` → "light flow's full pass" and "starts with 4 sessions spent"
  rewordings, "and on `state.lastStage`", the "heavy flow" → "full flow"
  renames (they now match the escalation string's `flow: full`), "the latter
  rendered", "as a usage error at the config-read seam" (consistent with
  Selecting the flow's pre-lock posture and #23), and the merged Revision-log
  entries, which restate body text accurately — except the r2 entry's "the width
  claim deleted", which the new comparative sentence above partially restores.
