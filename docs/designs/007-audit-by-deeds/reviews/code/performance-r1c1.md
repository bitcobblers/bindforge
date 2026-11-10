# Code review — performance, round 1, dissent pass c1

Scope: `git diff 95b73560fab5571ad65610ffe4f790d080d715c1..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md` (design 007) and the current source. This pass
re-checks my five round-1 (r1c0) findings against the revision written to
answer them, then scans the changed code for new performance issues only.

## Verdict: approve

All five r1c0 performance findings are fixed as prescribed, each in the
spirit and the letter of the requested fix, and four of the five gained a
deterministic stub assertion pinning the cost rule. No new performance
issue is introduced by the revision. Zero open blocker/major findings.

## Re-check of r1c0 findings

### #1 (major) — unconditional descendant walk — RESOLVED

`runner.ts:1247-1248`: the call site computes `countDelegationParts(lastSeen)`
and gates the walk on `delegations > walkedDelegations`. `countDelegationParts`
(runner.ts:859-872) counts every `subagent` part — any status, any age — exactly
the whole-list guard r1c0 asked for; its comment (851-857) records why a
completed-in-window key would open a detection gap. The zero-descendant
tripwire (1263-1271, keyed on `hasCompletedDelegation`) is retained for the
delegation-exists-but-enumeration-empty case. A non-delegating attempt now
makes zero walk requests — no throw, no `console.warn`, no `independenceAudit`
gate entry. Pinned by `child-session-deeds.test.ts:563-569`
(`listCalls() === 0`), and the resumed-attempt shape (pre-`since` part, in-window
child deed) is pinned end-to-end at `child-session-deeds.test.ts:693-708`,
which is what keeps the guard from being later narrowed away. The nine
pre-existing fakes additionally gained `session.list` stubs (Test strategy #9),
so deviation #2's per-attempt warn + gate entry no longer occurs even if a
future flow delegates.

### #2 (major) — walk once per completed turn instead of per attempt — RESOLVED

`runner.ts:1149-1152`: per-attempt `walkedDelegations` counter (the function is
attempt-scoped — `executeInference` starts it fresh at 1152 before the outer
loop). On `wait`/nudge re-entries the pure parent audit (1243) still runs every
pass — cheap, as the fix prescribed — but the network walk re-runs only when
the delegation count has grown (1248), bounding the attempt to one walk per new
delegation instead of up to 1 + MAX_NUDGES. Setting `walkedDelegations` before
the walk (1249) also means a failed walk is not re-paid on later same-count
passes. Pinned by `child-deeds-retry.test.ts:321-330`: at most one descendant
enumeration per attempt (`listCalls() <= attempts.length`, i.e. ≤ 2 across the
two-attempt escalation run).

### #3 (minor) — unbounded paged `session.list` crawl — RESOLVED

`runner.ts:554-560` `MAX_SESSION_LIST_PAGES = 50`; the crawl throws on a
repeated `cursor.next` (754-759) and on exceeding the page budget (760-766);
either throw lands in the caught partial-audit path at the call site
(1272-1297), which names the truncation in the warn and the gate line. Request
count and `byParent` size are now bounded by the walk's budget (≤ 50 pages ×
200 entries), not the server's lifetime store. Pinned as r1c0 asked: the
"one enumeration per walk" stub assertion is at
`child-session-deeds.test.ts:221-231` (two-level tree on the list route:
`listCalls() === 1`), and the spinning-cursor shape at
`child-session-deeds.test.ts:576-588` asserts `listCalls() < 100` and the
partial-audit degradation. Measured (I ran the two deed test files): 15/15 pass,
and with the looping stub the crawl terminates on the second page (derived:
page 2 echoes the cursor → throw), far inside the asserted bound.

### #4 (minor) — walk requests outside the stage deadline — RESOLVED

`ChildWalkOptions.deadline` (runner.ts:642-647) is threaded from the call site
(1256) with the attempt's `deadline` (1139). `budgetMs()` (668-677) clamps every
walk request's `AbortSignal.timeout` to `min(requestTimeoutMs, remaining)` and
throws `the child-session walk of … was truncated at the stage deadline` when
the deadline has passed; every request site on all three enumeration routes
funnels through it (688, 708, 726, 782, 800, 820). Expiry routes into the same
caught partial-audit path (1272), and the truncation is named in both the
`console.warn` and the `independenceAudit` gate line. Worst-case overshoot is
now one request site (≤ 2 × the clamped window through `bounded()`'s retry)
rather than unbounded — the fix's requested shape. Residual (test-side, not a
performance finding): no stub pins the deadline-expiry throw specifically,
unlike #3's two pins; the expiry path is reachable but currently unpinned.

### #5 (nit) — access-key value subtree escapes `MAX_PATH_SCAN_NODES` — RESOLVED

`independence.ts:239-266`: one shared `ScanBudget` now feeds both the
breadth-first payload traversal and the value subtrees under access keys via
`budgetedStringValues` (273-289), which decrements per value node and stops at
zero. The doc comments (118-124, 228-237) now state the shared budget honestly.
Nuance accepted, not flagged: expansion of a single node (`Object.entries`) is
O(node width), so the budget caps nodes-walked, not per-node entry enumeration;
work stays proportional to the already-parsed input, strictly better than the
pre-diff uncapped `stringValues` the unknown branch used. Not worth a finding.

## New-issue scan of the revision (perf checklist)

- Poll loop: the per-pass additions (`countDelegationParts`, delegation-count
  compare) are pure O(messages) over lists the poll already fetched — no new
  network on the hot path; the walk stays off the poll loop, post-reply, per
  design §2's cost paragraph.
- Unbounded growth: crawl pages (50), `byParent` map (≤ 10k ids), walk request
  sites (subtree size), scans (256 shared nodes), gate entries (≤ 1 per walk
  attempt ≤ 1 per delegation growth per attempt) — all bounded and documented.
- No caches introduced, so no invalidation surface (checklist item 5 N/A).
- Direct-HTTP route (`directChildSessions`, runner.ts:609-624): per-node GET
  with `bounded()` + clamped signal, same posture as the SDK routes; auth-header
  extraction in `opencode-runner.ts` is a pure refactor.
- No startup-path parsing added.

Measured runs (checklist item 6 — the diff's cheap count assertions, executed):
targeted `child-session-deeds` + `child-deeds-retry`: 15/15 passed, 753 ms.
Full package suite `npm test`: 250/250 passed, 2.66 s, 28 files — no
pre-existing test changed verdict.

## FYI (out of specialty, no severity)

- The walk-failure `console.warn` wraps the server-supplied error string in
  `brief(printable(...))` (runner.ts:1283) but the `gateLog` reason passes it
  raw (1292) — the security seat's printable posture, not a performance matter.
