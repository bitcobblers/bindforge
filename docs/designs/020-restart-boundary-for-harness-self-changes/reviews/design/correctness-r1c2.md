# correctness review r1 — c2 ratification pass

verdict: approve

Scope as given: `git diff 15ba04f5 0b135560` over `02-proposal.md` (6 hunks:
A7 row, Options RunStatus-rejection wording, Options per-commit-hooks wording,
§1 diff command + Test-#7 cross-reference, §2 path-copy sentence, §1 risk row,
matrix row, Test #2 wording, revision log r2 entries) plus the full current
document. Checking only that the r2 revisions did not break what r1 approved,
and that the sections r1 cited are still sound.

Checks run against the tree (read-only):
- `nulPaths` exists and is the right parser for the new command form —
  `runner.ts:262` splits a NUL-separated, status-prefix-free list (its current
  caller is `diff-tree -r --no-commit-id -z --name-only`), so `git diff -z
  --name-only` output feeds it unchanged; with `--no-renames` no rename
  continuation entries appear. §1's "the existing `nulPaths` parser" holds.
- r1 finding 1's fix landed: §1 now reads `git diff -z --no-renames
  --name-only <headBefore>..HEAD`, so a rename out of the two prefixes shows
  as delete+add and both sides pass the predicate. The Options section now
  cites "§1's net-diff listing" instead of restating the command, so it
  inherits the fixed form rather than the old one — closure is real, not
  worded away.
- A7's rewritten claim is supported by its new basis: `loadState` validates
  with `StateSchema.safeParse` but returns `parsed as PipelineState`
  (`state-store.ts:239–256`), so an unknown key survives into the in-memory
  state (inert to consumers that never name it), and `saveState` writes
  `JSON.stringify(state)` — the input object, not `checked.data`
  (`state-store.ts:356, 365`) — so the key does round-trip through later
  saves. "verified" is the right status.
- No contradiction created by the A7 rewording: the Options rejection of a new
  `RunStatus` member still holds (a widened `status` enum fails
  `z.enum([...])` at `safeParse`, hence "refuse the checkpoint"), and the risk
  row and the pre-020 matrix row now say "inert" consistently with §2.
- `printable` (`independence.ts:443`) and `boundedGateEvidence`
  (`runner.ts:3057`) exist; §2's citation of them is not invented.
- The finalize-window site claim still supports the moved check:
  `finalizeStage` runs `enforceOwnership` (which mints the revert commits)
  *before* `commitStage` (`runner.ts:1877–1890`), so a single
  `headBefore..HEAD` listing taken after the stage commit sees the reverts and
  the stage commit together, as A4 says.

## Findings

- [minor][re-raised] Chosen design §3 (with §2) — "`pipeline status` while the
  record stands gains one line … the stage that opened the boundary, how many
  harness files it changed, and the resume posture" — unchanged by this round,
  and still short of ADR-020 Decision 2's "marker naming the changed paths and
  **the stage that will run next**". Every artifact the design names (the §2
  record, the `restart-boundary` gate entry, this status line, the printed
  summary) names the stage that *completed*; the next stage is derivable from
  `lastStage` plus the flow but the design never says where the ADR's "stage
  that will run next" is carried. Fix: one sentence in §2 stating that the
  next stage is carried by the checkpoint's `lastStage` + flow and surfaced by
  the status line's resume posture — or add the next stage to the printed
  summary's content list in §3. (Carried from r1; the r2 revision log does not
  record it as rejected.)
- [minor][regression] Chosen design §2 — "the record keeps the
  `boundedGateEvidence` bounding (first N paths plus the count)" — this new
  sentence leaves the count with nowhere to live: the record shape §2 declares
  two paragraphs earlier is `restartRequired?: { stage, paths, at }`, so a
  record bounded to "first N paths" carries no total, and §3's status line
  ("how many harness files it changed") is read by a *fresh* process that can
  only see the checkpoint. The cited helper also does not fit the field:
  `boundedGateEvidence(evidence: GateEvidence, repoDir?)` bounds
  `evidence.legs` at `MAX_GATE_LEGS = 32` and encodes its elision count by
  pushing a synthetic entry *into the kept array* (`runner.ts:3057–3075`) —
  the analogue for `paths: string[]` would put a non-path string into the
  record's path list, which then flows into the commit body and summary that
  the same sentence says carry "every copy of a path". Fix: bound only at the
  copy sites — keep `paths` whole in the record (each entry `printable`, which
  is where the model-authorable-filename concern actually bites) so §3's count
  reads `paths.length`, and apply the first-N-plus-count posture to the
  gate-log entry and commit body; alternatively add an explicit count to the
  §2 shape and say the bounded `paths` is display-only.

## FYI (no severity)

- §1's closing line is now a bare cross-reference — "The ADR's 'test that
  fails when a new executable path is added' is Test strategy #7" — the
  mechanism moved entirely into Test #7. Nothing is contradicted (§1 still
  answers Decision 1 and the Consequences clause, and Test #7 states the
  walk), but the prose now answers the ADR's honesty-test requirement by
  pointer rather than in the section that makes the rule; a coherence seat may
  want the one-clause summary back in §1.
- Revision-log attribution for the `git diff -z --no-renames` decision cites
  "research commit-site-coverage risk; security r1". Record-keeping, not a
  technical issue.
