# security review r1c1 (dissent pass — sections changed since r1)

verdict: approve

Scope reviewed: `git diff 15ba04f5 0b135560` over `02-proposal.md`, then the full
current document, against `docs/adrs/020-restart-boundary-for-harness-self-changes.md`
and my own r1 findings. Checks run read-only against the code: `nulPaths` and
`porcelainPaths` (runner.ts:247–264, sweep call at runner.ts:307–309), `printable`
(independence.ts:434–445, imported in runner.ts:57), `boundedGateEvidence` /
`boundedFindings` / `boundedField` (runner.ts:3010–3075; state-store.ts:42–43),
`finalizeStage`/`finalizeReviewPass` signatures (runner.ts:1848–1892, 1940–1961),
`loadState`/`saveState` (state-store.ts:239–256, 353–386), the `git` helper
(spawnSync with an argument array — runner.ts:186–187), and a live
`git diff -z --no-renames --name-only <range>` in this repo to confirm the flag
form emits a NUL-separated, unquoted, non-collapsed list that `nulPaths` parses.

## My r1 findings — both closed

- **[closed] §1 detection command.** The call now reads
  "`git diff -z --no-renames --name-only <headBefore>..HEAD` through the existing
  `nulPaths` parser", which is exactly the posture the ownership sweep already
  uses (runner.ts:308, and the comment at runner.ts:247–252 states the two
  hazards: quoting without `-z`, rename collapse without `--no-renames`).
  Verified against this repo's git: the command outputs NUL-separated, unquoted
  paths, so a non-ASCII path under a named prefix and a rename out of the two
  prefixes both reach the predicate instead of disappearing from it. The
  Options-considered paragraph no longer quotes an unflagged `--name-only`
  variant, so there is no second, weaker spelling of the command left in the
  document for an implementer to copy. `--no-renames` also keeps the boundary
  fail-closed on evasion-by-rename: the deleted old path is listed and still
  hits the prefix.
- **[closed] §2 path sanitization.** The new sentence — "The paths are model-
  authorable, so every copy of a path takes `printable` into the record,
  gate-log entry, commit body and summary, and the record keeps the
  `boundedGateEvidence` bounding (first N paths plus the count)" — classifies
  the paths as untrusted where written and names the two existing helpers,
  which is what r1 asked for; both exist and are already reachable from
  `runner.ts`. The residual points below do not re-open this closure.

## Revision-log check

Both r2 entries record the two closures above as decisions; nothing in the log
rejects a security finding I am re-raising, and neither closure was made by
argument. No new rejected-but-open concern appears in the changed text.

## Findings

- [minor][new] Chosen design §2 — "the record keeps the `boundedGateEvidence`
  bounding (first N paths plus the count)" — the named helper carries its
  elision note as an *element of the bounded array*: `boundedGateEvidence`
  pushes a synthetic leg `{ command: '(gate legs elided)', … }`
  (runner.ts:3066–3073), and `boundedFindings` pushes a synthetic finding
  (runner.ts:3031–3038). Those arrays hold typed objects with a field to carry
  the label; `paths: string[]` has none, so taken literally the closure puts a
  runner-authored pseudo-path in the same untagged array as model-authorable
  untrusted path strings — and that array is what the boundary-commit body and
  §4's cross-reference read as "the changed paths". The same sentence leaves
  §3's claim that the status line prints "how many harness files it changed"
  without a number to print once the array is truncated: the total exists only
  inside a display string. Fix: keep `paths` as paths only — first N
  printable-copied entries — and carry the pre-bound total as a number on the
  `restartRequired` object (a field inside a field this change is already
  adding, so ADR-006 §2 is untouched); the gate-log entry and §3's status line
  print the count from that number rather than from a pseudo-path.

- [nit][new] Chosen design §2 — "every copy of a path takes `printable` into
  the record, gate-log entry, commit body and summary" — `printable` is
  `value.replace(/\p{Cc}/gu, ' ')` (independence.ts:443–445): it neutralizes C0/C1
  controls, ESC sequences, newlines and BEL, which is the spoofing class r1
  named, but it leaves format characters (`\p{Cf}`: the U+202A–U+202E
  directional isolates/overrides, zero-width joiners) untouched, and those are
  the other way a crafted model-authored filename reorders or hides characters
  on the operator's terminal and in the `git log` the operator reads for the
  boundary commit. This is the repo-wide posture, not a defect this design
  introduces, so it should not be fixed here; it should not be *claimed*
  beyond what the helper gives either. Fix: nothing new — in the same §2
  sentence, say the path copies take printable's Cc-only coverage and that
  BiDi/format characters stay out of scope repo-wide, so a later hardening of
  `printable` is the place it belongs.

## FYI (no severity)

- The boundary keys detection to commit ranges (`<headBefore>..HEAD`), matching
  ADR-020 Decision 1 ("After each stage commit…"). A surface-path edit left
  *dirty and uncommitted* by a failed attempt is not swept for `implement`/`fix`
  (stagePathAllowed admits those paths, gates.ts:575–582), and templates are
  read from the worktree at stage entry, so a later stage in the same process
  can render changed-but-uncommitted template text with the pre-change renderer
  before any commit exists to trip §1. That is the ADR's own commit-keyed rule,
  not a contradiction of it, and the changed text did not create it — noting it
  for the correctness seat only.
- Checklist #3 stays clean: the hit paths are only filtered and printed, never
  fed back as git arguments, and the `git` helper is `spawnSync('git', args[])`
  with no shell (runner.ts:186–187).
- A7's rewording is accurate against the code and makes the compatibility
  matrix's "the record inert to it" wording honest: `loadState` returns the raw
  parse (`parsed as PipelineState`, state-store.ts:256) and `saveState`
  stringifies the state handed in (state-store.ts:365), so the key is inert to
  consumers *and* round-trips. No security change from r1's wording; the
  pre-020-resume fail-open it describes is already carried in Risks row 2 and
  the matrix row, consistent with ADR-006.
