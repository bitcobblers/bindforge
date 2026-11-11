# security review r1 (dissent pass c3)

verdict: approve

Scope: `git diff 89027e8 6b21e2f` over `02-proposal.md`, then the full current
document, the ADR, and the Revision log. Checked first: my three r1 findings.

## r1 findings — closure check

- **major (containment: audit detects a plant but retry re-exposes it) — CLOSED.**
  "A hit ends the stage as `git-plant-detected: <path>`, escalating to the operator
  like `moves-rollback-failed`: no attempt 2 and no finalize, since both would run
  in that worktree under the planted config and the `since` window never re-detects"
  is the containment step I asked for, and #13 now pins the outcome, not just the
  detection: "with the plant in place the runner's next git call must not execute
  it — the hostile config is a `core.fsmonitor` that touches a marker file, which
  stays absent." The audit call site (runner.ts:839-849) precedes the reply/nudge/
  finalize path, so the ordering the design relies on exists.
- **major (subagent route bypasses both the advisory deny and the carrier) —
  CLOSED in substance.** The configs now append a `subagent` deny; A9's new clause
  matches the permissions doc (the `subagent` action gates launching a child agent,
  which carries its own permissions), and the doc confirms rule denies survive
  saved approvals ("Saved approvals ... never override a configured `deny`"), so a
  plain rule deny is enforceable here. #13 pins "any subagent launch is refused by
  the agent config." One closure defect survives — see finding 1.
- **minor (matcher overclaim) — CLOSED.** Risks row 1 is scoped to `.git`-segment
  writes and names "out-of-worktree config landings (no `.git` segment, e.g.
  `~/.gitconfig`) ... stay Deferred"; the Deferred sandbox bullet now carries
  "config writes that land outside the worktree". Consistent with the ADR.

Also checked the other changed text: rule 7's rewrite (relativizing the `.git`
test to the repo realpath) keeps the linked-worktree pointer and intra-repo
symlink-to-`.git` refusals and fixes the ancestor false positive — no regression;
dropping the segment-ends-in-`.git` clause and `**/*.git` deny is covered by the
cwd-pinning + `safe.bareRepository=explicit` story and #13's bare-repo clause; the
new pinned committer identity is a net improvement; no new dependency is named.

## Findings

- [minor][new] Test strategy #13 — "every completed `.git` write (`src/.git/config` plant, `git log --output=.git/config`, a subagent-launched one) escalates the stage with no attempt 2" — the parenthetical pins the wrong mechanism: the design's own sentence two paragraphs earlier says "the audit judges one attempt's own message list" (independence.ts audits one session's `lastSeen`), so a write completed in a child session is never in the matcher's input; and with the launch refused one clause earlier, no subagent-launched write can even reach "completed". The gate clause is unimplementable as written — or silently assumes child-session scanning the Chosen design never specifies. Fix: drop "a subagent-launched one" from the audit-escalation list — the preceding clause, "any subagent launch is refused by the agent config", already pins that vector — or add one sentence making the audit walk the child sessions the attempt spawned and say which messages it reads.
- [minor][new] Chosen design / Read-only git + Risks row 1 — "The same two configs append a `subagent` deny" vs "escalates the stage to the operator for any `.git`-segment write, however spelled" — the deny ships only on the two item-5 configs, but the audit matcher runs "over every attempt" and the doc's own Deferred bullet and code-correctness citation say the review/research/doc-sync seats already run `git diff*` and `git log*` in writable worktrees. Nothing denies those seats `subagent`, and the carrier cannot see child-session writes, so a plant routed by a non-item-5 seat through a subagent lands unopposed: the sweep cannot see it (A4) and the next runner git call re-reads it — while the mitigation row claims "any". Fix: append the same `subagent` deny to every seat whose shell can write, or scope the row-1 claim to the item-5 seats and state that subagent routing from the other seats sits on the Deferred sandbox line.

## FYI (no severity)

- Rule 7's change ("rule 2's `.git` segment test then runs on the resolved path
  made relative to that realpath") quietly fixes what would otherwise reject every
  move in a checkout under a `project.git` ancestor (a linked-worktree layout) —
  worth a correctness seat's eye on rule 2's raw-path test still running first;
  #2's symlink-into-`.git` case pins the endpoint.
- The Non-goal narrowing to "Any agent git **write** access" aligns the doc with
  ADR-005 item 5 (which itself grants read-only git); the ADR's Alternatives header
  "Agent git access (any scope)" reads broader than its own Decision — editorial,
  not mine.
