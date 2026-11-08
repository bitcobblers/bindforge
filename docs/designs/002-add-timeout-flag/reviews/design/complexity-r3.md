# complexity review r3

verdict: approve

Scope: the change set `reviews/design/editorial-r3.diff` applied to
`02-proposal.md` (the proposal file already contains the applied hunks).
Checklist: `references/complexity.md`. Previous complexity round:
`complexity-r2.md`.

## Findings

- [nit] 02-proposal.md:130 (Test strategy #10 — line edited by diff hunk
  `@@ -123`; also A19 at :47) — the fallback route is still written as an
  addition rather than a substitution: "fall back to an optional `clientFactory`
  parameter on `main()` reusing `stubClient()`, and record the spike as the
  basis". Nothing says the seam *replaces* the `vi.mock` route, so an implementer
  whose spike stumbles can ship both a mock-based integration test and a
  production dependency-injection parameter on `main()` with one test-only,
  speculative consumer — the one-implementation abstraction checklist item 6
  warns about. Carried unchanged from complexity-r2's `[nit]`; the diff touched
  this sentence (comma only) and did not close it. Fix: one clause — "if the mock
  proves unworkable, use an optional `clientFactory` parameter on `main()` **in
  place of** the `vi.mock` route (never both), and record the spike result as
  A19's basis".
- [nit] 02-proposal.md:165 (Revision log, r4 entry) — the change set correctly
  deleted the verbatim-duplicated "Deliberately not changed this round"
  paragraph from the r3 entry (:160), closing the first half of complexity-r2's
  `[nit]`, but the same decision list is still restated in full a second time by
  r4 at :165 ("including the full r3 not-changed list — …"). Two near-identical
  ~1 000-character paragraphs in the log cost every downstream reviewer reading
  time for zero added information. Fix: let :165 cross-reference instead of
  restate — "every r2/r3 not-changed decision stands (see the r3 entry), plus
  the yargs `describe` text left as static help copy; register statuses A8/A14/
  A17/A19 stay `unverified`".
- [nit] 02-proposal.md:126 (Test strategy #6 — clause edited by diff hunk
  `@@ -123`) — "pins the behaviour that research line 43 asks the designer to
  fix" reads as in-scope work on the polling loop, which the proposal elsewhere
  names as deliberately unchanged (explicit non-goal at :16/:150/:160/:165;
  "documented rather than 'fixed'", `opencode-runner.ts:259–271`). Research line
  43 asks the designer to *pin down* the bounded sub-poll behaviour, not change
  it. The wording invites an implementer to add a loop change that traces to no
  Goal (checklist item 2). Same wording was raised as a citation nit by
  correctness-r3; the complexity angle is the scope-creep risk. Fix: "pins the
  behaviour that research line 43 asks the designer to pin down".

## Checklist walk-through (basis for the verdict)

1. **Reuse:** the change set introduces no package, module, interface, constant
   or identifier — all seven hunks are prose edits (two `honor→honour`
   spellings, two comma/relative-clause fixes, one grammar fix, one inserted
   "that", one revision-log de-duplication). Re-checked `tools/` for anything
   newly duplicated: `tools/orchestration/src/` still has no validator/resolver/
   error base that the touched text now duplicates (the only numeric-validation
   precedent is `z.number().int().min(0)` in `pipeline/types.ts`; the only error
   class precedent is `StopDriving extends Error` in `pipeline/runner.ts`). The
   reworded :85 bullet still states the single-source rule correctly
   (`DEFAULT_REPLY_TIMEOUT_MS / 1000`, `MAX_REPLY_TIMEOUT_MS / 1000`). Pass.
2. **Goal trace:** no hunk adds a moving part. The touched sentences describe
   parts that trace to Goals 1–5 (resolver bounds → Goal 1; `runPrompt` boundary
   validation → Goal 2; `ReplyTimeoutError` report → Goal 3; Test #6/#10 →
   Goals 2/4). The one part that traces to no stated Goal — the `engines` line
   (:90) — is untouched by this change set (see FYI). Pass.
3. **Smaller design:** the change set is net-negative in size: it removes a
   ~1 000-character duplicated paragraph and completes a truncated sentence,
   and adds no new content. No 30–50 %-smaller variant of the *change set*
   exists — it is already pure deletion plus word substitution. The remaining
   restatement at :165 is the nit above. Pass.
4. **New dependencies:** none. Verified against `tools/orchestration/package.json`
   (47 lines, no `engines` key, deps `@opencode/client`/`yargs`/`zod`,
   `vitest ^5.0.5` at :40) — the change set touches no dependency or script
   content. Pass.
5. **Deferrability:** unchanged; the change set does not alter phasing, and the
   design remains one thin slice (Rollout #1). Pass.
6. **Knobs/abstractions:** no knob or abstraction is introduced or renamed. The
   only one-implementation abstraction in the touched lines remains the
   conditional `clientFactory` seam (nit above). `MAX_REPLY_TIMEOUT_MS` keeps its
   two real consumers (resolver + runner boundary) and the poll interval stays
   hard-coded, so no consumer-less knob appears. Pass with nit.
7. **Value check (stated plainly):** proportionate, and this change set slightly
   reduces cost. Production complexity is untouched at roughly the ~40-line core
   r1/r2 estimated (one exported constant pair, one optional `PromptOptions`
   field, one parameter, one error class, one resolver, one wiring change); the
   editorial pass adds none of it and removes duplicate prose that was slowing
   reviewers. All three open complexity findings are documentation nits, not
   structural over-engineering.

## Prior-round closure

- complexity-r2 `[nit]` (duplicated r3 log paragraph at :160): **closed** by this
  change set — the applied file has exactly one copy, and the tail now reads
  "…records as the fallback-triggering condition".
- complexity-r2 `[nit]` (`clientFactory` fallback not stated as exclusive):
  **open** — re-raised as finding 1.
- complexity-r2 `[minor]` (`engines` line traces to no Goal): **open**, and not
  addressable by a language-only change set — see FYI.
- complexity-r1's two `[minor]`s (error payload; cap labelled as a product
  decision) stay closed in the body.

## FYI (no severity)

- 02-proposal.md:90 (+ Goals, Compatibility matrix :115, Rollout #1/#4) — the
  `"engines": { "node": ">=22.12.0" }` line is still the only moving part with no
  Goal behind it (complexity-r2 minor). The change set is language-only and
  cannot act on it, so it is left for the proposal editor / closure audit rather
  than charged to this diff.
- Prose consistency (editorial specialty's call, noted only because it affects
  reader load): `honouring`/`honours`/`hard-coded` now match the document's
  prevailing en-GB style; no `honor`/`behavior`/`hardcoded` remnants were found,
  so the diff introduced no two-names-for-one-thing drift.
- Citation accuracy in the same hunks — 02-proposal.md:55's "prior-art
  human-facing interfaces are integer seconds" vs the curl man page accepting
  decimals, and the A10 `runner.ts:326` anchor — is correctness's business
  (correctness-r3 already carries them), not this review's.
- The revision log (:141–165) is now the single largest section of the document;
  whether to compress it wholesale is an editing decision, outside the
  complexity checklist.
