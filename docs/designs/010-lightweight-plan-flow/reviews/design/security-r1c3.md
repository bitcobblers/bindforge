# security review r1c3

verdict: request-changes

Scope: the c3 diff of `02-proposal.md` (`24b804a` → `4cc9acc`) plus the full
current document. Ratification read: only the changed text, the Revision log,
the ADR, and my own r1 security findings.

My r1 finding is **closed in part**: Test strategy #7 no longer asserts
ingestion behavior the predicate unit test cannot observe (it now reads "a
POSIX repo-root-relative path gets the verdict of its own spelling, and case
variants of `tests/`/`docs/` keep today's case-sensitive regex verdict"), and
the Ownership sentence drops the case-normalization claim ("case is **not**
normalized — as today"). On linux, the only supported platform
(Compatibility matrix: Windows / macOS → `no`), a case-sensitive `TESTS/`
spelling is a genuinely distinct path, so the widened test-ownership rule has
no case-variant bypass on the tested surface. That part is settled and is not
re-raised. The citation half of the finding is re-raised below as #2.

New in this round: the Ownership rewrite changes the code-gate verdict for the
new `plan` stage, and that is the section my r1 approval covered.

## Findings

- [major][new] Ownership — the security spine, widened in one flow — "the new
  name `plan` is governed by the **document-stage** rule in both flows —
  `return inDocs`, i.e. `inDocs = /^docs\//` (`gates.ts:304,315`), any path
  under `docs/`, the rule that lets `doc-sync` write `docs/workflows.md`;
  confining `plan` to the run's own directory is template and agent-config
  work, not predicate work." — The r1 text I approved put the confinement in
  the predicate ("writes confined to the run's `docs/designs/NNN-<slug>/`
  paths, `namesReservedSegment` first"); the revised text moves the code-gate
  verdict to "any path under `docs/`" and relocates the confinement to
  template and agent config — the layers this design itself declares advisory
  ("The gate is code — agent permissions and skill text are advisory (A13)",
  Options considered). Read as written, the ownership sweep will not revert a
  light `plan` write to `docs/adrs/**` (the decision record),
  `docs/prompts/**` (other runs' briefs), `docs/workflows.md`, or another
  run's `docs/designs/**` documents. The ADR's accepted widening is the
  coder's test paths (Consequences), not a docs-wide write surface for the
  flow's only design-writing stage — a stage the ADR says "inherits every
  small-model hazard of the stages it replaces" with the two code seats as
  the whole net; in the full flow the same rule is carried by `doc-sync`, a
  late stage whose diff a design panel has already read, and the light flow
  runs no `doc-sync` at all. The security tests bless the widened rule
  without asserting the denies that matter: #7 checks only "`plan` allows
  `docs/designs/**` and denies `src/**` in both flows (the `docs/**` rule,
  not the run's directory)"; #18 checks only "a light `plan` that writes
  `src/**` is reverted to its document paths". Fix: choose one, do not keep
  the claim and drop the enforcement. (a) Make the code gate equal the stated
  confinement — add a run-scoped branch to the document-stage rule for `plan`
  (`inDocs && inside the run's `docs/designs/<slug>/` prefix), replacing the
  "template and agent-config work, not predicate work" clause — and extend #7
  with the `plan` denies (`docs/adrs/**`, `docs/prompts/**`,
  `docs/workflows.md`, a sibling run's design directory) and #18 with a
  `plan` write to `docs/adrs/**` being reverted. (b) If `plan` on all of
  `docs/**` is what the author wants, state it as an accepted widening in
  Risks beside the coder's widened test surface, and pin it as allowed in #7,
  so the design stops asserting a confinement no code performs.

- [minor][re-raised] Ownership — the security spine (and Test strategy #7) —
  "Paths reach the predicate repo-root-relative and POSIX-normalized (A8/A9),
  so the `flow` parameter parses no path; case is **not** normalized — as
  today." — The #7 clause is fixed; the premise it rested on is unchanged and
  is still cited to A8/A9, which record the predicate's shape (A8:
  `namesReservedSegment` runs first, `TEST_PATH_RE` denials) and that move
  validation reuses that predicate (A9). No register row names the intake site
  that feeds `runner.ts:279` and `moves.ts`, and no behavior is stated for a
  path that arrives non-canonical (checklist 3, 7). The gap is sharper than in
  r1 because the allow rule is now a prefix regex: an un-normalized
  `docs/../../docs/adrs/x.md`, or `docs/../src/x.ts`, matches
  `inDocs = /^docs\//`, clears `namesReservedSegment`, and is permitted by a
  predicate that intended a path inside `docs/` — a fail-open traversal, not a
  clean rejection. Fix: add the verified assumption row naming the path-intake
  site (paths reach the predicate repo-root-relative and POSIX-normalized,
  absolute and `..` forms denied there), cite that row in place of
  "(A8/A9)" in the Ownership sentence, and add the traversal case to the
  integration sweep that drives the real code, #18.

## FYI (no severity)

- Reporting — "`plan` joins `CODE_STAGES` (A15)" — `plan` is a code-phase
  stage for escalation reporting while Ownership governs it by the
  **document-stage** ownership rule. Two classifications of one stage name in
  two subsystems; correctness/compatibility seat's call.
- Budget resolution — "`maxReviews.code: 6` is the ceiling the backstop
  coincides with, not a string an operator reads" — with the session cap never
  printing, every light code-phase stop is a pass/round backstop record. The
  diff's three-suffix coverage (`states.ts:183`, `187-194`, the runner's
  round-entry record) is what keeps each of them naming the flow; correctness
  and performance seats own the arithmetic.
- Deferred — "if it lands it counts path-shaped tokens through a **new**
  extractor (the ownership predicate tests one complete path; it extracts
  nothing from prose)" — the correction is honest and consistent with the ADR's
  two deterministic stops; no security content.
- Checklist 5: no new dependency is named anywhere in the design (A20), so no
  advisory/CVE lookup applies. Checklist 4: nothing in the diff sends data off
  the machine; the escalation and status strings carry flow name, seat names
  and line counts only.
