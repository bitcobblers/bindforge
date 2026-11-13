# complexity review r2

scope: `reviews/design/editorial-r2.diff` (ratification — the editorial language pass only)
verdict: request-changes

Read: `02-proposal.md` (post-diff), `docs/adrs/010-lightweight-plan-flow.md`, the Revision log,
the diff, and the agent configs the diff's changed sentence describes
(`.opencode/agents/design/designer.md`, `.opencode/agents/code/coder.md`,
`.opencode/agents/research/researcher.md`). My r1 findings are all recorded as closed in the
Revision log; the diff re-opens none of the r1 themes. Of the eleven hunks, ten are pure
language (serial commas, `heavy`/`heavier` → `full`, `the last` → `the latter`, `starts at spent 4`
→ `starts with 4 sessions spent`, the `--stage code-review` force-run clause, the Rollout 6
reflow). Every number in the touched text is unchanged: 4–6 sessions, 6 spent sessions,
`maxRounds: 2`, `2 + seats × 3` ≤ 8, ≤ 96 min, ≈ 9 sessions, 1–2 per `--extend-budget` grant.
One hunk changes technical meaning, and it is the one that breaks a claim.

## Findings

- [major][regression] The `plan` stage, agent bullet — "`mode: primary`, edits `docs/designs/**`,
  denies `.pipeline.json`, **reads and writes `transcripts/**`, and allows shell**" — the
  pre-diff text ("denies `.pipeline.json`, reading/writing `transcripts/**`, and shell") read the
  three items as denials, which is exactly the cited `design/designer` shape
  (`designer.md:11-22`: deny `docs/designs/**/.pipeline.json`, deny `edit` and `read` on
  `docs/designs/**/transcripts/**`, deny `shell: "*"`). The editorial rewrite scopes the list the
  other way, so the sentence now specifies **grants**: transcript read+write and unbounded shell.
  Three consequences, all complexity: (a) it is no longer "the `design/designer` shape … one line
  narrower" — designer denies all of it, so the bullet cites a source that says the opposite;
  (b) the grants have no consumer anywhere in the design — nothing in the `plan` stage reads a
  transcript, and the repo-wide rule is that transcripts are runner-written and not stage input
  (`coder.md:51-54`, `test-engineer.md:50-52`: "the runner audits every attempt's tool calls and
  fails it on any transcripts access"), so an agent configured as written is failed by the harness
  the Goals bullet promises to share "unchanged"; (c) `shell` with no resource scope widens the
  permission surface of a document-stage agent beyond every existing document agent — designer
  denies `shell: "*"`, researcher denies it and adds only `websearch`/`webfetch`, coder denies
  `shell: "*"` and adds narrow command allows. This is the security spine's neighbourhood that
  the Risks table already rates medium/high, and the widening is unrecorded there. Fix: restore
  the denials and drop the width claim — "`mode: primary`, edits `docs/designs/**`, denies
  `.pipeline.json`, denies `transcripts/**` for read and edit, and denies shell —
  `design/designer`'s rule set verbatim (`designer.md:4-22`)"; if shell is genuinely needed, name
  the scoped command allow the coder uses (e.g. `wc`/`npm run check*`), never `shell: "*"`, and
  add the row to Risks.

- [nit][new] Budget resolution, `validated` paragraph — "A violation throws at **the same pre-lock
  posture** as a bad `flow` value." Sharpening the r1 wording ("the bad-`flow` posture") asserts
  a lock-relative site for every caller of `configFor`, and the design places only two of the
  three: the flow parse is pre-lock (Selecting the flow), while A4's budget seams
  (`runner.ts:2068,2126`) and the `pipeline status` reader are never positioned relative to the
  lock — and `status` takes no lock, so a pre-lock posture cannot describe it. Test strategy #23
  asserts "throws", not where. Fix: name the seam, not the lock — "throws as a usage error at the
  entry seam that reads the config, the same shape as a bad `flow` value" (drop "pre-lock").

## Checklist walk (item 7 is the value check)

1. **Reuse before new surface** — passes, with one exception: the `plan` stage reuses
   `STAGE_BASES`, the `proposal` reply schema, `prompts/proposal.md`'s shape, the existing
   `codereview/code-correctness`/`code-testing` agents, the `code-review.md` template and both
   review-skill references (A12), and `funnelSpecialties`/`specialtiesFor` delegate to `seatsFor`
   rather than forking. The exception is the finding above: the planner's stated config is
   *wider* than the `design/designer` record it points at, which is the reuse the design claims.
2. **Every moving part traces to a Goal** — `flows.ts` + `flowOf` + `configFor` + `seatsFor` +
   `assertStageInFlow` map to Goals 2/3/5 and to the `--stage` misuse row; the five `cfg`
   parameters on the seat readers trace to Goal 3 (one authority for panel and budget); the
   pre-lock parse traces to Goal 2. No YAGNI in the diff's touched sections.
3. **A design 30–50% smaller?** Not inside ADR-010's Decision. A smaller shape is the ADR's own
   rejected config-only variant (no new stages, five seats remain) — the registry data plus the
   validation contract is what ADR-010 asks for, and the two discretionary pieces
   (`validated`'s seat rule, the `seatsFor` delegation) each close a named failure: vacuous
   `seatsConverged` on an empty panel, and light call sites falling back to the five-seat panel.
4. **New dependencies** — none, and the diff adds none; the one extractor the design would need
   (`planMaxFiles`) is Deferred with the reason it is not chosen (A24).
5. **Deferred second half** — Rollout 1 lands the registry as a no-op behind an unselectable
   `FLOWS.light` (#3), so the first merged slice is thin. Unchanged by the diff.
6. **Knobs with no consumer / single-implementation abstractions** — the knob table is
   consumed (`maxReviews.code: 12` is explicitly declined, the light `design: []` is registry
   data not a tunable, `maxParallelReviews` stays operator-side by spread order). The new
   single-implementation surface (`FlowDef`) is justified by ADR-010's Consequences: the next
   flow is a registry entry. The only consumer-less grant in the diff is the planner's `shell`.
7. **Value** — proportionate. The complexity is one registry module, four exported functions, two
   optional config fields, and a five-parameter threading, against a goal of moving a
   flag-sized change from 43 sessions to ≤ 8; every added part is data, and `full`'s behavior is
   pinned byte-equal (#3, #10). The planner bullet's grants are the one piece of added surface
   with nothing behind it.

## FYI (no severity)

- Context says "the six completed runs spent 43–96 stage sessions each"; ADR-010's table has six
  runs and says "The seven runs on disk". Consistency seat's call, untouched by this diff.
- Rollout 6's escalated ceiling "≈ 9 sessions — plan 1, implement 1, the fix rounds, and at most
  the code phase's 6 cumulative" sums to 10 with 2 fix rounds and 9 with 1; `≈` covers it, and
  the number is unchanged by the diff. Correctness seat's call.
- The diff's `heavy`/`heavier` → `full` normalization is correct against the proposal's own
  vocabulary and against the escalation strings it cites ("set `flow: full` and re-run with
  `--force`", "promote with `flow: full`"); ADR-010's prose still says "the heavy flow" at its
  `plan` bullet — the ADR is not this diff's surface.
