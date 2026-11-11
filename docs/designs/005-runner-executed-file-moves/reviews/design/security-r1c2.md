# security review r1 (dissent c2 — sections changed since r1)

verdict: request-changes

Scope: `git diff 64ae876f..89027e8a` of 02-proposal.md, then the full current
document, the ADR, and the Revision log. Inputs per the caller; no other
reviews or transcripts opened.

## Closures (my r1 findings, checked against the r2 text)

- r1-major 1 (global/system config sources): closed. `git()` now sets
  `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL` at an empty file, and
  `-c safe.bareRepository=explicit`; A6 names the extra sources; "closes every
  config sink" is gone; the Deferred bullet that parked the runner env is
  removed; #13 pins hostile `~/.gitconfig` and a nested bare repo.
- r1-major 2 (nested `.git` plant / prefix vs segment): closed.
  `stagePathAllowed` is now a case-folded segment rule (`.git` segment or
  segment ending in `.git`), the agent configs add `**/.git/**` and `**/*.git`,
  the floor is `git ≥ 2.38`, and #13 adds the `src/.git/config` plant and the
  nested bare repo.
- r1-major 3 (flag-only writes through allowed commands): the *detection* half
  is closed (audit command-text matcher + #13's `git log --output=.git/config`
  case; the false "worktree landings land in visible commits" row is gone) —
  but the adopted carrier introduces two new defects, below.
- r1-nit 4 (A9 matching model): closed, and independently re-verified: the
  OpenCode permissions doc's Matching section says whole-value wildcards and
  last-match-wins, and its Policies section describes the hard-deny-after-rules
  A9 now cites. The allowlist paragraph's "appended after their existing rules"
  ordering requirement is the right consequence.
- r1-nit 5 (#4 predicate assertion): closed — #4 now asserts
  `stagePathAllowed` false for `.git`, `.GIT/hooks/pre-commit`,
  `src/.git/config` for every stage.

## Findings

- [major][new] Chosen design — "Read-only git for implement/fix/test-update
  seats (item 5)" — "in the shape of its existing `transcripts` matcher, and a
  hit fails the attempt" — the audit detects the plant but does not contain it,
  and the code path the design leans on re-exposes it. An audit hit fails the
  attempt and nothing removes the planted `.git/config`: the design's own A4
  says the sweep can neither see nor undo `.git/**`. An `independence
  violation:` failure is not `invalid-reply`, so attempt 2 starts a **fresh
  session on the same worktree** (runner.ts:836-838 comment; retryInSameSession
  at 1121-1126); attempt 2's audit scans only messages since its own
  `auditStart` (independence.ts:44-50 `since`), so the attempt-1 write is never
  re-detected. If attempt 2 succeeds, finalize's `dirtyFiles` `git status` and
  `commitStage`'s `git add -A` are runner `git()` calls, and the pinned env
  (`GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL`, `safe.bareRepository`) does not
  stop git from reading the **repo-local** `.git/config` — the planted
  `core.fsmonitor` fires on the index refresh at runner privilege. If both
  attempts fail, the plant persists in the operator's checkout and executes on
  every later git command, run over. Concretely: plant in attempt 1 → attempt 1
  fails the audit → clean attempt 2 → the commit executes the plant at runner
  privilege. #13 as written passes through this whole chain (it asserts only
  that the attempt fails). Fix: name the containment step beside "fails the
  attempt" — a `.git`-matcher hit escalates the stage to the operator
  immediately (no attempt 2, no finalize until the operator clears the plant)
  — and extend #13's ship gate to assert containment, not just detection: the
  hostile fixture is an fsmonitor that touches a marker file, and after the
  audit-failed attempt the runner's next git call must not create the marker.

- [major][new] Chosen design — same paragraph — "agent-config permissions are
  only advisory: a model can route around them (gates.ts:256-258,
  independence.ts)" — the design demotes the config denies to advisory using
  the repo's own rationale that the bypass route is **subagents**
  ("a model can route around them via subagents"), but the audit it promotes
  to carrier sees only the attempt's own session. It "inspects what the
  session actually DID: the completed tool calls and shell commands in **its
  own message list**" (independence.ts:10-11), and the runner passes exactly
  one session's `lastSeen` list (runner.ts:757, 839). A `write`/`edit`
  completed inside a subagent's child session is not in that list — the base
  permission policy allows `*` and neither agent config denies `subagent` —
  so the one bypass vector the design cites to justify the audit slips past
  both the advisory deny and the carrier, and (per the previous finding) the
  surviving plant then executes. Fix: state the audit's session scope —
  either fetch and audit subagent sessions spawned by the attempt, or deny
  `subagent` launch for the two move-emitting seats (the doc itself notes a
  `policies` entry can hard-deny after rules and saved approvals) — and pin
  whichever is chosen as a #13 case (`task`/subagent write to
  `src/.git/config` must fail).

- [minor][new] Risks row 1 — "The audit's case-folded `.git` matcher fails the
  attempt for the write however it was spelled" — overclaimed, and the changed
  text makes it testably false. The matcher is a `.git` *segment* match
  (independence.ts:90-98 shape); an allowed command writing a git config file
  *outside* the worktree — `git log --output=~/.gitconfig` or an XDG config
  path — contains no `.git` segment (the segment is `.gitconfig`), matches no
  `**/.git/**`/`**/*.git` deny (all are Location-relative whole-value patterns,
  and the harness default has no `external_directory` ask/deny), and is outside
  the sweep. The impact is contained — `git()`'s pinned env means it never
  executes at runner privilege, and agent-privilege exec is the Deferred
  sandbox line — but a host-persistent config plant escapes every control this
  change ships, and the mitigation sentence asserts it is caught. Fix: scope
  the claim ("fails the attempt for any `.git`-segment write, however spelled;
  out-of-worktree config landings are contained by `git()`'s pinned env and
  left to the Deferred sandbox line"), or extend the matcher with a
  `.gitconfig` segment case and pin it in #13.

## FYI (no severity)

- The `.git`-segment rule now lives in `stagePathAllowed`, the global
  ownership predicate — it bars every stage from touching any path with a
  `.git` segment or a `*.git` segment name (e.g. a future `*.git` fixture
  directory), and the `**/*.git` edit deny mirrors that; broader than
  moves-only. Correctness/complexity seats' call whether that breadth is
  wanted; security does not object.
- Rule 6's universal case-fold means a repo with case-variant filenames can
  never move both in one reply; the text flags this as deliberate — fine,
  just noting it is a behavior change beyond the deferred case-only renames.
