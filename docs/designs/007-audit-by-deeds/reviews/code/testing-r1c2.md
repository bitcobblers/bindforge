# Code review — testing seat, r1 candidate 2 (ratification)

Scope: confirm the fixes that landed after the r1c1 approval did not break what
this seat approved, read against the current `02-proposal.md`.

## How this seat reviewed (method disclosure)

The shell tool was permission-denied again for this entire pass (multiple
attempts: `git diff 1ce23f4..HEAD`, `git log`, `npm run check` — all rejected
with `permission.rejected`). I therefore could not reproduce the scoped diff
byte-for-byte. I reviewed instead by reading the full current state of every
path my r1c1 findings and approvals touched (`src/pipeline/runner.ts`,
`src/pipeline/independence.ts`, `src/pipeline/cli.ts`, the prompts, and
`tests/pipeline/{independence,child-session-deeds,child-deeds-retry,prompts,gates,cli-max-parallel}.test.ts`
plus the nine legacy fakes), cross-checking each line range I quoted in r1c1
against the current file — they match (directChildSessions 609–624, third
route 789–825, no-route throw 827–830, budgetMs 668–677) — and re-verifying
every approved pin and behavior statically.

## What the later fixes changed, and did they break anything approved?

New/changed pieces visible in the current tree beyond the r1c1 state:

- Walk guard keyed to delegation: `countDelegationParts` (any age, runner.ts:859)
  gates the walk, `hasCompletedDelegation` keys only the tripwire (runner.ts:1265).
  Pinned: child-session-deeds.test.ts:563 (no delegation → zero enumeration)
  and :698 (stale part + in-window child deed still fails). No regression to
  the approved call-site shape.
- Walk options object `{ connection, deadline }` (runner.ts:642–659); the call
  site passes the stage deadline (runner.ts:1256). Default `{}` keeps every
  approved three-argument test call (`walk(client, root, 1_000)`) compiling and
  behaving exactly as approved — `budgetMs()` without `deadline` returns
  `requestTimeoutMs`, identical to the approved behavior.
- Paged list crawl hardened: repeated-cursor stop (754–758) pinned by
  child-session-deeds.test.ts:576; a 50-page budget `MAX_SESSION_LIST_PAGES`
  (560, 761–766) — the observable bounded-crawl degradation is pinned; the
  specific page-budget throw line is not (FYI below).
- Partial-walk evidence preservation: violations collected before a mid-walk
  throw survive into the failure (runner.ts:1272–1303), pinned by
  child-session-deeds.test.ts:719. Matches the approved §2 "failure on what is
  in hand" posture.

Re-verified intact from r1c1: `subagent: []` + `PATH_ACCESS_KEYS` matcher
(independence.ts:98,106,258); pins #1–#4 (independence.test.ts:245,264,281,293,305,315)
with the pre-existing nested-path pin at :124 untouched; §3 `failureNote` on
`StageHandle` (1021), appended only when `resume` is absent (1091–1093), set at
both non-resumable sites (1726 runWithRetry, 1935 runReviewPass) — retry budget
and escalation flow unchanged (pinned at child-deeds-retry.test.ts:251–284);
§4 prompt lines present (design-review.md:9, test-authoring.md:8–9,
test-update.md:5–6) with rendered-content pins (prompts.test.ts:207,358); the
nine legacy fake clients carry `session.list` (Test strategy #9); CLI wires
`server: ServerConnection` into both lanes (src/pipeline/cli.ts:175–217) with
no checkpoint footprint.

## Checklist walk

1. Every Test strategy item has a citing test: #1–#4 independence.test.ts,
   #5–#7 child-session-deeds/child-deeds-retry, #8 prompts.test.ts. #9 is the
   CI check itself — see item 6. Pass.
2. Unit (matcher, walk fakes) and integration (resumePipeline over temp git
   repos) layers both present. Pass.
3. Revision rounds: none of the previously addressed findings' tests were
   weakened or removed; every fix carries its own new pin. Pass.
4. Determinism: fakes only, `vi.stubGlobal('fetch')` where fetch is involved,
   temp-repo git in `mkdtempSync` dirs, monotonically ratcheted real clock for
   `since` compatibility; no live server, no ports. Pass.
5. Test names describe behavior ("fails an attempt on a child deed and feeds
   attempt 2 the failure reason"). Pass.
6. Could NOT run `npm run check` — shell denied for the whole pass. Finding.
7. The direct-HTTP route and the deadline clamp remain testable-but-untested
   (carried findings). The `server` option has real consumers (cli.ts), so no
   knob-without-consumer.

## Findings

1. **minor** — runner.ts:609-624,789-830; src/pipeline/cli.ts:167-177: the
   compat matrix's third enumeration route (`directChildSessions`, its BFS, and
   the no-routes throw) still has no test — no test passes `connection` to
   `listChildSessionMessages` and no `fetch` stub exercises the `/api/session/:id/children`
   path, auth header, or response parsing. Carried open from r1c1: the later
   fixes neither addressed nor regressed it; production still never reaches the
   branch (pinned SDK exposes `session.list`) and the call site catches it as a
   partial audit. Fix unchanged: one walk test with a fake client lacking both
   `session.children` and `session.list` plus `vi.stubGlobal('fetch', …)`,
   asserting the request path, that a child enumerated that way is audited, and
   that a no-route client without `connection` lands in the caught
   partial-audit path.
2. **nit** — runner.ts:668-677,1256: the deadline clamp (`budgetMs()`) and its
   "truncated at the stage deadline" throw remain unpinned — no test passes a
   `deadline` (expired or live) to the walk. Carried open from r1c1; the new
   call-site wiring makes the clamp live in production, so a pin would now be
   cheap at the call site (stage deadline already in hand).
3. **minor (procedural)** — checklist item 6 un-evaluated: this seat's shell
   tool was permission-denied for the whole pass, so I cannot paste a green
   `npm run check` tail, and could not mechanically re-derive the
   `1ce23f4..HEAD` diff. Static cross-checks of every current assertion against
   the implementation found no contradiction; the runner's independent record
   stands in.

FYI (no severity, in-specialty, pre-existing at approval): the
`MAX_SESSION_LIST_PAGES` throw (runner.ts:761-766) is the sibling of the
pinned repeated-cursor stop but no test reaches that exact throw; the observable
bounded-degradation behavior is pinned.

## Verdict

**approve** — the post-approval fixes preserved everything this seat approved
(matcher pins, both walk routes and their cost guards, the delegation-keyed
guard, partial-audit evidence preservation, failureNote budget invariants,
prompt pins, legacy-fake extensions) and broke nothing it touched. The two open
test gaps and the un-executable checklist item are unchanged carried findings,
non-blocking.
