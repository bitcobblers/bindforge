# security review r1c4

verdict: approve

Scope: the dissent-pass diff `4cc9acc → 640d904` (blob ids of `02-proposal.md`), then the full
current document. Inputs: the proposal, `docs/adrs/010-lightweight-plan-flow.md`, the Revision
log, my r1 findings. Checklist: `references/security.md` (path handling, least privilege,
fail-closed, untrusted input). No report or transcript was opened.

## My r1 findings — both closed

- **major (Ownership: `plan`'s code gate is `docs/**`, not the run's directory) — closed by the
  second option my `fix` offered.** The section now states what the gate does and names the
  consequence: "**not** the run's directory: no run-scoped branch, no new predicate. A light
  `plan` that rewrites `docs/adrs/**` (the decision record) or a sibling run's design directory is
  therefore **not** reverted — that is an accepted widening (Risks, Ownership row), pinned as
  **allowed** in #7, not a confinement this design claims." Risks row 3 carries the same
  disclosure, #7 pins the cross-run and `docs/adrs/**`/`docs/workflows.md` cases as allowed, and
  #18 asserts a `docs/adrs/**` write is **kept** (assert-not-reverted) in a real sweep. The
  claim-vs-gate mismatch that made my r1 finding a major is gone, and the Revision log's rebuttal
  ("the design says exactly that ... not a confinement asserted without enforcement") is sound.
  I verified the hard fence the widening cannot reach: `namesReservedSegment` (`types.ts:195-208`)
  is segment-wise and case-insensitive and runs first (`gates.ts:301-303`), so `plan` cannot own
  the checkpoint, a `saveState` tmp, the escalation marker or the lock at any depth or case, and
  #7 asserts exactly that for `plan` in both flows.
- **minor (the intake site behind the `docs/` prefix rule was uncited) — closed.** A28 names the
  site and is accurate against the code: the predicate's only two callers are
  `enforceOwnership`'s filter (`runner.ts:279`, fed by `nulPaths(diff-tree … -z --name-only)` over
  runner-authored commits at `runner.ts:261-269` plus `dirtyFiles` = `status --porcelain=v1 -z
  --untracked-files=all --no-renames` at `runner.ts:225-229`) and `moves.ts`'s Rule 3
  (`moves.ts:155-162`), whose Rule 1 (`moves.ts:107-131`) rejects absolute, backslash, colon,
  control-char, empty-segment and `.`/`..` shapes. Git reports the real filesystem path, so a
  `docs/../src/x.ts` write arrives as `src/x.ts` and is denied by the predicate — the traversal
  fails closed rather than passing the `inDocs` prefix, which was my concern. Ownership cites A28
  in place of "(A8/A9)", and #18 asserts the spelling lands as git's real path. Because intake is
  filesystem-derived, a planted symlink inside `docs/` is caught too: the real path it modifies is
  reported and reverted.

## Findings

- [minor][new] **Risks — Ownership row** — "the `docs/**` write surface is pinned as **allowed**
  there and recorded here, not claimed as a confinement no code performs; **ADR-010's Consequences
  records the widened surfaces**" — the acceptance this disclosure rests on is attributed to the
  ADR, which does not make it. ADR-010's Consequences records exactly one write-surface widening:
  "`stagePathAllowed` … gains a flow parameter, widening the **coder's** write surface in one
  flow"; its only statement about `plan` runs the other way ("the `plan` stage inherits every
  small-model hazard of the stages it replaces without the design panel's five pairs of eyes, so
  the two code seats are the whole safety net"). This design's own Rollout 4 cites the same clause
  at the narrow scope — "ADR-010's Consequences records the coder's widened surface" — so the
  document cites one ADR sentence at two scopes, and the broad reading is the one that closes the
  question of who accepted a plan rewriting the decision record. As written, the run's record
  implies the ADR pre-accepted the flow's only design-writing stage having a repo-wide `docs/**`
  write surface; it did not. Fix: scope the citation in that Mitigation cell to the coder's test
  surface (Rollout 4's wording) and state the `plan` `docs/**` widening as accepted **by this
  design's Risks row** — the record that actually accepts it. One clause; no code change.
- [nit][new] **Ownership — The `plan` stage** — "A light `plan` that rewrites `docs/adrs/**` (the
  decision record) or a sibling run's design directory is therefore **not** reverted" — the
  enumeration names the two out-of-run cases and skips the one that touches this run's own audit
  chain: the run's source document sits inside the allowed surface (`docs/prompts/**`,
  `docs/adrs/**`), so a light `plan` can rewrite its own task record, frontmatter included —
  including the `flow:` key that Selecting the flow reads once per entry and that Promotion
  re-reads on the `--force` re-run, which is the very key the escalation names as the promotion
  recipe. It is disclosed in aggregate (Risks names `docs/prompts/**`, #7 says "**every** path
  under `docs/`"), so nothing is hidden and the operator's `--force` is still the gate; the
  named-list is just silent on the highest-integrity path in the set. Fix: add the run's own
  source document (frontmatter included) to that Ownership sentence and one example path
  (`docs/prompts/<this-brief>.md`) to #7's allowed list.

## FYI (no severity)

- **State graph delta** — "The light suffix is appended at **one** site — `finalizeEscalate`
  (`runner.ts:2845`), which already computes `escalationPhase(state)`" combined with Reporting's
  "`plan` joins `CODE_STAGES` in `report.ts` **for reporting only**" means `escalationPhase`
  returns `code` for a plan stop. The sentence names the gate as the stage
  ("a code-phase escalation from `implement`/`fix`/`code-review`"), which is the correct,
  non-overlapping rule; an implementation that gates on the phase value it "already computes"
  appends the promotion suffix to the oversized-plan stop, whose string "already names the flow
  and the promotion recipe". Worth pinning as `lastStage`, not phase — that is the correctness
  seat's call.
- **Options considered** — "ADR-010's Rejected section … each rejected there on the
  ownership-matrix and session-count grounds": the ADR rejects `lane` naming on word-collision
  grounds and "no plan artifact" on audit-chain grounds, so the summary attributes grounds the
  ADR does not give. Accuracy, not security.
- Checklist items with nothing to evaluate: no new dependency (item 5), no secrets/PII surface
  (item 4). Parsing of the untrusted `flow:` value fails closed — an unreadable value throws
  pre-lock, absence defaults to `full` (the stricter flow), and `--stage` is checked against
  `flowOf(state)`, which closes the route from a light checkpoint to `doc-sync`'s repo-wide
  `docs/**` write. #6/#17 pin those.
