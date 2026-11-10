# correctness review r1c2 (ratification pass over ce95f3b..f4f64f9)

verdict: approve

Scope: the diff between the round-1-approved blob and the current document, plus
the full current text. Checked: whether the r1 revisions broke anything this
seat approved, and whether they made any round-1-cited section false.
Spot-checks this pass: https://opencode.ai/docs/server/ (still the V1 page,
banner: "These are the docs for OpenCode V1"), https://opencode.ai/docs/api/
(V2 reference: `GET /api/session` exists with `SessionsCursor`/`SessionsResponse`
pagination; **no** `children` route appears under any session path), and repo
anchors (`independence.ts:64–82,168–189`; `runner.ts:473–499,839–849,1144`;
`opencode-runner.ts:17–18,264–265`; `independence.test.ts:124–130,201–212`).

Round-1 closures verified: my pattern-residual finding is closed — §1 now says
"query-text `pattern` on an unknown tool is a known false positive" and drops
the wrong Risks-row-2 pointer. The old §2 sentence I flagged ("children need
no creation-time filter because the message-level `since` filter already
excludes...") is gone, replaced by the accurate "`since` bounds what the
matcher judges, not what the walk fetches".

## Findings

- [minor][re-raised] Compatibility matrix → "OpenCode server/SDK without a
  children method" row — "a direct `GET /session/:id/children` against the
  configured `baseUrl` with the client's auth headers — all three exist
  server-side (A3), so the walk always has a route (A4 decides which, not
  whether)" — this revision strengthened my round-1 finding instead of
  anchoring it: the row moved from "unknown / resolved at implementation step
  1" to an unconditional "yes... always has a route", and A3 is still cited
  to the same V1 docs page. The V2 API reference (opencode.ai/docs/api/),
  which is the surface the pinned `@opencode/client ^2.1.4` speaks (the repo
  itself records the V2 namespace at opencode-runner.ts:17–18, "`session.create`
  uses `/api/session`"), documents `GET /api/session` (with cursor pagination,
  so the fallback half of the chain is real) but lists **no** children route
  on any session path — `GET /session/:id/children` is a V1-only endpoint, so
  the third named route does not "exist server-side" on the V2 server as
  written, and "the walk always has a route" is a certainty the cited basis
  cannot carry (whether V2 `Session.Info` carries `parentID` is likewise
  unverified here). Fix: re-anchor the endpoint half of A3 to
  https://opencode.ai/docs/api/ — "V1 documents `GET /session/:id/children`
  (docs/server/); V2 documents `GET /api/session` with a cursor — children
  endpoint on V2 not found in the reference" — and reword the row's claim to
  "step 1 picks the route from the installed SDK's types (A4); the SDK's
  session-list + `parentID` index is the V2-documented route, the direct
  children call is the V1 path", rather than "always has a route".
- [minor][new] Risks row 2 (new snapshot row) vs Chosen design §2 snapshot
  bullet — the row says "A descendant still running at the snapshot
  contributes no completed parts", while §2 says "a descendant still running
  at that moment contributes only its completed parts". A running child can
  have completed parts at snapshot time (the message list returns them with
  `state.status === 'completed'`, which is exactly what the matcher judges at
  independence.ts:129–130), so one of the two statements is false as written;
  the real gap is only the parts *not yet completed* at the snapshot. Fix:
  reword the Risks row to "parts of a descendant not yet completed at the
  snapshot go unseen for that attempt".
- [minor][re-raised] Assumption register A8 + Risks "Child actions
  misattributed to this attempt" row — A8's basis is unchanged: "`SessionInfo`
  timestamp fields unverifiable here" remains contradicted by
  opencode-runner.ts:264–265, which already reads `session.time.created` /
  `session.time.updated` off the typed v2 client result; and the unchanged
  row's mitigation "the `since` filter skips pre-attempt messages in any
  fetched list (A7)" still does not cover a child spawned by attempt 1 that
  is *still running* on the resumable lane: the r1 text itself now grants
  both halves of that case — §2's Cost bullet admits the walk fetches
  "entirely pre-`since` lists on the resumable lane", and the new snapshot
  row's likelihood names "background delegations outliving the parent's
  reply" — so such a child's post-`since` deeds in attempt 2 are judged
  against attempt 2's `since` and attributed to it (false failure), while
  the row implies A7 closes misattribution. Fix: correct A8's basis to cite
  opencode-runner.ts:264–265, and either apply the creation-time filter in
  the BFS or add to the misattribution row's mitigation: "an older child
  still running across the retry can emit post-`since` deeds attributed to
  the new attempt; accepted, fence-primary (A10)".
- [nit][new] Chosen design §2 — "one enumeration per walk, not per node: per
  session `client.session.children({ sessionID })` when the SDK exposes it" —
  the headline claim is only true of the fallback branch; the primary path is
  still one call per node, as the Cost bullet itself says ("one children call
  per session"). Fix: scope the phrase to the fallback ("on the list-based
  fallback, one enumeration per walk, not per node").
- [nit][new] Chosen design §1 — "the violation names key and value, so a
  mis-scan is a one-line allowlist fix" — the violation strings as built at
  independence.ts:135 name the tool name and the value, not the key; and the
  "one-line fix" (dropping `pattern` from `PATH_ACCESS_KEYS`) also forfeits
  the glob-style reading the design just chose for unknown glob tools. Fix:
  either state that the matcher will also name the matched key, or reword to
  "diagnosable from the violation's tool name and value; the allowlist is the
  fix knob, at the cost of unknown-tool glob detection".

## FYI (no severity)

- The "node-capped queue that drops the deepest nodes" (§1) has no stated cap
  and the pinned nested fixture sits at input depth 2 (`{ tools: [{ path }] }`,
  independence.test.ts:126); the cap must be pinned above that for Test
  strategy #3 to be deterministic — a specification detail for the
  complexity/test seats.
- `brief()` is module-private in independence.ts:204; §3 uses it from
  runner.ts, so the implementation step must export it — implementation
  housekeeping, not a design defect.
