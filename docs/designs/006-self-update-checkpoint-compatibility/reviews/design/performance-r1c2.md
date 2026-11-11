# performance review r1c2

verdict: approve

## Findings

None. This pass is ratification-only: it checks the r1→c2 diff
(blob da725a4 → 305aa36 of `02-proposal.md`) for anything that broke the
r1-approved text or falsified a section this seat cited.

Diff walked (all hunks): Non-goals bullet removed; A12 reworded
(unverified → verified); the "Auto-reconcile" option moved from Options to
the Deferred bullet; §1's fence matcher rewritten (segment-wise,
case-insensitive, tmp shape narrowed to `.pipeline.json.<8 hex>.tmp`) and
paired with the author-agnostic sweep plus the in-memory snapshot-skip rule;
§2 refusal-message reword; §3 bullets condensed with unchanged meaning;
§4's redundant no-schema-change sentence deleted; compatibility matrix rows
relabeled; Test #1/#2 extended; Platform paragraph and Deferred items
reworded; Revision log populated.

Against the performance checklist:

1. No per-event/keystroke-path cost is added by the changed text. The
   segment-wise name test and the new authorship snapshot comparison both run
   only on the existing stage-boundary sweep; §3's `status` marker check and
   §4's one extra read + one `'wx'` retry under contention are unchanged from
   r1-approved text.
2. No O(n²) path: the matcher is linear in path segments per already-swept
   path; the broadening from "last segment" to "any segment" keeps it
   per-path linear.
3. Memory growth: the new "bytes `saveState` last wrote for it, kept in
   memory" retains one checkpoint-sized byte string per run process — the
   full state object is already in memory and re-serialized on every save
   (r1-approved), so this adds a constant, bounded copy. No new unbounded
   structure appears in the diff.
4. Startup cost: nothing in the diff adds boot-time reads or parsing.
5. Caching: the in-memory snapshot is written by `saveState` and compared at
   the sweep, both in the same single-flight process; the diff's §1 text
   keeps the invalidation story (skip-while-matching, revert-on-mismatch,
   marker-deletes not restored from `headBefore`) consistent.
6. Baseline gap: unchanged from r1 — the document states no cost expectation
   anywhere, and the research document is not among this pass's inputs, so
   the baseline-vs-expectation comparison still has nothing to cite in-text.

Citations this seat relied on in r1 are still true in c2:
§2 "…`stageCommits` keeps its bounded-window meaning … as named contract,
not comment" (verbatim); §3 "`resumePipeline` (`runner.ts:1983–1989`) writes
a `gateLog` entry (`terminalSaveFailed`, …) before deleting the marker"
(verbatim); "checks for the marker alongside the loaded state" (verbatim);
§4 "on `EEXIST`, read the holder and apply today's pid-liveness logic
unchanged" (verbatim). Nothing deleted or reworded in the diff falsifies any
r1-approved performance-relevant statement.

Open from r1, carried for the record (not re-reported as findings this pass —
the revisions neither addressed nor rejected them in the Revision log, and
did not make them false): the minor on naming the `gateLog` growth bound the
way §2 names the `stageCommits` bound, and the nit on the absence of any
cost sentence for the added per-operation checks. c2's §1 text is consistent
with the nit — it adds one more unstated-but-bounded cost (case-insensitive
matching over every segment, one byte comparison and one kept snapshot per
matched path per sweep) — without raising its magnitude beyond what r1
already rated nit.

## FYI (no severity)

- The "Auto-reconcile worktree vs committed copy" option was removed from
  Options considered and its rejection rationale folded into the Deferred
  bullet. Flagging for the proposal-template/complexity seat: confirm the
  Options section is still considered complete for a rejected-but-discussed
  option.
