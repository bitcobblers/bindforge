# Code review — correctness (dissent pass c1, round 2)

Scope: `git diff 6a5c4548..HEAD -- ':(exclude)docs/designs'`, read against the
full current `02-proposal.md`, `03-build-notes.md`, the ADR, and the runner's
gate record. This pass checks my round-2 findings first; anything else is
reported only where the changed code makes it true.

## Verdict: approve

No open blocker or major. All five of my round-2 findings are closed in this
diff, each declared in `03-build-notes.md` (deviations 10–14), each pinned by a
test. One minor stands on the build notes' own accuracy.

## My round-2 findings — closure check

1. **major — `sessionTokens` drops the `cache` group** — CLOSED.
   `prompts.ts:155–182` now flattens one nesting level into dotted keys
   (`addUsage` + inner loop, non-numeric and deeper nesting still skipped, so
   an unexpected shape can add a key but never lose one) — exactly the fix
   asked for. The fixture the finding demanded exists:
   `transcript-tokens.test.ts:53–71` uses the real `TokenUsageInfo` shape and
   pins `cache.read`/`cache.write` in the totals, and the writeTranscript test
   (`:97–113`) pins the flattened terms reaching the printed line. A13 honored:
   the walk is still generic, no field names enumerated in the summing code.
   Declared as deviation 12.

2. **minor — green-leg `test result:` match swallows a flattened tail** —
   CLOSED. `runner.ts` `greenSummary` takes the *last* `test result:`
   occurrence, ends the clause at cargo's ` finished in ` terminator, caps a
   window at 160 chars — the prescribed shape, declared in deviation 10 and
   pinned by `gate-evidence-render.test.ts` (chatter between two summaries is
   not emitted; the leg costs a line, not a tail). I re-ran that suite: green.

3. **minor — research counts fold into the `proposal:` aggregate** — CLOSED.
   The fix is the option the finding named: `StageResult.docName`
   (types.ts, additive-optional, matching `StateSchema.results` entry — A6
   kept), recorded at the meter point, and `proposalGrowth` filters
   `r.docName !== RESEARCH_DOC` (report.ts:184–187), so the operator line sums
   and counts only the document it labels; absence reads as the design doc, so
   golden checkpoints are unchanged. The runner's stage line now names the
   metered file. Declared as deviation 11; `proposal-size.test.ts` green.

4. **nit — elision marker overstates the count** — CLOSED. `gates.ts`
   `boundedTail` now solves the pair (`elided = chars.length − keep`,
   `keep = MAX − note.length`, `note.length = OVERHEAD + digits(elided)`) by
   fixed-point substitution. I verified the arithmetic: the iteration is
   monotone and settles at the unique fixed point within the 8-step guard for
   any real input, and at that point `note.length + keep = MAX_GATE_TAIL_CHARS`
   exactly, with `elided` equal to the characters actually dropped. Deviation
   13's "budget holds exactly" claim is true. (Cosmetic only: the code comment
   "the substitution is a contraction" reads the wrong direction — the seeded
   value is below the fixed point and the sequence increases to it. Not filed.)

5. **nit — design-review prompt kept "never browse to verify" unscoped** —
   CLOSED in the prompt. `design-review.md:10–14` now scopes the kept sentence
   to what it prohibits ("walking a directory to locate a path, or to check
   that a named path exists") and plainly states the bar is on finding paths,
   not on running checks — the scoping the finding asked for. Deviation 14.
   The build-notes half of that closure is inaccurate — filed below as new
   finding N1.

## New findings (changed code only)

### N1 — minor — 03-build-notes.md deviation 14 vs tools/orchestration/tests/pipeline/prompts.test.ts:233–237

**Problem:** deviation 14 states "It is still kept — `prompts.test.ts` pins
the wording" — but this diff *removed* the `expect(text).toContain('never
browse to verify one')` assertion; the test now says the opposite in its own
comment ("The pre-016 phrase is therefore not pinned verbatim") and pins only
the listing/globbing/grepping sentence and `Do not open review reports or
transcripts`. Checklist #5: a build-notes claim checked against the code and
found false. The code is right; the note sentence is stale.

**Fix:** correct deviation 14's sentence to name what is actually pinned (the
kept prohibition via `Do not open review reports or transcripts` and the
listing/globbing/grepping sentence), or add a `toContain` on the scoped browse
sentence so the pin claim becomes true again.

## Notes on the other round-2 additions (declared, faithful, not filed)

- Deviations 8–10 (checkpoint in the run-record fence on all seventeen
  shell-holding agents + `STATE_FILE_SEGMENT` in the audit; the line-selective
  citation rule applied at record write with repo-relative `cwd`; `$ `/`| `
  line ownership, fence neutralization, reserved closing fence) extend the
  Decisions on accepted security-r2 findings, are declared in the build notes,
  and their premises check out against the repo: `.gitignore` carries only
  `.pipeline.lock`, and the runner's own comments confirm `git add -A` sweeps
  `.pipeline.json` — so "the checkpoint stays local" was indeed the false
  premise the proposal's Risks row rested on. The audit segment is built from
  `STATE_FILE` (no drift possible with the writer) and fails closed; the
  permission/audit seam on a command naming own report *and* the checkpoint is
  closed by the audit, as designed.
- `citableCwd`/`citableTail`/`defuseFences` behaved correctly against the
  shapes the dirty-gate integration fixture plants; I re-ran the touched
  suites (render, tokens, agent-shell, independence, prompts, integration,
  proposal-size): 118 tests, all green, matching the runner's green `npm run
  check` gate record.
- Tiny capture-side wrinkle, not filed: `citableTail` can let a *stored* tail
  exceed 2,000 chars by the marker's own length (the 2,000 bound is enforced
  at capture, the marker added after). The checkpoint's real bound is the 8
  MiB loader cap and the deviation declares the marker; no behavior claim
  breaks. The save-state tmp name (`.pipeline.json.<hex>.tmp`) also falls
  outside the audit's segment regex (`.` is not a boundary char), though the
  shell permission rule `*pipeline.json*` covers it; the tmp file is never
  committed.

## FYI (no severity)

`greenSummary` takes the last `test result:` clause — which is what my round-2
fix prescribed and deviation 10 declares — but for `cargo test --workspace`
the last summary is the final doc-tests target's `ok. 0 passed` line (as the
runner's own gate record shows), so the one citable line under that leg will
often be the least informative summary; the first occurrence, or the largest
`N passed`, would serve Test strategy #10's "citable line for a behavior
claim" better if the panel ever wants to tune it.
