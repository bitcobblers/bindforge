# correctness review r1c3

verdict: approve

Scope as given by the caller: the blob diff `9b6b5ed → 4e18f4f` of
`02-proposal.md` plus the full current document. Inputs: `02-proposal.md`,
`docs/adrs/016-harness-evidence-and-read-rights.md`, the Revision log, my r1
findings. Correctness checklist, ratification framing: report only where a
revision broke something r1 approved or made a cited section false.

## Closure check of my r1 findings

- **minor (canonicalization input ambiguous in the shell branch) — closed.**
  The replacement text names one operand: "collect each whole command token
  containing a `reviews` segment — delimited as `REVIEWS_SEGMENT` delimits,
  prefix included, never the prefix-stripped `REVIEWS_KEY` fragment". The
  never-exempt reading of r1 is gone (the token carries its design-directory
  prefix, so it can resolve under `opts.ownReportPath`'s directory), and the
  cd-relative reading is no longer silently lost: it is now an explicit rule —
  "a prefix-less token resolves outside the attempt's design directory and
  fails closed" — and Revision log r2 records it as a deliberate choice.
  Test strategy #3 now pins the operative spelling
  (`git diff docs/designs/<own>/reviews/design/<specialty>-r1c1.md` clean,
  another run's same path failing, own + sibling failing), which is the
  spelling ADR Decision 1 names ("the repo-relative path the prompt handed the
  seat") and the one §3's re-open admits with a non-empty leading `*`. No
  test coverage was removed by the rewrite. Residual gap below, as a nit.
- **nit (A17 case-fold over-attribution) — unchanged, not re-raised.** §1's
  gloss ("normalized the way the permission layer normalizes paths (A17: `\`
  to `/`, case folded)") and A17's row ("case-insensitive on Windows") are
  byte-identical to r1, and Test #3 still pins case-variant read spellings.
  Nothing in the diff touched it, so per this pass's scope it is not reported
  as a finding (see FYI).

## Findings

- [nit][re-raised] Chosen design §1, shell branch — "a prefix-less token
  resolves outside the attempt's design directory and fails closed" — the
  fail-closed clause the r2 revision adds is the only place in the document
  that decides this case, and it is unpinned and unenumerated. It is
  reachable through rules the same document ships: §3's per-seat re-open
  `shell git diff *reviews/design/<specialty>-*` matches with A16's leading
  `*` spanning the empty prefix, and §3's inspection set (byte-for-byte
  `code/coder.md`, ADR Decision 3's `cd`) permits
  `cd docs/designs/<dir> && git diff reviews/design/<specialty>-r1c1.md`, so
  the permission layer admits a command the audit then fails. The Risks seam
  row enumerates only "the seat's *prior-round* reports and another design
  directory's same-named report", and §3's seam sentence only "prior-round
  and other runs' same-named reports", so the one own-report spelling the
  allow admits-but-§1-fails is described nowhere; Test #3 pins neither
  direction of it. Not a blocker: the behavior is deterministic,
  conservative, and no worse than pre-016 (the row's own "a burned attempt
  for an off-prompt read is today's behavior (no regression)"). Fix: extend
  Test strategy #3's fail set with that one compound command (or move it to
  the clean set if the intent was to exempt it), and add the prefix-less
  own-report spelling to §3's "the wildcard's extra reach over prior-round
  and other runs' same-named reports" enumeration.

## FYI (no severity)

- Checklist item 2 (spot-check research URLs) was not re-run: the diff touches
  no assumption row, citation, or research claim; the URLs were exercised in
  r1 and nothing in these revisions depends on them.
- The rewritten Risks row now asserts "the checkpoint and transcripts stay
  local and the build note is the one committed carrier". That is the
  load-bearing reason for shipping no redaction pass over the tails, but no
  assumption-register row names where the checkpoint and transcript files
  live; ADR Decision 2 says only that the capture "lives in the checkpoint and
  the documents stages already own". A register row for the checkpoint/
  transcript paths would put it on the record. Not evaluated here — not
  checkable from this pass's named inputs.
- The r2 Revision log entries replaced the r1 §1 and §2 entries rather than
  superseding them, and my r1 nit on the A17 case-fold gloss is recorded
  neither as adopted nor as deliberately kept; a one-line "not changed: audit
  folds case by its own rule" entry would close it.
- For the editorial seat: §2's sentence "…belongs in the committed
  `03-build-notes.md` 'Gate posture' section — cited line-selectively,
  dropping any credential-bearing or absolute-path line — with its leg
  command and exit, through the document the coder already owns" reads
  mangled after the inserted clause.

## Sections verified as unbroken by the diff

- Context's narrowing to "the exclusions are the Non-goals" still covers both
  exclusions it dropped by name — Non-goals bullet 1 (`--max-parallel`,
  `maxParallelReviews`, slots/KV, against ADR Decision 4) and bullet 2
  (`subagent` deny, ADR "Not decided here"), and Deferred repeats both.
- §2's gate-record shape, capture/render bounding, optional-field and
  ownership claims are otherwise untouched; the added "line-selectively"
  wording contradicts nothing in §2's rendering rules.
- §4's read-ceiling simplification matches the Deferred entry
  ("Machine-checking the research read ceiling … this design states the
  budget in the prompt only") and ADR Decision 4, which asks only that the
  prompt "states a ceiling".
- Risks, Compatibility matrix, Test strategy #1/#2/#4–#14, Rollout: no
  contradiction introduced; the review-stage audit rule and the transcripts
  test remain as approved.
