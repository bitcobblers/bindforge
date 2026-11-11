# Code review — correctness seat, round 1 ratification pass c2

Range: `95b73560fab5571ad65610ffe4f790d080d715c1..HEAD -- ':(exclude)docs/designs'`
(16 files: 4 src, 14 test paths). Judged against `02-proposal.md`, the round-1
addenda in `03-build-notes.md`, and `docs/prompts/audit-by-deeds.md`.
This pass checks only that the post-r1 fix/test stages did not break what this
seat approved at r1.

## Checklist results

1. **Components/interfaces vs proposal** — intact. §1 (`subagent: []`,
   `PATH_ACCESS_KEYS`, capped scan) untouched except the shared `ScanBudget`
   (declared, Perf #5). §2 walk: parent-first, `(via child session …)`
   suffixes, caught partial audit, tripwire — all present. §3 (`failureNote`,
   both attempt-2 sites) untouched in this range. §4 prompt lines and pins
   landed before the range base; this range only added `// proves:` citations.
2. **Assumption register** — honored. A4/A8 stay runtime-probed (deviation 1);
   A6's paging shape is reused; A7's `since` still bounds judgment, not fetch.
   The compat matrix's third route (`GET /session/:id/children`) is now
   implemented with the client's auth header — the matrix's "not whether, only
   which" is finally true for CLI lanes (`cli.ts` threads `server` into both
   `run` and `resume`; `drive` accepts `RunOptions | ResumeOptions`).
3. **Deviations declared** — the r1 addendum lists every behavior the fix
   stage changed (violation-keeping, third route, crawl bounds, walk guards,
   deadline clamp, `basicAuthHeader` extraction, optional `server` field). One
   documentation gap remains (finding 2).
4. **Scope creep** — none undeclared. The deadline clamp and crawl caps are
   declared cost bounds that degrade, never strengthen, the audit.
5. **Build-notes truth** — addendum 2's "npm run check green" re-verified in
   this seat: green, 250/250 in 28 files (up from the notes' 241 because the
   test-update stage added pins — consistent). The stale-test-claims issue
   from r1 has grown (finding 2). `typecheck` is still not covered by `check`
   (package.json:23) and was still not run in any seat (shell denied here).
6. **Error paths fail closed** — yes, and better than r1: a mid-walk failure
   now keeps collected violations (runner.ts:1272-1298) and the attempt still
   fails via line 1299; repeated-cursor and page-cap throws route into the
   same caught partial-audit path; deadline expiry names the truncation.
7. **Previous round's findings** — this seat had no blocker/major. Both
   minors are fixed and pinned: the walk now runs at most once per growth of
   the delegation count (counter at runner.ts:1152/1247-1249; test "pays at
   most one descendant enumeration per attempt"), and the list crawl is
   bounded by a repeated-cursor throw + 50-page budget (test "degrades to the
   partial audit when the session-list cursor never advances"). The gate-entry
   asymmetry the notes' addendum 3 diagnosed matches the code
   (`stageFailure` runner.ts:1911, `escalate` runner.ts:2439), and the fixed
   pin in child-deeds-retry.test.ts asserts exactly that pair — Test strategy
   #6's "one gate entry per failure" shape.

## Findings

1. **minor — the new walk guard keys the whole child walk on the recorded
   part name `subagent`.** `countDelegationParts` (runner.ts:859-871) gates
   the walk at runner.ts:1247-1248; the tripwire keys on the same name. If a
   real server records the delegation part under another name (the open
   question from my r1 nit 5 — established only by fixtures), the walk is now
   skipped *silently*: no enumeration, no warn, no gate line — while under the
   proposal's letter the walk ran after every passing parent audit and caught
   the child's deeds regardless of what the parent's part was called. The
   guard closes a cost hole but removes the design's name-agnostic safety net;
   the notes justify it for cost and the stale-window case, not this one.
   Fix: verify the recorded delegation part name against one real session
   message list, and map the real name in the guard (and `ACCESS_FIELDS`) if
   it differs.
2. **minor — 03-build-notes.md now contradicts the diff a reviewer receives
   (carried r1 finding 3, unfixed).** Deviation 2 still says "the nine
   pre-existing fakes … still expose only `session.create/prompt/remove`" and
   line 102 still says "No test files were modified", but this diff adds
   `session.list` to all nine fakes (Test strategy #9, done by the
   test-update stages) and modifies 14 test paths, including the
   child-deeds-retry pin rework addendum 3 prescribed. No addendum records
   the test-update stages' outcome. Fix: append one short addendum listing
   the test files in the final change set and their `// proves:` mapping.
3. **nit — `directChildSessions` drops a baseUrl path prefix.** `new URL`
   with an absolute path (runner.ts:614) replaces any path component of the
   configured `baseUrl`, while `createClient` hands the whole base URL to the
   SDK. A `--base-url` with a path prefix makes the third route 404 into the
   caught partial audit (fails closed, default URL has no path). Fix: join
   the SDK's prefix instead of the absolute path, or document the
   no-path-prefix constraint on `ServerConnection`.
4. **nit — test comments assert an unverified SDK fact.** The new pins say
   the pinned SDK "exposes no `session.children`" (child-deeds-retry.ts,
   child-session-deeds.ts). The package is not installed in this worktree and
   Rollout step 1 never ran (A4 is register-unverified), so the walk's real
   route is still a runtime probe, not a fact. Comment should say "assumed /
   one probed shape", not pin the claim.
5. **nit — carried: `listChildSessionMessages` still exported**
   (runner.ts:655) while the proposal's snippet keeps it module-private; the
   widened surface exists for the unit tests and is still not in the notes'
   additions list.
6. **nit — carried: the zero-descendant tripwire inspects only the parent
   list** (runner.ts:1265). A child list that itself holds a completed
   `subagent` part whose descendants went unenumerated still triggers no
   warning. Unchanged by this round; the narrower parent-only reading is
   still undeclared.
7. **nit — carried: typecheck evidence still missing.** `npm run check` is
   verified green here, but `check` does not include `typecheck`
   (package.json:23) and `npm run typecheck` has been denied shell in every
   review seat so far. One seat with shell should run `tsc -p tsconfig.json
   --noEmit` and record it.

## Verdict

The two fixes this seat had asked for landed correctly and are pinned; the
third-route and violation-keeping changes match the compat matrix and deepen
fail-closed behavior. Nothing the r1 pass approved was broken. Open items are
one new minor (name-keyed guard), the carried notes/diff mismatch, and nits.
