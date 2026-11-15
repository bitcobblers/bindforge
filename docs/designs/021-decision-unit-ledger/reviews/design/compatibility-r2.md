# Compatibility review r2

Scope: the editorial diff `reviews/design/editorial-r2.diff` (5 hunks: Goals
bullet, ¶1, ¶8 template bullet, ¶8 spec bullet, Risks row), confirmed against
`02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md` and the Revision log.
This is the editorial confirmation pass; previous compatibility findings: none.

verdict: approve

## Findings

None. No blocker, major, minor or nit.

Each hunk is verb, tense, article, quotation and punctuation only; none changes
a platform, an engine, a dependency, a path, a line-ending posture or a test's
platform assignment:

- Goals — `"The join, checked in both directions"` → `"checked in both
  directions"`: adds a preposition. Same claim 021-R2 makes.
- ¶1 — `"parses no path it did not list"` → `"parses no path it does not
  list"`: tense only. The compatibility-bearing sentence in the same paragraph,
  `"builds every path with `node:path` (`join` for the roots, `basename` for
  `NNN` and `template.md`), never writes"`, is untouched.
- ¶8 template bullet — `"never-re-specifies (021-L1)"` → `"the "never
  re-specifies" test (021-L1)"`: the note block still lists the same eight
  handles (C1–C3, L1–L5), so ¶4's handle shape and ¶5's note rule are
  unaffected; prettier's `proseWrap` default (`preserve`) means the longer line
  cannot make `format:check` rewrap it, and that leg runs with
  `working-directory: tools/orchestration`, so no `docs/**` markdown is on its
  path — consistent with Rollout 1's "no markdown enters the format leg's path".
- ¶8 spec bullet — `"the pairing `013-L2`/`013-R4` state gives"` → `"the
  pairing that `013-L2` and `013-R4` state"` plus `"the correction is recorded
  as a sixth `Backfill findings` entry"`: ungrammatical verb removed, subject
  restored. The technical claim is unchanged and holds: `schemas/manifests/v1/spec.md:14`
  does read `(ADR-016, ADR-017)`; `013-R4` reads `Owner: ADR-019 (reserved)` and
  `013-R1`–`R3` read `Owner: ADR-017 (reserved)`, matching the corrected pairing
  "the seed set's `(ADR-019)`, the codec's `(ADR-017)`"; `013` ships exactly
  five `Backfill findings` entries, so a sixth is right.
- Risks — `"editorial-only — no normative sentence changes —"` →
  `"editorial-only (no normative sentence changes),"`: dashes to parentheses.

## Checklist disposition

1. **Compatibility matrix** — pass. Eight rows, every one carries a status
   (`Yes`, `No — out of scope`, `Not tested — expectation only`,
   `Yes — partly unchecked`); no empty cell, no literal `unknown`. The
   macOS/Windows row's declared non-status is the author's r1 decision
   (Revision log: "the macOS/Windows row marked untested") and the r2 diff does
   not touch it; its stated basis is sound and verified — every file in
   `docs/adrs/` and `schemas/` is lowercase kebab (19 + 9 files, matching A23's
   counts), so no case-sensitivity is assumed.
2. **OS input APIs (X11/Wayland/Windows Raw Input/macOS HID)** — not
   applicable. The design reads markdown with `node:fs`; it touches no input
   subsystem. The abstraction justification the row needs is present: harness-
   side TypeScript over repo files, "`schemas/` stays language-neutral".
3. **Node version floor** — pass. `tools/orchestration/package.json:31` reads
   `"node": ">=22.23.5"`; the only engine feature the design adds is regex
   look-behind (¶2's `(?<![\dA-Za-z-])`), already in use in shipped code
   (`runner.ts:3290` `(?<![A-Za-z])`, with `gates.ts:63`'s `$(?![\r\n])`
   showing the same end-of-line anchor posture), so nothing
   is above the floor. `tsconfig.json` `target`/`lib` `ES2022` with `strict` and
   `noUncheckedIndexedAccess` — as the matrix row states; regex features are not
   downlevel targets, so ES2022 does not gate look-behind.
4. **Native modules** — pass. No new dependency and no native module:
   `dependencies` is `@opencode/client`, `yargs`, `zod` only (A14 verified), and
   the CI job installs nothing beyond `npm ci` (A24 verified:
   `.github/workflows/check.yml` `ubuntu-latest`, node `22.23.5`,
   `npm run check` = `lint && format:check && test` per `package.json:23`).
   Silent native dependency: none.
5. **Keyboard reality (layouts, dead keys, IME, non-ASCII names)** — not
   applicable as behaviour; the design parses and emits no key events. The
   adjacent text-decoding concern is covered: ¶1 drops a leading `\uFEFF`, lines
   split on `\r?\n`, and the id grammar is ASCII-digit/ASCII-letter, so a
   non-ASCII look-alike id surfaces as `unit-malformed` rather than silently
   vanishing.
6. **File paths** — pass. ¶1 names `node:path` `join`/`basename`, "never
   writes", and "parses no path it does not list" — no string concatenation; CRLF
   posture matches `parseSource` (`gates.ts:55` `/^---\r?\n([\s\S]*?)\r?\n---/`,
   A13 verified verbatim). Per-OS config-dir conventions (XDG/AppData): not
   applicable — every path is repo-relative under `docs/adrs/` and `schemas/`.
7. **Platform test per supported platform** — pass. The only platform the matrix
   claims as supported for execution is Linux, and Test strategy #13 names it
   ("Linux executes #1–#12"); macOS/Windows are explicitly not claimed, so #13's
   "no maintainer is asked to run the suite there" is consistent rather than a
   gap. No matrix row claims a platform the test strategy leaves unrun.

## Evidence and limits

- The cargo/rustfmt/npm check legs were not executed: this session's shell
  allow is `git diff*` only, so no leg (read or write) could be launched. The
  compatibility claims were instead verified by reading the cited source:
  `tools/orchestration/package.json` (engines, deps, scripts), `tsconfig.json`,
  `.github/workflows/check.yml`, `gates.ts:54–75`, `runner.ts:3290`,
  `schemas/manifests/v1/spec.md`, `docs/adrs/013-game-manifests.md`,
  `docs/adrs/template.md`, and glob/grep inventories of the two corpus trees.
  No compatibility property was left unestablished, so nothing blocks on
  missing evidence.
- Corpus inventories confirm the assumptions the matrix and Test #10 rest on:
  unit-id tokens inside `schemas/` occur only in `manifests/v1/spec.md` (A8);
  no unit-id line is 4-space-indented in either tree (A8); the mid-paragraph
  handles are at `spec.md:27`, `:168`, `:183` (A6 verified — `013-L5` is handled
  at `:151`, `013-C1` at `:49–50`, `013-C7` at `:148`, leaving `013-C2` the one
  unhandled unit ¶8 fixes); `ledger:` is not a frontmatter key in use and every
  `depends-on` value is smaller than its own number (A16).
- `docs/adrs/021-decision-unit-ledger.md` contradicts nothing the Deferred
  section hands back to it: the deferrals there (target resolution, prose
  `ADR-NNN` mentions, L1's judgement test, the 011/012/014/015 backfill order,
  `amends` symmetry, harness-ADR ledgers) restate 021-R1, 021-C3 and 021's
  "Not decided here" clause.

## FYI (no severity)

- Revision log / ¶8 — `013-L2` states the pairing as "A preset's *contents* are
  never recorded (the seed set's and the codec's — ADR-017, ADR-019)", which
  reads positionally as seed→017 / codec→019, the inverse of `013-R1`–`R4` and
  of 013's own Consequences line ("which preset sets ship as seeds (ADR-019)").
  ¶8's corrected pairing follows the R-units and is the sound reading, but the
  new wording "the pairing that `013-L2` and `013-R4` state" leans on L2, whose
  ordering is ambiguous — a citation-accuracy point for the correctness seat,
  not a compatibility one.
- A15 cites an external MDN URL as its basis while the repo already ships
  look-behind in `runner.ts:3290`; a same-repo citation would
  age better. Research/citation seat's call.
