# performance review r1 (dissent pass c1)

verdict: approve

Scope: `git diff ce95f3b f4f64f9` on `02-proposal.md`, then the full current
document. Checked my four r1 findings first; all four are closed in the revised
§2. One new defect sits in the text that closed them.

## Findings

- [minor][new] Chosen design → §2 Deeds of children — "New async-generator
  helper beside `listSessionMessages`, so peak memory is one message list" and
  the Cost bullet's "one children call per session (or one session-list call and
  a `parentID` index on the fallback)" — the peak-memory sentence is true only
  on the `session.children` route. On the named fallback the generator must
  accumulate every session the server has ever recorded and keep the `parentID`
  index alive for the whole BFS, so the walk's peak is one message list plus an
  O(S) index over S = the entire server session table — the same unbounded S my
  r1 finding named, now bounded in request count (one enumeration per walk) but
  not in footprint, and `SessionInfo` records are what the enumeration returns.
  The Cost bullet also under-counts the requests against the design's own
  wording: the fallback is "paged with `cursor.next` like `listSessionMessages`"
  (A6; runner.ts:480–496 pages at 200 until `cursor.next` is empty), so the
  enumeration is ⌈S/200⌉ list requests, each inside `requestTimeoutMs`
  (default 30 s, stages.ts:70) with `bounded()`'s single retry, and the review
  lane runs up to `maxParallelReviews` of those walks concurrently
  (runner.ts:1203). Fix: qualify the header — "peak memory is one message list
  on `session.children`; on the list+index fallback, one message list plus a
  `parentID` index" — and in the Cost bullet replace "one session-list call"
  with "⌈S/200⌉ paged session-list calls", with the index holding only
  `{ sessionID, parentID }` pairs (the walk needs nothing else) instead of the
  fetched `SessionInfo` records.

## Closed without re-opening

- r1 major (O(k × S) enumeration, bare "bounded") — closed: "one enumeration per
  walk, not per node … indexed by `parentID` into a Map the BFS walks", and the
  Cost bullet now enumerates requests instead of saying "bounded".
- r1 minor (`since` read as a fetch bound) — closed: "`since` bounds what the
  matcher judges, not what the walk fetches", and the Cost bullet now admits
  "one full paged message fetch per descendant — including entirely pre-`since`
  lists on the resumable lane". Verified against independence.ts:112–114 (the
  filter is inside the matcher) and runner.ts:473–499 (unconditional paging).
- r1 minor (whole subtree in memory) — closed: the helper is an
  `AsyncGenerator`, so the matcher receives one list at a time; the residual
  overclaim on the fallback route is the finding above, not a re-open.
- r1 nit (walk runs on an already-failed attempt) — closed: "on the parent list
  first; when it already returns violations, skip the descendant walk and fail
  on what is in hand", and the walk stays otherwise unconditional — the new
  subagent-part check is a `console.warn` tripwire, not a gate.
- The §1 change to a "node-capped queue" replaces an unbounded recursive
  `stringValues` scan (independence.ts:180–188) with a capped one — a cost
  improvement, no performance objection. §3's `brief(printable(...))` bounds
  what attempt 2's prompt absorbs; strictly better than the raw printable string.
- Deferred rows (off-allowlist keys, values beyond the node cap, the structured
  "rule broken" field) do not contradict any Decision or required change in
  `docs/prompts/audit-by-deeds.md`; not re-opened.

## FYI (no severity)

- §3's `failureNote: brief(printable(...))` calls `brief()` from `runner.ts`,
  but `brief` is module-private in `independence.ts:204`; only `printable` is
  exported (independence.ts:200). Correctness seat's call.
- §2's tripwire condition ("a completed `subagent` part yet the walk enumerated
  zero descendants") is also satisfied by the new parent-first short-circuit,
  where skipping the walk makes zero-descendants the normal state — a possible
  spurious warn, not a cost issue.
- §1's node cap carries no number, and the Revision log has no r1 entry for the
  cap or the `pattern`-false-positive rewording; the cap's value decides which
  inputs get judged, so it reads as an unlogged decision (complexity/process).
