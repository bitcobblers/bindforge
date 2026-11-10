# complexity review r1c2

verdict: approve

Ratification pass over `ce95f3b..f4f64f9` of `02-proposal.md`. Inputs: the diff,
the full current document, the Revision log, the source prompt
(`docs/prompts/audit-by-deeds.md`), and my own r1c1 findings. I re-checked the
three findings I carried in c1 first:

- **c1 finding 1 (derive `PATH_ACCESS_KEYS` from `ACCESS_FIELDS`)** — the
  allowlist literal and the Risks-row-2 promise ("Allowlist covers the key
  names the platform's tools actually use") are unchanged by the revisions.
  The finding remains open but was neither applied nor recorded as rejected in
  the Revision log, and the changed §1 text does not falsify it. Not
  re-reported as a finding under ratification rules; noted under FYI because
  the log is silent on it.
- **c1 finding 2 (test strategy #9's nine fake-client extensions)** — #9's
  text is unchanged, but the r1 §2 revision (fetch errors now emit a `gateLog`
  line in the run record, not just `console.warn`) gives those extensions a
  plausible purpose my c1 reasoning did not weigh: an unextended fake would
  now write into the gate log that existing tests pin the shape of. My c1
  claim "nine file edits buy nothing" is the statement made false by the
  revision — the design's nine edits are now the defensible choice. Withdrawn.
- **c1 finding 3 (nit: one enumeration route, not two)** — the revision goes
  the opposite way: §2 keeps the two-route choice and the Compatibility matrix
  adds a *third*. The c1 concern (route ambiguity → fake-client shape
  ambiguity) is now larger. Re-raised below as a regression.

Nothing in the diff breaks a blocker- or major-level decision I approved; the
Goals, Non-goals, Options, §3 seam shape, §4 pins, and the r0 Matcher decision
(allowlist, not payload scan) all survive intact.

## Findings

- [new][minor] Chosen design §1 (Matcher) — "instead of `stringValues(input)`,
  collect string values under an allowlist of access-naming keys, via a
  node-capped queue that drops the deepest nodes" — the r1 revision replaced
  the text I approved ("collect string values ... recursively, in any nesting
  depth") with a queue that silently drops nodes, and the cap's value appears
  nowhere in the document. Deferred now hands off "values beyond the §1 walk's
  node cap" as an accepted residual — a blind spot with no named bound cannot
  be reviewed, implemented, or pinned consistently, and no Goal, cost bullet
  (§2's cost bullet budgets fetches, not walk nodes), or Risks row names the
  input-size concern the cap defends against. It is also the only Matcher text
  change in the diff, and the Revision log carries no r1 Matcher entry for it.
  Fix: the smallest design that was already approved is a recursive collect
  over the existing `stringValues` traversal shape, keeping only values under
  allowlist keys — tool inputs are finite JSON trees and the current code
  scans them uncapped. If the cap stays, name the constant and its value (one
  inline constant, e.g. `MAX_WALK_NODES`) so the §3 pin and the Deferred row
  have a number to check, and add the r1 Revision-log entry saying so.

- [regression][minor] Compatibility matrix, row "OpenCode server/SDK without a
  children method" — "else SDK `session.list()` + `parentID` filter, else a
  direct `GET /session/:id/children` against the configured `baseUrl` with the
  client's auth headers" — the r1 revision grows my c1 nit from two routes to
  three, and the third is speculative machinery: it only fires if the pinned
  SDK exposes *neither* `session.children` *nor* `session.list`, yet A4's
  verified half already has the client exercised on `session.create/
  prompt/remove`, and A3 puts the `/session` listing server-side — a client
  SDK with session-create but no session-list is the assumption the row itself
  treats as impossible ("the walk always has a route"). Hand-rolling raw HTTP
  with auth-header handling outside the SDK adds a second integration surface
  to build, fake, and pin for a branch that cannot be reached unless the SDK
  assumption register is badly wrong — and if it were, step 1 should escalate,
  not route around it. Fix: drop the direct-GET clause from the row and keep
  the two SDK routes (my c1 nit preferred committing to `session.list` +
  `parentID` outright; two named SDK candidates decided at step 1 is the
  acceptable middle). §2's "implementation step 1 confirms which" and
  Test strategy #9's "the session-list method their flows need" both stop
  being ambiguous at three routes only if the matrix drops to two.

## Value check (checklist 7)

Proportionate. Against what the feature brings (killing three run-005 loss
modes the source documents with receipts), the design stays one hand-maintained
allowlist set, one async generator, one optional `StageHandle` field, one
prompt-append block, and prompt pins — no new package, no new dependency
(checklist 4: `@opencode/client ^2.1.4` already pinned), no config knobs, one
interface per concern, second half (`§4` prompt pins) already separable
(checklist 5). The two findings above are the only spots where r1 added
machinery (a cap, a third transport route) that no Goal or stated risk
requests.

## FYI (no severity)

- Revision-log hygiene: the diff edits §1 (node cap) and Risks/Deferred
  wording, but the log's only r1 entries cover Deeds-of-children and Retry
  feedback; there is no r1 Matcher entry, and my c1 allowlist-derivation minor
  is neither applied nor recorded as rejected. Editor/panel concern.
- §3: `failureNote: brief(printable(out.failure ?? 'no reply'))` truncates the
  note, but source §2 names only `printable()` sanitization and requires the
  note to carry the failure reason *and, for independence violations, the rule
  broken*; if `brief()`'s bound cuts mid-list, the rule name or the
  `(via child session …)` suffix could be the part that disappears — and the
  Deferred "no structured rule-broken field" entry leans on "the violation
  strings already name the rule" surviving that truncation. Correctness seat's
  call.
- §2 call site: the new short-circuit ("when it already returns violations,
  skip the descendant walk") interacts with the new tripwire ("...yet the walk
  enumerated zero descendants") — it is unstated whether a skipped walk counts
  as zero-enumerated (warn) or not-walked (silent) when the parent violates
  and holds a completed `subagent` part; Test strategy #5's new pin asserts
  the warn case. Correctness/test seat's call.
