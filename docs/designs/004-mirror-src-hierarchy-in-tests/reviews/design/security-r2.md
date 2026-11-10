# security review r2

Scope: `reviews/design/editorial-r2.diff` (editorial round 2 confirmation pass).
Inputs used: the diff, `02-proposal.md` as it now stands, the Revision log inside
it, and `docs/adrs/004-mirror-src-hierarchy-in-tests.md`. Prior security-seat
findings for this phase: none.

verdict: approve

## Checklist walk (against the diff's five hunks)

1. **Trust boundaries** — Chosen design ¶3's statement ("nothing here parses
   untrusted or network input — it moves only trusted, repo-local files, and
   their only security-relevant consumers are that gate and that grant,
   unchanged in scope (A7, A11, A19)") is untouched by the diff. Intact.
2. **Parsing untrusted data** — no parser, deserializer or decompressor is
   introduced or touched; the diff is punctuation plus one cross-reference
   correction. Nothing to evaluate beyond ¶3's already-stated boundary.
3. **Path handling** — hunk 1 (`62 `from '../src/...'` imports to
   `from '../../src/...'``) is the only path-text edit. It closes two stray
   backticks so both literals read as complete strings; it changes no count
   (62 matches A8's "62 in moved files" of 65) and adds no new path form. The
   rewritten `../` segments remain static, repo-local, author-controlled source
   text — no user- or network-controlled path enters the flow, so checklist
   item 3's canonicalize/reject-`..` clause stays not-applicable rather than
   unmet.
4. **Secrets / PII** — no new logging, storage or egress in the diff.
5. **Dependencies** — the diff names no dependency and changes no version pin;
   the pinned majors in the Compatibility matrix (`vitest ^5.0.5`,
   `TypeScript ^6.0.3`, `@trivago/prettier-plugin-sort-imports` per A10) are
   unchanged from what the design already carried. No new advisory surface.
6. **Least privilege** — the test-engineer `**/tests/**` grant (A11) and the
   `stagePathAllowed` denial for `implement`/`fix` (Test strategy #4) are not
   in any hunk's changed text. Intact.
7. **Fail closed** — Test strategy #2's file-listing check ("finds zero files
   named `pipeline-*.test.ts`") and Risk 1's mitigation are untouched. The
   r1 security closures (Test strategy #2 and #4, Chosen design ¶3, per the
   Revision log) all still read as approved.

Hunk 3 ("Risks 4" → "Risk 4") was checked because it is a security-relevant
cross-reference: the Rollout 1 sentence designates the Rollout-1 commit as the
baseline that Risk 4's mitigation consumes ("compares the vitest reporter's
file/test counts to the Rollout-1 baseline"). Singular "Risk 4" now points at
exactly that row; the plural it replaces pointed at no defined target. The edit
strengthens, not weakens, the reference. Hunk 4 ("from the merge") tracks ADR
Consequences' "From merge, the pipeline's test-authoring and test-update stages
place new tests in the mirrored layout" — no change in obligation.

Hunk 5 adds only the `Editor notes:` line flagging Chosen design ¶2's amendment
clause as ambiguous. It adds no behavior, no permission and no path form; the
clause it flags is unchanged text in the diff's context lines.

## Findings

None. No blocker or major finding is open against the diff: it changes no trust
classification, no path-validation behavior, no privilege surface and no
dependency set, and every r1 security closure survives it verbatim.

## FYI (no severity)

- Chosen design ¶2 — the clause the new Editor note flags, "a relative path
  reference — code or comment — naming a location this move passes takes its
  new depth", is the wording Rollout 2 inserts into ADR-004 Decision bullet 2,
  and Test strategy #1 / Risk 3 use ¶2's "allowed set" as the oracle that fails
  a hidden content change. If the author's clarification widens that clause
  beyond A8's closed enumeration, the safe way to write it is to name the set
  (62 imports in moved files, the 16 `new URL` paths, the one A14 comment) and
  exclude A8's 4 inert `'../escape'` fixture strings and the 2 non-import sites
  at `cli.test.ts:29-30`; that boundary is the editorial/correctness seat's call
  to land, not a finding here.
- Deferred items (fixture path strings, ADR Context's "14 modules", historical
  run artifacts) were re-checked only against the ADR's Decision and stated
  requirements; none contradicts it, so none is re-opened.
