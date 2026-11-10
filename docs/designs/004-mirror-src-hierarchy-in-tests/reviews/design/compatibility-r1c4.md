# compatibility review r1c4

verdict: approve

Ratification pass. Inputs: the `1f3a83c → 1d1e68a` diff of `02-proposal.md`, the
full current document, `docs/adrs/004-mirror-src-hierarchy-in-tests.md`, the
Revision log, and my r1 findings. Checked only that the revisions did not break
what r1 approved or falsify a section r1 cited.

Matrix state re-checked: all four rows carry a status (no `unknown`, no empty
cell); no OS input API, native module, or keyboard-facing surface is introduced;
no Node API above the `>= 22.12.0` floor appears in the changed text. Item 7
holds as in r1 — Linux executes #1–#5, macOS/Windows are hedged as
"Expected — assumption", not claimed supported. What changed under checklist
item 6 (path separators) is the *scope of paths the move edits*, which the
platform rows were not updated to answer.

## Findings

- [minor][new] Chosen design ¶2 — "the 16 `new URL('<rel>', import.meta.url)`
  paths — `../prompts/` → `../../prompts/`, `../../../.opencode/…` →
  `../../../../.opencode/…`" (with A8's "16 depth-sensitive
  `new URL('<rel>', import.meta.url)` paths in 11 moved files" and Test strategy
  #1's "depth-adjusted `new URL` lines") — the revision moved 16 runtime path
  resolutions from untouched into the diff. In r1 the only edited path strings
  were `import` specifiers, resolved by the toolchain; these 16 are resolved at
  run time against the moved file's own location, and Test strategy #7 keeps
  that execution on Linux ("Linux executes items 1–5"), so no non-Linux run ever
  exercises the new depth. Meanwhile the macOS/Windows rows still justify
  "Expected — assumption" solely with case (A16) and git rename (A15) — neither
  dimension speaks to the sites the revision put in scope, and the design never
  says how those URLs become paths (a `URL` handed to `fs`/`fileURLToPath` is
  `/`-separated and platform-safe; a `.pathname` read or a string-joined path on
  a case-insensitive checkout is the classic footgun, surfacing as ENOENT only
  for a maintainer off Linux). Checklist item 6 is unevaluated for the new scope.
  Fix: one clause in A8's Basis naming the form those 16 sites use (e.g. "each
  site hands the `URL` object to `fs`/`fileURLToPath`; no `.pathname` read, no
  string-joined path"), and carry that clause into the macOS/Windows Notes in
  place of the case-only basis. This names behavior in existing text; it adds no
  mechanism.

- [minor][re-raised] Test strategy #4 — "the existing `stagePathAllowed` block in
  the gate suite asserts the real post-move path
  `tools/orchestration/tests/pipeline/gates.test.ts`" (with A7's Basis
  "gates.ts:253,262" and the Windows row's Notes "Same basis as macOS (A16); git
  handles the rename identically (A15); not executed there") — my r1 separator
  finding is still open: A7's Basis and the Windows Notes are byte-identical to
  the text r1 reviewed, and the revision raised what rests on them. A19 now
  asserts "no pattern in `src/` keys on the flat `pipeline-*` filename shape" and
  #4 turns nested classification into an executed assertion — but its input is a
  hand-written `/`-separated literal, so it passes whether or not the pipeline
  ever hands `stagePathAllowed` (A19's `runner.ts:232` enforcement point) an
  OS-native path; and per #7 it runs only on Linux. The new text therefore gives
  the separator question the appearance of coverage without the evidence, so r1's
  exposure note "limited to gates keying off the `tests/` prefix" now carries
  more weight than it did at approval. Fix: the same clause asked for in r1 —
  name the separator form reaching `TEST_PATH_RE` in A7's Basis ("paths reach the
  gate in POSIX form — git and vitest report `/`-separated paths") and carry it
  into the Windows row's Notes; if the runner builds OS-native paths instead,
  #4's fixture needs one such path alongside the literal.

## FYI (no severity)

- Compatibility matrix Node row is unchanged since r1 — "discovery/compile
  behavior verified at these pinned majors (A4, A5; package.json:31,41,43)" still
  cites caret ranges (`vitest ^5.0.5`, `typescript ^6.0.3`) with A5 recording that
  no `vitest.config.*` exists; my r1 nit on the wording "pinned majors" stays
  open because the text it cited was not touched.
- Not this seat's call, flagging for scope/process: Rollout now lands two commits
  (#1's assertion-only commit, #2's move commit) and Rollout 2 also carries "the
  one-clause amendment to Decision bullet 2", i.e. the move commit edits the ADR
  that authorizes it, while ADR-004's Consequences still reads "one rename
  commit". The proposal's Deferred entry keeps the ADR edit narrow ("Only its
  Decision bullet 2 is edited"), which is consistent with the ADR's Decision, so
  it is not a Deferred contradiction.
