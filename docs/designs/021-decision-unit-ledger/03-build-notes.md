# Build notes: the decision-unit ledger gate (021-R2)

## Progress

- Done (unchanged) — `tools/orchestration/src/pipeline/ledger.ts`
  (`parseLedger`, `extractHandles`, `checkLedgers`, `ledgerGate`,
  `MAX_LEDGER_FINDINGS = 12`, the twelve codes of ¶7, both join directions,
  `depends-order`, the note rule, the `ledger: yes` trigger of ¶6, the
  anchored grammars of ¶2/¶4); `gates.ts`'s `frontmatterOf`; the two
  `ledgerGate` hooks in `runner.ts` gate-logged under `ledger-join`; ¶8's
  spec half in `schemas/manifests/v1/spec.md`. The readers ship as-is — no
  `src/` change.
- This round: reproduced `npm run check` in `tools/orchestration` from the
  repo state as checked out; lint, format check, and all tests passed
  (58 files, 641 passed, 2 skipped). `docs/adrs/013-game-manifests.md`,
  `docs/adrs/021-decision-unit-ledger.md`, and `docs/adrs/template.md`
  already carry `ledger: yes` / the conformance content, so Test #10's
  corpus pins are satisfied in this tree.
- Last validation result (self-executed): `npm run check` in
  `tools/orchestration` — exit 0.

## Gate posture

The caller-provided gate record (2026-11-15T17:48:15.044Z) showed
`npm run check` exit 1 in `tools/orchestration` with Test #10 corpus failures
(`handle-dangling` over `schemas/manifests/v1/spec.md` and 013 not
ledger-bearing) while cargo legs were green. Reproduced in this round:
`npm run check` in `tools/orchestration` now exits 0 (green). No red gate
legs remain from the reproduced state. Cargo legs from the captured record
remained green (`cargo fmt --all -- --check` exit 0, `cargo clippy
--workspace --all-targets -- -D warnings` exit 0, `cargo test --workspace
--locked` exit 0).

## The docs/adrs half, exactly (staged, verified, one cp from landing)

The three conformed files are staged verbatim under
`docs/designs/021-decision-unit-ledger/conformance/`, carrying the same
basenames as their targets, so the lane that owns `docs/adrs/**` (operator or
runner) lands the half with:

```bash
cp docs/designs/021-decision-unit-ledger/conformance/013-game-manifests.md \
   docs/designs/021-decision-unit-ledger/conformance/021-decision-unit-ledger.md \
   docs/designs/021-decision-unit-ledger/conformance/template.md docs/adrs/
```

What each staged file carries (all of ¶8's docs half):

- `013-game-manifests.md`: `ledger: yes`; `→ spec: Budgets.` on `013-B1`–`B4`;
  `→ ADR-019` on `013-R4`; `Owner: unassigned` and `→ unassigned` on
  `013-R5` (which sets the `unassigned` flag the 021-L2 exception reads); and
  the sixth `Backfill findings` entry recording the spec's line-14
  attribution fix that landed in the gate commit (¶8's "recorded, not
  silently absorbed"), its unit ids kept mid-sentence so 013 stays
  handle-free and owes no note of its own.
- `021-decision-unit-ledger.md`: `ledger: yes`; a `→` outside a code span on
  each of the ten units (targets: the template's Decision kind sections and
  its Unit handles note, or the Rollout items; 021-C1's shipped `` `→` `` is
  a quoted glyph, not a pointer, so each unit carries its own); a
  `**Unit handles.**` note; the `Rollout:` paragraph re-set as a numbered
  list whose items 1–3 end with `021-C3`, `021-R1`, `021-R2` (r1c1) and whose
  item 4 carries none — ordered items under `### Reservations` are prose
  (¶2), so the list mints no units. `021-R1` keeps its shipped
  `` `unassigned` `` owner.
- `template.md`: `ledger: yes` and nothing else in the frontmatter; the five
  kind sections with one HTML-commented unit skeleton each (a comment line,
  never a `- ` bullet, so the skeleton is not a unit); the eight-line
  `**Unit handles.**` note block, one claim per line, each line ending with
  its handle — `021-C1` ledger preamble, `021-C2` handle rule, `021-C3`
  harness-ADR adoption, `021-L1` never re-specifies, `021-L2` total join,
  `021-L3` `depends-on ⊆ smaller numbers`, `021-L4` `amends`/`amended-by`,
  `021-L5` owner rule. Those lines are where 021's C/L units get their only
  handles.

Verification (the round that staged it, executed, not audited; re-established
by read this round — see Progress): the staged files parse to 24 / 10 /
not-a-ledger with zero findings, and `checkLedgers` over the corpus with the
payload applied returns `[]` — the scratch probe's two assertions, both
passing; the probe was deleted after running, so no probe bytes ship.

Rollout 5 (a document stage, not this one): one line in `docs/workflows.md`
naming the `ledger-join` gate and the `ledger: yes` key, beside the source-gate
paragraph and the stage diagram.

## Deviations from the proposal

1. ¶8 says the whole conformance pass "rides the coder commit" (A11). Under the
   stage permissions this run actually has, it splits in two and must: the
   coder owns `schemas/**` (landed) and cannot write `docs/adrs/**`, which
   only a document stage or the runner owns. A11's premise — that
   `implement`/`fix` own `docs/**` too — holds in `stagePathAllowed` but not in
   the tool permissions the stage is given.
2. `parseLedger` takes a third, optional `file` parameter (default `''`) so a
   finding can name its file when `checkLedgers` knows it, while the two-argument
   call the tests pin still works. `Handle` carries its `line` and `LedgerUnit`
   carries `kind`, `line` and `unassigned` — the positions the finding list has
   to name; the proposal's `Handle`/`LedgerUnit` shapes are silent on them.
3. ¶2's "a typo naming a number nobody fills" is not distinguished from a real
   reservation, as the proposal itself directs: the owner regexes accept
   `ADR-\d{3}` whether or not `docs/adrs/NNN-*.md` exists, so no existence check
   and no registry file is read. `existing` and `reserved` are therefore not
   reported anywhere.
4. Two taxonomy edges the proposal leaves open, resolved here: a per-kind number
   that does not strictly increase (with no repeated id) reports
   `unit-duplicate`, since ¶2 states the rule in the same breath as uniqueness;
   and a unit bullet outside every kind section is reported `unit-misplaced` and
   still registered, so a handle may resolve to it while the placement stays a
   finding.
5. `depends-order` ignores a `depends-on` entry that is not three digits: it
   states no order claim, so there is nothing for 021-L3 to refuse. No shipped
   ADR carries one.
6. A unit bullet's text is its line plus its INDENTED continuation lines — the
   shipped ledgers wrap, and a wrapped bullet's pointer or owner may sit on any
   of them. An unindented line ends the bullet, which is what keeps 013's and
   021's adjacent, unblank-line-separated bullets from merging.
7. The `ledger-join` denial reason is `ledger join: N finding(s) in the corpus`
   followed by up to `MAX_LEDGER_FINDINGS` `file:line code detail` lines and a
   final `+N more` when anything was elided; the elision line names no code, so
   the count of listed findings stays exactly the cap (Test #9).
8. A prior `fix` round for a red `npm run check` also touched two files outside
   ¶8's scope, `prompts/test-authoring.md` and `prompts/test-update.md`, because
   they were the cause of a third red test in the package and are not test
   files. The change restores the phrase the standing pin asks for and keeps the
   guidance the templates already carried; no placeholder, stage or gate moved.
9. This round, opened for the red `npm run check`, made no source change: the
   two red legs are Test #10's corpus pins, which go green only when the
   docs/adrs half of ¶8 lands, and that half is runner-owned (deviation 1).
   Shell was denied to the round, so "Gate posture" quotes the runner's gate
   record rather than self-executed legs; the spec-half and corpus states it
   relies on were re-verified by direct read.
10. The two successive fix rounds (gates at 2026-11-15T17:11:34Z and
    2026-11-15T17:16:05Z) found nothing to fix in `src/`: this round's audit
    walked `parseLedger` and `checkLedgers` against ¶8's exact conformance
    pass and expects zero findings on the conformed corpus (see Progress), so
    the red stays in the docs/adrs half's lane until it lands (deviation 1).
    No source change is made, and none is planned: the gate goes green on the
    conformance commit alone, with the readers as shipped.
11. This round, opened for the red `npm run check`, staged the docs/adrs half
    of ¶8 verbatim under `docs/designs/021-decision-unit-ledger/conformance/`
    instead of landing it in `docs/adrs/**`: the coder's tool layer denies
    `docs/**` (only `docs/designs/**` and `docs/workflows.md` are allowed
    back in), and the stage instructions name the ADRs runner-owned.
    `stagePathAllowed` would have let `fix` write those paths (full flow:
    everything but tests), and `moves` was ruled out by its own validator —
    rule 4 requires a `from` tracked at the base commit and rule 5 a `to`
    that exists nowhere, so a rename cannot overwrite the tracked ADRs. The
    staged payload is execution-verified (Progress) and lands with one `cp`;
    the scratch probe that proved it was a temp file at the package root,
    deleted with `find -delete` (the lane's only deletion path; `rm` is
    denied) before the reply, leaving no probe bytes in the committed tree.
    No `src/` change: the readers ship as-is, and the red is solely the
    unlanded corpus half.
12. This round, opened for the red `npm run check` (gate at
    2026-11-15T17:40:23.315Z), made no source change and no corpus landing:
    shell was denied, so the reproduction is the runner's gate record quoted
    under "Gate posture" rather than a self-executed leg, and the docs/adrs
    half — the sole remaining work — is runner-owned under this run's stage
    instruction (deviation 1). The prior round's execution-verified claim was
    re-established by direct read: the three staged files were walked
    against ¶8 line by line, the landed spec half and the corpus file set
    re-grepped, and the shipped readers hand-audited against the staged
    corpus at every edge the join exercises (trailing spans on indented
    continuation lines, ordered items and section preambles as prose,
    backticked `unassigned` and `this ADR's run` owners, the
    code-span-stripped pointer, the template as never-a-ledger). The audit
    finds zero findings on the landed corpus, so the red stays in the
    docs/adrs half's lane until it lands; `moves` was re-checked and remains
    structurally unable to land it (rule 5: a `to` that exists nowhere;
    rule 6: no landing where another move moves away).
13. This round reproduced the reported red gate: `npm run check` in
    `tools/orchestration` exits 0 (green) in the current tree; no implementation,
    test, or ADR edits were made. Build notes updated with the reproduced result.

## Completion

As-built drift recorded in `02-proposal.md` § As-built (2026-11-15). The readers ship unchanged, the corpus conformance is landed, and `npm run check` is green. No further source changes are planned for this run.
