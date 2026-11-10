# compatibility review r1 (dissent pass c1)

verdict: approve

Scope as given: diff `ce95f3b..f4f64f9` of `02-proposal.md`, then the full current
document. Inputs: `02-proposal.md`, `docs/prompts/audit-by-deeds.md`, the Revision
log, my r1 findings. Checklist applied: `references/compatibility.md` only.

## Carried from r1 — closed

- [major] Compatibility matrix row "OpenCode server/SDK without a children
  method" read `unknown`, leaving the "no children method and no session-list
  method" combination unclassified. The revised cell now reads `yes` and carries a
  step-1 decision rule in the Notes cell, as my r1 fix asked; no matrix row is
  `unknown` or empty any more, so checklist item 1's gate condition is met. The
  two findings below are about the *wording* of that new cell, not its status.

## Findings

- [minor][new] Compatibility matrix — row "OpenCode server/SDK without a
  children method | yes | Determinate at step 1: SDK `session.children`, else
  SDK `session.list` + `parentID` filter, else a direct
  `GET /session/:id/children` …" — the row certifies an environment in which the
  *server* may lack a children method, yet remedy 1 and remedy 3 of the cascade
  are that same endpoint (its SDK wrapper, then a raw call to it), so in the
  environment the row names only one route is actually live, and "all three
  exist server-side (A3), so the walk always has a route" counts three routes
  where two are the same route. The `yes` still holds (A3 verifies
  `GET /session`, which the `parentID`-index branch needs), but the cell
  misdescribes the environment it certifies, and an implementer reading it
  cannot tell whether a server without the endpoint is in scope. Fix: retitle
  the row "OpenCode SDK without a children method" and, in that cell, name the
  one route that environment guarantees (`session.list` + `parentID`); if a
  server lacking `GET /session/:id/children` is a supported combination, give it
  its own row instead of covering it with a cascade that calls that endpoint.

- [minor][new] Compatibility matrix — "a direct `GET /session/:id/children`
  against the configured `baseUrl` with the client's auth headers" — this
  revision adds a mechanism to the basis of a `yes` cell: the premise that
  `runner.ts` can reach the server outside the typed `@opencode/client` and
  supply its auth headers appears in no assumption row (A3 verifies only that
  the endpoints exist server-side; A4 covers the typed methods only), and the
  Risks row that tracked exactly this gap ("The pinned client SDK exposes no
  children/list method, so the child walk needs another seam") was deleted in
  this same diff. The unverified leg of the support claim is therefore now
  untracked, and the branch is also out of order: a raw call to the children
  endpoint only helps when the SDK lacks a children wrapper, which the
  preceding `session.list` + `parentID` branch already covers. Fix: drop the
  third branch — the cascade already ends at `session.list` + `parentID`, which
  A3 verifies — or register the out-of-SDK request as a new assumption row and
  keep a Risks row pointing at it.

- [minor][new] Test strategy #9 vs Compatibility matrix — "the nine fake clients
  in `tests/pipeline/` … extended with the session-list method their flows
  need" — #9 fixes the fakes' surface to a session-list method while §2
  ("`client.session.children({ sessionID })` when the SDK exposes it, else …")
  and Rollout step 1 ("fix the walk's enumeration route (A4)") say the route is
  chosen only at implementation step 1. As written, the matrix's two OpenCode
  rows are backed by a pinned test surface that may not be the route taken: if
  step 1 resolves to `session.children`, #9's extension never exercises the
  primary path the matrix row 4 claims as `yes` (checklist item 7 — the Test
  strategy must back what the matrix claims). Fix: make #9 route-conditional in
  place — "extended with the enumeration method the step-1 route uses,
  `session.children` or `session.list`" — so the same sentence covers both
  matrix rows.

## Checklist state

- Item 1 (matrix statuses): pass — every row is `yes`/`no`, none `unknown`,
  none empty; the residual wording issues are the two minors above.
- Item 2 (OS input APIs): pass by justification already in the matrix — macOS and
  Windows are `no` with "no OS-specific code touched" / "pure string matching
  over server records"; no input API is targeted.
- Item 3 (Node floor): pass — nothing the revised §2 introduces (async
  generator, `Map`/`Set`, `for await`) sits above the stated Node ≥ 22.12.0.
- Item 4 (native modules): pass — no new dependency; `@opencode/client` is
  already the pinned client.
- Item 5 (keyboard reality): not applicable — no binding or input surface.
- Item 6 (file paths): pass for the changed text — no path construction added;
  the matcher compares recorded strings/globs only.
- Item 7 (platform test per supported platform): pass — Linux is the only `yes`
  platform and Test strategy #9 names the Linux CI `npm run check` run; see the
  third finding for the enumeration-route half of that test's coverage.

## FYI (no severity)

- §2 Deeds of children — "one enumeration per walk, not per node: per session
  `client.session.children({ sessionID })` when the SDK exposes it" contradicts
  itself on its first clause: the `children` route is one call per node, which
  the Cost bullet states correctly ("one children call per session"). Wording
  for the correctness/clarity seat.
- Deferred / out of scope — "values beyond the §1 walk's node cap": the node cap
  belongs to §1's input-value queue, the child walk is §2. Cross-reference slip,
  not a deferral that contradicts the source document.
- §3 Retry feedback — the note is now bounded by `brief(printable(...))`, while
  the source §2 requires attempt 2's prompt to carry "the attempt-1 failure
  reason … and for independence violations the rule that was broken"; if
  `brief()` truncates, the rule and path may be cut from the note. Whether the
  bound is tight enough is a source-fidelity/correctness question, not a
  compatibility one.
