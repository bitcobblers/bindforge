# performance review r1

pass: c0, round 1, scope: whole document
inputs: `02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`, the
proposal's Revision log. `01-research.md` was outside this pass's declared
input set, so checklist item 6 is evaluated against the baseline as quoted in
ADR-005 and in A12 (see Baseline below).

verdict: approve

## Baseline quoted (checklist 6)

- ADR-005 Context: run 004 "escalated with `tests not green at the code-review
  budget limit` after six `fix <-> test-update` rounds, all reproducing
  byte-identical failures"; "the approved change moves 23 test files".
- ADR-005 Alternatives: the status quo costs "a full livelocked code budget
  plus one escalation" per file-moving ADR — that is the number this design is
  buying back, and the design's mechanism for buying it back is sound.
- ADR-005 Decision item 3: "rename detection survives (A15 of run 004's
  proposal showed 91-99% similarity for content-identical moves)".
  The design's expectation ("run 004's 91-99% figures were move-plus-rewrite —
  a pure runner-side rename scores near 100%") explains the gap against that
  baseline rather than contradicting it, so item 6 raises nothing on similarity.

No design section states an expected cost or latency figure for the new
per-stage work; that gap is finding 4.

## Findings

- [minor][new] Chosen design → Validator, rule 4 — "`from` is a tracked **file**
  at `headBefore` (`git cat-file -t <headBefore>:<from>` equals `blob` — this is
  the file-only rule)" — the validator is specified as one git subprocess per
  endpoint: `git cat-file` per `from` for rule 4, plus a second headBefore
  membership query per `to` for rule 5, plus `existsSync`/`realpath` per pair.
  Run 004's motivating payload (23 pairs, A12) is ~46 subprocess spawns per
  validation pass, and the pass re-runs on every nudge cycle and again in
  attempt 2 (Wiring into retry), so a list that takes three nudges across two
  attempts costs several hundred spawns. It is not on a keystroke path — it is
  once per reply, inside finalize, against a stage that runs an inference
  session — so this is minor, not major, but the design never acknowledges the
  per-pair spawn cost and leaves the obvious batching unspecified. Fix: read
  headBefore once per validation pass instead of per pair — `git cat-file
  --batch` fed all `<headBefore>:<path>` endpoint queries through one process,
  or one `git ls-tree -r --name-only <headBefore>` snapshot held in a Set —
  replacing the per-pair `git cat-file -t` in rule 4 and the per-`to` tracked
  lookup in rule 5. Same answers, one spawn per pass.

- [minor][new] Chosen design → Reply field + Validator rule 6 — "`moves:
  z.array(MovePairSchema).default([])`" and "no listed path is a path-prefix of
  another" — the list length is unbounded: nothing caps the array, so n is
  model-controlled, and the prefix rule as written is naturally a pairwise scan,
  i.e. O(n²) in n for the disjointness/containment bookkeeping. Nothing in the
  design bounds the blast radius of a pathological list (thousands of pairs)
  beyond the reply's own token size. Fix: bound it at the schema —
  `z.array(MovePairSchema).max(N).default([])` with N stated (a few hundred
  comfortably covers run 004's 23) — and make rule 6 non-quadratic by sorting
  the normalized paths once and comparing each path only against its sorted
  neighbours, which replaces the pairwise scan rather than adding to it.

- [minor][new] Compatibility matrix → "Repository spanning mount points" row —
  "`rename(2)` fails `EXDEV` (A8): the pair fails at apply, the list rolls
  back, the stage fails with `moves-apply-failed`" — this routes a
  statically-detectable condition through the most expensive failure path the
  design has. `moves-apply-failed` deliberately drops the `invalid-reply`
  prefix (Apply: "so attempt 2 starts a fresh session"), so a cross-mount pair
  costs a full rollback of up to n−1 renames, then a fresh no-context session
  that re-derives the stage's whole reply, then escalation — in a system where
  a fresh inference session is the dominant cost. Rule 7 already realpaths
  `from` and `to`'s nearest existing ancestor, so the condition is knowable at
  validation for free. Fix: extend rule 7 to compare the `dev` of `statSync
  (from)` against the `dev` of `to`'s nearest existing ancestor and fail as
  `invalid-reply: moves` (the nudge path); keep the apply-time `EXDEV` handling
  as the race guard, not as the first line of detection.

- [minor][new] Wiring into retry / Rollout 4 — "The in-attempt nudge loop is
  the cheap correction path" — the document's cost claims are asserted, never
  quantified: "cheap" has no bound attached (the nudge cap the claim leans on
  is existing behavior that the proposal cites as A2 but never numbers), and no
  section states the expected added finalize cost at the run-004 baseline size.
  Rollout 4's dogfood criteria are outcome-only ("no `operatorForce`/…", "land
  as `R` entries"), so nothing in the acceptance net would notice if the new
  path made finalize slower — e.g. if the per-pair spawn cost of finding 1
  scales with a larger move set. This is the checklist's "input-path latency
  budget" item: the document has no budget line, so I could not evaluate it.
  Fix: one sentence, in Apply or Rollout 1, stating the per-stage cost model
  after finding 1's batching — one headBefore read + n `renameSync` calls +
  n stat checks, target under a second for a run-004-sized 23-pair list — plus
  the existing max-nudges-per-attempt number that makes "cheap" bounded. Note
  this does not re-open the budget-burn decision the Revision log already closed
  (r0: research OQ6): burning one code-budget unit per full failed attempt
  stands; only its bound is unstated.

- [nit][new] Chosen design → Read-only git — "`git status
  --no-optional-locks*` (the pinned form avoids the index-refresh side effect
  and the lock contention with the runner's own git calls, A6), `git log*`,
  `git diff*`, `git show*`" — the only cost-pinned form is `status`; the other
  three are granted in their unbounded spellings, and `git log -p` / `git log
  --follow` over full history is the most output- and CPU-hungry invocation
  available inside a stage turn, landing straight in agent context. Precedent
  exists (review seats already run the same three of four), so this is
  preference, not a defect. Fix: if the author wants it, mirror the `status`
  treatment with a one-line prompt/config note that log/diff calls be bounded
  (`-n`, path-scoped), the same way `--no-optional-locks` was pinned.

## Checklist coverage

1. Latency budget: no budget stated — finding 4. No new work lands on the
   agent's per-tool-call path; the added synchronous IO sits in finalize/reply
   validation — finding 1 sizes it (minor, not major).
2. Structure costs: the only superlinear step is rule 6's prefix scan, over a
   model-controlled n — finding 2.
3. Memory growth: nothing new is cached or accumulated across stages; the
   validator's endpoint sets and the reply's `moves` array are per-stage and
   bounded by the (uncapped) list — finding 2's cap closes the only unbounded
   edge. Checkpoint growth is additive per A13. No finding.
4. Startup cost: schema field + seven agent-config lines; no new boot-time file
   read or schema parse. No finding.
5. Caching/invalidation: no cache is introduced. The nearest analogue — the
   validation-time view of disk vs. apply-time disk — is handled by the
   per-rename `to`-absent re-check plus the `.pipeline.lock` single-writer
   invariant, which is the right shape: re-check at the mutation, not a cache
   with an invalidation rule. No finding.
6. Baseline gap: similarity expectation is explained; cost expectation is
   missing — finding 4.

## FYI (no severity)

- ADR-005 item 3 says the runner performs renames "through its own git
  plumbing"; the design uses `fs.renameSync` + the existing `git add -A` and
  rejects a per-pair `git mv` subprocess. From this seat that is the cheaper
  choice (one subprocess per pair removed, and the config surface A6 warns
  about), but whether the wording is a deviation from the ADR's Decision is the
  compatibility seat's call, not mine.
- `git status --no-optional-locks*` is the right trade for lock contention with
  the runner's git calls; the side it gives up is the persisted index refresh,
  so each agent `git status` redoes the stat/content walk of the dirty worktree
  instead of reusing a refreshed index. Correct trade at this scale — recorded
  here only so the checklist's invalidation item shows as considered.
- Rollout 4 makes "`R` entries (`git show --name-status -M`)" a pass criterion,
  but rename detection is a diff-time computation at the 50% similarity
  threshold (A5): a move whose content is rewritten past ~50% similarity lands
  as `D`+`A`, and the dogfood criterion would read as a failure with no
  mechanism in this design to change it. The 91-99% baseline sits far above the
  threshold, so no finding — flagging for correctness/compatibility to judge
  whether the criterion should be worded as "per pair that stays ≥50% similar".
