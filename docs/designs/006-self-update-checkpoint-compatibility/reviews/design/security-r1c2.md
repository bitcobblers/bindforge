# security review r1 (ratification pass c2)

verdict: approve

Scope: `git diff da725a42 305aa36b` over `02-proposal.md`, then the full current
document. Inputs: the proposal (incl. its Revision log), `docs/adrs/006-self-update-checkpoint-compatibility.md`,
and my own r1 security findings. Checklist: `references/security.md` only.

Closure check of my r1 findings:

- r1 minor (case-folding of the reserved-name rule): **closed** by the r1
  revision — §1 now reads "compared segment-by-segment and case-insensitively
  the way the moves validator already compares these names (`moves.ts:134–147`)",
  which is stronger than the `darwin`/`win32`-conditional I asked for (it folds
  everywhere, i.e. fails closed on case-sensitive platforms too), and Test
  strategy #1 pins `.PIPELINE.JSON`. No matrix note is needed now, so the
  macOS row's demotion to `partial` does not reopen it.
- r1 nit (Node engine floor advisory): the row is untouched by the diff;
  re-checked (checklist item 5) — `22.23.4` *is* the patched release for
  CVE-2026-111497 (the only exploited advisory), and the pre-alerted 22.x
  security release of 2026-11-11 (highest severity HIGH, CVEs not yet published)
  is still pending as of this review. A `>=` floor mandates no unfixed version,
  so it stays informational — see FYI.
- r1 nit (unquoted interpolated paths in printed commands): **still open**, and
  the diff added a third printed command string (§2's refusal message). Re-raised.

The closure edits are sound, but the new authorship-test sentence that replaced
the old unconditional `headBefore` restore introduces two edges of an
under-specified predicate. Both are new text, both narrow the fence's reach
without adding a mechanism, and neither blocks implementation.

## Findings

- [minor][new] §1 Fence runner-reserved paths in the ownership matrix — "a match
  is skipped while it still holds the bytes `saveState` last wrote for it, kept
  in memory (or is absent because the runner deleted it), and reverted otherwise"
  — the absence branch states a cause ("because the runner deleted it") but not
  the test that establishes it. Read as "absent ⇒ skip", a stage that deletes
  `.pipeline.json` (its write set covers `docs/**`, A1) escapes the revert, and
  the next `commitStage` commits the deletion — after which the §3 repair the
  design prints, `git checkout HEAD -- <designDir>/.pipeline.json`, restores
  nothing because HEAD now records the deletion, so the recovery anchor (A7,
  ADR-006:55–57) is destroyed by a model-authored action. Same shape for a stage
  deleting the `ESCALATION.md` marker: §3's split-record check ("`pipeline
  status` … checks for the marker") and `resumePipeline`'s gate-log entry both
  depend on the marker surviving a stage's reach. The previous text I approved
  restored unconditionally ("the sweep restores the file from `headBefore`"), so
  this is introduced by the revision. Test strategy #2 covers a planted edit, a
  planted tmp file, a directory replacement, and the runner's own save — no
  stage-side *deletion* case. Fix: in the same sentence make the skip require a
  recorded authorship by this process (its last `saveState` snapshot, or a delete
  this process itself performed), and state that every other state — including
  absence it did not author — is restored from `headBefore`; add one case to
  Test #2: a stage-deleted `.pipeline.json` and a stage-deleted `ESCALATION.md`
  are both restored. This qualifies the existing predicate; no new mechanism.

- [minor][new] §1 Fence (authorship test) vs §3 escalation — "the bytes
  `saveState` last wrote for it" — the exemption is keyed only to `saveState`
  snapshots and runner deletes, so the runner's own `ESCALATION.md` write is not
  exempt: §1 says the sweep filters "every touched path, runner saves included",
  and `finalizeEscalate` writes `ESCALATION.md` and then "still runs
  `commitStage`". Under the exemption as written, that marker is a reserved match
  holding nothing `saveState` wrote, so it is reverted and the terminal record
  never lands — contradicting §3's guarantee (and ADR-006 question 3) and Test
  #7's assertion that `ESCALATION.md` is written with no error escaping, and
  silently unmaking the `status` split-record line. Fix: generalize the exempted
  set in the same clause to "bytes this process last wrote at that path — the
  `saveState` snapshot for the checkpoint, the escalation report writer's bytes
  for `ESCALATION.md`" (the Revision log's intent, "skips what the runner itself
  wrote in-window", made operative). No new mechanism; it replaces the
  saveState-only wording.

- [nit][re-raised] §3 An escalation that survives its own failing write — "`git
  checkout HEAD -- <designDir>/.pipeline.json`, with the `rm -r
  <designDir>/.pipeline.json && git checkout HEAD -- …` variant" — unchanged from
  r1: interpolated paths printed unquoted, so a run slug with a space or shell
  metacharacter mis-tokenizes when the operator copy-pastes (checklist item 3,
  exposure via copy-paste only, the runner never executes them). The §2 revision
  widens the same pattern by one more printed string — "restore the committed
  copy (`git checkout HEAD -- …`)" — so the quoting rule is now worth stating
  once rather than per site. Fix: specify the printed commands with the
  interpolated paths single-quoted, and let Test #5 (which already asserts the
  message names the `rm -r` + `git checkout HEAD --` repair) assert the quoted
  form.

## FYI (no severity)

- §1's tmp matcher was narrowed from "starts with `${STATE_FILE}.`" to the exact
  `.pipeline.json.<8 hex>.tmp` shape. That is needed so the sweep does not revert
  the runner's own in-flight tmp file, and a near-miss planted name cannot touch
  the checkpoint — but such a file is no longer a reserved match, so it survives
  into the commit while the §1 bullet still promises the sweep keeps "`git add -A`
  clean". Leftover-file hygiene, for the correctness seat.
- A12 moved from `unverified` to `verified` on a basis that includes "repo glob
  finds no `.github/workflows/`"; that glob only excludes GitHub Actions, not
  another CI config or a scripted smoke job. Supporting evidence for the
  `no (unverified)` Windows row, for the correctness/compatibility seats.
- Compatibility matrix (Node row, unchanged by the diff): the floor `>= 22.23.4`
  is the release that patches CVE-2026-111497, and the pre-announced 2026-11-11
  22.x security release (highest severity HIGH, CVEs unpublished, not yet shipped
  at review time) is future work; a floor mandates nothing unfixed. My r1
  one-sentence suggestion ("run the newest patched 22.x") remains optional and is
  not re-reported as a finding.
- Deferred item "Automatic digest reconciliation … the §1 fence makes 'dirty ⇒
  runner' true" stays consistent with ADR-006's Decision and its interim posture
  (`git checkout HEAD -- …` or a fresh run, ADR-006:55–57); not re-opened.
