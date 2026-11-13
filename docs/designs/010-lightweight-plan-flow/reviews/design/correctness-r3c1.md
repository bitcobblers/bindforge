# Correctness review r3 (dissent pass c1)

Scope: `git diff 92f71e02b5d66c6da627a93779d50ed5decc402d 4635f0dc4efca2cf5a04189d48c3674ddcf07b50`
(both blobs of `02-proposal.md`), then the full current document. Inputs: the proposal,
`docs/adrs/010-lightweight-plan-flow.md`, the Revision log, my r3 findings.

verdict: approve

## Prior findings — closed for real

- r3 major (A12 document-stage universal) — **closed, verified in the repo**. The row now
  enumerates the four agents and names the exception. Checked each cited file:
  `researcher.md:4-22`, `designer.md:4-22`, `editor.md:4-22` each deny `edit "*"`, allow
  `docs/designs/**`, deny `docs/designs/**/.pipeline.json` (edit), deny
  `docs/designs/**/transcripts/**` (edit **and** read), deny `shell: "*"`;
  `doc-sync.md` denies the same three (11-19) then denies `shell: "*"` (20-22) and allows
  `git diff*` (23-25) — it is the one exception among the four. `stages.ts:114-151` puts exactly
  those four agents in the document stages (`research`→research/researcher,
  `proposal`/`proposal-revise`→design/designer, `proposal-final`→design/editor,
  `doc-sync`→ops/doc-sync), and the basis range `stages.ts:101-150` now reaches the doc-sync row
  (150), which the old `101-138` cut off.
- r3 minor (planner bullet's comparative sentence) — **closed**. "No document-stage agent has
  either privilege today." is gone; the bullet now reads "`design/designer`'s permission list
  verbatim (`designer.md:4-22`, shape in A12) — copied, not re-derived, and `ops/doc-sync`'s
  `git diff*` allow is not part of it", which is exactly what the files say (designer.md's
  permission block ends at line 22, so "4-22" is the whole list).

## Anti-hallucination gate

Three of the register's URLs fetched: abseil SWE-book ch. 9 (live; "one LGTM", "most code reviews
at Google are reviewed by precisely one reviewer", "Small changes should generally be limited to
about 200 lines" → A21 ✓), smartbear peer-review page (live; "lightweight code review takes less
than 20% the time of formal reviews and finds just as many bugs" → A21 ✓), google eng-practices
small-cls (live; "The number of files that a change is spread across also affects its 'size'" →
A22 ✓).

## Changed-text verification (passes)

- Budget resolution: "a caller-supplied `opts.cfg` is validated **pre-lock** — `validated` runs on
  it in `runPipeline`/`resumePipeline` once the flow is known, before `acquireLock` and
  `freshState`, the posture of `assertForceableStage`" — the posture is real: `assertForceableStage`
  is called at `runner.ts:2086` before `acquireLock` at 2089 (`resumePipeline`: 2137 before 2139),
  `deriveIdentity` is 2070, `freshState` 2091, so a validation inserted after the parse sits
  pre-lock on both entries. Feasible as written.
- Compatibility row and Test strategy #17 now both carry the `reviewSeats.code: []` pre-lock case;
  they agree with Budget resolution ("A violation throws as a usage error") and with #23's scoping
  (`code` non-empty for every flow, `design` under `full` only — light's `stages` list runs no
  design phase).
- A12's non-universal clauses re-checked: no `plan` in `STAGE_BASES`; `codereview/code-correctness`
  and `codereview/code-testing` exist; `prompts/code-review.md` exists;
  `.opencode/skills/code-review/references/{correctness,testing}.md` exist.

## Findings

- [minor][new] Assumption register A12 — "`doc-sync.md:23-25` is the one exception, an allow for
  `git diff*`" with basis `.opencode/agents/ops/doc-sync.md:20-25` — the row's universal part
  asserts that `doc-sync` **also** denies `.pipeline.json` and `transcripts/**` (read and edit),
  and those entries are at `doc-sync.md:11-19`; the cited range 20-25 covers only the `shell: "*"`
  deny and the `git diff*` allow, so the basis supports the exception but not the denials it is
  cited for. Fix: widen that one basis entry to `doc-sync.md:4-25` (the row's whole permission
  block), leaving the sentence as it is.
- [minor][re-raised] Assumption register A12 — "all four document-stage primary agents —
  `researcher`, `designer`, `editor` (`proposal-final`), `doc-sync`" — the claim is true for the
  four named, but "document-stage" is the document's own name for the `gates.ts:314` class
  (`return inDocs`), which that line labels "research, proposal*, reviews, doc-sync". Read in that
  sense the census is short by the ten review-seat agents, and every one of them is
  `mode: primary` **with** a shell allow (`review/design-*.md:29-31` allows `git diff*`;
  `codereview/code-*.md:26-30` allows `git diff*` and `git log*`), so the sentence's "one
  exception" reads as a class census that the repo contradicts. Fix: name the class by its stages,
  not by the gate's label — "the four agents of the document-writing stages (`research`,
  `proposal`, `proposal-revise`, `proposal-final`, `doc-sync`) deny ... ; `doc-sync.md:23-25` is
  the only shell allow among them; the review seats are separate agents and shell read-only git" —
  which keeps the planner's shape argument (copy `designer.md`, no shell) intact.

## FYI (no severity)

- Compatibility: the ADR's Context says a brief pays "the identical 13-stage path" while the
  proposal says "the twelve `STAGE_BASES` stages" (`STAGE_BASES` has exactly 12 rows); the Revision
  log already records the twelve-row reading, so the mismatch is the ADR's framing of sessions as
  stages.
- Compatibility: ADR-010 calls a bad `flow` value "a source-gate denial"; the design delivers a
  pre-lock usage error at the `assertForceableStage` posture. The ADR's binding requirement —
  "before the lock and before any inference" — is met, so this is naming, not a contradiction.
- The register's `verified` row for A21/A22 and the Deferred items (files-spanned ceiling,
  auto-detection, per-flow model routing) are consistent with ADR-010's Decision and Deferred list;
  nothing to re-open.
