# correctness review r1 — ratification pass c3

verdict: approve

Scope: `git diff 94fdca92 e66564c0` (¶1 BOM clause, ¶2 owner-grammar rewrite,
Test #9 rewording, Rollout 1 format-leg sentence, Revision log) plus the full
current document. Ratification bar applied: report only where the revisions
broke something approved in r1 or made a section cited there false.

## Checklist result

1. Assumption register: A13's basis (gates.ts:54–75) verified — `parseSource`'s
   regex is `/^---\r?\n([\s\S]*?)\r?\n---/` with `\r?\n` throughout, exactly as
   ¶1 quotes it. No new `verified` row was added or changed by this diff.
2. Research URLs: no URL is added or altered by this diff — not applicable to
   this pass's scope.
3. Internal contradictions: none found in the changed text —
   - ¶2 rewrite vs Test #5: the new rule ("the form immediately after `Owner:`"
     … "text after `Owner:` beginning with none of the three") parses every
     pin in Test #5 (`ADR-17` → `owner-unknown`; `ADR-019 (reserved)`, backticked
     `unassigned` + shipped trailing prose, bare `unassigned`, `this ADR's run.`
     → clean) and every shipped owner form (`ADR-017 (reserved).` at
     docs/adrs/013-game-manifests.md:147,150,153,155; `unassigned — a rollout
     task…` at 021:101; `this ADR's run.` at 021:108). My r1 nit #1 is closed
     for real, not just reworded: "matching" is gone, the search reading is
     excluded by "immediately after", and the shipped-suffix forms parse.
   - ¶8 Rollout re-set: the r1c1 entry's premise is true — the shipped
     `Rollout:` paragraph (021:110–112) is flat prose ending "adopt at
     landing.", no handle. The new claim is an editorial plan, and it is
     consistent: items 1–3 map to C3 (template adoption, 021-C3), R1
     (backfills "per 021-R1"), R2 (gate "per 021-R2"); numbered items stay
     prose under ¶2, so no pointer/unit obligation is created; as trailing
     code spans they are handles under ¶4, which is what makes R2 handled
     under Test #6's exemption rule (R1 is exempt anyway as `unassigned`),
     and 021 gains the ¶5 note in the same bullet — no `note-missing`.
   - ¶1 BOM clause: no register row or section contradicts it; frontmatterOf's
     shape still serves `parseSource`'s existing fallback (`body` when no
     frontmatter) unchanged.
4. Edge cases: empty `Owner:` still lands in `owner-unknown` under the new
   wording, as under the old. The "beginning with" looseness at the form's
   right edge (`ADR-0177`-style typos) is the residue class ¶2's closing
   paragraph already accepts explicitly, so it is not a new defect.
5. Interfaces: `frontmatterOf`, `checkLedgers(repoDir)`, `MAX_LEDGER_FINDINGS`
   keep single shapes across ¶1/¶7/Rollout 2; Test #9's "scratch repo of the
   existing `tests/pipeline/tmp-dir.ts` helper" verified —
   `tools/orchestration/tests/pipeline/tmp-dir.ts` exists, its comment names
   its directories "Scratch repos", they live under `os.tmpdir()` (outside the
   package tree) and are removed in `afterAll`, which is what supports
   Rollout 1's "no markdown enters the format leg's path" (format leg =
   `prettier --check .` scoped to `tools/orchestration`, package.json:20,23).
6. Source-document coverage: unchanged by this diff; 021-R2's taxonomy ask
   still answered.
7. Previous round: my r1 nit #1 is closed by this revision. Nits #2 and #3 sit
   in text this diff does not touch — see below.

## Findings

(none new; no r1-approved text broken by the diff)

## Carried, unchanged by this pass (not re-reported per the ratification bar)

- r1 nit #2: ¶8's spec.md bullet still attributes the corrected pairing to
  `013-L2`/`013-R4`; 013:96–97 still reads "the seed set's and the codec's —
  ADR-017, ADR-019", positionally the reverse of R1/R4. Unchanged text; still
  open as a nit.
- r1 nit #3: Goals' "(Deferred)" still under-cites the Rust-side and
  CommonMark rejections. Unchanged text; still open as a nit.

## FYI (no severity)

- ¶1's BOM drop is a new behavior with no pin: Test #3 pins CRLF tolerance but
  nothing exercises `\uFEFF` (test-strategy seat's ground, if it wants it).
