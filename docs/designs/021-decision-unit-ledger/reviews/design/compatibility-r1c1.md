# Compatibility review r1 — dissent pass c1

verdict: request-changes

Scope: `git diff 67e7b57 94fdca9` over `02-proposal.md`, then the full current
document. Inputs: `02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md`,
the Revision log, my r1 findings. Verification of the design's claims about the
tree was done with read-only greps/reads (`.github/workflows/check.yml`,
`tools/orchestration/package.json`, `tools/orchestration/tests/pipeline/*`,
`tools/orchestration/src/pipeline/gates.ts`, and a trailing-handle grep over
`docs/adrs/` + `schemas/`). No other run record was opened.

## Prior findings — all five closed

- **[major] absent-`ledger:`-key reach (matrix row / ¶6 / Test #7)** — closed.
  The row now reads "Yes — partly unchecked" and splits exempt from bound
  checks; ¶6 says "absent means no unit, pointer, owner or kind check … while
  ¶4–¶5's handle rule binds every corpus file and ¶7's `depends-order` every
  ADR that carries `depends-on`"; ¶7's self-contradictory "carries the key,
  not only ledger-bearing ones" is gone ("every ADR that carries that key,
  ledger-bearing or not"); Test #7 is the wording I asked for. Re-checked the
  widened reach against the corpus: a trailing `` `[0-9]{3}-[CLBRS][0-9]{1,2}` ``
  span (¶4's handle shape) occurs in `schemas/manifests/v1/spec.md` only — 28
  lines, and that file carries the `**Unit handles.**` note — and in no
  `docs/adrs/` file, so the rule is red nowhere today and ¶5's "three note
  lines in the whole corpus" holds for today's tree.
- **[major] committed markdown fixtures vs. the standing format leg** — closed.
  Test #9 now builds fixtures in the test module and writes them into the
  scratch dir of `tests/pipeline/tmp-dir.ts` (`mkTmpDir`/`mkTmpDirAsync`,
  `tests/pipeline/tmp-dir.ts:20,27` — the reference is real), Test #3 rewrites
  the body `\r\n` in memory, and Rollout 1 lands only `.ts`.
- **[minor] A24 unverified** — closed and accurate: `.github/workflows/check.yml`
  is `on: pull_request`, `runs-on: ubuntu-latest`, `npm run check` with
  `working-directory: tools/orchestration`, `node-version: "22.23.5"`
  (check.yml:13–14, 36, 50, 54–56); `package.json:23` chains
  lint → format:check → test. The Linux matrix row cites the same file.
- **[minor] no path library named** — closed: ¶1 names `node:path` with `join`
  for the roots and `basename` for `NNN`/`template.md`, and the macOS/Windows
  row adds "separators come from `node:path` (¶1)".
- **[nit] matrix cell** — closed: "Not tested — expectation only". Per
  checklist item 1 the row still reads as an unknown, and I keep it unscored
  for the r1 reason — it is the wording this seat prescribed, the row names its
  mitigations, and #13 names the fallback. Not re-raised.

## Findings

- [major][new] Chosen design ¶8, `docs/adrs/021-decision-unit-ledger.md` bullet
  — "the shipped `Rollout:` prose lines end with `021-C3`, `021-R1` and
  `021-R2`" — the sentence is false against the file it describes, and it is
  the only mechanism the revision leaves for `021-R2`'s handle. The shipped
  Rollout text is docs/adrs/021-decision-unit-ledger.md:110–112: "Rollout: (1)
  the template adopts the ledger sections; (2) the product / backfills land per
  021-R1; (3) the gate lands per 021-R2; (4) 017–019 / adopt at landing." — the
  two ids sit mid-line and unbackticked, the lines end with "the product",
  "(4) 017–019" and "adopt at landing.", and `021-C3` is not in the Rollout
  paragraph at all (it appears only at :63). Under ¶4 a handle is a trailing
  *backticked* span, so these are the prose mentions ¶7 excludes; the r0
  wording this clause replaced (re-set the paragraph as a numbered list whose
  items end with the ids) is what supplied them. Failure scenario: after
  ¶8's conformance pass, `021-R2` has zero handles — its owner is `this ADR's
  run`, so it is not the ¶6/Test #6 `unassigned` exception, and ¶8's template
  note carries only C1–C3 and L1–L5 ("where 021's C/L units get their
  handles"), and `grep 021-[CLBRS][0-9]` over `schemas/` is empty — so
  `unit-unhandled` fires and Test #10, "checkLedgers over **this repository**
  is green", stays red. That is the requirement Goals states ("The tree green
  at merge: 021's ten units each carry a `→` and a handle") and it breaks
  Rollout 4 ("`npm run check` green is the acceptance evidence") and Rollout 5:
  ¶7 denies at finalize when "one of its own units is unhandled", so #12's
  assert never passes and 021 never flips to `accepted`. Fix: replace the
  clause with the edit it asserts away — "the three `Rollout:` lines are
  re-wrapped so each ends with its backticked handle: `021-C3`, `021-R1`,
  `021-R2`" (r0's numbered-list wording, keeping `021-R1`'s owner at its
  shipped `unassigned`).

- [minor][new] Chosen design ¶2 — "A reservation's owner is the text from
  `Owner:` to the end of the line, backticks stripped" together with "an owner
  matching neither `ADR-\d{3}`, `unassigned` nor `this ADR's run` is
  `owner-unknown`" — read literally, the owner string is the whole rest of the
  line, and then every shipped reservation is `owner-unknown`: 021-R1 ships
  "Owner: `unassigned` — a rollout task, not an ADR slot." (:101), 021-R2 ships
  "Owner: this ADR's run." with a trailing period (:108), and the 013 form ¶2
  itself lists is "ADR-017 (reserved)" — none of the three equals any of the
  three alternatives, which would hold Test #10 red on text ¶8 leaves alone.
  Test #5 already pins the intended reading (clean for "`unassigned` with
  `021-R1`'s shipped trailing prose"), so the rule is missing only the match
  position the test assumes. Fix: state it — "the owner text must *begin* with
  `ADR-\d{3}`, `unassigned` or `this ADR's run`; the rest of the line is free
  prose. Anything else is `owner-unknown`", replacing "an owner matching
  neither … is `owner-unknown`".

- [nit][new] Compatibility matrix, "macOS / Windows checkout" — "separators
  come from `node:path` (¶1), and CRLF is handled by `\r?\n` splits (A13)" —
  the row now enumerates the portability axes and still omits the one that
  silently disables the whole check rather than mis-reporting it: ¶1 moves
  `parseSource`'s `/^---\r?\n([\s\S]*?)\r?\n---/` verbatim into `frontmatterOf`,
  and that regex anchors at offset 0 and strips nothing (gates.ts:55), so a
  UTF-8 BOM on a Windows-authored corpus file means no frontmatter block is
  found, `ledger: yes` is invisible, and the file is skipped as non-ledger with
  no finding — the same silent-under-coverage class as Risks row 2, reached by
  encoding instead of placement. Fix: one clause in the row's axis list —
  "no BOM in the corpus (reads are UTF-8; a BOM defeats `frontmatterOf`)" — or
  one clause in ¶1 saying `frontmatterOf` drops a leading `\uFEFF`.

## Checklist

1. Matrix statuses: all eight rows carry one, no empty cell, no literal
   `unknown`. The macOS/Windows row is a disclosed not-tested expectation —
   recorded above, not scored (see Prior findings).
2. OS input APIs — not applicable: the design reads markdown files and holds no
   input surface.
3. Node floor — clean: `node:fs`, `node:path`, regex look-behind and the
   trailing guard are all within `>=22.23.5` (A15, package.json:31); CI pins
   the same floor (check.yml:50).
4. Native modules — none; no new dependency, no markdown parser (A14,
   package.json:45–49).
5. Keyboard reality — not applicable. Non-ASCII reach is the `→` glyph, matched
   as a literal and pinned by #1–#4.
6. File paths — satisfied by ¶1's `node:path` (`join` for the two roots,
   `basename` for `NNN-` and `template.md`); corpus names are lowercase kebab;
   no XDG/AppData surface.
7. Platform test per supported platform — #13 names Linux for #1–#12; macOS and
   Windows are not claimed as supported, and #3 pins the CRLF split they would
   depend on.

## FYI (no severity)

- Rollout 1 — "Only `.ts` files land here, so the format leg is untouched":
  this repo has no `.prettierrc` or `.prettierignore`, so `prettier --check .`
  does cover the new `.ts` files (and auto-loads the sort-imports plugin);
  the claim reads correctly only as "no malformed markdown lands", which is
  what the closure needed. Every stage's check gate already requires
  `npm run check`, and repo-practices:27–29 tells the stage to format the
  files it owns, so nothing bites.
- Test #9 (`checkLedgers` + `ledgerGate` in the scratch repo) is assigned to the
  cross-module `ledger-join-gates.test.ts` while A21 reserves a module-named
  file for a module's own tests — placement is another seat's call.
- The r1 removals renumber cleanly: the twelve-code list matches ¶7, the
  `MAX_LEDGER_FINDINGS`/`MAX_GATE_LEGS` analogy holds (`MAX_GATE_LEGS` is
  exported, gates.ts:310), Risks/matrix/Rollout references all land on the
  renumbered tests, and dropping the four amendment codes contradicts neither
  021-R2's four named classes nor 021-L4 — so the new Deferred entry for
  `amends` symmetry is consistent and is not re-opened.
