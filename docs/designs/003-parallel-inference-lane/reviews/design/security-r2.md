# security review r2

Scope: `reviews/design/editorial-r2.diff` (round 2 editorial confirmation pass).
Inputs read: the diff, `02-proposal.md` (incl. Revision log),
`docs/adrs/003-parallel-inference-lane.md`. No other review report or transcript opened.

verdict: approve

## Findings

None. The diff is language-only; every hunk was checked against the security
checklist and none changes a trust boundary, a validation rule, a fail-closed
path, or a git/filesystem behavior.

Diff-by-diff confirmation (each `+` line verified present in the proposal):

- **Context** — "the run's dominant wall-clock cost"; "every other
  state-machine semantics stay as written". Word-wrap and plural agreement;
  the ADR-003 "stays as written" claim is unchanged.
- **Options considered / Model-driven subagent fan-out** — "The cap stays in
  runner code." The security-relevant invariant (cap enforced in runner code,
  never by model compliance, per ADR-003's standing rejection of subagent
  fan-out) is preserved and now reads unambiguously.
- **Chosen design / `finalizeReviewPass`** — "the message has today's fallback
  shape with one value: `docs: <nnn-slug> design-review stage` (A8: no
  `commitMessage` in replies)". The clause that blocks a reply-borne (untrusted
  model-controlled) commit string from reaching `git commit -m` is untouched.
- **Pass-level fan-out / Escalate after siblings settle** — the two edits are
  precision improvements on the fail-closed path, not behavior changes:
  "releases **its permit** and returns **without prompting**" disambiguates the
  earlier "releases it … unprompted" (a permit release, not a flag reset), and
  "no seat **that** today's `StopDriving` … would skip ever gets run" keeps the
  skip-after-`escalationPending` rule intact. The sweep-before-`finalizeEscalate`
  ordering and the commit qualifier "only complete in-policy sibling reports"
  keep both r1 security constraints (sweep enforced; partial/out-of-policy files
  excluded), so the Revision log's r1 security closure is not weakened.
- **Configuration and CLI** — "a range check (integer ≥ 1) as a separate error
  naming the source". The `^\d{1,5}$` format gate on `$OPENCODE_MAX_PARALLEL`
  that runs even under a flag override, the separate range error, and "all
  validation before side effects" all survive verbatim, matching ADR-003
  §Configuration's strict fail-closed requirement.
- **Test strategy #9** — "the pass produces one commit" (tense). The commit-count
  assertion that backs the pass-finalize ownership guarantee is unchanged.

Checklist coverage for this scope: (1) trust boundaries — no data-source text in
the diff; unchanged. (2) parsing untrusted data — unchanged. (3) path handling —
no path/shell text in the diff. (4) secrets/PII — no logging or egress text in the
diff. (5) dependencies — the diff adds none; the only third-party packages named
in the design (`p-limit`, `p-queue`) are rejected in *Options considered*, and
neither carries a known unfixed advisory, so no blocker. (6) privileges —
unchanged (`spawnSync` git, `.pipeline.lock`, no new FS surface). (7) fail closed —
the escalation-skip and env-validation paths the diff reworded still reject
cleanly.

## FYI (no severity)

- **Pass-level fan-out / Escalate after siblings settle** — the rewrap leaves a
  hard line break mid-sentence: "run the pass `enforceOwnership` sweep, then" /
  "`finalizeEscalate` once with today's …". Renders correctly in Markdown, but
  it is an orphaned line for the language pass to re-flow.
- **Assumption register A10 / Compatibility matrix** — the repo's pre-existing
  `node >= 22.12.0` engine floor predates the October 2026 Node security
  releases (dependency updates such as c-ares). This diff does not touch that
  text, and the design changes no dependency, so it is not a finding here —
  noting it only as repo-hygiene for a rollout/repo owner.
