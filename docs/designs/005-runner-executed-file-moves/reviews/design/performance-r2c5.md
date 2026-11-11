# performance review r2 (c5 ratification)

verdict: approve

Scope: diff 82b346f..2168fac of 02-proposal.md, then the full current document.
This pass had no open findings from my previous verdict, so I checked only
whether the revisions broke anything the performance seat had approved. They
did not.

## Checklist evaluation (against the changed text)

1. Event-path latency — the only runtime additions touched by the diff are
   unchanged-cost paths: validator rule 4's `git cat-file -t` and the rules
   4–7 apply guard (both predate this diff; the diff only re-scopes rule 2 to
   a validator-local `.git` check). Work is bounded by the unchanged
   `moves ... .max(64)` cap and runs in validate/finalize, not per keystroke.
   Nothing on the changed path adds per-event cost.
2. Structure costs — no loop nesting introduced by the diff; the removed
   `auditSessionIndependence` matcher would actually have been the only
   session-wide scan, and it is gone. No O(n²) over unbounded n.
3. Memory growth — the diff removes machinery (audit matcher, env pins,
   allowlists); nothing new is retained.
4. Startup cost — unchanged by the diff; no boot-time reads added or removed.
5. Cache invalidation — n/a; the diff removes the only caching-adjacent text
   (the runner git-env pin), reverting runner git calls to today's baseline
   behavior rather than regressing it. The dropped `-c core.fsmonitor=false`
   pin was an optimization on top of today, not something today relied on.
6. Baseline consistency — ADR-005 item 3's rename-detection baseline (run 004
   A15: 91–99% similarity, so `R` entries survive) is untouched and the new
   compatibility row "rename detection stays at git's default (A5)" is
   consistent with it, as is Rollout #4's `git show --name-status -M`
   acceptance check. The reworded git row ("no new floor: the only git call
   this change adds is the validator's `git cat-file -t`") is accurate as a
   call *kind* — validate and apply-guard both use only that helper — and
   dropping the 2.32 floor is coherent now that `GIT_CONFIG_GLOBAL` ships
   nowhere in this change.
7. Deferred check — the new Deferred entry ("ADR-005 defers the earlier item
   5 to a follow-up ADR") matches ADR-005's own "Deferred — read-only git in
   stage shells" section verbatim in substance, including the
   separable/non-terminating rationale and "items 1-4 stand without it".
   No contradiction.

## Findings

None.

## FYI (no severity)

- Assumption register — A6 still opens with "The four read-only-named
  commands", language that referred to the now-deferred item 5 allowlist;
  A6's config-surface substance is still load-bearing for the Options
  rejections, but the framing is stale for an editorial/compatibility seat
  to weigh.
