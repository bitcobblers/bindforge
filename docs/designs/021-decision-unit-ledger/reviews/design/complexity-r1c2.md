# complexity review r1c2

verdict: approve

Scope: the dissent diff `94fdca92` → `e66564c0` (¶1 BOM clause, ¶2 owner rule, ¶8's
`Rollout:` bullet, Test #9, Rollout 1, Revision log), then the full current
document. Inputs used: `02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md`,
the Revision log, my r1 findings, and read-only checks of the code the changed
text cites (`gates.ts`, `package.json`, `.github/workflows/check.yml`,
`tests/pipeline/tmp-dir.ts`). Checklist applied: `references/complexity.md` only.

## Closure check on my r1 findings

- **major (¶8, the Rollout handles) — closed, verified.** The bullet now reads
  "the `Rollout:` paragraph re-set as a numbered list whose items 1–3 end with
  `021-C3`, `021-R1`, `021-R2`", which is the r1 wording my fix asked to restore.
  Consequences re-checked against the shipped text: `docs/adrs/021-decision-unit-ledger.md:110–112`
  is one wrapped paragraph with four numbered clauses under `### Reservations`,
  ids mid-line and unbackticked and no `021-C3` at all, so the re-set is the
  handle edit and not a restatement. Items 1–3 map (template ← `021-C3` per
  021-C3's own "template.md adopts the ledger sections", backfills ← `021-R1`,
  gate ← `021-R2`); item 4 needs no handle (¶4 asks nothing of a line without
  one). Handle coverage closes: ¶8's template note block carries `021-C1`–`C3`
  and `L1`–`L5`, the rollout list carries `R1` and `R2` — all ten units handled,
  so Test #10's green claim is reachable. The re-set adds no mechanism, and
  ¶2 already makes ordered items prose, so a numbered list under
  `### Reservations` cannot read as units. Verified there is no format leg that
  could re-wrap the new line-final handles: the prettier leg is
  `prettier --check .` scoped to `tools/orchestration` (package.json:20,23;
  check.yml:55–56), the other format step is `cargo fmt --all` (check.yml:29),
  and there is no root `package.json` — `docs/**` is outside both.
- **minor (¶2, owner extent vs acceptance test) — closed, verified.** One rule at
  one extent now: "A reservation's owner is the form immediately after `Owner:`,
  backticks stripped … the rest of the line is prose". It parses every shipped
  owner line (`021-R1`: "Owner: `unassigned` — a rollout task, not an ADR slot.";
  `021-R2`: "Owner: this ADR's run."; 013's `ADR-017 (reserved).` form) and keeps
  Test #5's two `owner-unknown` cases honest (`ADR-17` cannot match `ADR-\d{3}`).
  Net complexity reduction: the extra reconciliation the r1 text implied is gone,
  no branch added.
- **nit (Rollout 1 / Test #9, the prettier reason) — closed in substance.**
  Test #9 now names the existing helper and the true reason; verified
  `tests/pipeline/tmp-dir.ts:21` creates the scratch dir with
  `mkdtempSync(join(tmpdir(), prefix))` — i.e. `/tmp`, not under the leg's path.
  One wording residue in the same clause is filed below as a nit.

## Findings

- [minor][new] Chosen design ¶1 — "so both readers share one regex and one CRLF
  posture, with a leading `\uFEFF` dropped (A13)" — the BOM clause is the one
  moving part this revision adds that traces to no Goal (checklist item 2). No
  Goal or risk asks for it; A13's basis is CRLF only (gates.ts:54–57 matches
  `\r?\n`; nothing under `tools/orchestration/src` mentions `\uFEFF` or a BOM,
  checked by grep), so the citation points at a precedent that does not exist;
  Test strategy has no BOM pin (#3 pins CRLF only); the only consumer would be a
  checkout on the platform the Compatibility matrix itself marks "Not tested —
  expectation only". It also opens an offset the text does not resolve:
  `parseSource` derives its body by slicing the *original* content at
  `front[0].length` (gates.ts:57), so a strip inside the shared `frontmatterOf`
  puts that slice one character out unless `parseSource` is switched to the
  returned `body` — the clause buys shared-code behavior plus a coupling for
  input the corpus, the harness and CI never produce. Fix: delete the clause and
  inherit A13's CRLF-only posture; nothing else in the document changes and Test
  #3 is unchanged. If the author keeps it: state that `frontmatterOf` returns
  already-stripped content and that `parseSource` uses the returned `body`
  instead of slicing `content`, add a BOM case to Test #3, and cite a basis that
  shows a BOM case exists rather than A13.
- [nit][new] Rollout 1 — "Only `.ts` files land here, prettier-formatted, and no
  markdown enters the format leg's path (Test #9)" — the second half is false as
  written: the leg's path is `tools/orchestration` (`prettier --check .`,
  package.json:20), which already holds committed markdown (`README.md`,
  `prompts/*.md`), so markdown does enter the leg's path. The narrower claim the
  revision established is true and is what the conclusion needs: no committed
  *ledger* markdown lands there, because Test #9's fixtures live in
  `tmp-dir.ts`'s `/tmp` scratch (tmp-dir.ts:21) and ¶8's markdown sits in
  `docs/**`, outside the leg. Fix: "and no committed markdown fixture lands in
  the leg's path (Test #9)". No design change.

## Checklist pass

1. **Reuse** — improved by this diff: Test #9 now names the existing
   `tests/pipeline/tmp-dir.ts` instead of implying a new scratch mechanism, and
   ¶1's `frontmatterOf` is an extraction out of `gates.ts` rather than a second
   frontmatter regex. No new package or directory; the only new module remains
   `ledger.ts`. No reusable markdown parser is needed (A14).
2. **Each part traces to a Goal** — the twelve codes map to 021-R2's four
   classes (¶7), the corpus pass to the "tree green at merge" Goal, `ledger:` to
   021-R1's split. One exception: the BOM clause, filed above.
3. **30–50% smaller design** — nothing here makes the design bigger; the diff is
   25 insertions / 25 deletions and ¶2's rewrite removes a rule reconciliation.
   No smaller shape is available for the Gate 4 Goal: the conformance pass is
   what makes Test #10 green.
4. **New dependencies** — none; the BOM strip needs none either.
5. **Second half deferrable** — no. ¶8's pass is not the second half; it is the
   Goal, and deferring it leaves Test #10 and the ADR's own units red.
6. **Knobs with no consumer / one-implementation abstractions** —
   `MAX_LEDGER_FINDINGS` is pinned by Test #9; `frontmatterOf` gains a second
   caller by construction. The BOM branch is the only knob-shaped addition with
   no consumer, filed above.
7. **Value** — proportionate. The mechanism count is unchanged from r1 and the
   grammar text got shorter; the remaining cost sits where the ADR asked for it
   (one module, one shared frontmatter helper, two hook calls, three note lines).

## FYI (no severity)

- ¶7's "unregistered unit — an id used, or defined outside a kind section" is
  unchanged text, so I do not re-raise it, but the re-set makes it worth the
  correctness seat to pin the reading: if "an id used" covers *handles* placed
  outside kind sections, then every handle in `schemas/**`, the template's note
  block and now 021's rollout list would be `unit-misplaced`. The intended
  reading is surely "a unit *definition* (a `- ` bullet) outside a kind section".
- ¶2's leading-form `ADR-\d{3}` would read `Owner: ADR-0170` as `ADR-017` with
  `0` as prose; ¶2's own id grammar guards that with `(?![\dA-Za-z])`. Correctness
  seat's call, one character of regex.
- Revision log r1c1 entry — "…ends with no handle — its re-set as a numbered
  list…" reads without a verb; language only, for the proposal-editing seat.
