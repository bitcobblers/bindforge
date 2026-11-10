# correctness review r1

verdict: approve

Scope: whole document, panel pass c0, round 1. Inputs: 02-proposal.md,
docs/prompts/audit-by-deeds.md, the Revision log, no prior findings.

Verification performed (basis for the verdict, not findings): every `verified`
register row was read against its cited code — independence.ts:44–50, 64–82,
112–155, 168–189 (A1, A7, A9, A10); runner.ts:267–285, 473–499, 629–647,
693–710, 839–849, 1121–1126, 1133–1140, 1144–1161, 1239–1284, 1453–1497
(A6, A11, and every line anchor in Context/§2/§3, including "exactly two
attempt-2 sites" — `attempt: 2` occurs at runner.ts:1264 and 1473 only);
independence.test.ts:110–122, 124–130, 134–153, 201–212; fix-trigger.test.ts:15–20,
58–67; prompts.test.ts:179–210 (base sentence at line 206); design-review.md:3–8;
package.json:19–23, 31, 35; test-engineer.md:29–31; design-security.md:17–25.
All hold. The nested-path pin (execute → tools[].path) survives the path-key
rule as §1 claims, and no existing test pins a `subagent` part (adding
`subagent: []` breaks nothing). Both cited URLs are live and support A3's
subagent-as-child-session claim; V1 server docs show `parentID` and
`/session/:id/children`; V2 client docs show `client.session.list` for the
fallback. The nine fake clients named in Test strategy #9 are exactly the
nine `message:` stubs that drive executeInference (runner.test.ts's stub only
feeds listSessionMessages paging tests).

## Findings

- [minor][new] Assumption register A8 / Chosen design §2 ("Deeds of children") —
  A8: "`SessionInfo` timestamp fields unverifiable here", and §2: "children
  need no creation-time filter because the message-level `since` filter already
  excludes messages from before this attempt (A7, A8)". The stated basis is
  contradicted by repo evidence: `src/opencode-runner.ts:264–265` already reads
  `session.time.created` / `session.time.updated` off the typed v2
  `client.session.create` result, so the source §1 filter — child sessions
  "created at or after the attempt's `since` stamp" — is implementable at the
  enumeration level, not just at the message level. The deviation itself is
  defensible for fresh attempts (children of a just-created session are always
  newer than `since`), but on the resumable lane (same sessionID across
  attempts) a still-running background child spawned by attempt 1 emits
  post-`since` messages during attempt 2, and the message-level filter does not
  exclude that; Risks row 3's mitigation ("the `since` filter skips
  pre-attempt messages in any fetched list") covers only pre-attempt messages.
  Fix: correct A8's basis to point at `src/opencode-runner.ts:264–265` and
  either apply the creation-time filter in the BFS (the session objects the
  walk already fetches carry the stamp), or state the still-active-older-child
  residual explicitly in Risks row 3 instead of implying `since` closes it.

- [minor][new] Chosen design §1 ("Matcher: paths only, not payloads") —
  "the grep-style reading of `pattern` as query text is a known residual
  (Risks row 2)". Risks row 2 is "An unknown tool names its access under a key
  outside `PATH_ACCESS_KEYS` → missed violation" — a false-negative class. The
  `pattern`-as-query residual is a false-positive class (an unknown grep-like
  tool whose query string contains `transcripts/` fails the attempt — the
  defect this design exists to remove, narrowed to unknown tools), and no Risks
  row covers it, so the pointer anchors the residual to a row that does not
  describe it. Fix: add a Risks row ("an unknown search-like tool names its
  query under `pattern` → false violation; likelihood Low–Medium, impact Low:
  named `grep` is already exempt at independence.ts:74 and pinned at
  independence.test.ts:201–212, so only unknown search tools carry it") and
  point §1 at that row.

- [minor][new] Assumption register A3 — "the server exposes
  `GET /session/:id/children` and `GET /session` | verified |
  https://opencode.ai/docs/agents/, https://opencode.ai/docs/server/". The
  server-docs page is the OpenCode V1 docs (its top banner: "These are the
  docs for OpenCode V1. The latest docs, for OpenCode V2, are at
  .../llms.txt"), while the design pins `@opencode/client ^2.1.4` (V2) and the
  repo records that the v2 client namespaces its routes (`session.create`
  hits `/api/session`, src/opencode-runner.ts:17–18). The V1 page therefore
  supports `parentID` and a children route for V1 only; the Compatibility row
  "OpenCode server with `GET /session/:id/children` | yes (child walk's
  primary path (A3))" and Risks row 1's "both exist server-side (A3)" rest
  that V2 claim on a V1 page. Fix: re-anchor the endpoint half of A3 to the
  V2 API reference (opencode.ai/docs/api/) or mark it "verified on V1 server;
  V2 surface covered by A4/implementation step 1", and note that the V2
  client docs (opencode.ai/docs/build/client/) already show
  `client.session.list(...)` exists, which firms up the named fallback.

- [nit][new] Chosen design §1 — "Update the two doc comments that promise the
  full scan (independence.ts:64–68 and the header scope-discipline block)".
  Only the 64–68 comment promises the full scan; the header scope-discipline
  block (independence.ts:18–28) states the opposite ("Only the fields that
  NAME where a tool looks are scanned") — it is already aligned with the new
  rule and is where a line about delegation would be added, not a promise to
  retract. Fix: one clause, e.g. "update the 64–68 comment to state the
  path-key rule and extend the scope-discipline block with the delegation
  rule".

## FYI (no severity)

- Chosen design §2 cost note ("once per attempt after the reply — off the
  poll loop"): the audit runs after each completed reply turn, so a
  nudged attempt re-runs the child walk per reply, not strictly once —
  same frequency as today's matcher, but the cost row's "per attempt"
  phrasing slightly understates it; cost framing belongs to the
  performance seat.
- Risks row 2's "the same accepted class as obfuscated shells" and the
  Deferred section are consistent with the source's fence-primary posture;
  no deferral contradicts docs/prompts/audit-by-deeds.md.
