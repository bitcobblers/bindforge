# security review r3 (ratification c0)

verdict: request-changes

Scope: `git diff c1b351d 92f71e0` of `02-proposal.md`, then the full current document. My
round-3 verdict carried no security findings, so this pass reports only what the revisions
changed: the `planner.md` permission-list rewrite (r2), the A12 addition, the `validated`
throw-site rewrite, and the "heavier flow" → "full flow" language swaps.

## Checklist walk

1. Trust boundaries — unchanged by the diff. Source frontmatter is operator input, `opts.cfg`
   is caller data (validated), the only path intake is git plumbing output (A28), transcripts
   are closed to document-stage agents. No shared binding-file parsing enters scope.
2. Parsing untrusted data — unchanged: CRLF-tolerant regex scan, no YAML, case-sensitive value
   match, absent → `full`, `planMaxLines` bounds the plan, no evaluation, no decompression.
3. Path handling — unchanged: intake is repo-root-relative POSIX with `.`/`..`, absolute and
   backslash shapes rejected by `moves.ts` Rule 1; the size meter is a JS line count, not a
   shell-out; case stays sensitive, pinned in #7.
4. Secrets/PII — nothing in the changed text touches storage or logging of secrets.
5. Dependencies — the design names no new dependency (a stage row, an agent file, a skill
   directory, local registry data), so there is no advisory/CVE to check.
6. Privileges — the r2 rewrite repairs the inverted deny list; the planner now denies
   `transcripts/**` (read and edit) and `shell: "*"`. One support gap, below.
7. Fail closed — the throw-site rewrite moves the seat-panel rejection off the posture that
   guaranteed a clean rejection. Below.

## Findings

- [major][new] Budget resolution — "A violation throws as a usage error at the config-read
  seam, as a bad `flow` value does." — this revision replaced "throws at the bad-`flow`
  posture", which the document defines (Selecting the flow) as pre-lock and
  "never a half-started run: no lock, no design directory, no session", with a site the
  document places **after** the lock. `configFor`'s first argument is `state.flow`
  (`flowOf(state)`), and `state.flow` is set "immediately after `freshState`, so the entry save
  carries it (`runner.ts:2102`)"; ADR-010 likewise resolves budgets "from `state.flow` inside
  `runPipeline` and `resumePipeline`". So the config-read seam is necessarily post-`freshState`
  — after the lock, after the design directory, after the entry checkpoint save — and a
  caller-supplied `opts.cfg` with `reviewSeats.code: []` (the fail-open the r1 security closure
  exists for: `seatsConverged` vacuously true, a code review that converges with zero sessions)
  now throws as a partially-applied run, not as the clean usage error the sentence claims
  equivalence with. Checklist 7 ("reject cleanly, never crash or partially apply"). No test
  covers it: #23 asserts only that it "throws", #17's clean-failure assertions (`no lock file,
  no design directory`, zero client calls) name only `flow: sideways` and `--stage research`.
  Fix: state that the caller-supplied panel is validated at the same pre-lock posture as the
  `flow` value — validate `opts.cfg` through the existing `validated` rule before the lock and
  before `freshState` — and extend #17 to a `cfg` with `reviewSeats.code: []` asserting no lock
  file, no design directory, no entry checkpoint, zero client calls. One sentence plus one test
  case; no new mechanism.

- [minor][new] Assumption register A12, with The `plan` stage — "no document-stage primary
  agent reads, writes or shells — `designer.md` and `researcher.md` both deny
  `transcripts/**` (read and edit) and `shell: \"*\"`" — the clause added in this diff states a
  universal over document-stage primary agents and marks it `verified`, but the Basis cites two
  agent files. The design's own flow description (Context; ADR-010's `full` stage list) names
  further document-stage primary agents — the editor and the doc-sync stage — whose configs are
  not cited, so the enumeration supporting the claim is incomplete as written. It matters
  because the claim is what licenses copying `designer`'s list to the planner: if a document
  stage outside the two cited files holds shell, the "no document-stage agent has either
  privilege" sentence is false and an implementer working from the summary (not from
  `designer.md:4-22`) has cover to grant the planner shell. Fix: cite every document-stage
  primary agent in A12's Basis, or narrow the sentence to "neither `designer.md` nor
  `researcher.md` grants it" — the claim the citations actually carry.

- [nit][new] The `plan` stage / A12 — "`design/designer`'s permission list verbatim —
  `mode: primary`, edits `docs/designs/**`, and denies `.pipeline.json`, `transcripts/**` for
  read and edit, and `shell: \"*\"` (A12)" — A12's summary of the same list omits
  `.pipeline.json`, so the two places that state the planner's shape state it twice, with
  different entries. Fix: add `.pipeline.json` to A12's parenthetical so the one authority
  names all four entries.

## Closed by this revision (no action)

- The r2 fix is sound: the planner's text now reads as a deny list, `transcripts/**` for both
  read and edit plus `shell: "*"`, and the sentence that read as grants is gone; the width
  claim in the heading is the Ownership widening, which the Ownership section states as the
  `docs/**` document-stage rule and pins as **allowed** in #7/#18 — claim and gate agree.
- The "heavier flow" → "full flow" swaps (The `plan` stage, Risks, Goals) keep the escalation
  strings naming the flow the promotion recipe names (`set flow: full and re-run --force`), so
  an operator reading a light stop is pointed at a flow that exists. No security meaning moves.

## FYI (no severity)

- The plan cap's move from the `plan-note` skill to the registry, with the prompt interpolating
  `{{planMaxLines}}`, is disclosed as superseding ADR-010's "the `plan-note` skill carries the
  template and its length cap" (The `plan` stage; Revision log, r1 OQ3/OQ4 entry). It strengthens
  enforcement — skill text is advisory (A13) — so the deferral does not contradict the ADR's
  requirement; the wording-vs-Decision mismatch is an ADR-consistency seat's, not mine.
- `plan` joining `CODE_STAGES` in `report.ts` for reporting only, while ownership governs the
  same name by the document-stage rule, is a deliberate split (Reporting); it is the reason the
  escalation suffix is gated on the **stage**, not `escalationPhase`. Consistent as written.
