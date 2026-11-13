# Code review — compatibility, round 2 ratification (c2)

Scope: `git diff 6a5c454827e1ee63b910e3c937b114129784d3c8..HEAD -- ':(exclude)docs/designs'`
(4 commits: `873204e`, `c3c085f`, `c2fd15e`, `fe424f1`), read against the current
`02-proposal.md` and `docs/adrs/016-harness-evidence-and-read-rights.md`.
Pass posture: ratification — does the post-r2 fix work break what this seat approved?

Verdict: **approve** (no blocker, no major). One carried-forward finding of this
seat regressed in severity of consequence rather than in kind (finding 1).

## Gate evidence used

The runner's gate record quoted into this pass's inputs (2026-11-13T19:39:13Z,
green): `npm run check` exit 0, `cargo fmt --all -- --check` exit 0,
`cargo clippy --workspace --all-targets -- -D warnings` exit 0,
`cargo test --workspace --locked` exit 0 with `test result: ok. 29 passed; …`
and `test result: ok. 2 passed; …` legs. That record closes my r2 finding 3
(the "no gate record for this run yet" half): the block now renders into review
prompts and is citable. Cited line-selectively, no absolute-path lines.

## Checklist

1. **Platform vs the Compatibility matrix** — no new OS-specific code without a
   guard. The fix's only host-sensitive additions are spelling handlers
   (`citableCwd` normalizes `\` to `/` and recognizes `C:\…`;
   `ABSOLUTE_PATH_LINE` covers POSIX and Windows absolute paths), which fail
   safe (drop, not leak). No new shebang fixtures or POSIX-only shell scripts
   were added in this range; the pre-existing POSIX-only stub-cargo and
   PATH-shadow legs in `gate-evidence-capture.test.ts` are untouched, so the r2
   guard nit stands (finding 6).
2. **Node floor** — nothing in the range uses an API above the raised floor:
   regex lookbehind (`runner.ts:3096`), `Array.from`, `Object.entries`, replacer
   callbacks, `Array.prototype.at` (tests) are all inside Node 22. The floor
   *text* drift this seat reported in r2 is untouched by these commits
   (finding 2).
3. **Paths** — one new hand-rolled path computation, `citableCwd`
   (`runner.ts:3133-3147`), where the repo's own idiom is
   `relative(repoDir, target).split(/[\\/]/).join('/')` (`runner.ts:222`,
   `identity.ts:59`, `moves.ts:175`) — finding 4. No string-concatenated path
   *construction* was introduced.
4. **Guarded platform imports** — none added; nothing in the range imports a
   platform-only module.
5. **Locale/layout reality** — tails survive non-ASCII: `printable` only
   replaces `\p{Cc}`, `defuseFences` and the line-dropping filter are byte-
   agnostic, and the render's hard cap slices by code points
   (`runner.ts:3273`). One new slice bounds by UTF-16 units — finding 5.
6. **Native dependencies** — none introduced (no `package.json` dependency
   changes in this range).
7. **Tests run on the platforms they claim** — the new integration fixture
   (`gate-evidence-integration.test.ts`, `dirty` mode) uses only `git`, `node`
   and the `check` npm script, so it is not platform-locked; the new render and
   agent-shell tests are pure TS. Test strategy #14's real-workspace cargo claim
   is still unpinned and the new fixture widens the same gap (finding 3).

## What the fixes do not disturb (approved posture re-checked)

- Checkpoint-fence additions (`.opencode/agents/**`, 19 files): the new
  `shell *pipeline.json*` + `read`/`glob` denies sit **before** the per-seat
  `git diff *reviews/<phase>/<specialty>-*` re-open, so last-match-wins keeps
  Decision 1's route open (`code-compatibility.md` confirms the order);
  `agent-shell.test.ts:415-450` pins all three commands denied for every
  `SHELL_HOLDING` agent and the run's own `01-research.md`/`02-proposal.md`/
  `03-build-notes.md` still allowed for the doc families.
- The checkpoint really lives at `join(designDir, STATE_FILE)`
  (`state-store.ts:14`) = `docs/designs/<nnn>-<slug>/.pipeline.json`, so the
  deny pattern's reach matches the file's location; the audit's
  `STATE_FILE_SEGMENT` is built from `STATE_FILE` so rule and writer cannot
  drift, and `grep` stays judged on `path`/`include`, never its query
  (`independence.ts:108`), so searching *for* the name is not a violation.
  Only standard rule types are used, so the matrix's "Pre-ADR-016 server
  config: yes" row still holds.
- Back-compat rows hold: `docName` is `optional()` (`state-store.ts:152`),
  `proposalGrowth` reads absence as the design document
  (`report.ts:180-188`), and `gateEvidenceText` re-bounds a record written
  before the cap; `docFileFor` naming is accurate — the plan stage really
  writes `02-proposal.md` (`prompts/plan.md:3`).
- `citableTail` runs before `printable` at write (so it can split lines) and is
  idempotent under the render-time re-bound; `boundedTail`'s elision marker
  never matches `unshipped`, so the cap's own statement survives the citation
  filter.

## Findings

1. **minor — the green leg's citable line is now the weakest summary in the
   tail.** `runner.ts:3188-3195` (`greenSummary`) takes
   `tail.lastIndexOf('test result:')`, then cuts at ` finished in `. For a
   `cargo test --workspace --locked` leg the last target cargo reports is a
   **doc-test** target: in this pass's own gate record that is
   `Doc-tests bindforge_transforms … test result: ok. 0 passed; …`, while the
   crate totals (`29 passed`, `2 passed`) sit earlier in the flattened tail. The
   round-2 render this seat approved emitted the 29-passed summary (visible in
   the gate block quoted into this pass's inputs), so the fix for the
   too-long-join problem removed the artifact Test strategy #10 and A18 promise
   — "a behavior claim has a citable `test result:` line" — and replaced it with
   a line that reads as *no tests ran*. `tests/pipeline/gate-evidence-render.test.ts:150-165`
   pins the new shape against a fixture whose last summary is the meaningful
   one, so the workspace ordering is exactly the case left unpinned.
   **Fix:** choose the first non-vacuous summary (`0 passed; 0 failed` skips
   forward, lastIndexOf as the fallback), or render the first summary and a
   count of further ones; pin it with a fixture that ends in a doc-test
   `0 passed` target after an integration-target summary, i.e. the shape the
   gate record in this pass shows.
2. **minor, carried and unfixed — the Node floor still reads 22.23.4 in two
   of its four statements.** `README.md:24` ("`npm ci && npm run check` in
   `tools/orchestration` (Node 22.23.4)") and
   `.opencode/skills/repo-practices/SKILL.md:65-66` ("Node floor for the
   orchestration gate: 22.23.4") against `tools/orchestration/package.json:31`
   (`>=22.23.5`) and `.github/workflows/check.yml:50` (`node-version: "22.23.5"`).
   None of these four paths is in the c2 range, so the drift my r2 finding
   named is untouched; `crates/core/tests/gate_contract.rs` keeps only the
   package.json/CI pair in lockstep, and the skill copy is the live trap because
   every coder and test-engineer stage loads it.
   **Fix:** one-line change in each of the two files to `22.23.5`.
3. **minor, carried and widened — Test strategy #14's real-workspace cargo
   claim is still not shipped.** `02-proposal.md:393-395` claims "the cargo legs
   run against the real workspace in the integration tests that need a green
   gate". The c2 range adds a third mkdtemp integration fixture
   (`gate-evidence-integration.test.ts:38-95`: `dirty` mode of the same
   `gateScript`, `tmpGateRepo` still writes a `package.json` and no
   `Cargo.toml`), so `hasCargoWorkspace` is false there too and the persisted
   record still contains only `npm run check` legs; the only cargo capture in
   the suite still stubs `cargo` on PATH. A18's premise is therefore still
   pinned only against synthetic summaries — which is how finding 1 slipped
   through.
   **Fix:** one leg against a fixture workspace (`[workspace] members = []`
   plus a trivial member) or the repo's own workspace, asserting a real
   `cargo test --workspace --locked` leg's tail and its rendered citable line;
   or amend #14 to the shipped posture and register it under Deferred.
4. **nit — `citableCwd` hand-rolls relativization and its Windows branch
   degrades silently.** `runner.ts:3133-3147` compares
   `path.startsWith(root + '/')` case-sensitively after normalizing separators,
   so a host that spells the root differently by case (`C:\Work\…` vs
   `c:\work\…`) falls through to `'(cwd withheld)'` and every leg's package dir
   vanishes from the committed record and from any future render — while
   `path.win32.relative`, the repo's idiom elsewhere, is case-insensitive. On
   Linux (the only exercised platform) the dirs come from `join(repoDir, …)`, so
   the prefix match is guaranteed; hence nit.
   **Fix:** `relative(repoDir, cwd).split(/[\\/]/).join('/')` (the
   `runner.ts:222` idiom) and keep `'(cwd withheld)'` only when the result
   escapes the root (`..` segment).
5. **nit — the new citable-line slice bounds by UTF-16 units, unlike its
   sibling.** `runner.ts:3192-3194` slices `tail` between two indices, so a
   160-unit window ending inside a surrogate pair (a non-BMP character in a test
   name or path — checklist item 5) puts a lone surrogate into the rendered
   block; `printable` runs at write, before the slice, and nothing re-sanitizes
   after it. `boundedTail` in `gates.ts` was written with the explicit
   "code points, not UTF-16 units" rule for exactly this reason.
   **Fix:** `Array.from(tail).slice(a, b).join('')`, or re-apply the code-point
   guard after slicing.
6. **nit, carried and unfixed — the POSIX-only stub legs stay unguarded.**
   `tests/pipeline/gate-evidence-capture.test.ts:218-247` (shebang `cargo`
   stub, mode 0o755, PATH prefixed) and `:166-178` (PATH swapped to a
   non-existent bin dir) gained no `describe.runIf(process.platform !== 'win32')`
   guard in this range; `moves.test.ts:753/:765` is the repo's spelling. No
   support claim breaks — the matrix lists macOS and Windows/WSL2 as `not
   exercised` — but a contributor on an unclaimed host gets a red
   `npm run check` instead of a skipped-and-labelled leg.

## FYI (out of specialty, no severity)

`boundedGateEvidence` flattens tails through `printable`
(`independence.ts:443`, `\p{Cc}` → space), so the renderer's
`leg.tail.split('\n')` loop in `runner.ts:3239` always yields one element: the
"one line per captured line" intent of the fence fix is inert for failing legs —
a failing leg's whole tail still arrives as a single `| ` line.
