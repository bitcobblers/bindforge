# security review r3c1 (dissent pass)

verdict: approve

Scope: `git diff 92f71e0..4635f0d` of `02-proposal.md` (A12 row, Budget resolution
paragraph, The `plan` stage first bullet, one Compatibility matrix row, Test strategy
#17, Revision log), then the full current document. Inputs: `02-proposal.md`,
`docs/adrs/010-lightweight-plan-flow.md`, the Revision log, my r3 findings. Checklist:
`references/security.md`.

## Findings

- [nit][new] Assumption register A12 — "all four document-stage primary agents —
  `researcher`, `designer`, `editor` (`proposal-final`), `doc-sync` — deny
  `.pipeline.json`, `transcripts/**` (read and edit) and `shell: "*"`;
  `doc-sync.md:23-25` is the one exception, an allow for `git diff*`" — the universal
  and the exception clause collide on a literal reading: as written, the sentence first
  asserts doc-sync denies `shell: "*"`, then names it "the one exception". Both readings
  of `doc-sync.md:20-25` are open to a reader of the register alone — a wildcard deny
  plus one narrow allow (so the universal holds, and only `git diff*` is added), or an
  allow map with no wildcard deny (so the universal is false for doc-sync and the
  exception clause understates it, covering only `git diff`). A12 is the document's one
  stated shape authority for the planner's permission list, so the shape it licenses
  should not be readable two ways. It does not reach the planner as written: the
  `plan` stage bullet says the list is `designer.md:4-22` verbatim and "`ops/doc-sync`'s
  `git diff*` allow is not part of it", so the operative security claim (planner: no
  shell) is pinned. Fix: make the denied thing the wildcard key and the exception
  additive — "deny `.pipeline.json`, `transcripts/**` (read and edit) and the
  `shell: "*"` wildcard; `doc-sync.md:23-25` adds one allow, for `git diff*`".
- [nit][new] Budget resolution — "a caller-supplied `opts.cfg` is validated
  **pre-lock** — `validated` runs on it in `runPipeline`/`resumePipeline` once the flow
  is known, before `acquireLock` and `freshState`" — "runs on **it**" names the bare
  caller value, while the rule being enforced is scoped to the *resolved* config: the
  non-emptiness check reads "keep `reviewSeats` non-empty for every phase the flow's
  `stages` runs", and a CLI-shaped `opts.cfg` (`{ ...DEFAULT_CONFIG,
  maxParallelReviews }`) carries no `reviewSeats` at all (A3: the field is new and
  optional), so on the bare value the check is vacuous for absence and binding only for
  a caller who supplies a panel. That is enough to close the fail-open either way —
  `reviewSeats.code: []` is present-and-empty, so #17's assertion (no lock file, no
  design directory, no entry checkpoint, no `ESCALATION.md`, zero client calls) holds
  under both readings — but it leaves two call sites of `validated` whose subject is
  unspecified, and the pre-lock check is exactly the same predicate `configFor` runs at
  the read seam. Fix: name the resolved value as the subject, one clause —
  "the resolved `configFor(flow, opts.cfg)` is computed pre-lock, the same call the
  read seam makes".

## Closures checked (my r3 findings)

- major (budget-resolution throw site) — **closed**. The throw site is no longer placed
  after the lock: "validated runs on it in `runPipeline`/`resumePipeline` … before
  `acquireLock` and `freshState`", anchored to the `assertForceableStage` posture that
  Selecting the flow defines as "no lock, no design directory, no session"; the
  post-lock `configFor` validation is retained (the `validated` in the canonical
  formula), so the pre-lock check is a clean fast-fail, not a relocation. The flow is
  knowable at that point on both entry paths — the pre-lock parse in `runPipeline`,
  `flowOf(state)` at the same pre-lock points `--stage` already uses. Test strategy #17
  is extended to a caller's `reviewSeats.code: []` panel and asserts no entry
  checkpoint and zero client calls, and the Compatibility matrix row names the same
  case. The requested test case and the one sentence are both present; nothing new is
  added mechanistically. Residual, not a finding: a concurrent `--force` promotion
  between the pre-lock flow read and `acquireLock` can move the validation's scope, so
  the read seam's throw (unchanged behavior) stays reachable; the checkpoint remains
  authoritative, which is what the design states.
- minor (A12's document-stage universal) — **closed**. The enumeration is now the four
  document-stage primary agents (researcher, designer, editor/`proposal-final`,
  doc-sync), matching the full flow's document stages, with all four cited to their
  agent files, and the `doc-sync` shell exception is named in the register and
  explicitly excluded from the planner's copied list. What is left is the wording
  collision noted above (nit).
- nit (A12 omitted `.pipeline.json`) — **closed**: the parenthetical now lists
  `.pipeline.json`, `transcripts/**` (read and edit) and the shell wildcard, so the
  single stated shape authority lists every entry.

## Checklist notes

- Trust boundaries / fail-closed (1, 7): the seat panel is treated as caller data,
  validated at the pre-lock seam and at the read seam; unknown seat names,
  `planMaxLines: 0` and an empty `code` panel throw for every flow (#23), and an empty
  `design` panel resolves only where no stage consumes it. Malformed `flow` values stay
  a pre-lock usage error, case-sensitive, never silently `full`.
- Least privilege (6): the planner copies `designer.md:4-22` verbatim, denies
  `.pipeline.json`, `transcripts/**` (read and edit) and the shell wildcard, and the
  one document-stage shell allow in the repo (`git diff*`, doc-sync) is named as not
  part of it — and doc-sync is not a light stage, so it is unreachable in the flow.
- Path handling (3): unchanged from the r1 closure — `plan` keeps the document-stage
  rule (`inDocs`), the intake of A28 is repo-root-relative POSIX output with `.`/`..`
  and absolute shapes rejected by `moves.ts` Rule 1, and #18 asserts
  `docs/../src/x.ts` is reverted as git's real path; `docs/**` writes by a light `plan`
  are pinned as the **accepted** widening, not claimed as confinement. Nothing in the
  changed text touches path handling.
- Parsing (2): one new frontmatter key through the existing CRLF-tolerant scan; no new
  untrusted-data surface, no code evaluation.
- Dependencies (5): the design names no new dependency (A20 — a stage, a seat list, a
  config record on the existing Node floor), so there is no advisory to fetch.
- Secrets/PII (4): unaffected by the changed text.

## FYI (no severity)

- A12's Basis now cites `stages.ts:101-150` while Flow registry cites the same site as
  `stages.ts:101-138`; two ranges for one site after this diff — a citation-consistency
  detail for the correctness seat.
- Test strategy #17 asserts "no design directory, no entry checkpoint" for
  `--stage research` on a **light checkpoint**, a resume where both already exist from
  the prior save; the assertion reads cleanly for the two fresh-entry cases. How to
  phrase the resume case (pre-existing checkpoint byte-equal) is a test-authoring call.
