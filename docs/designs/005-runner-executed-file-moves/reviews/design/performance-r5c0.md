# Performance review r5c0 (ratification)

verdict: approve

Scope: the blob diff `2168fac… → 967d5e8…` of `02-proposal.md`, then the full
current document. Inputs: the proposal (incl. its Revision log),
`docs/adrs/005-runner-executed-file-moves.md`, and my previous findings
(none — my prior verdict held no findings). No open blocker or major findings.

## Checklist walk-through (against the changed text)

1. **Input-path latency.** Nothing in the diff puts cost on a keystroke/event
   path; all move work stays on the inference/finalize path. The per-pair cost
   changed shape but not class: rule 4's `git cat-file -t` → `git ls-tree
   <headBefore> -- <from>` is still one bounded-depth git subprocess per
   `from`, and rule 5's `existsSync` → `lstatSync` try/catch is still a single
   final-component syscall (marginally cheaper: no symlink-follow second
   stat). The rules 4–7 apply-guard re-run was already there pre-diff and is
   bounded by the schema caps.
2. **Structure costs (O(n²)).** Rule 6's pairwise disjointness scan remains
   O(n²) in pairs, but the pair cap stays `.max(64)` and the endpoint length
   cap 200 is unchanged; the reworded "**Reply field**" passage ("The bounds
   cap the validation and apply work one reply can make the runner spend")
   now states the bound more sharply than the r5 text, not less.
3. **Memory growth.** The `moves` array cap is untouched; no new cache or
   table is introduced anywhere in the diff.
4. **Startup cost.** Unchanged. The Compatibility matrix still says the only
   git call added is one (`git ls-tree` now instead of `git cat-file -t`) and
   "no new floor" — true for `ls-tree`, which predates every git this repo
   runs.
5. **Caching/invalidation.** No cache added or altered by the diff.
6. **Research baseline gap.** The diff introduces no new expected numbers, so
   there is no new gap against the run's baseline to explain.

## Regression check on revised passages

- A10 basis rewritten to "an `lstatSync` path lookup + `core.ignoreCase`" —
  consistent with rules 5 and #12's "rule 5's disk check"; performance-neutral.
- Apply-paragraph rewordings ("as that of a rename error", "which is exact
  because the pairs are disjoint") are prose-only; ordering (single guard
  re-run, then per-pair `fs.renameSync`, reverse rollback) is byte-for-byte the
  approved semantics.
- Revision log entries match the revised text (validator r4 change, Node
  floor entry); nothing in the Deferred section contradicts ADR-005's Decision
  or Deferred clause.

## Findings

None.

## FYI (no severity)

- Compatibility matrix — "Node ≥ 22.23.4 (engines raised from ≥22.12.0…) …
  (Brotli RCE CVE-2026-111497 et al.)": the engines bump is performance-
  neutral, but a runtime floor raise riding inside a file-moves change is a
  scope question (complexity seat) and the CVE citation is unverifiable from
  this seat's permitted inputs (correctness/security seat); the Revision log
  at least records it deliberately ("r4: … engines included").
