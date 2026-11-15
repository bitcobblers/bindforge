# correctness review r1 (dissent pass c1)

verdict: approve

All ten r1 findings were checked against the revised text and the shipped
corpus, and all are closed for real, not by rewording: the widened guard
`(?![\dA-Za-z])` refuses every case Test #1 pins (re-derived per case); ¶2's
owner rule now matches the three shipped `Owner:` spellings (013-game-manifests
.md:147/150/153/155, :156–158 has none — the `owner-missing` shape; 021's :101
backticked `unassigned`, :108 `this ADR's run.`) and Test #5 pins them; the
pointer rule is `→`-outside-a-code-span, which makes A2's no-pointer claim
true (021-C1's quoted `→` sits in a span at :56); A5's four kinds now cover
every backticked unit-id span in the corpus (verified by grep: line-final,
cell-final, the three mid-paragraph lines :27/:168/:183, and exactly the two
mid-sentence examples spec.md:17 and 021:59); Test #7's fixture is buildable
under the reworded ¶6; the `- `-bullet rule keeps the shipped `Rollout:`
paragraph prose while ¶8's trailing handles still bind; A3 cites `:50–158`
(7C+8L+4B+5R = 24 ✓); A24's basis checks out line for line against
`.github/workflows/check.yml` (npm run check, tools/orchestration,
pull_request, ubuntu-latest, node 22.23.5); `MAX_GATE_LEGS` (gates.ts:310) is
the real analogue for `MAX_LEDGER_FINDINGS`; the twelve-code list counts
twelve; spec.md:14's corrected pairing matches 013-R1 (codec→ADR-017) and
013-R4 (seed set→ADR-019); and `frontmatterOf(content): { front, body } |
undefined` can serve `parseSource`'s use of both `front[1]` and
`content.slice(front[0].length)` (gates.ts:55–57). Deferred items re-checked:
none contradicts ADR-021 (021-R2's four classes are all covered by the
twelve-code taxonomy; the `amends` deferral is outside R2's list).

## Findings

- [nit][new] Chosen design ¶2 (Unit grammar) — "an owner matching neither
  `ADR-\d{3}`, `unassigned` nor `this ADR's run` is `owner-unknown`" — every
  owner the design itself accepts carries a suffix beyond the shape name: the
  shipped forms are `ADR-017 (reserved).`, `this ADR's run.` and — for
  021-R1 — `unassigned — a rollout task, not an ADR slot.`, so exact equality
  cannot accept any of them while ¶2 says they "all parse" and Test #5 pins
  them clean; the rule must mean *begins with*, but "matching" also admits a
  search reading, under which `Owner: see ADR-017` would parse clean. Fix:
  one word — "an owner not **beginning with** `ADR-\d{3}`, `unassigned` nor
  `this ADR's run` is `owner-unknown`".

- [nit][re-raised] Chosen design ¶8 (spec.md bullet) — "line 14's \"ADR-016,
  ADR-017\" is corrected to the pairing `013-L2`/`013-R4` state gives" — the
  spec.md half of my r1 nit is closed; the other half is not: 013-L2 does not
  *state* that pairing positionally (docs/adrs/013-game-manifests.md:96–97
  reads "the seed set's and the codec's — ADR-017, ADR-019", positionally
  seed set→ADR-017, codec→ADR-019, the reverse of both the corrected spec:14
  and its own R1/R4). After ¶8 lands, spec.md:14 is the only half of the
  shipped pair that reads right. Fix: extend the same editorial edit to
  group L2's parenthetical ("the seed set's (ADR-019) and the codec's
  (ADR-017)"), or attribute the pairing to `013-R1`/`013-R4` alone and note
  L2's flat order inside the sixth `Backfill findings` entry.

- [nit][new] Goals / Non-goals — "no Rust-side re-implementation, no
  CommonMark dependency (Deferred)" — Deferred / out of scope lists neither
  item; they are rejections, recorded in Options ("A CommonMark parse …
  Rejected") and the Compatibility matrix ("Rust product … out of scope").
  With ¶9 — the old second pointer — deleted, "(Deferred)" is the only
  citation and it under-cites two of the items it covers. Fix: point the
  parenthetical at "(Deferred; Options and matrix)", or add the two one-line
  entries the heading ("Deferred / out of scope") already admits.

## FYI (no severity)

- ¶5 now defines the corpus purely positively ("every markdown file under
  `docs/adrs/` and under `schemas/`") and drops the named exclusions of
  `tools/orchestration/prompts/` and `docs/workflows.md`; behavior is
  identical under the positive definition, but the explicit exclusions were a
  clarity hedge another seat may want restored.
- Rollout 1's "Only `.ts` files land here, so the format leg is untouched" —
  prettier (with the sort-imports plugin) does still check the new `.ts`
  files; what the change achieves is that prettier cannot rewrite fixture
  *content*. The sentence slightly overstates; the mechanism is sound.
- Checklist item 2 (research-URL spot-check): not applicable this pass —
  01-research.md is not among the dissent-pass inputs; the CommonMark and MDN
  URLs in the body are unchanged since r1.
