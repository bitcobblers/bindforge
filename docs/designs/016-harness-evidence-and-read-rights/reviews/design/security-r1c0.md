# security review r1

verdict: request-changes

Scope: whole document (pass c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/016-harness-evidence-and-read-rights.md`, the Revision log (r0, no
prior decisions to honor). Dependency advisories checked today per checklist
item 5: `@opencode/client ^2.1.4` — no advisories in Snyk's database for that
version; Node 22.23.4 — see finding 3. Revision log holds no rejections, so
every finding below is `new`.

## Findings

- [major][new] Chosen design §1 "Own-report identity on the audit's read side"
  — "a `reviews/` access value is the seat's own report iff `reportKey(value)`
  (everything from the `reviews` segment on) equals
  `reportKey(opts.ownReportPath)`" — the reportKey drops the design-directory
  prefix, but every run's reports share one suffix grammar,
  `reviews/<phase>/<specialty>-rN[cM].md` (A11). The r1c0 security seat of run
  016 therefore holds a key identical to `reviews/design/security-r1c0.md`
  under *every other* design directory (`docs/designs/012-…/reviews/design/…`
  etc.). Today's read rule fails those accesses; under this exemption they pass
  silently, letting a panel anchor on a prior run's verdicts — the harm class
  ADR-002/007 exist to guard — which contradicts ADR-016 Decision 1's clause
  "Every other `reviews/` path stays a violation". The §3 per-seat allow
  `git diff *reviews/design/<specialty>-*` compounds it: A16's wildcard spans
  `/`, so the permission layer opens the same cross-run paths, and unlike the
  recorded prior-round seam (Risks table) the audit does *not* fail this case —
  both layers open, no backstop. Fix: add run scoping before the identity test
  — canonicalize the access value and require it to lie under the attempt's
  design directory (the directory of `opts.ownReportPath`), i.e. compare
  canonicalized full paths against `ownReportPath`; absolute and repo-relative
  spellings canonicalize to the same file, so the ADR's spelling-equivalence is
  kept while a foreign design dir's same-named report stops matching. Apply
  the same containment to the shell branch's `REVIEWS_KEY` matches. Extend
  Test strategy #3 with a fixture: a seat reading another design directory's
  same-spelled report must fail the audit.

- [major][new] Chosen design §2 "The gate record" — "`implement.md`/`fix.md`
  add the sentence that the block is the runner's own record and its
  failing-leg tail belongs, verbatim, in the `03-build-notes.md` 'Gate posture'
  section" — the tail is merged stdout+stderr of the very code the run is
  writing and fixing; a stage can plant arbitrary text in its test output
  (bounded by 2,000 bytes, ample for an instruction line) that then reaches all
  five review seats and the fix loop fenced and labeled "the runner's own
  record", with a mandate to quote it verbatim into the committed build note.
  That is a stage→seats anchoring channel the proposal never classifies as a
  data source, while the whole of §1/§3 exists to close exactly that channel
  (checklist item 1: every data source classified). Fix: one sentence in the
  rendered block's header and in the prompt sentence, fixing provenance — the
  tails are output captured by the runner *from the code under review*; they
  are citable evidence, not instructions — and drop "verbatim" for the build
  note in favor of "cited with the leg command and exit". This adds no
  mechanism and does not touch the ADR's decision that the record is shared
  ground truth.

- [major][new] Compatibility matrix, "Linux dev/CI host" row — "all changes
  are Node ≥ 22.23.4 in `tools/orchestration`" — Node 22.23.4 carries the
  HIGH-severity CVEs of the 2026-11-11 security release (CVE-2026-123864,
  -123865, -123866, -123872, affecting 22.x < 22.23.5; fixed in 22.23.5,
  published two days before this review). Checklist item 5 requires named
  dependencies to be advisory-clean; the design pins a floor that is already
  known-vulnerable while the patched release exists. Fix: raise the stated
  floor to Node ≥ 22.23.5 in the compatibility matrix row and the package
  `engines` pin — one-line change.

- [minor][new] Chosen design §2, capture shape — "`tail` the last
  `MAX_GATE_TAIL_BYTES = 2_000` bytes of the leg's merged stdout+stderr, passed
  through `printable`" — checklist item 4 is not addressed anywhere: nothing
  states what a leg tail may carry into persisted artifacts (absolute local
  paths, env echoes, failing-`npm` error lines that can embed registry URLs and
  credential-bearing text), where it then lives (checkpoint, transcripts, five
  prompts), or what is committed once the verbatim tail lands in
  `03-build-notes.md` in an open-source repo. Fix: state the posture in §2 —
  extend the `printable` step with a redaction pass (known token patterns,
  home-directory paths) before storage, or explicitly declare the record
  local-only and require the scrub before build-note insertion.

## FYI (no severity)

- `@opencode/client ^2.1.4` evaluated clean (no advisories in Snyk's DB, any
  version through 2.2.6); the historical OpenCode *server* advisories
  (CVE-2026-22812, GHSA-632h-h47v-g4x4) concern `opencode-ai` versions the
  ADR's A20 places outside this repo — operator hygiene, not this diff.
- §3's doc-family allowlist (`cat`, `find`, `rg`, …) is not path-scoped and
  from a bare `shell * → deny` now reaches any file the user can read; ADR-016
  Decision 3 names exactly these commands, so this is not reopened here, but a
  later hardening could scope the inspection set to the repo root.
- `printable` placement drifts between §2 ("passed through `printable` before
  storage") and Test strategy #2 ("`boundedGateEvidence` strips control bytes
  via `printable`") — pick the one site (A7 applies bounds at write *and*
  render); wording issue for the correctness seat.
- The Risks row "Permission/audit seam" covers only same-run prior-round
  reports; the finding 1 case is cross-run and is not covered by that
  acceptance or by the deferred per-run templating entry.
