# performance review r1

verdict: request-changes

Scope: whole document (full-panel pass c0). Checklist: latency/event-path cost,
structure cost with unbounded n, memory boundedness, startup cost, cache
invalidation, quoted research baseline. First pass for this seat: no prior
findings; nothing in the Revision log (r0 entries) addresses cost, so every
finding here is `new`. Startup cost is unaffected (module-level `Set`, no boot
work). No cache is proposed, so invalidation is untestable here — see FYI.

## Findings

- [major][new] Chosen design §2 "Deeds of children" — "else `client.session.list()`
  filtered by `parentID === sessionID`" plus the Cost bullet "one children call
  plus one paged message fetch per descendant per attempt, once per attempt
  after the reply — off the poll loop, bounded". The fallback is written per BFS
  node: every session in the subtree triggers a full `GET /session` — the whole
  session table the server has ever recorded, which grows without bound across
  runs and is never enumerated by this repo today. Cost is O(k × S) for a
  subtree of k descendants against S total sessions, repeated per attempt and
  multiplied by `maxParallelReviews` concurrent seats (runner.ts:1203, 1292).
  The word "bounded" is the design's only cost argument, and it is a restatement
  of the research note that already assumed it (research Platform notes:
  "bounded and off the poll loop") — no subtree size, session count, or page
  count anywhere, so the checklist's baseline-vs-expectation comparison has
  nothing to compare. The claim holds for the `session.children` endpoint (cost
  proportional to the real subtree) but not for the named fallback.
  Fix: keep one enumeration per walk, not per node — when the SDK exposes
  `children`, keep the per-node call (subtree-proportional); when it does not,
  call `client.session.list()` **once per attempt**, index the result by
  `parentID` in a `Map`, and BFS that map, then fetch messages per descendant.
  In the Cost bullet, replace bare "bounded" with the explicit worst case (one
  session-list call + one children call + one paged message fetch per
  descendant, each request inside `requestTimeoutMs`, `bounded()`'s single
  retry).

- [minor][new] Chosen design §2, child-filter bullet — "children need no
  creation-time filter because the message-level `since` filter already excludes
  messages from before this attempt (A7, A8)". That justifies correctness, not
  cost, and the sentence reads as a cost justification because it sits next to
  the Cost bullet. `since` is applied inside the matcher
  (independence.ts:112–114) *after* `listSessionMessages` has paged the
  descendant's entire history (runner.ts:473–499 pages at `limit: 200` until
  `cursor.next` is empty; the server API exposes no time-bounded message fetch).
  So each attempt pays a full fetch per descendant even when the whole list is
  older than `since` — and on the resumable lane (`retryInSameSession`, the
  session survives into attempt 2) the attempt-1 children are re-enumerated and
  their full lists re-fetched and thrown away.
  Fix: split the two claims in that sentence — the `since` filter bounds what the
  matcher *judges*, not what the walk *fetches* — and state the fetch honestly
  in the Cost bullet: one full paged message fetch per descendant per attempt,
  including descendant lists that are entirely pre-`since`.

- [minor][new] Chosen design §2, helper signature —
  "`Promise<Array<{ sessionID: string; messages: SessionMessageInfo[] }>>`".
  The shape forces the whole subtree's message history into memory at once: the
  runner holds the parent's `lastSeen` (already fetched) plus every descendant's
  complete list simultaneously before the matcher sees the first one, and
  memory grows with subtree size, which §2 deliberately left uncapped (transitive
  BFS, custom agents may fan out). The matcher is per-list and stateless
  (`auditSessionIndependence(messages, opts)`), so nothing needs them together.
  Fix: hand lists over one at a time instead of collecting them — take a
  per-list callback (`(sessionID: string, messages: SessionMessageInfo[]) => void`
  that appends `${violation} (via child session …)` and drops the list), or
  return an async generator; same walk, same violations, peak memory = one list.

- [nit][new] Chosen design §2, audit call site — "run the pure
  `auditSessionIndependence` once per message list — parent first, then each
  descendant". The walk is unconditional, including on attempts the parent
  audit has already failed: the attempt fails on the parent's violations either
  way, so the descendant fetches only add round-trips (each up to
  `requestTimeoutMs` with `bounded()`'s retry) to the failure path that leads
  into attempt 2. Fix: state that the parent list is audited first and, when it
  already returns violations, the walk is skipped and the attempt fails on what
  is in hand. Keep the walk unconditional otherwise — do not gate it on the
  absence of a completed `subagent` part, which would narrow detection.

## FYI (no severity)

- The poll loop already re-fetches the parent's whole message list every
  `pollIntervalMs` (runner.ts:763–780) — pre-existing, untouched by this design;
  mentioned only because it is the one place where an implementer might be
  tempted to cache message lists across the two attempts of a resumable retry.
  If they do, child sessions keep producing messages until the attempt settles,
  so the cache needs an invalidation rule this design does not currently give.
- Reading an unknown tool's `pattern` as a glob-style access while `grep`'s
  `pattern` stays query text is a judgment call about what counts as an access,
  not a cost question — correctness/security seat's ground.
