# security review r2

verdict: request-changes

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial pass over
`02-proposal.md`), read against `docs/adrs/010-lightweight-plan-flow.md` and the
proposal's Revision log. My security seat carried no open finding into this pass,
so the only admissible findings are ones the changed text creates. Six hunks are
pure language and touch no security property (serial comma in Goals; the
seat-reader sentence; "the last" → "the latter"; `heavy`/`heavier flow` → `full
flow` in the plan-pointer, plan-stop and Risks row 6, which now matches the
escalation strings; the two Rollout commas; the Rollout 6 `--extend-budget`
reflow, which stays state-side per A27). One hunk is not language-only.

## Findings

- [major][new] The `plan` stage, first bullet — "`mode: primary`, edits
  `docs/designs/**`, denies `.pipeline.json`, reads and writes `transcripts/**`,
  and allows shell." — this line is in the diff (`denies `.pipeline.json`,
  reading/writing `transcripts/**`, and shell` → `denies `.pipeline.json`, reads
  and writes `transcripts/**`, and allows shell`), and the editorial rewording
  turned two denials into grants. The bullet declares the planner is "the
  `design/designer` shape (A12), one line narrower", and that shape is the
  opposite on both axes: `designer.md:14-22` denies **edit** and **read** of
  `docs/designs/**/transcripts/**` and denies `shell: "*"`, and
  `researcher.md:14-22` is identical, so no document-stage primary agent holds
  either privilege. Two security properties break. (a) Audit integrity / least
  privilege (checklist 6): transcripts are the run's record — the runner writes
  them (`prompts.ts:79`, `runner.ts:1640`), and `runner.ts:1436-1441` states
  "Transcripts are out of band **for every stage**", with `independence.ts:6-16`
  failing the attempt outright on a completed transcripts access. An agent
  configured to *write* `transcripts/**` can forge the run record the
  ownership matrix and the independence audit exist to protect, and configured
  to *read* it is exactly the run-003 behaviour that produced the fence.
  (b) Functionality of the stated privilege set: a planner built to this line
  fails its own stage audit on the first transcripts access, so the spec is not
  implementable as written. The `allows shell` grant is the widening with no
  stated need — a plan-writing stage runs no commands, `designer.md:20-22`
  denies shell outright, `coder.md:26-46` denies `shell "*"` and then lists
  seven named commands, and `independence.ts:39-48` records that an
  obfuscated shell command is the one access the audit provably cannot catch, so
  an unrestricted shell grant on a primary agent is precisely the privilege the
  defence-in-depth does not cover. The same edit also makes the bullet's
  "one line narrower" claim false: the sentence adds grants, not lines removed.
  Fix: restore the pre-diff privileges — `denies `.pipeline.json`,
  reading/writing `transcripts/**`, and shell` (`shell: "*"` deny,
  `docs/designs/**/transcripts/**` read+edit deny, mirroring `designer.md:11-22`).
  If a light `plan` genuinely needs a command, name the explicit allowlist
  `coder.md:29-46` uses and the need for it, in this line; that replaces the
  `allows shell` clause rather than adding to it.

## Checklist coverage

- 1 Trust boundaries — not touched by the diff; the proposal keeps the seat panel
  as caller data (`validated(merged, flow)`), unchanged.
- 2 Parsing untrusted data — the diff's hunk at Budget resolution ("A violation
  throws at the same pre-lock posture as a bad `flow` value") sharpens the
  fail-closed claim and matches "Selecting the flow" and ADR-010's "before the
  lock and before any inference". No finding.
- 3 Path handling — A28/Ownership untouched by the diff.
- 4 Secrets/PII — nothing in the diff touches logging or shared artifacts.
- 5 Dependencies — the diff names no new dependency, and A20 keeps "no new
  platform surface", so there is no advisory to check.
- 6 Privileges — the finding above.
- 7 Fail closed — untouched by the diff.

## FYI (no severity)

- The planner's `docs/designs/**` edit grant stays narrower than the Ownership
  section's code gate for `plan` (`inDocs = /^docs\//`, so the gate permits all
  of `docs/`); pre-existing, not created by this diff, and agent permissions are
  advisory (A13) — an ownership/complexity seat may want the gap recorded.
- Terminology: this diff removes the last "heavy flow"/"heavier flow" prose and
  aligns the escalation-adjacent sentences with the `flow: full` strings; that is
  the editorial seat's win, noted only so a later seat does not re-litigate it.
