# Code review — correctness (r2, pass c0)

Seat: correctness. Inputs: `git diff 22ce12bf…HEAD -- ':(exclude)docs/designs'`,
`02-proposal.md`, `docs/adrs/016-harness-evidence-and-read-rights.md`,
`03-build-notes.md`, and the read-only checks I ran myself (no gate record was
supplied for this run).

Checklist verdict per item:

1. **Components and interfaces exist and match the proposal's names** —
   `checkGate`/`testsGreen` (gates.ts:300–478), `GateLeg`/`GateEvidence`/
   `PipelineState.gateEvidence` (types.ts:149–167, 302), the optional zod schema
   (state-store.ts:45–55, 109), the one gate site before `checkpoint()`
   (runner.ts:2865–2884), `boundedGateEvidence`/`gateEvidenceText`/
   `MAX_EVIDENCE_BLOCK_CHARS` (runner.ts:3036–3115), `documentOutgrewFlow`
   (gates.ts:146–165), `documentSize`/`docFileFor` (runner.ts:193–227),
   `docMaxLines` + `validateDocMaxLines` (stages.ts:79–86, flows.ts:77,
   260–268), `sessionTokens` and the `- tokens:` line (prompts.ts:115,
   136–157, 188), the own-report identity in `auditSessionIndependence`
   (independence.ts:160–252, 262–298), the `gateEvidence` var in
   `stageVars`/`reviewVars` (runner.ts:2983, 3141) and the four templates, the
   run-record block on all 17 shell-holding agents and the per-seat re-open on
   the 10 review seats. All present and wired.
2. **Assumption register honored** — A1–A12, A14–A24 hold in the code. A13
   (`TokenUsageInfo`'s field names, marked *unverified*) is where the code went
   wrong: see finding 1. A6 (additive-optional checkpoint), A7 (bounding at
   write and again at render), A16/A17 (matcher grammar, deny-then-reallow),
   A18 (tail kept from the end), A21/A23 (coarse shell deny plus the per-seat
   re-open) are all visible in the diff.
3. **Deviations declared** — build-notes 1–6 all check out against the code:
   `TEMPLATE_DEFAULTS.gateEvidence` (prompts.ts:53–56), the tail elision marker
   (gates.ts:332–346), the CI node pin, which the gate contract genuinely
   requires since it reads both files (`crates/core/tests/gate_contract.rs`),
   the reason wording (`gates.ts:165` vs the proposal's example), the unread
   `cfg` (runner.ts:3073–3077), the named `GateRun`. Deviation 7's scratch file
   is gone from the tree. Two undeclared gaps: findings 1 and 3.
4. **Scope creep** — nothing outside Decisions 1–4 plus Rollout 1–3; the
   `fix-tests.md` "Gate posture" sentence (the test engineer does own
   `docs/designs/**`, gates.ts:556–558) and the two extra `docs/workflows.md`
   lines sit inside the change's own documentation surface.
5. **Build-notes claims true** — checked against the code and against a live
   run of the checks: `npm run check` at HEAD is green
   (`Test Files 51 passed (51) / Tests 557 passed | 2 skipped (559)`), which
   post-dates the note's red leg (see FYI).
6. **Error paths fail closed** — spawn failure → `exit: null` with the failure
   spelled in the tail (gates.ts:356–364, 395–401, 403–415); no
   `ownReportPath` → the shell branch denies (independence.ts:247); a
   prefix-less `reviews/…` token fails closed (independence.ts:226–230); a
   directory, glob, grep path or listing under `reviews/` is a violation;
   transcripts are never exempted (independence.ts:272, 288); an absent cap is
   the unchanged measure-and-warn posture (gates.ts:158–160); over-cap is a
   normal stage failure whose reason reaches `gateLog` through `stageFailure`
   (runner.ts:2097, 2481) and the retry's `failureNote` (runner.ts:1360–1361).
7. **Previous round's findings** — none held by this seat.

## Findings

1. **major** — `tools/orchestration/src/pipeline/prompts.ts:136–157`
   (`sessionTokens`; the skip is at :148). The sum walks only the top-level keys
   of a message's `tokens` and skips anything non-numeric. The pinned client
   types that field as
   `TokenUsageInfo = { input; output; reasoning; cache: { read; write } }`
   (`node_modules/@opencode/client/dist/promise/generated/types.d.ts:64–70`,
   with `SessionMessageAssistant.tokens?: TokenUsageInfo`), so every
   transcript's `- tokens:` line silently drops `cache.read`/`cache.write` —
   the largest term for a stage session whose prefix is cached, which is exactly
   the quantity Decision 4 exists to make checkable (ADR-016: "most turns
   re-prefill past the resident prefix"; Rollout 4: "the ADR's prompt-size
   claims become numbers the operator can re-read"). A13 is marked *unverified*,
   and the code resolved that open assumption by omission; nothing notices,
   because the fixture in `tests/pipeline/transcript-tokens.test.ts:27–38` uses
   a flat shape. Fix: recurse one level — flatten nested numeric objects to
   dotted keys (`cache.read`, `cache.write`) while still skipping non-numeric
   values — and pin a fixture in the real `TokenUsageInfo` shape so a group can
   never be dropped silently again.
2. **minor** — `tools/orchestration/src/pipeline/runner.ts:3102`. §2 renders
   green legs "as one-liners (`command exit 0` plus any tail line starting
   `test result:`)", but the tail it reads has already been flattened:
   `boundedGateEvidence` passes every field through `printable`
   (independence.ts:418 replaces all of `\p{Cc}`, newlines included) at write,
   and `gateEvidenceText` re-applies it at render, so no `\n` survives and
   `/test result:[^\n]*/` matches from the *first* summary to the end of a
   single-line tail — up to `MAX_GATE_TAIL_CHARS` of joined per-target
   summaries for `cargo test --workspace`. A green leg stops being a one-liner
   and can crowd other legs' lines out under `MAX_EVIDENCE_BLOCK_CHARS`. Fix:
   bound what is emitted — take the last `test result:` occurrence and slice a
   fixed window (or split the flattened tail on ` finished in ` and emit the
   last summary) so a green leg stays one citable line.
3. **minor** — `tools/orchestration/src/pipeline/runner.ts:1723–1726` with
   `flows.ts:71` (`docStages` gains `research`) and the unchanged `report.ts:178`
   (`proposalGrowth` filters on `typeof r.docLines === 'number'`). Putting
   `research` in the metered set makes the research document's counts flow into
   the existing design-scope aggregate, so the operator-facing line
   `proposal:  N lines (+A/−B across K design stage(s))`
   (`report.ts:208–221`) now sums `01-research.md` growth into a total labelled
   `proposal:` and counts research as one of the "design stages". Neither the
   proposal nor `03-build-notes.md` names it; the Non-goals only rule out *new*
   `report.ts` display, not this shift in an existing one. Fix: keep the growth
   aggregate on the `02-proposal.md` stages (name the metered document on
   `StageResult`, or filter `proposalGrowth` by stage), or declare the mixed
   total in the build notes as intended.
4. **nit** — `tools/orchestration/src/pipeline/gates.ts:340–341`. The elision
   marker's count is computed from a fixed 64-character reservation
   (`chars.length - (MAX_GATE_TAIL_CHARS - 64)`) while the kept span is
   `MAX_GATE_TAIL_CHARS - note.length`, so the marker overstates the elided
   characters by `64 - note.length` (≈ 20 for a real note). The budget itself
   holds (`note.length + keep ≤ 2 000`). Fix: size `keep` first, build the note
   from `chars.length - keep`, then trim the kept span to
   `MAX_GATE_TAIL_CHARS - note.length`.
5. **nit** — `tools/orchestration/prompts/design-review.md:10`. §3 asks for the
   "never browse to verify" wording to be *replaced*; the sentence is kept
   verbatim with the allowance added after it ("What is open to you instead…").
   The intent — prompt, permission layer and audit saying one thing — is met,
   and the build notes do record the keeping, but they file it under "no
   deviation". Fix: scope the kept sentence to what it actually prohibits,
   locating a named path by browsing, so it cannot read as a bar on running the
   checks the next sentence grants.

## FYI

The build note's "Gate posture" table still records the npm leg at exit 1 with
`testsNeedUpdate: true`; that fixture was fixed in the test-update commit and
`npm run check` is green at HEAD (51 files / 557 tests passed, run in this
pass), so the committed carrier of the gate record now reads staler than the
branch it describes — worth one line from the coder or doc-sync, and `docs/`
sits outside the diff this panel was handed.
