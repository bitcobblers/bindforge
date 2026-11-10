# Code review — performance seat, round 1, pass c0

Inputs: `git diff 7844c17…HEAD -- ':(exclude)docs/designs'` (read through the
changed files, since the shell tool is denied in this seat),
`02-proposal.md`, `docs/prompts/audit-by-deeds.md`. No other review report or
transcript was opened. First pass for this seat — no prior findings to re-check.

Scope reviewed: `src/pipeline/independence.ts`, `src/pipeline/runner.ts`
(`listChildSessionMessages`, the audit call site, `failureNote` sites),
`tests/pipeline/child-session-deeds.test.ts`,
`tests/pipeline/child-deeds-retry.test.ts`, the three prompt files.

## Checklist evaluation

1. **Hot path (per-event fs/network):** clean in the matcher —
   `auditSessionIndependence` stays pure and synchronous; no fs, no network,
   regexes are linear alternations. `gateLog` (runner.ts:1411) is in-memory.
   The added network I/O sits on the attempt's post-reply path, not the poll
   tick — but see findings 1, 2, 4 for how often that path runs.
2. **Complexity:** no O(n²) over an unbounded dimension. The walk's BFS uses a
   head index rather than `Array.shift` (runner.ts:598, 670) — correct.
   `pathKeyValues`' `queue.shift()` (independence.ts:235) is capped at 256
   nodes, so it is noise. The `byParent` index (runner.ts:637) is the right
   shape for identity lookup.
3. **Unbounded growth:** `visited`/`queue`/`byParent` are per-walk and released
   with the generator — bounded per walk, but the walk's *fetch* cost scales
   with the whole session store, not with the attempt's subtree (finding 3).
   `MAX_PATH_SCAN_NODES` (independence.ts:230) is documented and bounded, but
   leaks under access keys (finding 5).
4. **Startup:** nothing new runs at boot; the walk is per attempt, after the
   reply.
5. **Cache invalidation:** no cache is introduced, so nothing to invalidate.
   Any fix that memoizes the enumeration must scope it to one attempt (noted
   in finding 2).
6. **Benchmark/timing note:** the diff carries no benchmark or timing
   assertion, and the seat's shell was denied, so nothing could be run and
   quoted; the numbers below are arithmetic from the package's own defaults
   (`DEFAULT_REQUEST_TIMEOUT_MS = 30_000`, `DEFAULT_MIN_NUDGE_INTERVAL_MS =
   20_000`, `MAX_NUDGES = 8`, `stageTimeoutMs = 30 min`, page `limit: 200`).

## Findings

### 1. major — the child walk runs for every clean attempt, even one that delegated nothing

`runner.ts:1070–1083`: whenever the parent audit is clean, the walk runs
unconditionally. Nothing keys it to delegation. Most stage attempts (proposal,
revise, editorial, seats that only read named paths) contain no `subagent` part
at all, and still pay a `session.children` call per BFS node — or, on the
fallback route, a full paged crawl of the server's whole session list
(runner.ts:639–668). Deviation #2 in `03-build-notes.md` makes the waste
concrete in this repo's own suite: every attempt in the nine pre-existing fake
clients takes the neither-route throw, so each clean attempt burns a throw, a
`console.warn`, and a `gateLog('independenceAudit')` entry it cannot act on.

The cheap check already exists and is only used after the fact:
`hasCompletedDelegation` (runner.ts:690). Fix: compute the delegation presence
over the whole `lastSeen` list — any `subagent` part, **regardless of status or
`since`** (a resumed attempt's part sits pre-`since` while its child's deeds sit
in-window, and a still-running child is exactly §2's Risks row, so restricting
the guard to completed in-window parts would open a detection gap) — and skip
the walk when the list holds none. Keep the tripwire at 1086 for the case where
a delegation part exists but enumeration returns nothing.

### 2. major — the audit (and therefore the walk) runs once per completed turn, not once per attempt

The audit block sits inside the outer stage loop opened at `runner.ts:983`, so
every iteration that reaches a `done` turn re-runs it. The `continue` at 1174
and 1212 (after a nudge) and — more sharply — the `if (gate === 'wait') continue`
at 1148 and 1190 re-enter the block with *no new prompt and no new reply* at
all. Worst case per attempt: 1 + `MAX_NUDGES` = 9 audit passes, each re-running
the full descendant walk. Design §2's cost paragraph is explicit that this is
"once per attempt after the reply, off the poll loop"; the implementation pays it
up to nine times, and with the fallback route that is nine whole-store session
crawls plus nine × (descendants) full message-list fetches per attempt.
Consequence in the failure mode of finding 1: up to 9 extra `gateLog` entries
per attempt in the persisted checkpoint's `gateLog` array.

Fix: make the walk at most once per attempt. Keep a per-attempt counter of the
`subagent` parts already walked; on a later pass, re-audit the parent list (pure,
cheap) and re-walk only if that count has grown — that keeps a nudge that
*causes* a new delegation in view while bounding the repeated network cost. If
instead the child lists are memoized across passes, the memo must be per-attempt
state, discarded when the attempt ends, and its staleness documented (the walk
is a point-in-time snapshot).

### 3. minor — the fallback enumeration is unbounded in the server's total session count, with no page cap

`runner.ts:637–668`: one paged `session.list` crawl per walk, `limit: 200`,
looping until `cursor.next` is absent, indexing every parentID-bearing session
into `byParent`. The request count and the map's size therefore scale with every
session the server has ever recorded for that location — prior runs' stage
sessions and sibling review seats included — not with this attempt's subtree.
There is also no guard that the cursor actually advances, so a server that
echoes the same `cursor.next` turns the crawl into an endless loop that grows
`byParent` forever (the `visited` set guards the BFS, not this crawl).
Fix: cap the crawl (a page budget, e.g. 50 pages) and break when
`next === cursor`; on hitting either, throw into the existing catch at 1093 so
the attempt degrades to the design's logged partial audit instead of spinning.
The "one enumeration per walk" claim in §2 is met and worth pinning: add a stub
assertion that `session.list` is called once for a two-level tree.

### 4. minor — the walk's awaits are outside the stage deadline

`deadline` is only consulted inside the inner poll loop (runner.ts:996, 1129);
everything at 1074–1083 awaits without checking it. With the package defaults,
one request site costs up to 2 × `requestTimeoutMs` = 60 s (`bounded()`'s retry,
runner.ts:712), and the walk makes one site per page plus one per descendant:
a half-open server turns a clean attempt into several minutes past
`stageTimeoutMs` — and multiplied by finding 2's nine passes. `bounded()`'s own
comment says "the caller still owns the stage deadline"; the walk never exercises
that ownership. Fix: give the walk a whole-walk budget — pass
`AbortSignal.timeout(Math.max(1, deadline - Date.now()))` (or check the deadline
before each `descendantMessages`) and treat expiry as the same caught path at
1093, naming the truncation in the gate line.

### 5. nit — the node cap does not bound the work under an access key

`independence.ts:249–251`: once a key is in `PATH_ACCESS_KEYS`, its whole value
subtree goes through the uncapped recursive `stringValues` (259–268), so the
256-node budget doesn't apply to that subtree. Harmless for real tool inputs;
the comment at 118–123 promises more than the code delivers. Fix: apply the same
`nodes` budget inside `stringValues` for the path-key case, or narrow the comment
to "caps the traversal of payload nodes, not the values under access keys".

## What this seat checked and found sound

- `subagent: []` plus the `PATH_ACCESS_KEYS` scan removes a full
  `stringValues(input)` walk over every unknown tool's payload — a strict cost
  reduction on the matcher's hot loop, with the 256-node cap keeping the
  remaining traversal bounded.
- The generator yields one message list at a time (runner.ts:563–577, 620–623,
  680–683), so peak memory really is one descendant list; violations are
  appended as strings, not lists.
- Parent-first short-circuit (1070–1071) correctly skips the walk when the
  parent already fails — the design's cost rule, implemented.
- `failureNote` at 1539 and 1748 is a one-shot `brief(printable(...))` per
  attempt: bounded, no repeated work.
- No new work at startup, no per-poll-tick addition.

FYI (no severity, outside this seat's checklist): `listSessionMessages` is still
called on every poll tick (runner.ts:988), re-fetching and re-sorting the whole
session list each tick; that is pre-existing and untouched by this diff.

## Verdict

request-changes — two open major findings (1, 2), both about the walk's call
frequency rather than its per-walk shape. The per-walk design (one enumeration,
one list at a time, parent-first short-circuit) is sound and worth keeping.
