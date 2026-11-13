# security review r1c2

verdict: approve

Scope: diff 9b6b5ed→4e18f4f of `02-proposal.md` (Context, §1 shell branch, §2
build-note sentence, §4 read-ceiling sentence, Risks tail row, Test strategy
#3, Revision log), then the full current document. Inputs: proposal, ADR-016,
Revision log, my r1 findings.

## r1 finding closure check

- **major (my r1 #1, §1 shell branch operand) — closed.** The replacement
  text names the operand exactly as the fix asked: "collect each whole command
  token containing a `reviews` segment — delimited as `REVIEWS_SEGMENT`
  delimits, prefix included, never the prefix-stripped `REVIEWS_KEY` fragment"
  with "every token is an identity match to `own` above" (so the §1 canonical
  rule's repo-root resolution and design-directory containment apply to the
  full token, containment is not vacuous), and the fail-closed clause "a
  prefix-less token resolves outside the attempt's design directory and fails
  closed" is present and arithmetically correct (a repo-root-resolved
  `reviews/design/...` cannot lie under `docs/designs/<run>/`). Test
  strategy #3 now pins both spellings §3's allow opens: own-report repo path
  clean, same path under another run fails, own + sibling fails. Neither
  r1 escape (a) (exemption never fires on the shell route) nor (b)
  (vacuous containment) survives the new text.
- **minor (my r1 #2, build-note publication claim) — closed.** §2 now reads
  "the committed `03-build-notes.md` 'Gate posture' section — cited
  line-selectively, dropping any credential-bearing or absolute-path line —"
  and the Risks tail row names the committed set: "the checkpoint and
  transcripts stay local and the build note is the one committed carrier."
  The contradiction between the row and §2 is gone; the no-redaction-pass
  decision stands and is not re-argued here.

## Findings

- [nit][new] Chosen design §1 shell branch + Test strategy #3 — "a
  prefix-less token resolves outside the attempt's design directory and fails
  closed" — this r2 sentence is the load-bearing fail-closed half of the new
  rule (checklist 7), and it is the only direction of the new shell rule with
  no fixture: #3 pins the clean own-spelling and the cross-run/sibling
  failures, but not the prefix-less spelling (`git diff
  reviews/design/<specialty>-r1c1.md`), which §3's `*`-prefixed wildcard
  allow does let through the permission layer, so it reaches the audit and
  must be the audit that fails it. Fix: one added assertion in #3 — the
  prefix-less spelling fails the audit — no new mechanism.

## Checklist pass notes

- Trust boundaries / fail-closed: the exemption stays identity-scoped (own
  report under the attempt's design directory only); transcripts never
  exempted; empty-token-set commands keep today's violation behavior;
  malformed/prefix-less spellings fail closed. Pass.
- Path handling: canonicalization rule unchanged (A17 normalization, repo-root
  resolution); the shell operand is now explicitly the whole prefixed token.
  Pass.
- Secrets (checklist 4): tails live in the local checkpoint/transcripts; the
  one committed carrier (build note) is line-selective with
  credential-bearing and absolute-path lines dropped; no redaction pass, as
  already accepted. Pass.
- Privileges: §3 widening unchanged since r1 (read-only inspection set, write
  legs denied, run-record denies); nothing in this diff widens privileges.
  Pass.
- Dependencies: no dependency text changed in this diff (`engines.node` and
  `@opencode/client` rows are r1 text, unchanged, and the Node move is toward
  the patched release). No new advisory exposure in scope.
- Deferred entries: the machine-checking of the research read ceiling moved
  from an inline note to the Deferred list; ADR-016 Decision 4 only requires
  the ceiling be prompt-stated ("states a ceiling on what it may read"), so
  the deferral contradicts no ADR clause. Not a finding.

## FYI (no severity)

- The §3 wildcard re-open intentionally allows more than the §1 audit exempts
  (prior-round / other-run same-spelled reports pass the permission layer,
  fail the audit); the Risks seam row already owns this and the Deferred
  per-run-templating entry covers its closure — consistency is the
  correctness/compat seats' call, not mine.
