# compatibility review r2

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r2.diff` (four prose hunks + one
added Editor notes line), read against `02-proposal.md`,
`docs/adrs/004-mirror-src-hierarchy-in-tests.md`, and the Revision log. Previous
compatibility findings in this phase: none. Inputs used: the diff, the proposal, the
ADR, and my own static checks in `tools/orchestration/`.

Method note: one repo-wide grep intended to locate convention surfaces incidentally
listed other seats' report and transcript paths under `reviews/` and `transcripts/`.
I did not use their contents as review input; every statement below is from my own
reading of the proposal, the ADR, and the source tree.

## Findings

None. Every hunk in this diff is language-only and leaves the compatibility-bearing
text intact:

- ¶2 (`from '../src/...'` backticks): the fix puts the closing quote inside the code
  span, so the specifier now renders exactly as the ADR's Decision bullet 2 arrow form
  (`../src/...` -> `../../src/...`). I re-verified the counts the sentence rests on:
  62 `from '../src/...'` specifiers in moved files (65 total minus 3 root-tier files),
  and 16 `new URL(...)` sites — `../prompts/` and `../../../.opencode/…` only — all 16
  wrapped in `fileURLToPath(new URL(..., import.meta.url))`. `../../../` from
  `tests/` reaches the repo root, so `../../../../` from `tests/pipeline/` is the
  correct rewrite. Nothing in the hunk changed the scope of the rewrite.
- ¶4 (added comma): the naming rule is unchanged, still bound to
  `repo-practices/SKILL.md:21` and `test-strategy/SKILL.md:27`; I confirmed those are
  the only two live convention surfaces still stating the flat form.
- Rollout 1 (`Risks 4` -> `Risk 4`): the singular now points at one row — the
  acceptance-gate/26→26-count row — which is precisely what a pre-move baseline commit
  supplies. No cross-reference was broken.
- Rollout 4 (`from the merge`): matches ADR Consequences ("From merge, the pipeline's
  test-authoring and test-update stages place new tests in the mirrored layout").
- Added Editor notes line: flags ¶2's amended clause as ambiguous, author to clarify.
  Compatibility-wise the operative scope is unaffected: A8's closed enumeration, ¶2's
  named forms, and Test strategy #1's allowed-set assertion together decide exactly
  which relative references change depth. See FYI.

## Checklist walk-through

1. Compatibility matrix — unchanged by this diff; all four rows carry a status
   (`Yes`, `Expected — assumption`), no `unknown`, no empty cell, and the two
   assumption rows state their basis (A16, A15) and their non-execution.
2. OS input APIs — not applicable to this design, and it does not claim otherwise: the
   change moves and rewrites repo-local files; no X11/Wayland/Raw Input/HID surface is
   touched, so no per-platform input API needs naming.
3. Node version floor — floor is `engines.node: ">=22.12.0"` (package.json:31); the
   design adds no API calls at all. The path forms it edits (`import.meta.url`,
   `fileURLToPath`, `new URL`) sit far below the floor.
4. Native modules — none introduced; the diff adds no dependency and the move touches
   no build step (`tsconfig` excludes `tests/`).
5. Keyboard reality — not applicable; no binding-name or text-entry surface is in scope.
6. File paths — separators are handled by libraries, not string surgery: every one of
   the 16 depth-sensitive sites is `fileURLToPath(new URL('<rel>', import.meta.url))`
   (my grep of `tools/orchestration/tests`), renames go through `git mv`, and the
   rewrites keep POSIX-relative specifiers as text. Case: the 23 stripped names are all
   lowercase and pairwise distinct, and directory `pipeline` collides with nothing in
   `tests/`, so A16's case-insensitivity claim holds on macOS/Windows.
7. Platform tests vs. matrix — Test strategy #7 names Linux executing items 1–5 and
   names the fallback for the two assumption rows (A16, optional maintainer run of
   item 2); the matrix does not claim macOS/Windows as supported, so no platform is
   claimed without a named test path.

## FYI (no severity)

- Chosen design ¶2 — "a relative path reference — code or comment — naming a location
  this move passes takes its new depth" — is the clause Rollout 2 copies verbatim into
  ADR-004 Decision bullet 2, where it outlives this commit and governs the next
  subsystem move. Its compatibility behavior is pinned here by A8/¶2/#1, but the ADR
  copy would be the only surviving rule; the editor note already routes the wording to
  the author, so this is an editorial/correctness call, not a compatibility one.
