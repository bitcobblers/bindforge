# correctness review r1

verdict: request-changes

Scope: whole document (pass c0, round 1). Inputs used: `02-proposal.md`,
`docs/adrs/021-decision-unit-ledger.md`, the Revision log, and read-only checks
against the code and corpus the assumption register cites (`gates.ts`,
`runner.ts`, `identity.ts`, `flows.ts`, `package.json`, `docs/workflows.md`,
`docs/adrs/013`, `docs/adrs/template.md`, `schemas/manifests/v1/spec.md`).
No other run record was opened.

## Checklist coverage

1. **Assumption register.** Re-checked every `verified` row against its cited
   basis. Confirmed true as stated: A1 (19 files in `docs/adrs/`: 001–016, 020,
   021, template; no 017–019), A4 (only `013-R5` lacks a handle in the spec),
   A6 (the three mid-paragraph lines are `:27`, `:168`, `:183`; `013-L5` also
   ends lines 51/151, `013-C1` lines 49/50, `013-C7` line 148 — so exactly
   `013-C2` is orphaned), A7 (spec.md:16 `**Unit handles.**`), A8 (no other
   `schemas/` file carries a unit-id token; no 4-space-indented id span in the
   corpus), A9 (single `sourceAdmissible` call site, runner.ts:2438, after the
   entry checkpoint at 2428 and before `drive()` at 2444), A10
   (`finalizeDone`/`flipAdrStatus`/`finalizeEscalate`, runner.ts:3570–3603),
   A11 (gates.ts:608–624 — document stages own `docs/**` only), A12
   (`SURFACE_SRC_PREFIX`, gates.ts:551; `surfacePathsInWindow`,
   runner.ts:1861–1867), A13, A14, A15, A16 (`ledger:` absent; `depends-on`
   only on 011–015 with exactly the listed values), A17 (only 013 and 021 carry
   unit-shaped text), A18 (`nextNnn` max+1, identity.ts:66–67 filename number),
   A19 (spec.md:14 "ADR-016, ADR-017" vs 013-L2/R4's ADR-017/ADR-019), A20
   (`config: { docMaxLines: 900 }`, flows.ts:77), A21, A23 (19 + 9 = 28
   markdown files). A22 and A24 are marked `unverified` with reasoning that
   holds (`docs/workflows.md`:57–60 "the runner never merges" supports A22).
   Rows whose basis mis-states or under-scopes the text they cite: findings 3,
   4, 7.
2. **Anti-hallucination gate.** The document cites two research URLs. Both
   fetched live: CommonMark (served as version 0.31.2) — "The contents of a
   code block are literal text, and do not get parsed as Markdown", which is
   exactly the property ¶4 and the Options row borrow; MDN's regex guide lists
   `(?<=y)x` / `(?<!y)x`, supporting A15's look-behind claim at the Node floor.
   No dead or contradicting source. Not applicable: no other URL is cited in
   the proposal.
3. **Internal contradictions.** Findings 1, 2, 5, 6, 10.
4. **Missing edge cases.** Findings 2, 3, 6. Fence/frontmatter exclusion,
   CRLF, empty ledger, name collisions (`unit-duplicate`, `identityClash`) and
   concurrent-modification posture (read-only reader, A12 boundary) are covered.
5. **Interfaces.** Findings 1 and 10; `LedgerFinding[]`/`GateResult` naming is
   otherwise consistent across ¶1, ¶7 and Test #10, and the sixteen codes are
   counted correctly.
6. **Does it decide what the source asked?** Yes. 021-R2's four classes map
   onto named codes (¶7), the entry hook sits at the deterministic source step
   before any inference (A9) and finalize before the status flip (A10), and the
   taxonomy ADR-021 defers to "021-R2's run" (lines 153–154) is delivered.
7. **Revision rounds.** Not applicable: the Revision log holds only the r0
   entry ("no `reviews/design/` findings yet") and this is round 1.

Deferred section: consistent with ADR-021 — target resolution, 021-L1's
judgement and the 011/012/014/015 order are all things 021 itself leaves open
("Not decided here", lines 152–154; `unassigned` legality, line 92). No
deferral contradicts the ADR. The ¶8 re-mark of `021-R1` is the run amending
its own ADR, which 021-L5 permits; noted in FYI only.

## Findings

- [major][new] Chosen design ¶2 (Unit grammar) — "anchored with a look-behind
  and a trailing guard so it cannot match inside a longer token" — the guard as
  written, `(?![\d])`, blocks only digits, so the canonical id grammar matches
  *inside* `013-C1x` (`\d{1,2}` takes `1`; the next character `x` is not a
  digit, so the assertion passes and `013-C1` is returned). Test strategy #1
  pins the opposite — "refuses `1234-C1`, `013-C1x`, `013-C123`" — and ¶2's own
  stated purpose. (`013-C123` and `v013-C1` are genuinely refused; `013-C1x`
  is not.) Impact: a typo id like `**013-C1x**` reads as a valid id, so the
  uniqueness and strictly-increasing checks consume a phantom unit instead of
  reporting `unit-malformed`, and the pin in #1 fails as written. Fix: widen
  the guard in ¶2 to `(?![\dA-Za-z])` and keep #1's two refusal cases as the
  pin; nothing in the corpus depends on the narrower guard (cell-final ids are
  followed by a backtick, multi-id spans by a space, and `013-B1–B4` by an en
  dash).

- [major][new] Test strategy #5 (reservations) — "`owner-missing` for
  `021-R1`'s pre-run `unassigned` text with no marker" — the shipped text *is*
  an owner marker: docs/adrs/021-decision-unit-ledger.md:101 reads "Owner:
  `unassigned` — a rollout task, not an ADR slot.", which A2 itself records
  ("`021-R1`'s owner is `unassigned`"). Under ¶2's own taxonomy `owner-missing`
  is the no-owner-at-all code (013-R5's shape) and an unparseable owner string
  is `owner-unknown`, so the pin is wrong whichever way the parser resolves the
  form. The deeper hole is that ¶2 never says what may follow the owner marker:
  the shipped ledgers use a backticked value plus trailing prose
  ("`unassigned` — a rollout task…"), a bare period ("Owner: this ADR's run.")
  and a `(reserved)` suffix, and #5 pins none of them. Fix: change #5 to pin
  `owner-unknown` for the shipped 021-R1 form and reserve `owner-missing` for a
  reservation bullet with no `Owner:` (013-R5), and state in ¶2 the one rule
  that decides it — the owner value runs to the end of the line, backticks
  stripped, so `ADR-017 (reserved)`, `` `unassigned` `` and `this ADR's run.`
  all parse.

- [minor][new] Assumption register A2 and ¶2 — "021 declares ten units (C1–C3,
  L1–L5, R1–R2) and none carries a `→` pointer" — the cited basis contains a
  `→`: 021-C1's bullet, line 56, "plus a `→` pointer to its target". ¶2's rule
  — "The pointer is the last `→` in the bullet and must be followed by
  non-space text" — counts that code-span arrow (it is followed by a backtick),
  and "or another prose target" accepts the residue, so the gate sees 021-C1 as
  pointer-bearing today and `pointer-missing` can never fire for a unit that
  merely quotes the arrow convention. The run's conformance pass survives only
  by accident, because ¶8's appended `→` happens to be the last one. Fix: in
  ¶2, "the last `→` outside a code span", and in A2 "no unit carries a pointer;
  `021-C1` quotes the `→` glyph in a code span".

- [minor][new] Assumption register A5 vs A6 — "Unit-id text in the corpus lives
  in exactly three kinds of place: a trailing code span at end of line, a
  trailing code span before a cell's ` |`, and two mid-sentence examples
  (`spec.md:17`, `021-…:59`)" — A5's third kind is two lines, but A6 itself
  records three more handle lines that are neither line-final nor cell-final
  ("Three spec lines carry handles mid-paragraph (`spec.md:27,168,183`)";
  verified, e.g. spec.md:27 "and a geometry. `013-C2 013-L5` The id grammar is
  lowercase-only"). So the register states an exhaustive taxonomy that omits
  three shipped lines, and ¶4's claim that the rule "excludes the mid-sentence
  examples (A5) without a special case" rests on that count. Fix: make A5 four
  kinds — add "a handle span in the middle of a wrapped paragraph line
  (`spec.md:27,168,183`)" — and cite A6 from it.

- [minor][new] Test strategy #7 vs ¶6 — "a ledger-bearing ADR with no
  `ledger:` key of its own still gets the check" — ¶6 defines the opposite: "A
  ledger is `docs/adrs/NNN-*.md` whose frontmatter carries `ledger: yes`", so
  "a ledger-bearing ADR with no `ledger:` key" is an empty class and the pin
  cannot be built as written. ¶7's intent is clear ("checked on every ADR that
  carries the key, not only ledger-bearing ones") but the fixture is not. Fix:
  "a non-ledger ADR that carries `depends-on` (013's shape without the key)
  still gets `depends-order`".

- [minor][new] ¶2 vs ¶8 (kind-section bullets) — ¶2: "a ledger's `Decision`
  must carry at least one of them, and every top-level bullet under one must be
  a unit"; ¶8: "the Rollout paragraph re-set as a numbered list whose items 1–3
  end with `021-C3`, `021-R1`, `021-R2`". In the shipped file the Rollout
  paragraph sits *inside* `### Reservations` (lines 110–112, before `##
  Alternatives considered`), so an implementation that reads "top-level bullet"
  as "top-level list item" reports `unit-malformed` on this run's own
  conformance text and Test #11 cannot go green. The numbered list is most
  likely deliberate, but the document does not say so. Fix: one clause in ¶2 —
  "only `- ` bullets are units; ordered items and paragraphs under a kind
  section are prose" — and ¶8 is safe.

- [nit][new] Assumption register A3 — basis "docs/adrs/013-game-manifests.md:128–158" —
  that range is Budgets and Reservations only; it cannot support the row's
  first claim, "013's ledger carries 24 units in four kind sections" (7 C + 8 L
  + 4 B + 5 R, Contracts at line 50 onward). The claim checks out, the basis
  does not reach it. Fix: cite `:50–158`.

- [nit][new] ¶7 (the checks) — "denies with the first twelve findings as
  `file:line code detail` lines plus `+N more`" — next to the sixteen-code list
  immediately above, "the first twelve" reads as twelve of the sixteen *codes*,
  which would drop every `amends-*` finding from the reason. Test #10 ("lists
  at most twelve findings") shows the intent is a count cap. Fix: "denies with
  at most twelve findings, `file:line code detail` each, plus `+N more`".

- [nit][new] ¶8 (spec.md conformance) — "line 14's \"ADR-016, ADR-017\" is
  corrected to \"ADR-017, ADR-019\" to match `013-L2`/`013-R4`" — it does match
  L2, but both texts then read "the seed set's and the codec's (ADR-017,
  ADR-019)", whose positional pairing is the reverse of R1/R4 and spec.md:125
  (codec → ADR-017, seed set → ADR-019). Fix: write "the seed set's (ADR-019)
  and the codec's (ADR-017)" in the same edit, so the shipped pair is right in
  both files rather than equally ambiguous in both.

- [nit][new] ¶1 (Components) — "`parseSource`'s `/^---\r?\n([\s\S]*?)\r?\n---/`
  is extracted verbatim into an exported `frontmatterOf(content)` that
  `parseSource` itself now calls" — `parseSource` uses the whole match, not
  just the capture: `body = content.slice(front[0].length)` (gates.ts:55–57). A
  `frontmatterOf` that returns the frontmatter slice cannot serve that line, so
  "extracted verbatim" and "parseSource itself now calls" cannot both hold with
  the shape implied. Fix: say `frontmatterOf(content): { front: string; body:
  string } | undefined` (or return the match), so the shared-regex claim is
  implementable as stated.

## FYI (no severity)

- ¶8 gives `docs/adrs/template.md` a `series:` key, while Options rejects
  number-range/`series:` detection because "`series:` already means the 011–015
  series (A16)". A template-borne key that names one closed series is a
  semantics question for the compatibility/complexity seat.
- ¶8 also gives the template a `depends-on:` key without stating its placeholder
  value; ¶7 runs `depends-order` "on every ADR that carries the key" and ¶6
  makes template.md corpus. Whether a template placeholder is checked is
  unspecified; a compatibility seat may want it pinned.
