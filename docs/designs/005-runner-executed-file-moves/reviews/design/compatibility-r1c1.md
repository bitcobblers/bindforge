# compatibility review r1 (dissent pass c1)

verdict: request-changes

Scope: the r0→r1 diff of `02-proposal.md` plus the full current document. Inputs:
the proposal, `docs/adrs/005-runner-executed-file-moves.md`, the Revision log, and
this seat's r1 findings.

## Closures (r1 findings of this seat)

- macOS row `unknown` → `yes (unverified)` with Test strategy #12 naming the two
  observations (round trip + case-only move) as the A10 evidence. Closed — the row
  now carries a status and a named check (checklist 1, 7).
- Windows row `unknown` → `not exercised`, with rule 1's name checks pinned on Linux
  in #2. Closed — the status is decided and the row no longer promises an
  observation the pipeline will never make.
- Rule 1's undefined "normalized" → `path.posix` segment semantics plus the `\`,
  NUL, control-character and `:` bans, pinned in #2. Closed; checklist 6's
  separator half is now explicit.
- Rule 1's missing per-OS name legality → Windows reserved names and trailing
  `.`/space rejected, pinned in #2. Closed.
- Rule 6 case-folded distinctness: added, but the qualifier's scope is narrower
  than the claims built on it — see finding 2.
- Cross-mount EXDEV now named as a case in #5: observation now exists, feasibility
  caveat in finding 3.
- Git floor: new `git ≥ 2.16` row. Correct — the global `--no-optional-locks`
  shipped in the 2.15 series, so 2.16 is a safe (slightly conservative) floor;
  `git cat-file -t` and default rename detection do predate it. Closed. The Node
  row's `22.23.4` parenthetical is a real 2026-10-21 LTS patch release. Closed.

## Findings

- [major][new] Chosen design / Validator rule 2 + Read-only git — "no endpoint
  `.git`-denied (see Read-only git — the same `stagePathAllowed` rule)" and
  "`stagePathAllowed` gains, as its first line, `path === '.git' ||
  path.startsWith('.git/') → false` (gates.ts:260)" — the guard is a case-sensitive
  string test, and the row that makes it load-bearing is now a supported row
  (macOS `yes (unverified)`). On a case-insensitive checkout, `to =
  '.GIT/hooks/pre-commit'` clears every rule: rule 1 (clean POSIX shape), rule 2
  (`.GIT` ≠ `.git` as a string), rule 3 (`stagePathAllowed` returns true for
  `implement`/`fix` — the new first line does not match either), rule 5
  (`existsSync` is false for a hook that does not exist, no blob at `headBefore`,
  and its parent `.GIT/hooks` resolves to the real `.git/hooks`), rules 6–7
  (disjoint; the realpath is inside the repo). `applyMoves` then renames a
  worktree file *into the real git directory* — invisible to the sweep (A4:
  `.git/**` is never a dirty path) and executed by the runner's own next
  `git commit`. The r1 revision makes this worse to note, not better: it moved
  item 5's enforcement to the harness-side `edit` deny on `.git`, which governs
  what an *agent* writes and cannot constrain a rename the *runner* performs, so
  rule 2 is the only control on this path — and it also added A10's blanket claim
  "nothing else in the design depends on the checkout being case-sensitive", which
  this contradicts. (r1 finding #3's `\` half of this same guard is closed; the
  case half is not.) Fix: make rule 2's test and the `stagePathAllowed` first line
  case-folded — `path.toLowerCase() === '.git' ||
  path.toLowerCase().startsWith('.git/')` — which is platform-independent and
  cannot reject a legitimate name (a segment equal to `.git` ignoring case cannot
  coexist with `.git` on any checkout), and add the `.GIT/hooks/pre-commit`
  endpoint to #2's rejection list, pinned on Linux.

- [minor][re-raised] Chosen design / Validator rule 6 — "all `from`s distinct and
  all `to`s distinct compared case-folded, no `to` equals any `from`, and no listed
  path is a path-prefix of another — which rejects … case-only collisions" — the
  case-folded qualifier grammatically covers only the two distinctness clauses; the
  `to`-vs-`from` and prefix clauses stay string-exact. A single case-only pair
  (`README.md` → `readme.md`) is then rejected nowhere on the reference platform:
  rule 5's `existsSync` guard — the one rule 5 calls "the case-insensitive-checkout
  guard, A10" — is inert on case-sensitive ext4, rule 1's `from !== to` is exact,
  and `applyMoves` lands it as an `R` entry. That makes the Risks mitigation "Rule
  5's disk check rejects case-only endpoints and rule 6 rejects case-folded
  collisions on every platform" false as written for the reference platform, sits
  under the Deferred entry "Case-only renames … support, not just rejection,
  deferred" (which presumes rejection is today's behavior), and leaves the claim
  "apply order is irrelevant" unproven for a list containing
  `A→B` and `a→C`. #2's "duplicate endpoints including case-only collisions" pins
  duplicates *within* a list, not the single case-only pair. Fix: state the scope
  in one clause — "every comparison in this rule is case-folded" — and add one #2
  case: a lone `README.md` → `readme.md` pair rejected on Linux.

- [nit][new] Test strategy #5 / Compatibility matrix (Repository spanning mount
  points, `partial`) — "a pair whose `to` parent is on another mount (tmpfs under
  the worktree) fails `EXDEV`" — the observation now has a named case, but mounting
  a tmpfs needs root/`CAP_SYS_ADMIN`, which the vitest tmpRepo suite cannot assume;
  if CI cannot mount, the row keeps an evidence-less `partial` under a different
  name. Fix: say who runs it, the way #12 does for macOS ("operator-run, mount
  required"), or gate it on mount capability so it skips rather than silently
  vanishes.

## FYI (no severity)

- Rule 1's ban on `:` and on segments ending in `.`/space is stricter than POSIX
  requires (both are legal names on Linux); that is a deliberate allow-pattern, but
  it belongs to the correctness/complexity seats to confirm no in-repo path hits it.
- ADR-005 item 3 says the runner performs the renames "through its own git
  plumbing"; the design's Options 2 rejects `git mv` and uses `fs.renameSync`. The
  ADR's observable requirement (one stage commit, rename detection intact, before
  the sweep and `commitStage`) is met, so this is a wording deviation for the
  ADR-conformance seat, not a compatibility issue.
