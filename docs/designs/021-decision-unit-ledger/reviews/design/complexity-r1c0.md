# Complexity review r1 (pass c0, whole document)

verdict: request-changes

Checklist mapping: #1 reuse checked against `tools/` (no markdown/frontmatter
reader, no ADR linter and no corpus walker exists in `tools/orchestration/src`
beyond `parseSource`'s frontmatter regex, which the design reuses by extraction;
`GateResult`/`allow`/`deny` are reused — no duplicate-invention finding). #4 new
dependencies: none, and A14's claim holds (`dependencies` is `@opencode/client`,
`yargs`, `zod`; `engines.node` is `>=22.23.5`, so A15 holds). #2/#3/#5/#6
produce findings 1–4. #7 value check at the end.

## Findings

- [major][new] Chosen design ¶8 (`docs/adrs/021-decision-unit-ledger.md`
  bullet), with Options ("**`021-R1`'s owner stays `unassigned`.** Rejected") and
  Deferred — "`021-R1`'s owner becomes `ADR-022 (reserved)`, 021-L5's legal form
  for 'not written yet' (A18), with a one-line `Reserved: 022` note in its text" —
  nothing but the design's own grammar requires this moving part, and it is the
  one reservation the number allocator will hand out. (a) 021-L5 as shipped lists
  two legal forms — "an existing or reserved ADR number, or this ADR's own run —
  **or carries an explicit `unassigned` marker**" — and 021-R1 already carries one
  (`docs/adrs/021-decision-unit-ledger.md:101`: "Owner: `unassigned` — a rollout
  task, not an ADR slot"). The re-mark is needed only because ¶2 accepts the bare
  spelling `Owner: unassigned` and so rejects the shipped backticked one — test #5
  says so outright ("`owner-missing` for `021-R1`'s pre-run `unassigned` text with
  no marker"). So the Options rebuttal, "021-L5's whole point is a named owner",
  misstates the law it cites: this is a rejected finding I re-report because the
  rebuttal is unsound. (b) Its cost is not zero: it rewrites the run's own source
  ADR's Decision against that unit's stated "a rollout task, not an ADR slot", and
  it re-points the four product backfills and their order at "ADR-022 (reserved,
  021-R1's new owner)", contradicting 021-R1's Decision text. (c) ADR-022 is
  precisely the number the allocator emits next: A18's own `nextNnn` =
  `max(existing)+1` = `022`, and `identity.ts:73` gives that value to every
  non-ADR-sourced (brief) run — so the next brief run mints
  `docs/designs/022-<slug>/` while ¶2's reserved test (presence of
  `docs/adrs/NNN-*.md`) keeps reporting 022 reserved, so the squat is invisible to
  the gate that is supposed to make reservations checkable. The 013-R1–R4 precedent
  works only because 017/019 sit below the counter's reach, and ¶2's
  bounded-residue argument ("a mistyped number cannot be handed out to someone
  else") holds only for numbers ≤ the counter's next value. Fix: accept the shipped
  spelling — ¶2's owner rule reads `Owner: unassigned` backticked or bare — then
  ¶8 drops the R1 re-mark, the `Reserved: 022` note and the ADR-022 reservation;
  Deferred's backfill entry keeps pointing at 021-R1's `unassigned` rollout task as
  the ADR words it; test #5's first case becomes a clean case. If a named owner is
  genuinely wanted, reserve a number the allocator cannot reach (≤ 021) and add the
  R1 re-mark to Goals with its reason.

- [minor][new] Chosen design ¶8 (template bullet) — "`docs/adrs/template.md`:
  frontmatter gains `series:`, `depends-on:`, `ledger: yes`" — only `ledger: yes`
  traces to a Goal (bullet 3: "`ledger: yes` as the machine-readable marker of a
  ledger-bearing ADR"); the other two are knobs with no consumer. No code in
  `tools/orchestration/src` reads `series` (grep: no match), and in the corpus it
  is a literal label, not a template: all five series ADRs spell
  `series: 011-015`, so the template would tell 017–019 — which Rollout 4 makes
  "ledger-bearing at birth" — to declare membership in the 011–015 series they are
  not in. `depends-on:` is consumed by ¶7's `depends-order`, but 021-L3 makes it
  optional ("Where a `depends-on` key is present"), so a template value is a
  default nobody asked for. Fix: the template gains `ledger: yes` only; if the
  optional keys should be visible, use the template's existing commented style for
  `depends-on:` and leave `series:` out of it.

- [minor][new] Chosen design ¶3 + ¶7 taxonomy — "No shipped unit uses either
  (research, Risks), so the check is symmetry, not a workflow" — four of the
  sixteen codes (`amends-dangling`, `amends-backlink`, `amends-order`,
  `unit-id-reuse`), the `amends`/`amended-by` parsing and all of test #8 serve a
  notation with zero corpus instances and none of the four classes 021-R2 names;
  the ADR leaves "the gate's exact error taxonomy" to this run, so the deferral is
  ours to make. Smaller design sketch (checklist #3/#5): the same two readers, the
  same two hook points, ¶3 reduced to the one sentence that defines the notation
  021-L4 asks for, twelve codes and no test #8 — the check rides the first
  amendment that actually carries `amends`, and nothing in Goals, Risks or the
  corpus test (#11) loses coverage, since #11 can only exercise shapes the corpus
  ships. Fix: move ¶3's symmetry checks to Deferred ("Owner: the first amendment
  that carries `amends`"), drop the four codes from ¶7's list and the twelve-line
  denial count with them, and delete test #8.

- [minor][new] Chosen design ¶4 — "A handle is the maximal trailing run of
  backticked unit ids on a source line: one or more code spans, separated by single
  spaces" — the "one or more code spans" clause has no consumer: every shipped
  handle is a single trailing span, including the multi-id case, whose ids sit
  *inside* one span (`013-B1 013-B2 013-B3 013-B4`, `spec.md:111`), and test #2
  pins only "a multi-id span" and "a table cell whose final span closes the cell" —
  never two spans on one line. The clause adds a grammar branch no fixture
  exercises and makes "maximal trailing run" ambiguous for the span-text-span
  shape. Fix: define a handle as one trailing code span whose content is ids
  separated by single spaces, keeping the trailing-spaces-and-one-`|` allowance;
  C2's "a line may carry several" stays satisfied inside the span, and #2 is
  unchanged.

- [nit][new] Chosen design ¶7 — "denies with the first twelve findings as
  `file:line code detail` lines plus `+N more` — bounded the way `boundedLegs`
  bounds a gate record" — the analogy is right but the bound is an unnamed literal
  stated in two places (¶7 and test #10), while the repo already exports
  `MAX_GATE_LEGS = 32` for exactly this and every other bounded list goes through
  it (`boundedLegs` in `gates.ts:457`, `boundedSurfacePaths`/`elisionMarker` in
  `runner.ts:1881–1892`). Fix: name one exported constant (or reuse
  `MAX_GATE_LEGS`) and have ¶7 and #10 cite the constant instead of "twelve", so
  the two statements cannot drift.

## Value check (checklist #7)

Proportionate overall: one new pure module, no new dependency, two hook calls
beside the existing `sourceAdmissible`/`flipAdrStatus` points, a fence toggle
instead of a markdown parser, and a corpus pass that is four file edits — the
reuse and dependency checks are clean, and the Options section discharges the real
alternatives with reasons. The trimmable surplus is finding 1 (a reserved ADR the
allocator hands out, plus a shipped-ADR rewrite no goal asks for), finding 3 (four
codes and one test for an unused notation) and finding 2 (two template keys);
removing all three costs nothing any Goal or corpus test can see, and takes the
taxonomy from sixteen codes to twelve. That is roughly 15–20% of the design's
moving parts, not 30–50% — no restructure is warranted.

## FYI (no severity)

- ¶2's id regex: the trailing guard `(?![\d])` lets `013-C1` match inside
  `013-C1x`, which Test strategy #1 claims to refuse — correctness seat.
- ¶2 makes the pointer "the last `→` in the bullet", but shipped bullets carry
  `→ spec: …` on continuation lines (`docs/adrs/013-game-manifests.md:55–56`,
  `:99–100`) and the bullet-extent/join rule is never stated — unstated
  implementation surface, correctness seat.
- Test strategy #11 puts a repo-wide corpus assertion inside `npm run check`, and
  the check gate runs only after `implement`/`fix`/`test-update`
  (`runner.ts:3065–3071`); if ¶8 lands in a separate round from `ledger.ts`, the
  red #11 sets `testsGreen` false and routes the run into a `fix` round — the
  Bootstrapping risk's mitigation names the entry hook and finalize, not this
  route — rollout seat.
- A24 and Rollout 4 call #11 "a standing CI leg", but the repository carries no CI
  workflow (`.github/` does not exist), which is why A24 is logged unverified —
  research seat.
