# Research: Mirror the src/ hierarchy in the orchestration test tree (ADR-004)

## Summary

ADR-004 proposes moving the 23 `tests/pipeline-*.test.ts` files in
`tools/orchestration/tests/` into `tests/pipeline/<name>.test.ts`, dropping the
filename prefix so the test tree mirrors `src/`, with the import-depth rewrite
as the only content change (source: docs/adrs/004-mirror-src-hierarchy-in-tests.md).
Direct inspection confirms every load-bearing claim the ADR makes about the
toolchain: `tsconfig.json` excludes `tests/` from compilation, no vitest config
exists so the recursive default `include` discovers nested tests automatically,
and `gates.ts`'s `TEST_PATH_RE` already matches nested `tests/` paths (source:
tools/orchestration/tsconfig.json:18, https://vitest.dev/config/include,
tools/orchestration/src/pipeline/gates.ts:253). All 26 test files import source
only via relative `../src/...` paths and never import each other, so the move
reduces to `git mv` plus one uniform path edit per import. The two friction
points the ADR underplays are a stale cross-reference comment inside
`pipeline-parallel-escalation.test.ts` (the "only import depth" rule leaves it
pointing at a path that no longer exists) and a one-off count error in the ADR
text (`src/pipeline/` has 13 modules, not 14). The chosen shape — a tests/ tree
mirroring src/ subsystems — matches the Maven-style mirroring tradition and sits
comfortably inside what vitest and Jest discover by default.

## Findings

Current shape of `tests/` (flat, 26 files, prefix-carried namespace)

- `tools/orchestration/tests/` is flat with exactly 26 test files: 23 match
  `pipeline-*.test.ts` and the three package-root tests are `cli.test.ts`,
  `opencode-runner.test.ts`, `opencode-runner-timeout.test.ts`
  (source: tools/orchestration/tests/ directory listing).
- The 23 names the ADR lists for the move match the on-disk files one-for-one
  after prefix stripping; no two stripped names collide inside `tests/pipeline/`
  (source: tools/orchestration/tests/ vs docs/adrs/004-mirror-src-hierarchy-in-tests.md:33-38).
- `src/` has the two tiers the ADR describes: package root (`cli.ts`,
  `opencode-runner.ts`) and `src/pipeline/` — but `src/pipeline/` holds 13
  modules (`cli`, `concurrency`, `gates`, `identity`, `independence`, `panel`,
  `prompts`, `report`, `runner`, `stages`, `state-store`, `states`, `types`),
  not the "14 modules" the ADR text states
  (source: tools/orchestration/src/pipeline/ directory listing vs
  docs/adrs/004-mirror-src-hierarchy-in-tests.md:16-17).
- Several moved names will not correspond to a `src/pipeline/` module file
  (`funnel`, `pass-finalize`, `parallel-fanout`, `stage-lifecycle`,
  `checkpoint-writer`, `reply-contract`, …); these are cross-module behavior
  suites, exactly the case the ADR's naming rule carves out for descriptive
  names (source: tools/orchestration/tests/pipeline-funnel.test.ts:10-12,
  docs/adrs/004-mirror-src-hierarchy-in-tests.md:48-52).

Nothing pins the current layout (the ADR's context claims, verified)

- `tsconfig.json` sets `rootDir: "./src"`, `include: ["src/**/*.ts"]` and
  `exclude: [..., "tests"]`, so tests are outside every compile/typecheck pass
  and their location cannot affect the build (source:
  tools/orchestration/tsconfig.json:8,17,18).
- No `vitest.config.*` (or `vite.config.*`) exists anywhere in the repo, so
  vitest runs on defaults; the default `include` is the recursive glob
  `['**/*.{test,spec}.?(c|m)[jt]s?(x)']`, which discovers
  `tests/pipeline/*.test.ts` with no config change (source: repo-wide glob for
  `vitest.config.*` — no matches; https://vitest.dev/config/include).
- The CLI scoping the ADR promises works as stated: vitest positional args are
  plain substring filters on test-file paths (not globs), so `vitest run
  tests/pipeline` selects every file under the new directory (source:
  https://vitest.dev/guide/cli).
- `gates.ts`'s `TEST_PATH_RE = /(^|\/)tests\/|\.test\.ts$/` matches any nested
  `tests/…/x.test.ts` path, so the fix/test-authoring path gates behave
  identically after nesting (source: tools/orchestration/src/pipeline/gates.ts:253,262).
- Test files never import each other: every relative import in all 26 files is
  a `../src/...` specifier, so the only content change is `../src/` →
  `../../src/` in files one level deeper (source: grep for `from ['"]\.\.?/`
  across tools/orchestration/tests — 65 matches, all `../src/...`).
- The lint/format gate is layout-agnostic: `eslint.config.js` ignores only
  `dist` and `node_modules`, and `npm run check` chains lint, format:check and
  test (source: tools/orchestration/eslint.config.js:5,
  tools/orchestration/package.json:23).
- Because every `../src/...` specifier in a file shifts identically to
  `../../src/...`, the prettier import-sorting plugin's relative order within a
  file is unchanged by the rewrite (source: package.json:35 lists
  `@trivago/prettier-plugin-sort-imports`; uniform-prefix argument from the
  grep above).
- The agent permission globs are already recursive: the test-engineer agent
  binds resource `**/tests/**`, and `TEST_PATH_RE` drives the ownership
  in-test checks, so nested tests are governed exactly as flat ones (source:
  .opencode/agents/code/test-engineer.md:9, tools/orchestration/src/pipeline/gates.ts:262).
- The renumbering the ADR cites is real in the tree: `docs/adrs/005-self-update-checkpoint-compatibility.md`
  exists alongside 001–003, and slot 004 is this ADR (source: docs/adrs/
  directory listing).
- The test-authoring stage prompt is directory-generic ("the affected package's
  tests/ directory") with no flat-file naming rule, so it needs no edit for
  nesting; the pipeline stage prompts stay compatible (source:
  tools/orchestration/prompts/test-authoring.md:3).

Convention surfaces the move touches

- `repo-practices` SKILL.md states "tests in `tests/*.test.ts`" — the ADR says
  this line becomes the recursive form in the same change; it also cites
  `tools/orchestration/tests/opencode-runner.test.ts` as an example, a root-tier
  file that stays put and keeps its path valid (source:
  .opencode/skills/repo-practices/SKILL.md:21,28,
  docs/adrs/004-mirror-src-hierarchy-in-tests.md:53-54).
- The `test-strategy` skill also hard-codes the flat form: "Follow
  `repo-practices` conventions: vitest, `tests/*.test.ts`" — the ADR's
  convention sentence names only the repo-practices line, so this second site
  is arguably in scope too (source: .opencode/skills/test-strategy/SKILL.md:27).

Stale references the "import depth only" rule will strand

- `pipeline-parallel-escalation.test.ts:45` carries a comment naming
  `tests/pipeline-reply-contract.test.ts`; after the move that path does not
  exist, and the ADR's no-content-change rule forbids fixing it (source:
  tools/orchestration/tests/pipeline-parallel-escalation.test.ts:45,
  docs/adrs/004-mirror-src-hierarchy-in-tests.md:42-44).
- Other in-test path mentions are inert fixture data, not references:
  `pipeline-independence.test.ts:145` uses a never-existing
  `pipeline-lane.test.ts` path, and `pipeline-gates.test.ts:298-303` uses
  `tests/cli.test.ts` (a root-tier file that does not move) — both stay green
  either way (source: tools/orchestration/tests/pipeline-independence.test.ts:145,
  tools/orchestration/tests/pipeline-gates.test.ts:298-303).
- Historical run artifacts cite old paths heavily — run 003's build notes name
  at least ten `tests/pipeline-*.test.ts` files, and its review reports and
  transcripts cite more; the ADR explicitly leaves these as records
  (source: docs/designs/003-parallel-inference-lane/03-build-notes.md:21,37,54,
  docs/adrs/004-mirror-src-hierarchy-in-tests.md:80-82).

## Prior art

Lens applied: how comparable projects solve "tests/ layout vs src/ layout".

- **Maven-style mirroring (closest match to the ADR):** Maven's standard
  directory layout puts test sources under `src/test/java`, mirroring the
  package structure of `src/main/java`; its stated rationale is that "having a
  common directory layout allows users familiar with one Maven project to
  immediately feel at home in another Maven project" — the same inventory-
  readability argument the ADR makes for `ls tests/pipeline/` (source:
  https://maven.apache.org/guides/introduction/introduction-to-the-standard-directory-layout.html).
- **Go (colocation):** Go co-locates `*_test.go` files inside the package
  directory itself; `go build` ignores files ending in `_test.go`, and there is
  no separate tests tree at all — the hierarchy signal is the package directory
  (source: https://pkg.go.dev/cmd/go#hdr-Compile_packages_and_dependencies).
- **Rust (two-tier split):** cargo puts integration tests in a top-level
  `tests/` directory while unit tests live inside the `src/` files themselves
  (`#[cfg(test)] mod tests` in `src/lib.rs`); `tests/` subdirectories are only
  used for multi-file test targets — hierarchy follows test targets, not source
  modules (source: https://doc.rust-lang.org/cargo/guide/project-layout.html,
  https://doc.rust-lang.org/book/ch11-01-writing-tests.html).
- **JS ecosystem (suffix-based discovery, layout is free):** Jest discovers
  tests by recursive default `testMatch` (`**/__tests__/**` and
  `**/?(*.)+(spec|test).*`), and vitest by its recursive default `include`;
  neither tool's defaults encode hierarchy, so both a flat and a mirrored tree
  work out of the box — the flat `pipeline-` prefix in this repo is a
  convention the tooling never required (source:
  https://jestjs.io/docs/configuration#testmatch, https://vitest.dev/config/include).
- No cross-ecosystem format "won" outright: mirroring (`src/test/java`),
  colocation (Go), and target-based `tests/` (Rust) coexist. The ADR's hybrid —
  mirrored subsystem directory, module-named unit file, descriptive names for
  cross-module suites — is the Maven mirroring pattern plus the JS
  `.test.ts` suffix convention; research found no documented regret cases
  attached to mirroring itself (the ADR's own Alternatives section rejects the
  strict one-file-per-module variant for the right reason, losing behavior
  pinning) (source: docs/adrs/004-mirror-src-hierarchy-in-tests.md:62-66).

## Platform notes

Lens applied: toolchain specifics that gate the move (the input-path/OS
lenses do not apply to a repository-layout change).

- Node floor for the package is `>=22.12.0`; nothing in the move touches Node
  APIs (source: tools/orchestration/package.json:31).
- vitest `^5.0.5` with no config file: discovery is the recursive default
  `include` glob, so nested directories need no change; the CLI file filter is
  a path-substring match, making `vitest run tests/pipeline` valid after the
  move (source: tools/orchestration/package.json:43,
  https://vitest.dev/config/include, https://vitest.dev/guide/cli).
- TypeScript `^6.0.3` never compiles tests at all (`exclude: ["tests"]`,
  `rootDir: "./src"`), and vitest transpiles imports itself, so the rewritten
  relative specifiers are resolved at runtime only — no tsconfig edit is needed
  and none exists for a test tree (source: tools/orchestration/tsconfig.json:8,18,
  tools/orchestration/package.json:41).
- Git history following: `git mv` stages the rename (commit still required);
  rename detection is on by default with a 50% similarity threshold, and a pure
  move plus a one-token import change keeps similarity near 100%, so diffs and
  logs follow the files; `git log --follow` continues a file's history beyond
  the rename but only for a single path at a time (source:
  https://git-scm.com/docs/git-mv, https://git-scm.com/docs/git-diff,
  https://git-scm.com/docs/git-log).
- The move is a same-directory-tree rename with no case-only or
  case-colliding renames among the 23 names, so case-insensitive checkouts
  (macOS/Windows) see nothing special; all 23 stripped names are distinct
  (source: docs/adrs/004-mirror-src-hierarchy-in-tests.md:33-38).
- The gate the ADR proposes as acceptance — `npm run check` green with an
  unchanged test count — maps exactly to the package's check chain
  `lint && format:check && test`, so the count check is observable in the
  default vitest reporter output (source: tools/orchestration/package.json:23).

## Risks & unknowns

- **ADR text error:** the Context says `src/pipeline/` has "14 modules"; the
  directory holds 13. Cosmetic, but a design run copying the ADR verbatim
  would propagate the error (source:
  docs/adrs/004-mirror-src-hierarchy-in-tests.md:16 vs
  tools/orchestration/src/pipeline/ listing).
- **Stranded comment:** the "only content change is the import depth" rule
  forces the commit to ship with `pipeline-parallel-escalation.test.ts:45`
  pointing at a path that no longer exists; either the rule needs a stated
  carve-out for comment path-references or the comment knowingly goes stale
  (source: tools/orchestration/tests/pipeline-parallel-escalation.test.ts:45,
  docs/adrs/004-mirror-src-hierarchy-in-tests.md:42-44).
- **Convention edits are plural:** the ADR names one convention line
  ("tests in `tests/*.test.ts`") but the same flat form appears in two skill
  files; missing one leaves the pipeline's own test-authoring stages writing
  flat names (source: .opencode/skills/repo-practices/SKILL.md:21,
  .opencode/skills/test-strategy/SKILL.md:27).
- **Concurrent-edit window:** the ADR lands the rename "directly between runs"
  (source: docs/adrs/004-mirror-src-hierarchy-in-tests.md:84-85), but the
  pipeline's fix/test-update stages edit these exact test files; a run in
  flight across the rename commit produces rename/add churn on 23 files.
  Unquantified — depends on whether any run is mid-branch at merge time
  (unknown from research).
- **Fixture-data staleness:** in-test path strings that mimic the old flat
  naming (the synthetic `pipeline-lane.test.ts` fixture) will describe a
  layout that no longer exists; they stay green but teach the wrong convention
  to future readers and model-authored tests (source:
  tools/orchestration/tests/pipeline-independence.test.ts:145).
- **No verification of the acceptance gate yet:** research confirmed the
  mechanisms (default include, gates regex, tsconfig) statically but did not
  execute `npm run check`; the 26→26 count claim rests on inspection of the
  file list, not a run (unknown until the change is made).

## Open questions

1. Does the "import depth only" rule get a carve-out for the path reference in
   the comment at `pipeline-parallel-escalation.test.ts:45` (and any like
   it), or does the ADR intend those comments to go stale like the run-003
   artifacts?
2. Are the skill files the only convention surfaces to update — should
   `.opencode/skills/test-strategy/SKILL.md:27` be amended in the same commit
   alongside `.opencode/skills/repo-practices/SKILL.md:21`?
3. Should the ADR's "14 modules" be corrected to 13 when the decision is
   implemented, or is the ADR frozen as written (status: proposed)?
4. One rename commit for all 23 moves (ADR's "one rename commit") or per-file
   `git mv`s in a single commit — does the implementer need a prescribed
   ordering so the diff review stays cheap?
5. Should fixture-data path strings that mimic old naming (e.g.
   `pipeline-lane.test.ts` in `pipeline-independence.test.ts:145`) be left as
   written, or renamed to the new convention while preserving assertions?
6. Is there any protection needed against a concurrent pipeline run editing
   old test paths across the merge (branch freeze, or "land only when no run
   is in flight")?
