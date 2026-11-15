# compatibility review r1 — dissent pass c2

verdict: approve

Scope: `git diff 94fdca92 e66564c0` of `02-proposal.md` (¶1 frontmatter slice +
path building, ¶2 owner rule, ¶8's `docs/adrs/021-decision-unit-ledger.md`
bullet, Test #9, Rollout 1, Revision log), then the full current document.
Inputs: the proposal, `docs/adrs/021-decision-unit-ledger.md`, the Revision log,
my r1 findings. The shell allow in this session covers `git diff*` only, so the
`npm`/`cargo`/`prettier` legs could not be executed; every claim checked below
was checked by reading the named source, config and workflow files.

## My r1 findings — status against the changed text

- **major, CLOSED.** ¶8 now reads "the `Rollout:` paragraph re-set as a numbered
  list whose items 1–3 end with `021-C3`, `021-R1`, `021-R2`; `021-R1`'s owner
  stays its shipped `unassigned`" — the edit my finding asked for, and the
  Revision log records the rejected premise ("the shipped `Rollout:` prose ends
  with no handle"). Consequences re-checked: an ordered item under
  `### Reservations` is prose under ¶2, so no `unit-misplaced`; a line ending in
  one backticked id is a handle under ¶4; `021` gains the `**Unit handles.**`
  note ¶5 demands of a handle-bearing file. Handle coverage for 021's ten units
  is now complete without a grep against `schemas/`: the template note lines carry
  C1–C3 and L1–L5, the Rollout items carry C3, R1 and R2.
- **minor, CLOSED.** ¶2 now states the position — "the form immediately after
  `Owner:`, backticks stripped — `ADR-\d{3}`, `unassigned` (bare or backticked,
  as 021-R1 ships) or `this ADR's run` — the rest of the line is prose", with
  `owner-unknown` for "text after `Owner:` beginning with none of the three".
  Every shipped owner line in the corpus parses under it:
  `docs/adrs/013-game-manifests.md:147,150,153` ("Owner: ADR-017 (reserved)."),
  `:155` ("Owner: ADR-019 (reserved)."),
  `docs/adrs/021-decision-unit-ledger.md:101` ("Owner: `unassigned` — a rollout
  task, not an ADR slot.") and `:108` ("Owner: this ADR's run."). Test #5's pins
  (`Owner: ADR-17` / `Owner: someday` red; the four shipped shapes green) are
  the same reading, so #10 holds nothing red that ¶8 leaves untouched.
- **nit, CLOSED.** ¶1 adds "with a leading `\uFEFF` dropped (A13)". Verified the
  fix lands at the only choke point: `/^---\r?\n([\s\S]*?)\r?\n---/` at
  `tools/orchestration/src/pipeline/gates.ts:55` is the repo's only frontmatter
  regex (grep over `tools/orchestration/src`), and the entry path reaches it
  through `parseSource(readFileSync(sourceAbs, 'utf8'), kind)`
  (`runner.ts:2367`), so the ledger reader and the source reader get the same
  BOM posture. The matrix row's axis list need not change as well.

## Findings

- [nit][new] Rollout 1 — "Only `.ts` files land here, prettier-formatted, and no
  markdown enters the format leg's path (Test #9)." — the last clause states a
  property of the format leg that the tree contradicts:
  `.github/workflows/check.yml:54-56` runs `npm run check` with
  `working-directory: tools/orchestration`, that package's `format:check` is
  `prettier --check .` (`tools/orchestration/package.json:20`), and
  `tools/orchestration/.prettierignore` holds only `dist`, `node_modules`,
  `package-lock.json` — so the 14 markdown files already in that tree
  (`README.md`, `prompts/*.md`) are in the leg's path today. What the sentence
  needs is the narrower claim it is actually arguing: nothing in this commit
  adds markdown there. Fix: replace "and no markdown enters the format leg's
  path (Test #9)" with "and no markdown is added to the format leg's path by
  this commit (Test #9)". The load-bearing half stands and I confirmed it: the
  corpus ¶8 rewrites (`docs/adrs/*.md`, `schemas/manifests/v1/spec.md`) sits
  outside the leg's `cwd`, and `.prettierrc` sets no `proseWrap`, so prettier's
  default `preserve` would not re-wrap a line that must end with a handle span
  even if it were inside it.

## Checklist

1. Matrix rows — every row carries a status; no empty cell. The
   macOS/Windows row's "Not tested — expectation only" is an unverified status,
   but it is untouched by this diff and is already recorded as a decision in the
   Revision log ("the macOS/Windows row marked untested"); not re-opened on a
   dissent scope that does not touch it.
2. OS input APIs — not applicable: the design reads markdown files and holds no
   input surface.
3. Node floor — nothing above `engines.node: >=22.23.5`
   (`tools/orchestration/package.json:31`); the changed ¶1 adds only a leading
   `\uFEFF` strip, and the pinned CI node is `22.23.5`
   (`.github/workflows/check.yml:50`).
4. Native modules — none; no dependency change (A14,
   `tools/orchestration/package.json:45-49`).
5. Keyboard reality — not applicable.
6. File paths — ¶1 builds every path through `node:path` (`join`/`basename`) and
   parses no path it did not list; unchanged in substance by this diff.
7. Platform test per supported platform — Test #13 states Linux executes #1–#12
   and what macOS/Windows rely on; unchanged.

## FYI (no severity)

- ¶1's new BOM clause cites A13, whose stated basis is CRLF only ("matches
  frontmatter and section bodies with `\r?\n`"); no register line carries the
  fact the fix rests on — a `'utf8'` read yields a U+FEFF character and the
  `/^---\r?\n/` anchor is offset-0 (`gates.ts:55`, `runner.ts:2367`). Citation
  and assumption-register seat, not this one.
- Test #9 calls `tests/pipeline/tmp-dir.ts` "the scratch repo" helper; it
  exports `mkTmpDir`/`mkTmpDirAsync`, which hand back a plain auto-cleaned temp
  directory that the importing test populates (as `tests/pipeline/gates.test.ts:77`
  already does). The intended use is right, the noun is loose — test-authoring
  readability, not compatibility.
