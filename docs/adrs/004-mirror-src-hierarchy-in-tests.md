---
status: accepted
date: 2026-11-10
---

# ADR 004: Mirror the src/ hierarchy in the orchestration test tree

Mechanical reorganization, no semantic change: `tools/orchestration/tests/`
stops encoding subsystem names as filename prefixes and starts mirroring the
`src/` directory layout. (This slot was freed by renumbering the
self-update checkpoint ADR, now ADR-006.)

## Context

`tools/orchestration/src/` has two tiers: the package root (`cli.ts`,
`opencode-runner.ts`) and `src/pipeline/` (the pipeline state machine, 14
modules). `tests/` is flat — 26 files — so the namespace migrated into the
filenames: 23 tests are `pipeline-<name>.test.ts`, while the three
package-root tests (`cli`, `opencode-runner`, `opencode-runner-timeout`)
already mirror the root. The prefix is the only hierarchy signal, each name
must carry subsystem *and* concern (`pipeline-parallel-fanout`,
`pipeline-pass-finalize`), and the next `src/` subsystem would add a third
prefix to a list that is already hard to read.

Nothing pins today's shape in place: `tsconfig.json` excludes `tests/` from
typecheck, vitest's default include is recursive (no vitest config exists),
and the tests never import each other or reference each other's paths. The
move is a rename plus the import depth.

## Decision

Move every file matching `tests/pipeline-*.test.ts` to
`tests/pipeline/<name>.test.ts`, dropping the `pipeline-` prefix — 23 files:
`budget`, `checkpoint-writer`, `cli-max-parallel`, `concurrency`,
`escalation`, `fix-trigger`, `funnel`, `gates`, `identity`, `independence`,
`ownership`, `panel`, `parallel-escalation`, `parallel-fanout`,
`parallel-integration`, `pass-finalize`, `prompts`, `proposal-size`,
`reply-contract`, `runner`, `stage-lifecycle`, `state-store`, `states`.
The three package-root tests stay where they are.

- Moves go through `git mv` so history follows each file.
- The only content changes permitted are depth-sensitive path references: a
  relative path reference - code or comment - naming a location this move
  passes takes its new depth. No test logic, test names, fixtures, or file
  splits: 26 test files before, 26 after.
- No config change: vitest discovers nested tests with its default include;
  `tsconfig.json` excludes `tests/`; `gates.ts`'s `TEST_PATH_RE` already
  matches nested `tests/` paths.
- Naming rule going forward: a directory under `tests/` mirrors a subsystem
  directory under `src/`. Within it, a module-named file (`state-store.test.ts`)
  is that module's unit test; cross-module behavior suites keep descriptive
  names (`funnel.test.ts`, `pass-finalize.test.ts`). New subsystems follow
  the same shape.
- Conventions follow the code in the same change: the repo-practices line
  "tests in `tests/*.test.ts`" becomes the recursive form.

## Alternatives considered

**Keep the flat `pipeline-` prefix:** rejected - the prefix is a namespace
without a hierarchy, already carried by 23 of 26 files, and it grows: every
future subsystem adds its own prefix to a single flat list.

**Strict one-file-per-src-module** (merge the behavior suites into
module-named files): rejected - most pipeline tests pin cross-module
behavior introduced by ADR-002 and ADR-003 (`funnel`, `pass-finalize`,
`parallel-fanout`); merging them would lose exactly what the current names
provide, which behavior is pinned where.

**Make `src/pipeline/` its own package with its own tests:** out of scope -
the layout fix should not wait on a packaging decision, and if that split
ever happens, the test tree will already be shaped for it.

## Consequences

Becomes easy: `ls tests/pipeline/` reads as the subsystem's test inventory;
`vitest run tests/pipeline` scopes naturally; subsystem names stop eating
filename budget.

Accepted costs: it is churn - one rename commit, gate is `npm run check`
green with an unchanged test count. Historical artifacts (run 003's build
notes and review reports, run 002's postmortem) cite the old paths and are
left as written; they are records, not references. From merge, the pipeline's
test-authoring and test-update stages place new tests in the mirrored layout.

Ordering: small enough to land directly between runs without a design run.
If one is ever executed against this ADR, its artifacts carry the same 004
number.
