# security review r1c3 (ratification pass)

verdict: approve

Scope as given: `git diff 67e7b57 e66564c` over `02-proposal.md`, then the full
current document, plus `docs/adrs/021-decision-unit-ledger.md`, the Revision log,
and my r1 findings. Checks run against the code (read-only): `gates.ts`
`parseSource` / `sourceAdmissible` (54–124) / `MAX_GATE_LEGS` `MAX_GATE_TAIL_CHARS`
(300–318), `runner.ts` entry hook (2427–2445) and `finalizeDone` (3570–3603),
`boundedFindings`/`boundedField` (3206–3232), `tests/pipeline/tmp-dir.ts`,
`tools/orchestration/package.json` (45–49), `.github/workflows/check.yml`,
`.github/workflows/advisories.yml`.

Nothing in the diff broke what I approved in r1: ¶1's cited property survives and
is strengthened (paths via `node:path`, `basename` for `NNN`/`template.md`, still
read-only, still "parses no path it did not list"); ¶4's fence/frontmatter text is
unchanged; ¶7's deny → escalate posture at both hooks is unchanged and still
matches the code (`runner.ts:2438–2443` escalates with a gate log on a deny,
`3590–3603` is the "escalation with a record, not a crash" posture); the twelve
codes still cover all four classes 021-R2 names (ADR-021:102–107); dropping the
four `amends` codes opens no hole, because ¶2's "`NNN` equals the file's number"
plus `unit-duplicate` makes an id impossible to reuse across files, and 021-R2
never named amendment symmetry; `021-R1` keeping `unassigned` matches 021-L5's
explicit-`unassigned` branch (ADR-021:90–92) and 021-L2's handle exemption
(ADR-021:77–79), so neither the Deferred entry nor ¶8 contradicts the ADR;
Test/Rollout renumbering is internally consistent (#10/#12/#2/#3 references all
resolve); A24's new `verified` basis checks out (`check.yml` runs `npm run check`
in the orchestration job, `on: pull_request`, `ubuntu-latest`, node `22.23.5`).

## Findings

- [minor][re-raised] Chosen design ¶1/¶7 — "`ledger.ts` reads with `node:fs`,
  builds every path with `node:path` …, never writes, and parses no path it did
  not list" — the revision rewrote this sentence and ¶7 but still specifies a
  fail path only for files that parse *wrong*: a corpus file that cannot be read
  or decoded at all (broken symlink, permission error, non-UTF-8 bytes in a
  `.md`) has no handling, so the escaping exception reaches `runPipeline` beside
  `sourceAdmissible` with no `ledger-join` gate log and no `finalizeEscalate`,
  and crashes the completion path inside `finalizeDone` — the class the harness
  legislated for itself at `runner.ts:2430–2431` and `3590–3593`, and the one
  `sourceAdmissible` already converts into a clean deny
  (`gates.ts:105–107`: `deny(\`source document not readable: …\`)`. The revision
  also opened a second silent route through the same class: ¶1's
  `frontmatterOf(content): { front; body } | undefined` returns `undefined` for a
  file whose `---` block does not match, and revised ¶6 now states that an absent
  key means "no unit, pointer, owner or kind check" and that `depends-order`
  binds only "every ADR that carries `depends-on`" — so a ledger-bearing ADR
  whose frontmatter block is malformed reads as *not ledger-bearing*, and neither
  its units nor its `depends-on` are checked, with no finding in any code.
  Checklist item 7 ("reject cleanly, never crash or partially apply").
  Fix: one sentence in ¶7, no new taxonomy code — a corpus read or decode
  failure, or a `docs/adrs/NNN-*.md` whose first line is `---` but whose
  frontmatter block `frontmatterOf` returns `undefined` for, is denied as
  `ledger corpus not readable: <file>`, the way `sourceAdmissible` turns its own
  read failure into a deny, so both hooks route through the escalate path a
  finding already takes.

- [minor][re-raised] Chosen design ¶4 — "Handles in a fenced block (lines whose
  trimmed start is ``` ``` or `~~~`, toggled) … are not handles" — the toggle and
  its lack of any end-of-file validation are unchanged by this diff, so my r1
  finding stands; what the diff *did* change is its blast radius. Revised ¶6
  ("absent means no unit, pointer, owner or kind check … while ¶4–¶5's handle
  rule binds every corpus file") and the rewritten compatibility row ("it does
  not exempt it from ¶4–¶5's handle and note rule") now make the handle rule
  explicitly load-bearing on every file in `docs/adrs/` and `schemas/`, including
  the ledger-less ADRs 001–016 and 020 that last round's text left ambiguous. One
  stray fence marker anywhere in such a file still suppresses handle detection to
  its end, and a dangling handle past it joins green: a check that partially
  applies, on a now-wider set of files, which is the silent-under-coverage defect
  the Risks table names.
  Fix: unchanged from r1 — add `fence-unbalanced` to ¶7's code list (a file whose
  fence state is still open at EOF is a finding, which turns the stray marker red
  instead of mute) and pin it with one fixture in Test strategy #2.

- [minor][new] Chosen design ¶2 — "The pointer is the last `→` outside a code
  span and must be followed by non-space text" — this sentence is new in the diff
  and introduces a second scanning state that the design never defines. ¶4
  defines exactly two non-prose regions (fence toggle, frontmatter block);
  inline code-span state is a third, with no stated toggling rule, no per-line
  reset, and no end-of-input or unbalanced case — the same shape as the ¶4
  toggle my open finding names. On malformed input the outcomes split: an
  unmatched backtick before the real pointer makes it invisible and yields
  `pointer-missing` (fail closed, so it is merely a false positive that the
  green-corpus test #10 would surface), while an unmatched backtick *before* a
  code-span arrow — the corpus quotes glyphs in spans, 021-C1 itself quotes `→`
  (A2) — lets an arrow inside a span be read as the pointer. Since targets are
  presence-checked and never resolved (Deferred), that unit is then green with a
  pointer no reader would accept, and nothing reports it. Checklist item 7:
  malformed input silently changes which text the check applies to.
  Fix: one clause in ¶2 — code-span state for the pointer is computed per line
  and resets at the line terminator, and a bullet with an unbalanced number of
  span delimiters is `unit-malformed` — reusing a code ¶7 already has; the same
  clause can say whether a backticked target (`→ \`ADR-019\``) counts.

- [nit][new] Chosen design ¶7 — "`ledgerGate` denies with at most
  `MAX_LEDGER_FINDINGS` findings — an exported constant, the way `MAX_GATE_LEGS`
  bounds a gate record" — the cap bounds the *count*, and the revision is the
  place the design leans on it hardest, but a finding line is
  `file:line code detail`, so per-line size is still whatever the corpus line
  contributes, and Test strategy #9 dropped the old #10's clause "so no reason
  string is unbounded". The harness's own findings cap does both:
  `boundedFindings` slices to `MAX_RECORDED_FINDINGS` *and* runs every field
  through `boundedField`/`printable` (`runner.ts:3206–3232`) — because, as
  `gates.ts:300–307` states, one gate record is cloned into every later
  checkpoint and rendered into every later prompt. Item 4 (what may be logged).
  Fix: one clause — each `file:line code detail` line goes through the existing
  `printable`/`boundedField` posture (which also normalizes the non-UTF-8 case),
  or ¶7 states that `detail` never quotes corpus text, only coordinates and code.

## Checklist

1. Trust boundaries — n/a for new sources: the corpus is repo-local
   `docs/adrs/` + `schemas/`, repo-authored; `docs/designs/**` stays outside it,
   and ¶5's dropped exclusions (`prompts/`, `docs/workflows.md`) were already
   outside the two prefixes, so scope is unchanged in effect.
2. Parsing untrusted data — no new dependency, no code evaluation, no parser
   (A14 verified against `package.json:45–49`); widened trailing id guard
   `(?![\dA-Za-z])` and the existing look-behind only shrink matches, and the
   corpus is bounded at 28 files (A23). Output bound: finding 4.
3. Path handling — improved by the revision: every path through `node:path`,
   `basename` for `NNN`/`template.md`, no path parsed that was not listed, no
   shell interpolation anywhere; owner strings are matched to `ADR-\d{3}` before
   they reach a path, so the existence probe `docs/adrs/NNN-*.md` takes three
   digits and nothing else.
4. Secrets and PII — the gate log carries repo paths, line numbers and ledger
   text only; nothing leaves the machine. Finding 4 is the size half of this item.
5. Dependencies — unchanged: no dependency is added or named newly by this diff;
   the three runtime deps match A14. No unfixed advisory applies to them
   (`@opencode/client` ^2.1.4 is the client library, not the `opencode` server
   package the September 2026 `opencode serve` advisory covers, and that one is
   fixed upstream).
6. Least privilege — read-only `node:fs` in `ledger.ts` stands; the revision's
   fixture change is a strict improvement in this direction: fixtures are written
   into `mkTmpDir`'s scratch dir under `os.tmpdir()` with `afterAll` removal plus
   the tmpfiles age-out (`tests/pipeline/tmp-dir.ts`, which exists and is the
   helper Test #9 claims), and Rollout 1 keeps `.ts`-only files on the format
   leg's path.
7. Fail closed — the gate's deny → gate-log → escalate routing is intact at both
   hooks and matches the code; findings 1–3 are the three places where malformed
   input still crashes or applies only partially.
8. ADR conformance of the deferrals — the two Deferred entries the diff rewrote
   (backfills as 021-R1's `unassigned` task; `amends` symmetry) contradict
   nothing in ADR-021: 021-L5 permits the explicit `unassigned` marker, 021-L2
   exempts such a reservation from a handle, and 021-R2's named gate classes are
   exactly the four ¶7 maps.

## FYI (no severity)

- Dependency advisories: `.github/workflows/advisories.yml` scans `Cargo.lock`
  with cargo-deny only; no npm leg runs an audit. Pre-existing repo posture,
  untouched by this design (which adds no dependency), so not a finding here —
  but it is a standing gap the harness's own CI story should own.
- ¶2's "an ordered item or a paragraph there … is prose" means a unit *shaped*
  as `1. **013-C9** …` inside a kind section is not a unit, is not registered,
  and produces no finding of its own — the escape is silent, though pointing a
  handle at it does yield `handle-dangling`. Coverage judgement for the
  correctness seat; the direction is at least fail-closed.
- Naming only: for a findings cap, `boundedFindings` (`runner.ts:3213`, with the
  elision-note leg) is the closer precedent than `MAX_GATE_LEGS`, which bounds a
  leg array. The analogy ¶7 reaches for is accurate but one step further from
  what it caps.
