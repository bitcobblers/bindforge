# performance review r1 (ratification c5)

verdict: approve

Scope: ratification pass over diff `188d705..44c52fa` of `02-proposal.md`,
against my r1 performance findings. Findings are reported here only where the
revisions made them worse or broke something previously approved.

## Findings

- [minor][re-raised] Chosen design → Apply — "Immediately before the first
  rename it re-runs rules 4–7 as one guard across the inference→finalize
  window" — the new apply guard re-executes validator rules 4–7 a second time
  for **every** successful move stage, re-incurring exactly the unbatched
  per-pair git spawn cost my r1 finding flagged: rule 4's `git cat-file -t
  <headBefore>:<from>` per pair and rule 5's per-`to` headBefore membership
  query. My r1 note covered only the reply-validation pass repeated on nudge
  cycles and attempt 2; the revision adds a mandatory second full pass on the
  happy path, doubling the spawns. The new `.max(64)` caps n but not the
  per-pair spawn count: worst case ≈ 2×128 subprocess spawns per finalize
  (≈ 2×46 for run 004's 23-pair payload, A12). Still finalize-time and off
  any keystroke/event path, so it stays minor, but the revisions amplified an
  open finding without acknowledging it. Fix: read headBefore once per stage,
  not once per rule pass — one `git ls-tree -r --name-only <headBefore>`
  snapshot (or `git cat-file --batch` fed all `<headBefore>:<path>` endpoint
  queries) consumed by both `validateMoves` and the apply guard. headBefore is
  an immutable commit, so the tree read is shareable across passes; the
  disk-dependent halves of rules 4–7 (existsSync, parent, realpath) still
  re-run at apply. This replaces the per-pair `git cat-file -t` in rule 4 and
  the per-`to` tracked lookup in rule 5 with one spawn per stage.

## Disposition of earlier r1 findings (not re-reported)

- Bounded list (was: unbounded array + O(n²) prefix scan) — **resolved**:
  `moves: z.array(MovePairSchema).default([]).max(64)` with per-endpoint
  `.max(200)`, an explicit cost rationale, and cap cases pinned in Test
  strategy #1. With n ≤ 64 the pairwise prefix scan is trivial; the sort-and-
  neighbours refinement is moot.
- Unquantified "cheap" claim (Wiring into retry) — **largely resolved**: the
  bare cost claim was deleted from the Wiring section, and the reply-field
  bounds now state the cost role ("The bounds cap the validation and apply
  work one reply can make the runner spend"). A numeric per-stage latency
  target and the max-nudges number remain unstated, but with the cap in place
  the worst case is bounded; no re-report.
- Cross-mount pair detection (EXDEV via expensive `moves-apply-failed` path) —
  **unchanged, not a regression**: the compatibility row and Test #5 pin
  apply-time EXDEV as before; the new rule-5 parent-exists check follows my
  fail-at-validation philosophy but does not contradict keeping EXDEV as the
  race guard. Re-reporting it here would violate the ratification scope.
- Unbounded `git log*`/`git diff*`/`git show*` spellings (nit) — **closed by
  deliberate decision**: "No argv refinement ships: flag patterns do not
  survive abbreviation" (A9's whole-value wildcard matching), which also
  undercuts my r1 deny-pattern fix as unsound on its own terms. Not
  re-raised.

## FYI (no severity)

- The new `auditSessionIndependence` case-folded matcher over one attempt's
  session (tool-call paths + shell command text) is linear and negligible
  next to the inference session it audits — no performance action.
- The shared config-free git env (`GIT_CONFIG_NOSYSTEM`, pinned
  `GIT_CONFIG_GLOBAL`, `-c core.fsmonitor=false`) is cost-neutral for the
  runner and is security-driven; ADR-005 item 5 scope is respected. No
  contradiction with the ADR's Decision clauses.
