# security review r2c1 (dissent pass)

verdict: approve

Scope: `git diff` of 02-proposal.md blob 44c52fa → 82b346f (the Read-only git
sub-paragraph, Risks row 1, Test strategy #13, the Deferred sandbox bullet, the
failure-classes log line, and the r2/r3 Read-only git log entry), then the full
current document. Inputs per caller: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log, and my r2 findings.

## Findings

None open.

## Closure check (my r2 findings)

- [major][closed] Read-only git / Risks row 1 / #13 — planted repo-local
  `.git/config` executing at runner privilege via the escalation's own
  `ESCALATION.md` commit and later `pipeline resume`. The revision takes my fix
  in full: for this class `finalizeEscalate` "records the escalation in the run
  state instead of committing `ESCALATION.md`, so the escalation path runs no
  git over the plant"; the env paragraph no longer overclaims — "That reaches
  only config sources outside the repo (A6) and those two keys; a planted
  repo-local `.git/config` survives every pin and is read by every runner git
  call — hence the halt"; and plant removal is stated as "the operator's
  remediation step before the worktree sees another git call." The old false
  inertness claims ("the plant is inert before any git there", "pinned inert
  before the escalation's own commit") are gone from both the Chosen design and
  row 1. #13's hostile config is upgraded from `core.fsmonitor` to
  `alias.status='!…'` and `filter.evil.clean='!…'` plus a planted
  `.gitattributes`, both marker-touching — the exact unplanted exec-capable
  keys I named, now pinned to fail if any git runs over the plant (marker
  absent, "the escalation makes no commit"). The Revision log's r2/r3 entry
  records the deliberate change. Sound as written; the only post-halt exposure
  is an operator who resumes without removing the plant, which the document
  discloses as the named remediation step rather than claiming the design
  suppresses it — the r1 posture my fix asked for.
- [minor][closed] Audit-blind child-session `.git` route for non-item-5 seats.
  The text now says "the `subagent` deny below covers the item-5 seats' route,
  the review seats' same route is the Deferred sandbox's," and the Deferred
  sandbox bullet gains "and the review seats' child-session `.git` route the
  `subagent` deny does not reach." Risks row 1 matches ("the review seats'
  child route Deferred"). The deferral does not contradict ADR-005: the ADR's
  Decision itself parks this planted-config family as "a separable,
  non-terminating problem," so handing this uncovered route to the sandbox
  follow-up is consistent with the source document, not a re-openable
  contradiction. The item-5 seats' own-session route keeps the runner-side
  audit (the enforced control); the advisory-only cover on their child route is
  the disclosure I accepted in r2.

## Regression check on changed text

- The halt-before-finalize sequencing means a plant hit never reaches the
  sweep (`git status`) or `commitStage` (`git add -A`), so #13's chosen hostile
  keys are the right tripwires; skipping the commit does not contradict ADR-005
  (the ADR says nothing about escalation commits).
- Dropping "a planted bare repo is inert anyway under the pinned
  `cwd: repoDir`" removes a rationale but keeps no false claim; the bare-repo
  behavior stays pinned as a #13 expectation.
- Env honesty ("survives every pin") plus the halt is now internally
  consistent with Risks row 1's "no escalation commit, so no runner git
  touches the plant" and the operator-remediation clause.
- Security checklist: trust boundary (stage output untrusted → halt +
  escalate, fail closed, no attempt 2), least privilege (runner git privilege
  no longer spent over a known-infected worktree in the automation path), no
  new dependency named in the changed text — all satisfied.

## FYI (no severity)

- ADR-005's Decision defers item 5 (read-only git in stage shells) "to a
  follow-up ADR," while this proposal's Context says "this proposal also lands
  item 5" with controls and a ship gate — scope/consistency question for the
  correctness/compatibility seats; unchanged text, outside my specialty, and I
  did not flag it in r2.
