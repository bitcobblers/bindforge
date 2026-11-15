# Security review r2

verdict: approve

Pass type: editorial confirmation (round 2), scope = `reviews/design/editorial-r2.diff`
only, plus `02-proposal.md`, `docs/adrs/021-decision-unit-ledger.md` and the Revision
log. Previous security findings: none, so nothing to re-check as dissent. A finding is
reportable here only if an editorial edit broke something security-approved or made a
security-relevant sentence false. All five hunks are language-only: none adds a data
source, a path, a privilege, a dependency, or a parser, and none weakens a check.

## Diff hunks, security read

1. Goals — "checked both directions" → "checked in both directions". Grammar; still
   matches 021-R2's "checks the join in both directions". No security content.
2. ¶1 — "parses no path it did not list" → "parses no path it does not list". This is
   the only edit touching a security sentence, and it does not weaken it: the present
   tense is the stronger reading (the invariant holds at the moment of parsing, not only
   retrospectively). Together with "never writes" and the `join`/`basename` construction
   it keeps checklist item 3 and 6 intact.
3. ¶8 template note — `never-re-specifies` → the "never re-specifies" test (021-L1).
   Names the same law; ADR-021 L1 calls it "The test", so the wording now matches the
   source. No check, target or authority changed.
4. ¶8 spec bullet — A19's correction reworded ("the pairing that `013-L2` and `013-R4`
   state", "the correction is recorded as a sixth `Backfill findings` entry in 013").
   Same claim, same consequence clause ("recorded, not silently absorbed"), so drift is
   still recorded rather than absorbed. Fixing spec.md:14 removes a shipped
   contradiction instead of adding one.
5. Risks row — em-dashes to parentheses: "Both are editorial-only (no normative sentence
   changes)". The mitigation's substance is unchanged, and it remains true of the
   changes it covers: `Owner:`, `→` and handle re-wrapping are ledger metadata, not
   normative sentences.

## Checklist disposition

1. Trust boundaries — no change. Inputs remain the repo-local corpus (`docs/adrs/*.md`,
   `schemas/**/*.md`, A23); no network or user-supplied source enters the diff's text.
2. Parsing untrusted data — no change: line-based anchored regexes, no markdown parser,
   no code evaluation, no decompression; bounded file count (A23); findings output bounded
   by `MAX_LEDGER_FINDINGS`.
3. Path handling — satisfied, and unchanged in substance by hunk 2: fixed roots via
   `node:path` `join`, `basename` for `NNN`, no user-controlled component, no shell
   interpolation, "parses no path it does not list".
4. Secrets and PII — no change; reason lines print `file:line code detail` only.
5. Dependencies — not applicable to this diff (it adds none). Verified the named set
   against `tools/orchestration/package.json:45–49`: `@opencode/client`, `yargs`, `zod`
   only, no markdown parser (A14 holds); dev legs `prettier`, `eslint`, `vitest`,
   `typescript`, `tsx`. GitHub Advisory Database queries for the exact runtime package
   names returned no entries. A transitive `npm audit`-depth sweep is a
   whole-document activity and was not run — recorded as missing evidence, not a
   finding, because no dependency enters through this diff and no security property of
   the design depends on it.
6. Privileges — unchanged: readers never write; the gate's only effect is deny/escalate.
7. Fail closed — unchanged: entry hook refuses before the first stage, finalize escalates
   instead of flipping status; no hunk alters a hook point or an error code.

Deferred section: re-read against ADR-021's Decision and 021-R2's four named classes.
No deferred item contradicts the ADR; pointer-target resolution in particular is
consistent with 021-L2, which makes only the handle direction mechanical, and the Risks
table states the residual invisibility. Not re-opened.

## Findings

None. No blocker, major, minor or nit arising from this diff.

## FYI (no severity)

- ¶8 `docs/adrs/021-decision-unit-ledger.md` / `schemas/manifests/v1/spec.md` bullets —
  "`013-R4` gains `→ ADR-019`" against A19's "`013-L2` and `013-R4` assign ADR-019 and
  ADR-017" — read distributively, R4 would point at ADR-017, so the sentence the
  editorial pass rewrote still leaves the unit-to-number pairing ambiguous; this is a
  correctness/consistency question, not a security one (pointer targets are names, and
  the Risks table accepts that class).
