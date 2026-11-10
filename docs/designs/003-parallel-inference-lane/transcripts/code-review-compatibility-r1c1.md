# Transcript: code-review:compatibility r1c1

## Provenance
- stage: code-review:compatibility (round r1c1)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-10T05:09:33.429Z
- ended: 2026-11-10T05:13:03.093Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/003-parallel-inference-lane/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 91dfbea2f0ab34c063a14036b3afdc1f4b4227d9..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/003-parallel-inference-lane.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "docs/workflows.md:83-91 (and tools/orchestration/README.md)",
    "problem": "The operator-facing compatibility story for the new knob never lands, and the docs now contradict the shipped code. workflows.md:83-85 still says \"Seats execute one at a time (single local inference lane)\", which is false for any --max-parallel >= 2 and never notes the pass-commit shape change at the default; workflows.md:87-91 still lists ADR-003 under \"Proposed, not implemented\", the exact entry Rollout step 3 says must move out with the change; and `--max-parallel` / `$OPENCODE_MAX_PARALLEL` with the `N <= server slots` guidance and the per-seat-deadline/queueing note appear nowhere in workflows.md or either README — only in a code comment (src/pipeline/cli.ts:56-64) and yargs describe strings. Since A1/A2 make the runner unable to verify slot count, that guidance is the only mitigation for the change's top-rated risk, and the matrix row \"Run with N > 1 — yes, conditional on the server exposing >= N slots\" has no discoverable support story.",
    "fix": "Add the knob to the workflows.md CLI block (`pipeline run/resume <slug> --max-parallel <n>`, `$OPENCODE_MAX_PARALLEL`, default 1) with the `N <= server slots` guidance and the note that each seat's deadline starts at its own prompt and counts queue waiting; delete the ADR-003 bullet from \"Proposed, not implemented\"; rewrite the \"one at a time\" sentence to \"up to N seats at a time (--max-parallel, default 1 — the serial lane), launch order fixed, one commit per pass\"; and add the knob to tools/orchestration/README.md's flag table or state why it does not list pipeline knobs."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:281-306 (formatTouchedFiles)",
    "problem": "The new hygiene step resolves the formatter as a hard-coded path `join(pkg, 'node_modules', '.bin', 'prettier')` instead of npm's own resolution, and when that path is absent it does a bare `continue` with no log line. Under any hoisted layout (npm workspaces, pnpm, yarn) or a checkout without devDependencies, packageDirs finds the package, the shim is one level up, formatting silently stops happening, and the run-003 deadlock the function exists to remove returns with no signal — the comment \"the gate will say so instead\" only means the gate goes red while nothing says the formatter never ran. The shim is also the POSIX sh script; on Windows npm writes `prettier.cmd`, so this is OS-specific behavior added against a matrix row whose stated basis is \"no OS-specific code added\".",
    "fix": "Resolve the formatter the way npm does — walk up from the package to the nearest `node_modules/.bin/prettier`, or `spawnSync('npm', ['exec', '--no', '--', 'prettier', '--write', ...], { cwd: pkg })` — and console.warn naming any package that has dirty formattable files but no formatter, so the skip is never silent."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/runner.ts:281-303 (formatTouchedFiles file list)",
    "problem": "The formatter's file list comes from dirtyFiles(), which strips the porcelain status letters (porcelainPaths, runner.ts:167-172), so git-deleted paths go to `prettier --write` alongside modified ones. Prettier exits non-zero for a pattern matching nothing (\"No files matching the pattern were found\") and treats each positional arg as a glob, so one stage that deletes a .ts/.md file makes the whole --write batch for that package fail — the `res.status !== 0` branch only warns, leaving every other dirty file in that package unformatted, i.e. the deadlock case returns silently — and a dirty path containing glob metacharacters (`[`, `]`, `*`, `?`, `{`, `}`) is reinterpreted as a pattern matching nothing or the wrong siblings. pipeline-fix-trigger.test.ts:87-108 only dirties modified, plainly-named files, so CI is green and the failure is latent.",
    "fix": "Keep the porcelain status letters for the formatter's list (or filter on existsSync(join(pkg, p))) so deleted paths are skipped, and pass the batch after `--` or format per file with `--stdin-filepath` so argv are not glob patterns; extend the fixture with one deleted .ts and one `[weird].ts` asserting the remaining files still get formatted."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/states.ts:29-39 (gateNameFor)",
    "problem": "The gate labels change from `allDesignReviewsApproved`/`allEditorialReviewsApproved`/`allCodeReviewsApproved` to `design-review`/`editorial-review`/`code-review`, but those strings are persisted values in `.pipeline.json`'s gateLog (`gate: z.string()`, state-store.ts:99), printed by `pipeline status` (runner.ts:1974-1976) and the escalation report (report.ts:279), and docs/adrs/001-code-driven-agent-pipeline.md:190 still documents the old names as the identity of the aggregated-verdict gates. The matrix row \"Checkpoints written before this change — yes — No checkpoint schema change\" and Rollout step 2 \"No migrations\" do not cover a value-vocabulary change, so a run resuming on this code shows both vocabularies in one gate log and anything keyed on the documented name stops matching. Nothing in the change needed the rename: the entry's stage context and reason already name the pass.",
    "fix": "Revert gateNameFor to the ADR-001 names, or keep the labels and record them: update the ADR-001 gate table / docs/workflows.md and add a Rollout line saying gate-log labels change and pre-existing entries keep the old names."
  },
  {
    "severity": "minor",
    "location": "tools/orchestration/src/pipeline/state-store.ts:156-173 (saveState)",
    "problem": "The `state.updatedAt = Date.now()` line is deleted with no replacement writer: `updatedAt` is now set exactly once, at state creation (runner.ts:435-436), so every checkpoint written after this change carries `updatedAt === startedAt` for the whole run while the field stays required (types.ts:224, state-store.ts:107) and `version` stays 1. The stated rationale is correct for the drained writer — a queued snapshot must not masquerade as newer — but the fix drops the timestamp rather than moving it, an unannounced semantic change to a persisted operator-facing artifact under a \"no checkpoint schema change\" claim. It also leaves the new ordering test relying on hand-set values (pipeline-checkpoint-writer.test.ts:52-84) instead of the writer recording snapshot time.",
    "fix": "Stamp at snapshot time: set `snapshot.updatedAt = Date.now()` inside `createStateWriter.enqueue` before queueing, and keep (or restore) the stamp in `saveState` for the sequential non-pass path, so both write paths mean \"when this snapshot was taken\"; then assert ordering in the writer test without hand-setting updatedAt in the fixture."
  },
  {
    "severity": "nit",
    "location": "tools/orchestration/src/pipeline/runner.ts:290-296",
    "problem": "The package-prefix matcher is built as `prefix = rel === '' ? '' : `${rel}/`` from `relative(repoDir, pkg)` and compared with `p.startsWith(prefix)` against git porcelain paths, which are always `/`-separated. On a platform where `relative()` yields `\\`, the prefix never matches and formatTouchedFiles becomes a silent no-op — acceptable under the Linux-only matrix, but another OS assumption added against a matrix row that says \"no OS-specific code added\".",
    "fix": "Normalize once: `const rel = relative(repoDir, pkg).split(sep).join('/')`, or record the POSIX-only assumption in the matrix row's Notes so the formatter's non-portability is explicit."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/003-parallel-inference-lane/reviews/code/compatibility-r1c1.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"nit","location":"docs/adrs/001-code-driven-agent-pipeline.md:121,190,197","problem":"Residual of my r1c0 finding 4: the gate-label rename stands (states.ts:34-38) and docs/workflows.md:116-119 now records the new vocabulary and the old/new coexistence rule, verified sound in code (gate names are write-and-display only; gate: z.string() keeps old entries loading). But the document workflows.md:3 calls normative still names the aggregated-verdict gates allDesignReviewsApproved/allCodeReviewsApproved in its gate table with no pointer to the shipped labels, so an operator keyed on ADR-001 alone targets a name this code no longer writes.","fix":"Add an amendment note to ADR-001's gate table (in the style ADR-003 uses to amend ADR-001) recording that since ADR-003 the runner logs these gates under `<phase>-review`, or state that ADR-001's table is the historical record and docs/workflows.md is the live label authority."}]}
```
