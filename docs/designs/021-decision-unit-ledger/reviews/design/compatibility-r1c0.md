# compatibility review r1

scope: whole document (full panel, pass c0)
inputs used: `docs/designs/021-decision-unit-ledger/02-proposal.md`,
`docs/adrs/021-decision-unit-ledger.md`, the proposal's Revision log; plus the
named repo sources the design cites for its environment claims
(`tools/orchestration/package.json`, `tsconfig.json`, `.prettierrc`,
`.prettierignore`, `vitest.config.ts`, `src/pipeline/gates.ts`,
`src/pipeline/runner.ts`, `src/pipeline/states.ts`,
`.github/workflows/check.yml`, `docs/adrs/*`, `schemas/**`).
Revision log read first: it holds one r0 entry ("no `reviews/design/` findings
yet"), so nothing below was rejected by the author. All findings are `new`.

verdict: request-changes

## Findings

- [major][new] Compatibility matrix, row "ADRs without `ledger:`" —
  "Yes — unchecked | Absent key means no ledger checks (¶6); their prose and
  handles behave as today" — this is the design's backward-compatibility
  statement for the corpus it does not convert, and ¶5 and ¶7 contradict both
  halves. ¶5 binds every corpus file, key or no key: "A document that carries
  a handle must state the convention in a `**Unit handles.**` note", and ¶4
  makes any trailing id span on any line of any `docs/adrs/` or `schemas/` file
  a handle — so a unit-shaped trailing span added to, say, ADR-005 or
  `schemas/profile/v2/spec.md` becomes a `handle-dangling` / `note-missing`
  finding even though that file will never carry `ledger:`. ¶7 adds a code that
  does not require the key at all: "`depends-order` (a `depends-on` violating
  021-L3, checked on every ADR that carries the key, not only ledger-bearing
  ones)", which Test #7 restates as "a ledger-bearing ADR with no `ledger:` key
  of its own still gets the check" — a phrase that contradicts itself (a
  ledger-bearing ADR carries `ledger: yes` by ¶6), so the one test that pins
  the row's exception cannot be written as stated. The Risks table already
  concedes the wider reach ("A red corpus blocks every new run at entry,
  including runs unrelated to the ledger"). Nothing is red today — A17 and A8
  check out (grep `[0-9]{3}-[CLBRS][0-9]{1,2}` over `docs/adrs/` matches only
  013 and 021, over `schemas/` only `manifests/v1/spec.md`) — so the failure is
  a wrong compatibility contract, not a broken tree: a maintainer reading this
  row will believe a 011/012/014/015 backfill run (Rollout 6) enters a tree
  that does not judge that file at all. Fix: split the row's Notes into what
  the absent key exempts (unit, pointer, owner and amendment checks) and what
  it does not (the ¶4–¶5 handle and note rule on every corpus file;
  `depends-order` on every ADR carrying `depends-on`), and re-word Test #7 to
  "an ADR that carries `depends-on` but no `ledger:` key still gets the check",
  replacing the self-contradictory phrase.

- [major][new] Test strategy #10 + Rollout 1, ledger fixtures — "the corpus
  fixture set (temp dirs under `tests/pipeline/fixtures/ledgers/`)" /
  "`tests/pipeline/ledger.test.ts` (#1–#9, fixtures under
  `tests/pipeline/fixtures/ledgers/`)" — committed markdown under
  `tools/orchestration/tests/` lands inside the standing format leg:
  `.prettierignore` holds only `dist`, `node_modules`,
  `package-lock.json`, `format:check` is `prettier --check .`, and
  `npm run check` chains lint, format:check and test
  (`tools/orchestration/package.json:23`) — the same command the harness runs
  as `checkGate` after every `implement`, `fix` and `test-update` stage
  (runner.ts:3065–3088, `states.ts:11` "executing `npm run check` after each
  mutating stage") and CI runs on every PR
  (`.github/workflows/check.yml:54–56`). Two collisions follow. (a) The
  fixtures must hold markdown that is malformed *by design* — a unit bullet
  outside a kind section, a `C` unit under `### Laws`, an id with a trailing
  word, a preamble-inheriting pointer — and prettier's markdown formatter
  canonicalizes exactly that class of shape, so a non-canonical fixture makes
  `format:check` red on the Rollout-1 commit, which the design calls its first
  landing commit ("test-authoring lands the pins first"), and `npm run format`
  then rewrites the pinned bytes; ¶9 states the opposite rule for the corpus
  ("nothing rewrites a ledger or a spec, so no check can be satisfied by a
  formatter") and never applies it to the run's own fixtures. (b) Test #3's
  CRLF corpus cannot be a committed file at all: `.prettierrc` sets no
  `endOfLine`, so prettier's `lf` default fails a CRLF markdown file, and the
  two statements also disagree about whether the tree is committed at all (#10
  "temp dirs under", Rollout 1 "fixtures under"). Fix: state the suite's
  existing posture — build fixture bodies in the test module and write them
  into the scratch repo the `vitest.config.ts` header describes ("each file
  builds a scratch repo under /tmp"), generating the `\r\n` variant in memory —
  which replaces the committed `tests/pipeline/fixtures/ledgers/` tree
  (Rollout 1 then lands only `.ts` files, which are already leg-clean); if
  committed `.md` fixtures are kept instead, name the tree in
  `.prettierignore`, which replaces `format:check`'s coverage of it, and say
  there that #3's CRLF case is synthesized, not stored.

- [minor][new] Assumption register A24 — "the standing CI leg is what keeps
  the entry hook from blocking unrelated work | unverified | package.json:23
  chains lint, format:check and test; no CI workflow run was observed" — the
  leg is verifiable in this repository and true: `.github/workflows/check.yml`
  triggers `on: pull_request`, runs `npm run check` with
  `working-directory: tools/orchestration`, on `runs-on: ubuntu-latest`, with
  `node-version: "22.23.5"` pinned to the engines floor. The design leans on
  this leg twice as load-bearing — Risks row 1's mitigation ("Test strategy #11
  makes the join an `npm run check` leg, so the tree is green before a run
  starts (A24)") and Rollout 4's acceptance ("#11 turns the join into a
  standing CI leg") — while its register entry records it as unverified, and
  the matrix's "Linux (dev and CI host)" row states the CI fact with no
  citation. Fix: set A24's status to verified with
  `.github/workflows/check.yml` (`on: pull_request`; `npm run check` in
  `tools/orchestration`; `ubuntu-latest`; node pinned `22.23.5`) as the basis,
  replacing "no CI workflow run was observed", and cite the same file in the
  matrix's Linux row.

- [minor][new] Chosen design ¶1, corpus listing and number derivation —
  "`ledger.ts` reads files with `node:fs`, never writes, and parses no path it
  did not list itself" — no path library is named anywhere in the design, while
  the corpus is written as `/`-separated patterns ("`docs/adrs/*.md` +
  `schemas/**/*.md`"), ¶2 tests an owner by whether "`docs/adrs/NNN-*.md` is
  present", and ¶6 needs "`template.md` is corpus but never a ledger: it
  matches no `NNN-`" — all of which derive a name, and a prefix of a name, from
  a listed path. The repo's "no path parsing" precedent does not cover this:
  `stagePathAllowed`'s comment (gates.ts:589–592) rests on paths arriving "only
  as git plumbing output — repo-root-relative, POSIX separators", and
  `moves.ts` imports `posix` alongside `node:path` for that reason; a gate that
  builds its own paths from `fs` listings gets platform separators instead,
  which is `\` on the Windows checkout the matrix expects. The matrix row
  lists only two portability axes — "No case-sensitive path or filename
  assumptions … ; CRLF handled by `\r?\n` splits" — and separators are not one
  of them. Checklist item 6 is the one this leaves open. Fix: one clause in ¶1
  naming `node:path` for the two corpus roots (`join`) and for the `NNN`
  prefix / `template.md` tests (`basename`), replacing the bare "reads files
  with `node:fs`", and add "separators via `node:path`" to the macOS/Windows
  row's axis list.

- [nit][new] Compatibility matrix, row "macOS / Windows checkout" — "Expected
  — assumption" — the cell is a hedge rather than a status; the matrix reads
  Yes / No / unknown, and this cell's own note ("not executed there") is what
  `unknown` means. I am not scoring it major, because the row states its
  mitigations and Test #14 names the fallback, so nothing is hidden from a
  reader. Fix: one word — "Not tested — expectation only", or `unknown` with #14
  named as the sole evidence.

## Checklist

1. Matrix completeness — pass with the nit above: 8 rows, every status cell
   filled, no empty cell, no literal `unknown`.
2. OS input APIs (X11 / Wayland / Raw Input / HID) — not applicable: the gate
   reads markdown; the matrix keeps the Rust product and the `schemas/*/v*/`
   harness out of scope, and the design adds no input surface.
3. Node floor — pass. `engines.node` is `>=22.23.5`
   (`tools/orchestration/package.json:31`) and CI pins exactly `22.23.5`; the
   only engine feature beyond the current gate regexes is look-behind (ES2018),
   and `tsconfig.json` targets `ES2022` with `lib: ["ES2022"]`, so A15 holds and
   no API above the floor appears in ¶1–¶7. `strict` and
   `noUncheckedIndexedAccess`, vitest `^5.0.5` and TypeScript `^6.0.3` all
   match the matrix row.
4. Native modules — pass. Pure TS, no new dependency: dependencies are
   `@opencode/client`, `yargs`, `zod` (`package.json:45–49`), confirming A14;
   nothing in the design needs a build step or a CI matrix change.
5. Keyboard reality (layouts, dead keys, IME, non-ASCII names) — not
   applicable to input. The design's one non-ASCII normative token is `→`, read
   under the harness's uniform `readFile(…, 'utf8')` posture (gates.ts:104,
   runner.ts:2367, runner.ts:3578); no new encoding path is introduced, so no
   finding.
6. File paths — finding 4 (minor); case is genuinely not load-bearing (every
   file under `docs/adrs/` and `schemas/` is lowercase kebab — verified by
   listing: 19 + 9 = 28 markdown files, matching A23), and there is no per-OS
   config dir in play.
7. Platform test per supported platform — pass. #14 names Linux for #1–#13;
   the non-Linux row is not claimed supported; the "Light flow, brief source"
   row is matched by #12's brief case, the flow rows by #12 and #13, and the
   mid-branch row by the two it cites ("the corpus test and finalize").

Spot-verified as claimed: A1 (19 files, 001–016 + 020 + 021 + `template.md`, no
017–019), A2, A8, A13 (the frontmatter regex in ¶1 is verbatim
`/^---\r?\n([\s\S]*?)\r?\n---/`, gates.ts:55), A14, A17, A23; the entry hook's
single host call site (runner.ts:2438, after `checkpoint`, before `drive`, on
both flows' entry path, and not on `resumePipeline`) — A9's claim; and coder
stages' right to write `docs/**` (gates.ts:610–617), which ¶8 and Rollout 3
depend on.

## Missing evidence

The shell tool is denied in this session (`Permission denied: shell`), so the
read-only cargo / rustfmt / npm legs could not be executed; every environment
claim above was established by reading the named sources instead, and each
finding cites the file that settles it. Nothing in the verdict depends on an
unexecuted command. Two claims I could not test and do not fault the design
for: whether any corpus file carries a UTF-8 BOM (a byte scan needs a shell),
and whether the harness's `→` targets resolve to real sections — the latter is
Deferred by the design and consistent with 021-L2's handle-only mechanical half.

## FYI (no severity)

- ¶2's trailing guard `(?![\d])` still matches `013-C1` inside `013-C1x`, so
  ¶2's "so it cannot match inside a longer token" and Test #1's "refuses …
  `013-C1x`" point opposite ways — correctness seat's, not mine.
- Risks row 3's bootstrapping mitigation names only the entry hook and
  finalize; the leg that actually goes red between Rollout 1 and Rollout 3 is
  `checkGate` after `implement`/`fix`/`test-update` (runner.ts:3065–3070),
  whose red is what summons the `fix` round ¶8 already allows — worth one
  clause, but a complexity/correctness call.
- ¶5 excludes `tools/orchestration/prompts/` and `docs/workflows.md`, neither
  of which ¶1's corpus contains; neither carries unit-shaped text today (grep).
  The dead exclusions hint at a wider corpus than ¶1 defines — precision seat.
- A11's "implement/fix own everything else" is looser than the gate (coder may
  write `docs/**` and may not write tests, gates.ts:610–617); ¶8's conclusion
  survives, so no finding.
