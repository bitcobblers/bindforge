# Code review — compatibility — r1c0

Range reviewed: `git diff d5092f39...HEAD -- ':(exclude)docs/designs'`
(26 files: 23 renames under `tools/orchestration/tests/`, 2 skill files, 1 ADR line-set).

Verdict: **approve** — no blocker or major findings; zero open findings.

## Checklist walk

1. **Platform code vs Compatibility matrix** — Pass. The diff introduces no
   OS-specific code, so no guards are required, and every "supported" row of the
   matrix (Linux yes; macOS/Windows expected; Node/vitest/TS majors) has a real
   code path: plain Node fs/child_process APIs plus the git CLI, all available on
   every claimed platform. No config change was made (vitest/tsconfig/eslint
   untouched), consistent with the matrix's "move touches no Node APIs" claim.
2. **Node floor (`engines: >=22.12.0`, package.json:31)** — Pass. New APIs in
   the added test code: `fs.promises.readdir(path, {recursive: true,
   withFileTypes: true})` (recursive added v16.1.0), `Dirent.parentPath` (added
   v20.12.0, and the code even falls back with `?? dir`), `child_process.execFileSync`
   (ancient). Moved files add no APIs at all.
3. **Paths** — Pass. The new code uses `join()`/`new URL()` joins, not string
   concatenation, for filesystem access (`join(gitRoot(), '.opencode', 'skills',
   skill, 'SKILL.md')`; `join(gitRoot(), dirRel)`). The literal `/` joins in
   `gates.test.ts` (`nested`/`rootTier` path lists, `includes('tests/pipeline')`)
   are correct-by-construction: they are compared against `git ls-files
   --full-name` output, which is repo-root-relative and always `/`-separated on
   every platform — using `path.join` there would in fact break Windows
   comparison, so the literal slashes are the compatible choice. Depth rewrites
   verified arithmetically: `../../src/` and `../../prompts/` from
   `tools/orchestration/tests/pipeline/` land at `tools/orchestration/`;
   `../../../../.opencode/…` lands at the repo root. Case assumptions: all 23
   moved names are lowercase and distinct, no file↔directory name collision with
   the new `tests/pipeline/` dir (`pipeline` vs `pipeline-*.test.ts`), so A16's
   case-insensitive-checkout claim holds statically.
4. **Guarded platform imports** — Pass. `node:child_process` is cross-platform;
   no platform-only module is imported anywhere in the diff.
5. **Locale/layout reality** — Pass. All new file/dir names are ASCII; the
   pre-existing non-ASCII fixture (`'Über Brief'`, `identity.test.ts:65`) is
   untouched and survives the move. Slug/ID handling code is unchanged.
6. **Native dependencies** — Pass, none introduced. The added tests shell out to
   `git`, which is not new to this package: existing tests (`ownership`,
   `funnel`, `escalation`, `identity`, …) and `src/pipeline/{runner,gates,
   identity}.ts` already require the git CLI. Packaging/CI story unchanged.
7. **Tests run on the platforms they claim** — Pass, evaluated statically (I
   could not re-execute the suite in this worktree — no local `node_modules`;
   Linux execution is the recorded `npm run check` gate-reproduction commits).
   The new tree-pinning and convention tests carry no platform skip or guard:
   they are platform-neutral and run wherever the suite runs, so nothing "skips
   everywhere"; Test strategy #7's macOS/Windows "not executed, relies on A16"
   claim matches the code (no claim of execution there). The `git ls-files`
   fail-closed check uses `--full-name` + `/`-separated expectations, so it
   behaves identically in a linked worktree or a Windows checkout.

## Out-of-specialty FYI (no severity)

`gates.test.ts`'s "tracks exactly 26 test files" asserts set *equality* against a
hard-coded 26-path list, not just the proposal's zero-flat-leftover rule — so any
future run that adds a test file will fail `npm run check` on every platform until
the list is edited (test stages may edit it, so it is a tripwire, not a wall).
That is a test-strategy judgment for the testing seat, not a compatibility defect.
