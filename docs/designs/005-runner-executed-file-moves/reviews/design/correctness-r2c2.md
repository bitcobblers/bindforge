# correctness review r2c2

verdict: request-changes

Scope: the blob diff 44c52fa → 82b346f of `02-proposal.md` (Read-only git item 5
paragraph, Risks row 1, Test strategy #13, one Deferred bullet, Revision log)
plus the full current document. Checklist coverage: items 1–2 (assumption
register, research URLs) were verified in r1/r2 and the changed text leans only
on A6, already checked against git(1); items 3–7 evaluated below against the
changed sections only.

## Findings

- [major][new] Chosen design — Read-only git (item 5) — "Nothing clears it
  in-process; removing it is the operator's remediation step before the worktree
  sees another git call" — the closure of my r2 over-breadth finding replaced
  "plant pinned inert" with the halt, but nothing in the design makes the halt
  unresumable, and this revision deleted the sentence that admitted that
  ("nothing clears it in-process, so a `pipeline resume` (runner.ts:1376) runs
  over it until the operator removes it"). Test strategy #6 still files
  `git-plant-detected` with `moves-rollback-failed` as an operator escalation,
  and the Apply section keeps `finalizeEscalate` as the mechanism, so an operator
  `pipeline resume` over an unresolved plant runs the next finalize's sweep and
  `git add -A` (runner.ts:298) over a config the same paragraph now says "survives
  every pin and is read by every runner git call" — executing the planted
  `filter.evil.clean` at runner privilege. The `.gitattributes` that selects that
  filter is not a `.git`-segment path, so no later audit re-detects it either. The
  Risks row states the same unenforced ordering as a consequence ("no escalation
  commit, so no runner git touches the plant"), and #13's containment assertion
  stops at the escalation, so the ship gate the Goals row makes item 5 conditional
  on never exercises the resume path. Fix: name the enforcement in one sentence —
  "the escalation leaves the run unresolved: `pipeline resume` runs no stage until
  the operator records the plant removed" — and add a #13 case asserting the
  marker stays absent across a resume over an unresolved plant; or restore the
  deleted hazard sentence and mark resume-over-plant as the accepted residual in
  the same row. Either way narrow the Risks row to "so the escalation path itself
  runs no git over the plant".

- [minor][re-raised] Chosen design — Read-only git (item 5) — "`git()`'s env
  (106-112) is the shared env for every runner git call — `gitOk` (143-145) and
  the raw `merge-base` spawn (1417-1420) included" (and Risks row 1, "the shared
  env over every runner git call reads no config outside the repo") — my r2
  enumeration finding is untouched by this revision, and the changed text
  re-asserts the universal claim. `writeEditorialDiff` (called from runner.ts:1507)
  spawns `git diff --no-index` with only `cwd: repoDir` at runner.ts:1933-1937,
  outside `git()`/`gitOk`, so it inherits the operator environment and does read
  ambient system/global config at runner privilege — the Risks row's sentence is
  false as written. It also makes the new sentence "a planted repo-local
  `.git/config` survives every pin and is read by every runner git call" inexact
  for that same site, which discovers no repository. #13 still enumerates only
  `gitOk` and `merge-base`. Fix: state the rule as "every `spawnSync('git')` site
  goes through the one env helper", name `writeEditorialDiff`'s call in the
  paragraph, and add it to #13's list.

- [nit][re-raised] Chosen design — Read-only git (item 5) — closing line "a
  planted repo-local `.git/config` survives every pin and is read by every runner
  git call — hence the halt" — my r2 nit about the dropped merge-blocking clause
  is still open, and the revision strengthens the gate's load: Goals still
  promises item 5 "conditional on the controls and ship gate named under
  Read-only git" and #13 still labels itself "Item 5's ship gate asserts
  containment, not just detection", but the section that names the gate states
  runtime behavior only, never the consequence of failing it. Fix: restore the
  one sentence at the end of the paragraph — "If Test strategy #13 lets a vector
  land a plant, or a runner git call run over one, the change does not merge."

- [nit][new] Revision log — "rollback failures escalate; `.git` audit hits halt"
  and "r2/r3: Read-only git — a `.git`-plant hit halts the run with no attempt 2
  and no escalation commit" — `halt` is a new name the body never defines or uses:
  the audit paragraph still routes the hit through `finalizeEscalate` "for this
  class", and #6 still says a `'git-plant-detected: …'` audit hit "escalates to
  the operator and starts no attempt 2". One class now carries two names, and the
  log's "halts the run" reads as terminal while #6 reads as an ordinary operator
  escalation — exactly the ambiguity finding 1 turns on. Fix: use one term
  everywhere ("escalates with no attempt 2 and no commit"), or define `halt` once
  in the audit paragraph and use it in #6 and Risks row 1.

## FYI (no severity)

- Test strategy #13's `alias.status` vector fires only if some runner git call
  actually runs `git status`; the document never says which command the sweep's
  dirty-path listing (A4) issues, so that half of the fixture may be vacuous —
  test-strategy seat.
- ADR-005's Decision defers item 5 itself "to a follow-up ADR" while this
  proposal ships it and Rollout 1 flips ADR-005 to `accepted`; unchanged by this
  diff, and outside this pass's scope — scope/consistency seat.
