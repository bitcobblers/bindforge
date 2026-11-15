# Design: The decision-unit ledger — parser, join gate, and self-conforming corpus

## Context

ADR-021 (docs/adrs/021-decision-unit-ledger.md) makes a product ADR's
`Decision` section a **decision-unit ledger** of typed, numbered units, each a
bounded choice with a `→` pointer to the normative text that carries it, and
makes the ledger↔spec join checkable by pattern in both directions: at the
deterministic source step before any inference, and again at finalize
(021-R2). This run is 021-R2's named owner, so it designs the gate — grammar,
check set, error taxonomy, hook points — and the small corpus work that makes
the gate green the day it lands: 021's own ledger carries no pointer
(research, Findings), `013-R5` has neither handle nor `unassigned` marker, and
`docs/adrs/template.md` still shows an undifferentiated `Decision`.

## Goals / Non-goals

- Goals:
  - One deterministic parser for the ledger shape and one for handles, in the
    TypeScript harness, with the anchored grammar 021-C1 needs (021-C1, C2).
  - The join, checked in both directions plus the laws 021-L3 and 021-L5 make
    decidable, at run entry and at `finalizeDone`, with this run's error
    taxonomy — the item ADR-021 defers to "021-R2's run".
  - `ledger: yes` as the machine-readable marker of a ledger-bearing ADR, so
    021-R1's "backfill / forward-only" split has an extension and 017–019
    adopt by construction (021-R1, rollout 4).
  - The tree green at merge: 021's ten units each carry a `→` and a handle, the
    template carries the convention, `013-R5` is re-marked (021-C3, L2, L5).
- Non-goals:
  - No check of 021-L1's "never re-specifies" test — a reading judgement
    (Deferred), enforced by the design-review seats.
  - No resolution of `→` targets, no check of prose `ADR-NNN` mentions, no
    Rust-side re-implementation, no CommonMark dependency (Deferred).
  - No backfill of 011, 012, 014, 015 (021-R1's `unassigned` rollout task), and
    no ledger for harness ADRs 001–010, 016, 020 (021-C3: next amendment).
  - No change to `stagePathAllowed` or to the ADR-020 boundary rule: this
    design runs under both (¶8, Rollout 2).

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | `docs/adrs/` holds 001–016, 020, 021 and `template.md`; 017–019 have no file | verified | directory listing via glob `docs/**/*` |
| A2 | 021 declares ten units (C1–C3, L1–L5, R1–R2) and no unit carries a pointer — `021-C1` only quotes the `→` glyph in a code span; `021-R1`'s owner is backticked `unassigned` | verified | docs/adrs/021-decision-unit-ledger.md:49–112 |
| A3 | 013's ledger carries 24 units in four kind sections; `013-B1`–`B4` inherit the section preamble's `→`, and `013-R4` and `013-R5` carry no `→` | verified | docs/adrs/013-game-manifests.md:50–158 |
| A4 | Every 013 unit except `013-R5` appears as a handle in the manifests spec | verified | grep `013-[CLBRS][0-9]` over schemas/manifests/v1/spec.md |
| A5 | Unit-id text in the corpus lives in four kinds of place: a trailing code span at end of line, a trailing code span before a cell's ` \|`, a handle span mid-paragraph in a wrapped line (`spec.md:27,168,183`, A6), and two mid-sentence examples (`spec.md:17`, `021-…:59`) | verified | grep `` `[0-9]{3}-[CLBRS][0-9]{1,2}` `` and `013-[CLBRS][0-9]` over `docs/adrs/` + `schemas/` |
| A6 | Three spec lines carry handles mid-paragraph (`spec.md:27,168,183`); of them only `013-C2` has no other handle, so a strict end-of-line rule makes exactly one shipped unit unhandled | verified | schemas/manifests/v1/spec.md:24–30, 164–171, 181–185 + A4's inventory |
| A7 | The manifests spec states the convention in a `**Unit handles.**` note that itself quotes an example id mid-sentence | verified | schemas/manifests/v1/spec.md:16–20 |
| A8 | No file under `schemas/` other than `manifests/v1/spec.md` contains a unit-id token, and no corpus file carries an id inside an indented code block | verified | grep `[0-9]{3}-[CLBRS][0-9]{1,2}` over `schemas/`; grep for a 4-space-indented line carrying a unit-id span over `docs/adrs/` and `schemas/` (no match) |
| A9 | `sourceAdmissible` has exactly one call site — `runPipeline`, after the entry checkpoint, before `drive()` — so an added sibling gate inherits every entry path | verified | gates.ts:93–124; runner.ts:2438–2443 |
| A10 | `finalizeDone` reads the source ADR and flips its status before the completion commit; it already has an escalation path | verified | runner.ts:3570–3603 |
| A11 | Document stages may write only `docs/**`; `implement`/`fix` own everything else, so spec-side handle edits are coder-stage work | verified | gates.ts:594–625 |
| A12 | `tools/orchestration/src/` is executable surface: a stage commit there opens an ADR-020 restart boundary | verified | gates.ts:551–570; runner.ts:1861–1867; docs/adrs/020-restart-boundary-for-harness-self-changes.md |
| A13 | `parseSource` matches frontmatter and section bodies with `\r?\n`, so CRLF tolerance is the established posture to inherit | verified | gates.ts:54–75 |
| A14 | `tools/orchestration` depends on no markdown parser (`@opencode/client`, `yargs`, `zod` only) | verified | tools/orchestration/package.json:45–49 |
| A15 | Regex look-behind and `\b` are available at the package's Node floor (`>=22.23.5`) | verified | tools/orchestration/package.json:30–31; https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions |
| A16 | `ledger:` is not a frontmatter key in use; `depends-on` appears only on 011–015 (`[]`, `[011]`, `[011]`, `[011,013]`, `[011,012,014]`) — every entry smaller than its own number, so 021-L3 is green today | verified | grep `^(series\|depends-on\|status\|flow\|ledger):` over `docs/adrs/` |
| A17 | 011, 012, 014, 015 contain no unit-shaped text today, so the frontmatter key is the only possible trigger for checking them | verified | grep `[0-9]{3}-[CLBRS][0-9]` over `docs/adrs/` (matches only 013 and 021) |
| A18 | `nextNnn` hands out `max(existing)+1` = `022`; a run whose source is an ADR file takes its number from the filename, so 017–019 stay reservable | verified | gates.ts:528–535; runner.ts:2385 with identity.ts:73 |
| A19 | `schemas/manifests/v1/spec.md:14` attributes preset contents to "ADR-016, ADR-017" while `013-L2` and `013-R4` assign ADR-019 and ADR-017 — a live contradiction in the shipped pair | verified | spec.md:14 vs docs/adrs/013-game-manifests.md:93–98, 154–155 |
| A20 | The full flow's document cap is 900 lines, which this document must stay under | verified | flows.ts:77; gates.ts:157–166 |
| A21 | Test layout: a directory under `tests/` mirrors `src/`, a module-named file is that module's unit test, cross-module suites keep descriptive names | verified | .opencode/skills/repo-practices/SKILL.md:32–41 |
| A22 | "The merged tree" in 021-R2 means the run's worktree at finalize: a run never merges, and the branch is the prospective merge | unverified | docs/adrs/021-decision-unit-ledger.md:105–107; 01-research.md Open questions 10 — read as author intent, stated in ¶7 |
| A23 | The corpus scan is cheap: the two handle-corpus prefixes hold 28 markdown files (19 in `docs/adrs/`, 9 under `schemas/`), so entry and finalize each read a bounded, repo-local set | verified | counts from glob `docs/adrs/*.md` and `schemas/**/*.md` |
| A24 | Once Test strategy #10 is a `npm run check` leg, no run starts against a red tree — the standing CI leg is what keeps the entry hook from blocking unrelated work | verified | package.json:23 chains lint, format:check and test; .github/workflows/check.yml runs `npm run check` in `tools/orchestration` `on: pull_request` on `ubuntu-latest` with node `22.23.5` |

## Options considered

**A YAML sidecar per ADR, Kubernetes-KEP style.** One structured file per ADR
that the gate reads. Rejected: it is a second copy of every unit, so the drift
the ADR exists to close (013's seven units made in the run and never written
back) becomes a drift between the ledger and its sidecar. Research, Prior art,
names KEP's `kep.yaml` as automation-facing, not as a duplicate of the body.

**A derived index generated from the ADRs.** Rejected: it adds an artifact that
must be regenerated, and the freshness question ("is the index current?")
needs its own gate. Parsing the ADR is the whole cost of the check.

**A line scan with no fence awareness.** Rejected: a fenced example that quotes
a handled line yields a handle the join must then resolve (research, Platform
notes; https://spec.commonmark.org/0.31/), so a doc example could hold the
gate red. The middle path — a fence toggle, no parser — is ¶4.

**A CommonMark parse (mdast/markdown-it).** Rejected: one property of one
construct is needed (fenced blocks are literal), the corpus uses no other
(A8), and the package depends on no markdown parser (A14).

**Loosen 021-L2's exception to "every reservation" so 021-R1 and 021-R2 need
no handle.** Rejected: it amends a shipped law to suit a corpus gap, when the
corpus can carry the handle (¶5's scope puts 021's own rollout text in it),
and it would make 013-R1–R4's shipped handles pointless.

**Ledger-bearing-ness by number range, by `series:`, or by the presence of kind
sections.** Rejected: a range hardcodes the rollout into code; `series:` already
means the 011–015 series (A16) and 021 carries none; presence-of-sections makes
a half-written ADR ledger-bearing and blocks its own run before its backfill
lands. A frontmatter key is the repo's existing shape for exactly this
(`series:`, `flow:`) and is what a backfill commit flips (¶5).

**Exempt 021 from the join until the backfills land.** Rejected: the exemption
is the standing hole the research asks about (Open question 1), and it is
unnecessary — 021's targets are this run's own artifacts, so the corpus can be
green at merge (¶8).

## Chosen design

Two pure readers, one gate shape, two hook points, and one
conformance pass over the corpus. Everything below is TypeScript in the
harness (`tools/orchestration/src/pipeline/`), which is where every
deterministic gate lives; `schemas/` stays language-neutral (research,
Platform notes).

1. **Components.** New `tools/orchestration/src/pipeline/ledger.ts` exports
   `parseLedger(content, nnn): LedgerFile | undefined`,
   `extractHandles(content): Handle[]`, `checkLedgers(repoDir): LedgerFinding[]`
   and `ledgerGate(repoDir): GateResult` — the `GateResult` shape every other
   gate returns (`allow`/`deny` with a reason), so the gate log and `pipeline
   status` print it unchanged. From `gates.ts` it imports the frontmatter
   slice: `parseSource`'s `/^---\r?\n([\s\S]*?)\r?\n---/` moves into an exported
   `frontmatterOf(content): { front: string; body: string } | undefined` that
   `parseSource` calls, so both readers share one regex and one CRLF posture,
   with a leading `\uFEFF` dropped (A13). `ledger.ts` reads with `node:fs`,
   builds every path with `node:path` (`join` for the roots, `basename` for
   `NNN` and `template.md`), never writes, and parses no path it does not list.
   Flow: `docs/adrs/*.md` + `schemas/**/*.md` → `parseLedger`/`extractHandles`
   → unit registry + handle set → `checkLedgers` → findings → `ledgerGate`.

2. **Unit grammar (021-C1).** A unit is a `- ` bullet in a kind section of
   `## Decision`, opening with a bold unit id: `- **NNN-Kn** text … → target`.
   The id is `(?<![\dA-Za-z-])\d{3}-[CLBRS]\d{1,2}(?![\dA-Za-z])` — the ADR's
   grep form, anchored by look-behind and trailing guard so it cannot match
   inside a longer token (A15). Kind letters map to section headings:
   `C`→`### Contracts`, `L`→`### Laws`, `B`→`### Budgets`,
   `R`→`### Reservations`, `S`→`### Seeds`; a ledger's `Decision` must carry at
   least one of them, and every `- ` bullet under one is a unit — an ordered
   item or a paragraph there, such as the shipped `Rollout:` text under
   `### Reservations`, is prose. Per file: the `NNN` equals the file's number,
   ids are unique, and per-kind numbers strictly increase. The pointer is the
   last `→` outside a code span and must be followed by non-space text;
   accepted targets are `spec: <name>` (or another prose target), `ADR-NNN`,
   and `unassigned`. A reservation's owner is the form immediately after
   `Owner:`, backticks stripped — `ADR-\d{3}`, `unassigned` (bare or
   backticked, as 021-R1 ships) or `this ADR's run` — the rest of the line is
   prose. A bullet with no `Owner:` is `owner-missing` (013-R5's shape); text
   after `Owner:` beginning with none of the three is `owner-unknown`. A named
   owner is *existing* when `docs/adrs/NNN-*.md` is present and *reserved* when
   it is not: 021-L5's rule with no registry file to keep in step, because the
   reservation lives in the ledger text the gate already reads — the way
   013-R1–R4 already reserve 017 and 019 (A1, A18). The residue is a typo
   naming a number nobody fills, which the gate cannot tell from a real
   reservation: the ledger line is the only registry, one line above the
   reader who would notice.

3. **Amendments (021-L4) — the notation, no check.** A unit may carry
   `amends NNN-Kn` and its target a matching `amended-by`; 021-L4 fixes that
   notation and the template states it (¶8). No shipped unit uses it, so
   nothing here parses it: the symmetry checks ride the first amendment that
   carries it (Deferred).

4. **Handle grammar (021-C2) — the canonical statement.** A handle is one
   trailing code span on a source line whose content is unit ids separated by
   single spaces, followed only by spaces and at most one table-cell `|` before
   the line terminator. That covers both shipped spellings — the multi-id span
   (`` `013-B1 013-B2 013-B3 013-B4` ``, spec:111) and the end-of-cell form (a
   table cell that closes just after its span, spec:115–118) — and excludes the
   mid-sentence and mid-paragraph spans (A5, A6) without a special case. Handles
   in a fenced block (lines whose trimmed start is ``` ``` or `~~~`, toggled)
   and in a document's
   frontmatter block are not handles: fenced content is literal to a markdown
   parser (https://spec.commonmark.org/0.31/), and frontmatter is not prose.
   Lines are split on `\r?\n` (A13). Indented code blocks are out of the rule
   and out of the corpus (A8); Deferred.

5. **Handle corpus and the declaration note — the canonical scope.** The corpus
   is every markdown file under `docs/adrs/` and under `schemas/`: the two
   trees where normative text lives. `docs/designs/**` is outside it — a design
   document is a record, not a contract, and quotes ids freely. A document
   that carries a handle must state the convention in a `**Unit handles.**`
   note (the RFC 8174 posture: normative by syntax plus a declaration line;
   research, Prior art); the shipped note (A7)
   already satisfies it, and this run adds the other two — the template's and
   021's own (¶8) — so the rule is three note lines in the whole corpus.

6. **Ledger-bearing ADRs.** A ledger is `docs/adrs/NNN-*.md` whose frontmatter
   carries `ledger: yes`. The key is the only new key this design adds: absent
   means no unit, pointer, owner or kind check (A16, A17) — while ¶4–¶5's
   handle rule binds every corpus file and ¶7's `depends-order` every ADR that
   carries `depends-on` — and it is 021-R1's split stated mechanically: 013 and
   021 gain the key in this run, 011/012/014/015 gain it with their backfills,
   harness ADRs at their next amendment, and 017–019 are born with it because
   the template's frontmatter carries it (rollout 4 needs no code).
   `template.md` is corpus but never a ledger: it matches no `NNN-`. Handles
   resolve against the registry of **all** ledger-bearing ADRs, not only the
   governing one, which is what 013 Backfill findings 5's cross-attributions
   need before 011/012/014/015 are backfilled.

7. **The checks, the taxonomy, and where they run.** `checkLedgers` walks every
   ledger-bearing ADR and every corpus file and returns findings in twelve
   codes: `ledger-empty`, `unit-malformed`, `unit-duplicate`,
   `unit-kind-mismatch`, `unit-misplaced`, `pointer-missing`, `owner-missing`,
   `owner-unknown`, `handle-dangling`, `unit-unhandled`, `note-missing` and
   `depends-order`. The four classes 021-R2 names map onto `handle-dangling`
   (dangling handle), `unit-unhandled` + `unit-misplaced` (unregistered unit —
   an id used, or defined outside a kind section), `depends-order` (a
   `depends-on` violating 021-L3, checked on every ADR that carries that key,
   ledger-bearing or not) and `owner-missing`/`owner-unknown` (021-L5).
   `ledgerGate` denies with at most `MAX_LEDGER_FINDINGS` findings — an exported
   constant, the way `MAX_GATE_LEGS` bounds a gate record — each as a
   `file:line code detail` line plus `+N more`, and both hooks gate-log it
   under `ledger-join`, the name `pipeline status` prints, so a stuck run
   explains itself from the gate log. Two hook points, one function:
   `runPipeline` calls it immediately after `sourceAdmissible` at its single
   call site (A9), inheriting every entry path and every source kind — a brief
   is checked against the same tree — and `finalizeDone` calls it before
   `flipAdrStatus` (A10), denying into `finalizeEscalate`. Finalize is what
   makes the ADR a work order: a run cannot complete while one of its own units
   is unhandled, which is the seven-units failure mode closed. Per A22 the
   check runs against the run's worktree, which at finalize is the branch the
   merge would take.

8. **Corpus conformance landed by this run.** Document stages own `docs/**`;
   the spec half is coder-stage work (A11), so both halves land in `implement`
   or `fix`:
   - `docs/adrs/template.md`: frontmatter gains `ledger: yes` and nothing else
     — `series:` names the closed 011–015 series (A16) and 021-L3 makes
     `depends-on` optional, so neither belongs in a template; `## Decision`
     gains the five kind sections with one
     commented-out unit skeleton each, and a `**Unit handles.**` note block
     stating, one claim per line and each line ending with its handle, the
     ledger preamble (021-C1), the handle rule (021-C2), the harness-ADR
     adoption rule (021-C3), the "never re-specifies" test (021-L1), the total join (L2),
     `depends-on ⊆ smaller numbers` (L3), `amends`/`amended-by` (L4) and the
     owner rule (L5). The template is the convention's single normative
     statement, so these lines are where 021's C/L units get their handles.
   - `docs/adrs/021-decision-unit-ledger.md`: `ledger: yes` in frontmatter; a
     `→` on each of its ten units, pointing at the template section or the
     rollout artifact; a `**Unit handles.**` note (¶5); the `Rollout:` paragraph
     re-set as a numbered list whose items 1–3 end with `021-C3`, `021-R1`,
     `021-R2`; `021-R1`'s owner stays its shipped `unassigned`.
   - `docs/adrs/013-game-manifests.md`: `ledger: yes`; `→ spec: Budgets.` added
     to `013-B1`–`B4` so every bullet carries its own pointer; `013-R4` gains
     `→ ADR-019`; `013-R5` gains `Owner: unassigned` and `→ unassigned`, the
     L2 exception it already asked for.
   - `schemas/manifests/v1/spec.md`: the three mid-paragraph handles
     (`:27`, `:168`, `:183`) move to the end of the last source line of the
     claim they govern (A6 — this is what makes `013-C2` handled), the
     `**Unit handles.**` note takes the template's canonical wording, and
     line 14's "ADR-016, ADR-017" is corrected to the pairing that `013-L2` and `013-R4` state
     — the seed set's `(ADR-019)`, the codec's `(ADR-017)` (A19) — and
     the correction is recorded as a sixth `Backfill findings` entry in 013, per the
     "recorded, not silently absorbed" consequence.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A red corpus blocks every new run at entry, including runs unrelated to the ledger | Medium | High | Test strategy #10 makes the join an `npm run check` leg, so the tree is green before a run starts (A24); the denial reason names `file:line` and code |
| Silent under-coverage: a handle placed where the rule cannot see it just stops existing (research, Risks) | Medium | Medium | The rule is the same for humans and machine (¶4), the corpus covers both normative trees (¶5), `unit-unhandled` turns an invisible omission into a named finding, and #2 plus #10 pin the placements the corpus already ships |
| Bootstrapping: the corpus is red between the wiring commit and the conformance commit | High inside this run | Low | Both halves land in one branch; the entry hook cannot fire for the already-running process, and finalize escalates if the conformance half is missing |
| 021's `→` targets are names, not references, so a pointer to a section that never exists stays invisible | Medium | Low | Accepted and Deferred: the pointer's mechanical half (presence) is checked now; A19 shows the drift class is prose, which no handle check sees either |
| Landing the gate crosses the ADR-020 boundary and the run pauses mid-flow | High | Low | Expected, handled by the existing mechanism: Rollout 2 names it, and the doc/spec conformance commits are not surface (A12) |
| Re-marking `013-R5` and re-wrapping spec lines touches a shipped, accepted ADR and its contract | Low | Medium | Both are editorial-only (no normative sentence changes), the ADR's consequence "the backfill is a mechanical, reviewable diff" is the precedent, and A19's correction is recorded in 013's `Backfill findings` |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux (dev and CI host) | Yes | The only platform this run executes on; Test strategy #1–#12 all run there, and `.github/workflows/check.yml` runs `npm run check` there (A24) |
| macOS / Windows checkout | Not tested — expectation only | No case-sensitive path or filename assumptions (corpus names are lowercase kebab per repo-practices), separators come from `node:path` (¶1), and CRLF is handled by `\r?\n` splits (A13) and pinned by #3 |
| Node `>=22.23.5`, vitest ^5.0.5, TypeScript ^6.0.3 (`strict`, `noUncheckedIndexedAccess`) | Yes | Look-behind is the only engine feature beyond the current gate regexes (A15); no new dependency (A14) |
| Rust product and `schemas/*/v*/` conformance harness | No — out of scope | The parser is harness-side only; `schemas/` stays language-neutral and the Rust harness keeps walking fixtures (research, Platform notes) |
| Full flow, ADR source | Yes | Entry hook sits beside `sourceAdmissible` (A9); finalize hook sits before the status flip (A10) |
| Light flow, brief source | Yes | The join is a tree invariant, so a brief run is checked against the same corpus (¶7) |
| ADRs without `ledger:` (001–012, 014–016, 020) | Yes — partly unchecked | The absent key exempts the file from the unit, pointer, owner and kind checks (¶6); it does not exempt it from ¶4–¶5's handle and note rule, and `depends-order` runs on every ADR carrying `depends-on` |
| Harness runs already mid-branch at merge | Yes | The entry hook is not re-run on resume; the corpus test and finalize cover them |

## Test strategy

1. Unit: the anchored id grammar accepts `013-C1`, `021-R12`, a multi-id span
   and a cell-final id, and refuses `1234-C1`, `013-C1x`, `013-C123`,
   `013-X1` and `v013-C1` (¶2).
2. Unit: `extractHandles` returns the ids of a line ending in a handle, of a
   multi-id span (spec:111's shape), and of a table cell whose final span
   closes the cell, and returns nothing for a mid-sentence id, a fenced-block
   line, a frontmatter line, or an id with a trailing word after it.
3. Unit: parsing is CRLF-tolerant — the same fixture body rewritten `\r\n` in
   memory yields the identical handle set and findings list (A13).
4. Unit: `parseLedger` accepts the 013 shape and reports `pointer-missing` for
   the pre-run 013 shape (`B1`–`B4` inheriting a preamble pointer,
   `R4`/`R5` with none), `unit-kind-mismatch` for a `C` unit under `### Laws`,
   `unit-misplaced` for a unit bullet outside any kind section, `unit-duplicate`
   for a repeated id, and `ledger-empty` for `ledger: yes` with no kind section.
5. Unit: reservations — `owner-missing` for a bullet with no `Owner:`
   (013-R5's shape), `owner-unknown` for `Owner: ADR-17` and `Owner: someday`,
   and clean for `ADR-019 (reserved)`, `` `unassigned` `` with `021-R1`'s
   shipped trailing prose, bare `unassigned` and `this ADR's run.` (¶2).
6. Unit: the join is total in both directions — a handle with no unit is
   `handle-dangling`, a unit with no handle is `unit-unhandled` except a
   reservation carrying `unassigned`, and a handle resolving to a unit in a
   *different* ledger-bearing ADR is clean (013 Backfill findings 5's shape).
7. Unit: `depends-on: [021]` on a fixture `013-*.md` is `depends-order`, and an
   ADR that carries `depends-on` but no `ledger:` key still gets the check
   (¶6); A16's real values are the clean case.
8. Unit: `note-missing` fires for a handle-bearing corpus file with no
   `**Unit handles.**` note and not for a file with ids only in prose
   positions (A5).
9. Integration: fixture ledgers built in the test module and written into the
   scratch repo of the existing `tests/pipeline/tmp-dir.ts` helper — no
   committed markdown for prettier to reformat out from under a shape pin —
   drive `checkLedgers` end to end, and a `ledgerGate` denial's reason lists at
   most `MAX_LEDGER_FINDINGS` findings and states the elided count.
10. Integration (standing CI cost): `checkLedgers` over **this repository** is
    green — no dangling handle, no unhandled unit, no missing pointer, note or
    owner, `depends-on` in order — which is the ADR's "coverage is a grep"
    made permanent, and it holds 013's 24 units, 021's ten units and the
    template's handle lines to the same rule.
11. Integration: the entry hook escalates a run whose tree is red — a temp repo
    with a `ledger: yes` ADR carrying an unhandled unit refuses before the first
    stage and gate-logs the finding under `ledger-join` — and a green tree
    proceeds; a brief source in the same red tree is refused too (¶7).
12. Integration: `finalizeDone` escalates rather than flipping the ADR status
    when the run's own ledger has an unhandled unit, and flips it when the tree
    is green (A10) — 021-R2's machine-verified acceptance.
13. Platform: Linux executes #1–#12. macOS and Windows rely on #3's CRLF pin
    and the lowercase-kebab corpus names; no maintainer is asked to run the
    suite there for this change.

## Rollout

1. `test-authoring` lands the pins first — `tests/pipeline/ledger.test.ts`
   (#1–#8), `tests/pipeline/ledger-join-gates.test.ts` (#9, #11, #12) and
   `tests/pipeline/ledger-corpus.test.ts` (#10), module-named for the new
   module and descriptive for the cross-module suites (A21). Only `.ts` files
   land here, prettier-formatted, and no markdown enters the format leg's path
   (Test #9). #10 is red from this commit until ¶8's conformance pass lands;
   #1–#9 run on in-module fixtures, so they go green with `ledger.ts`.
2. `implement` lands `ledger.ts`, the `frontmatterOf` extraction in `gates.ts`,
   and the two hook calls in `runner.ts`. That commit touches
   `tools/orchestration/src/`, so it opens one ADR-020 restart boundary (A12):
   the run pauses, the operator relaunches in a fresh process, and the resumed
   run carries the new gates. Commit: `feat: add the decision-unit ledger gate`.
3. `implement` (or a `fix` round) lands ¶8's conformance pass: the template,
   021, 013 and the manifests spec. The docs half could land in a document
   stage, but the spec half is coder-owned (A11), so the whole pass rides the
   coder commit:
   `docs: make the ledger corpus conform to the join`.
4. `npm run check` green is the acceptance evidence: #10 turns the join into a
   standing CI leg, so from this merge any red ledger fails the check gate
   before a run can start. `fix` and `test-update` follow the flow as usual.
5. `doc-sync` adds the ledger gate to the source-gate paragraph and the stage
   diagram in `docs/workflows.md` (lines 25–27 and 65) — one line naming
   `ledger-join` and the `ledger: yes` key — and finalize flips 021 to
   `accepted` once #12's assert passes.
6. After this merge, in order (021-R1, an `unassigned` rollout task): the
   backfills of 011, 012, 014 and 015, each a `Backfill findings`-carrying
   re-slice that adds `ledger: yes` and its handles in the same commit; 017–019
   arrive from the amended template and are ledger-bearing at birth; harness
   ADRs 001–010, 016 and 020 gain a ledger at their next amendment. Their order
   is the backfill run's to choose — 021 defers it, and this design does not
   take it.

## Deferred / out of scope

- Resolving `→` targets to headings or files — 021-L2 makes only the handle
  direction mechanical; target resolution needs a target grammar 021-C1 does
  not define. Owner: ADR-021's next amendment.
- Checking prose `ADR-NNN` cross-references (A19's class) — outside the join by
  construction. No follow-up yet; A19's instance is fixed by ¶8 as editorial.
- 021-L1's "never re-specifies" test — reviewer judgement, not a pattern.
  Owner: the design-review seats (docs/workflows.md's review phase).
- The 011/012/014/015 backfills, their order and their findings — 021-R1's
  `unassigned` rollout task, as 021 words it.
- `amends`/`amended-by` symmetry (¶3) — the notation ships, its checks do not.
  Owner: the first amendment that carries `amends`.
- Adding `depends-on` to the harness ADRs whose prose states an order (005 "run
  after 004", 006 carved from 003) — those ADRs' next amendments; 021-L3 bites
  going forward (research, Findings).
- Ledgers for harness ADRs 001–010, 016, 020 — 021-C3: their next amendment.
- Handles inside indented (4-space) code blocks — the corpus has none (A8);
  adding a rule now would be untested. No follow-up yet.
- A ledger writer/formatter that normalizes handle placement — ¶8 is the last
  hand edit; a writer would let a check be satisfied by reformatting.
  No follow-up yet.
- A per-run cap on restart boundaries — ADR-020's own "Not decided here".

## Revision log

- r1 ¶2 / Options / ¶8: ¶2 owns the grammar — guard widened, `- ` bullets,
  code-span pointer, owner is the form after `Owner:` — no ADR-022 reservation.
- r1 ¶3 / ¶7: 021-L4 keeps its notation; the four symmetry codes, their test
  and the "twelve findings" literal go, deferred to the first `amends`.
- r1 ¶6 / matrix / Test #7: the absent `ledger:` key exempts only the unit,
  pointer, owner and kind checks; the handle rule and `depends-order` do not.
- r1 ¶1 / Test #9 / Rollout 1: paths via `node:path`, `frontmatterOf` drops a
  BOM; fixtures live in `tmp-dir.ts`'s scratch, off prettier's committed path.
- r1 A2 / A3 / A5 / A24 / Test #1 / #5: bases and pins matched to the shipped
  text, the CI leg verified, the macOS/Windows row marked untested.
- r1c1 ¶8: the shipped `Rollout:` prose ends with no handle — its re-set as a
  numbered list whose items 1–3 end with `021-C3`, `021-R1`, `021-R2`.

## As-built (2026-11-15)

Drift from the approved design, recorded per the change set 1bda5c34…HEAD:

- **API surface / shapes** (¶1, ¶2). `parseLedger(content, nnn, file = '')` ships with an optional third `file` argument so findings can be named when `checkLedgers` knows the path; the proposal lists `parseLedger(content, nnn)` only. `Handle` carries `line`, `LedgerUnit` carries `kind`, `line` and `unassigned`. The proposal left these shapes unspecified. Reason: findings need file/line context and the join needs line numbers for the denial reason and tests; no design change, only implementation detail. Affected sections: 1. Components, 2. Unit grammar.

- **Owner existence check omitted** (¶2). The design states a named owner is *existing* when `docs/adrs/NNN-*.md` is present and *reserved* when it is not, with no registry file to keep in step. The implementation accepts `ADR-\d{3}` without checking file existence, so `existing`/`reserved` are not distinguished and no existence check is performed. Reason: the ledger line is the only registry and the gate cannot reliably distinguish a typo from a real reservation without a second read; the proposal itself notes the residue is a typo the gate cannot tell. Affected sections: 2. Unit grammar, 7. The checks.

- **Bullet text aggregation** (¶2). A unit's text is built from its bullet line plus indented continuation lines, so the pointer and owner may sit on any continuation line. The proposal describes a unit as `- **NNN-Kn** text … → target` without stating continuation handling. Reason: shipped ledgers wrap; continuation lines are needed for the pointer/owner to be found on wrapped bullets. No normative change. Affected sections: 2. Unit grammar.

- **Per-kind number monotonicity reporting** (¶2). A per-kind number that does not strictly increase but has no repeated id is reported as `unit-duplicate`. The proposal states per-kind numbers strictly increase and ids are unique, but does not name the finding code for a monotonicity violation. Reason: reuse the existing duplicate code to surface the violation. Affected sections: 2. Unit grammar, 7. The checks.

- **Unit-misplaced registration** (¶2). A unit bullet outside every kind section is reported `unit-misplaced` and still registered, so a handle may resolve to it while placement remains a finding. The proposal requires placement inside a kind section but does not specify whether the unit is registered. Reason: keep the join total while surfacing placement errors. Affected sections: 2. Unit grammar, 7. The checks.

- **Conformance landing split** (¶8, Rollout 3). The design states the whole conformance pass “rides the coder commit” (A11). In this tree the spec half landed in the coder commit and the `docs/adrs/**` half landed in a later document stage (`docs: apply the ledger conformance payload to the live ADRs and template`). Reason: stage permissions deny `docs/adrs/**` to the coder stage; the split preserves the corpus green state without changing design. Affected sections: 8. Corpus conformance, Rollout.

No design decisions were changed; the differences are implementation details and process constraints recorded for traceability.
