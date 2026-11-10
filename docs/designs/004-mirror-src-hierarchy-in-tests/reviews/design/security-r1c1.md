# security review r1c1 (dissent pass)

verdict: approve

Scope: `git diff 1f3a83c 5996360` of `02-proposal.md` (the sections changed since
r1), then the full current document, ADR-004, and the Revision log. Inputs kept to
those; no other review report or transcript opened.

## Closure check on my r1 findings

- **[major] Test strategy #4 — nested classification asserted, not inferred — CLOSED.**
  #4 now reads "the existing `stagePathAllowed` block in the gate suite asserts the real
  post-move path `tools/orchestration/tests/pipeline/gates.test.ts` — allowed for
  `test-authoring`/`test-update`, denied to `implement`/`fix`", which is the real-nested-path
  assertion asked for, and the round-1 gap in the assumption register is filled by A19
  ("static enumeration — grep for `tests`/`test.ts`/`TEST_PATH_RE`/`stagePathAllowed` over
  tools/orchestration/src (only the cited sites)"; "no pattern in `src/` keys on the flat
  `pipeline-*` filename shape"). The demonstrative claim that the moved gate tests' green run
  proved nested handling is gone, so the Deferred fixture strings no longer carry the claim.
  Residual, nit-level: see finding 2.
- **[minor] Chosen design ¶3 — missing trust-boundary statement — CLOSED.** ¶3 now states
  "Trust boundary: nothing here parses untrusted or network input — it moves only trusted,
  repo-local files, and their only security-relevant consumers are that gate and that grant,
  unchanged in scope (A7, A11, A19)." Checklist items 1, 2 and 4 are now evaluable and pass.
- **[minor] Chosen design ¶2 / Rollout 2 — unspecified rewrite mechanism — CLOSED.** ¶2 is
  now "rewrite only the 65 `from '../src/...` specifiers A8 enumerates … specifier-scoped, keyed
  to that grep pattern, never a content-wide `../src/` substring substitution, which would also
  rewrite the fixture path strings fed to the gate (A14)", and Test strategy #1 bounds the moved
  files' diff to "¶2's allowed set: `../../src/` import lines and its one comment line" — the
  diff assertion asked for, replacing reliance on the manual diff read (Risks row 3 now points
  at #1). The gate-fixture inputs my finding worried about are explicitly out of scope of the edit.
- **[minor] Rollout 1 — human pre-condition with no mechanical check — CLOSED, and the added
  guard is itself defective: finding 1.**

## Findings

- **[minor][new] Test strategy #2 — "fails closed on a half-applied move: a grep over
  `tools/orchestration/tests/` finds zero leftover `pipeline-*.test.ts` paths".** The same
  document guarantees this grep matches. A14 says "other in-test path mentions are inert fixture
  data" and cites `pipeline-independence.test.ts:145` and `pipeline-gates.test.ts:298-303`;
  Deferred keeps them: "Refreshing fixture path strings that mimic the old naming (e.g.
  `pipeline-lane.test.ts` at `pipeline-independence.test.ts:145`) … no follow-up". So `pipeline-*.test.ts`-shaped
  strings persist inside files under that directory after a fully applied move, and a content
  grep for that pattern can never report zero. Under checklist item 7 this is the guard that is
  supposed to fail closed on partial application, and as written it either always fails (the
  rollout's step-3 verification of items 1–6 can never pass) or is "fixed" by loosening the
  pattern until it passes — which is exactly how a half-mirrored tree slips through the only
  mechanical check for it. Fix: replace the content grep with a name-level filesystem check the
  deferred fixture strings cannot satisfy — `find tools/orchestration/tests -name 'pipeline-*.test.ts'`
  prints nothing — or keep the grep only when anchored to the path shape `tests/pipeline-` and
  applied to a file listing rather than file contents.
- **[nit][re-raised] Test strategy #4 — "the existing `stagePathAllowed` block in the gate suite
  asserts the real post-move path".** A19 enumerates two consumers of the single
  `TEST_PATH_RE`: `stagePathAllowed` ("`gates.ts:262`, enforced at `runner.ts:232`") and "the
  revert check (`runner.ts:1031`)". The new assertion exercises the first; the revert check's
  nested-path behavior is still carried only by A7's regex argument. Fix: one more expectation in
  the same block over the same nested path at the revert-check call site, or one A19 clause
  stating the revert check shares that one regex and so inherits the same classification — the
  latter makes the inheritance explicit rather than inferred by the reader.

## Checklist evaluation

1. **Trust boundaries** — pass. ¶3's Trust boundary line classifies everything the change moves
   as trusted repo-local files and names the only security-relevant consumers (the `TEST_PATH_RE`
   gate, A7/A19; the test-engineer `**/tests/**` grant, A11), both unchanged in scope. No shared
   binding file is read by anything this change touches.
2. **Parsing untrusted data** — not applicable, and now explicitly stated as none.
3. **Path handling** — no path-handling code changes; the only path-shaped mutations are the
   specifier-scoped import rewrites (¶2), now bounded to A8's 65 enumerated sites, so no
   `..`-containing fixture input is constructed or reinterpreted by this change.
4. **Secrets / PII** — pass: a rename commit in `tools/orchestration/` moves no secret and no
   network egress is introduced ("nothing here parses untrusted or network input").
5. **Dependencies** — the diff changes no dependency surface; the Compatibility matrix row
   (`Node >= 22.12.0, vitest ^5.0.5, TypeScript ^6.0.3`) is unchanged from r1 and the design adds
   no package, so item 5 has nothing new to check here.
6. **Least privilege** — pass: no grant widens (A11, A19); Rollout 1's new test-only commit edits
   only files under `**/tests/**`, already inside the existing grant.
7. **Fail closed** — the design now asserts a fail-closed check for a partially applied move, but
   the check as specified is unsatisfiable/ambiguous: finding 1.

## FYI (no severity)

- Chosen design ¶ says "one commit in `tools/orchestration/`" while ¶2 and Rollout 2 put the two
  `.opencode/skills/**` convention amendments and the "one-clause amendment to Decision bullet 2"
  in that same commit — all outside that path. Accuracy of the commit-scope statement belongs to
  the completeness/correctness seat; from here it only matters that the ADR edit is disclosed.
- Rollout ordering (test-only commit before the move commit) means the "pre-move test count" /
  "pre-move baseline" in Test strategy #2 and Risks row 4 has to be taken after Rollout 1, or
  "the one added assertion block in item 4" shifts the count that #1 and #2 compare.
- #4's stage names (`test-authoring`/`test-update` vs `implement`/`fix`) encode an expectation
  about the stage list in `gates.ts`; those files are outside my input set, so I took the
  expectation as stated. It fails loudly if wrong, which is the safe direction.
