# Complexity review r1c1 (dissent pass)

verdict: request-changes

Scope: `git diff 67e7b57 94fdca9` over `02-proposal.md`, then the full current
document. Inputs: `02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md`,
the Revision log, my r1 findings. Corpus/code claims were checked read-only
against `docs/adrs/`, `schemas/manifests/v1/spec.md`, `docs/adrs/template.md`,
`tools/orchestration/{package.json,src/pipeline/gates.ts,tests/pipeline/}` and
`.github/workflows/check.yml`.

## Findings

- [major][regression] Chosen design ¶8, 021 bullet — "the shipped `Rollout:`
  prose lines end with `021-C3`, `021-R1` and `021-R2`, and `021-R1`'s owner
  stays its shipped backticked `unassigned`, which ¶2 accepts" — the premise is
  false on the shipped text, and with it the work that made 021-R2 handled is
  gone. `docs/adrs/021-decision-unit-ledger.md:110–112` is one wrapped
  paragraph whose lines end with "the product", "017–019" and "adopt at
  landing."; the ids appear mid-line, unbackticked ("backfills land per
  021-R1; (3) the gate lands per 021-R2"), and `021-C3` does not appear in the
  Rollout text at all. ¶4's handle is *one trailing code span*, so none of
  these is a handle. Consequence chain: the template note block (¶8's first
  bullet) carries handles for C1–C3 and L1–L5 only; ¶7's exemption covers
  `021-R1` alone (a reservation carrying `unassigned`); so `021-R2` —
  "Owner: this ADR's run", the run's own unit — ends the conformance pass with
  no handle → `unit-unhandled` → Test #10 ("`checkLedgers` over **this
  repository** is green") red → Rollout 4's "`npm run check` green is the
  acceptance evidence" unreachable, and the Goal "The tree green at merge:
  021's ten units each carry a `→` and a handle" unmet. Regression: the r1 text
  ("the Rollout paragraph re-set as a numbered list whose items 1–3 end with
  `021-C3`, `021-R1`, `021-R2`") did the needed edit; the r1 revision dropped it
  and asserted it was already true. Fix: keep ¶8's simplification (no
  re-marking, no ADR-022) and restore only the handle edit, in the same
  bullet: "the Rollout paragraph's three lines are re-wrapped so they end with
  `` `021-C3` ``, `` `021-R1` `` and `` `021-R2` ``" — one line of ¶8, no new
  mechanism, and Test #10's green claim becomes true. (A17 confirms `docs/adrs/`
  carries unit-id text only in 013 and 021, so 021's Rollout prose is the only
  place `021-R2` can be handled; ¶5's "three note lines" does not change.)

- [minor][new] Chosen design ¶2 — "A reservation's owner is the text from
  `Owner:` to the end of the line, backticks stripped, so `ADR-017 (reserved)`,
  `unassigned` — bare or backticked … all parse" plus "an owner matching
  neither `ADR-\d{3}`, `unassigned` nor `this ADR's run` is `owner-unknown`" —
  the capture and the acceptance test are two different extents, and nothing
  says how they reconcile. `021-R1`'s shipped line is
  "`Owner: `unassigned` — a rollout task, not an ADR slot.`" (ADR:101), so the
  captured text is `unassigned — a rollout task, not an ADR slot`, which matches
  none of the three forms exactly; Test #5 nevertheless requires it clean
  ("`` `unassigned` `` with `021-R1`'s shipped trailing prose") while requiring
  `owner-unknown` for `Owner: someday`. So acceptance is silently an unstated
  prefix/leading-form match — and an over-broad one, since under containment
  `Owner: ADR-017, ADR-019` and `Owner: unassigned forever` would pass. The
  capture rule is wider than any shipped line needs: across the corpus the
  owner form is always what immediately follows `Owner:` (013-R1–R4:
  `ADR-017 (reserved).`; 021-R1: `` `unassigned` ``; 021-R2: `this ADR's run.`).
  Fix: state one rule with one extent — "the owner is the form immediately
  after `Owner:`, backticks stripped: `ADR-\d{3}` | `unassigned` |
  `this ADR's run`; whatever follows it on the line is prose; a line whose text
  after `Owner:` begins with none of the three is `owner-unknown`." Same code,
  no extra branch, Test #5 unchanged and now unambiguous.

- [nit][new] Rollout 1 + Test #9 — "Only `.ts` files land here, so the format
  leg is untouched" and "no committed fixture tree, so prettier never rewrites a
  deliberately malformed ledger" — the stated reason is inverted. The leg is
  `format:check` → `prettier --check .` run from `tools/orchestration`
  (package.json:20, 23; `.github/workflows/check.yml:54–56`), so it *does*
  check the landing `.ts` files, and it never sees the corpus at all
  (`docs/**`, `schemas/**` are outside that directory). The defensible reason is
  narrower: committed `.md` fixtures *would* sit under the leg's path —
  `tests/pipeline/fixtures/` is the repo's existing fixture convention
  (`tests/pipeline/fixtures/checkpoints/*.json`) — and prettier owns markdown
  table padding and line-final whitespace, which is exactly the shape Test #2
  pins. Fix: reword the two clauses to that reason ("fixtures live in the test
  module and are written to the scratch repo of the existing `tmp-dir.ts`
  helper, so no committed markdown under `tools/orchestration` can be reformatted
  out from under a shape pin") — and note that `tmp-dir.ts` exists, which is the
  part of #9 that reads as reuse rather than new scaffolding.

## Checklist

1. Reuse in `tools/`: checked. `frontmatterOf` is a faithful extraction of
   `parseSource`'s own slice and body handling (gates.ts:54–74), `body` has a
   consumer (locating `## Decision`); `GateResult`, `boundedLegs`-style bounding,
   `MAX_GATE_LEGS` naming, `tmp-dir.ts` and the A21 test-naming rule are all
   reused. Clean.
2. Tracing to Goals: ¶2's code-span-aware pointer survives on a real case —
   pre-run `021-C1` is the only `→` in 021 and it is inside a code span
   (ADR:56, A2), so without it `pointer-missing` would not fire for C1 during
   this run's own red window. The line-end owner capture is the one part with no
   consumer beyond `021-R1` (minor above).
3. Smaller design: the r1 cuts (four `amends` codes + their test, the ADR-022
   reservation, the two template keys, the multi-span handle branch, the
   twelve-finding literal) are the right ones — the design is visibly smaller
   and no Goal lost coverage. No further slice found.
4. Dependencies: none added (A14 verified: `@opencode/client`, `yargs`, `zod`).
5. Defer-ability of the second half: ¶8 must land with the gate or #10 is red;
   already minimal.
6. Knobs/abstractions: `MAX_LEDGER_FINDINGS` replaces a duplicated literal —
   fine; no knob without a consumer remains except the owner capture above.
7. Value: proportionate — two pure readers, twelve codes, two hook points for a
   law the ADR says must be mechanical. The one disproportionate element is gone;
   what remains open is a missing work item (¶8), not excess machinery.

## FYI (no severity)

- Correctness/consistency, on changed text: ¶8's last bullet — "line 14's
  'ADR-016, ADR-017' is corrected to the pairing `013-L2`/`013-R4` state gives —
  the seed set's `(ADR-019)`, the codec's `(ADR-017)`". `013-L2`
  (013:96–98) reads "the seed set's and the codec's — ADR-017, ADR-019", which
  positionally pairs seed set→ADR-017 and codec→ADR-019, the reverse of the
  chosen fix; only `013-R1`/`013-R4` (013:147, 155) support it. So the chosen
  pairing is defensible but the citation is not, and leaving L2 untouched keeps
  A19's contradiction class inside 013 (L2 vs R1/R4) after the pass.
- Test strategy #9 is written with a leading space in the ordered list (" 9.
  Integration"); cosmetic, but docs/ is outside the prettier leg, so nothing
  will catch it.
- `unit-malformed` is in ¶7's twelve codes but no test names it; #1 pins the id
  grammar without naming the code.
