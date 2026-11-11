# Security review r4

verdict: request-changes

Inputs used, exactly as named by the caller: `02-proposal.md`,
`docs/adrs/005-runner-executed-file-moves.md`, the proposal's Revision log,
and the named diff path. No review reports or transcripts were opened, listed,
globbed, or grepped. My seat has no previous findings (editorial phase, single
language pass per round), and the Revision log (r0/r1/r3 entries) records no
security decision I am re-opening; the deferred items were checked against
ADR-005 and are consistent (see FYI).

## Findings

- [blocker][new] Scope target — the caller names this pass's scope as
  `docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r4.diff`,
  but that file does not exist: reads at the named path, both absolute and
  repo-relative, return "File not found" (the surrounding `02-proposal.md`
  reads fine, so the run directory is present). A confirmation pass cannot
  confirm: checklist items 1–7 cannot be evaluated against r4's changed text,
  and per the skill a checklist item I could not evaluate counts as a finding,
  not a pass. Everything below was derived from the full `02-proposal.md`,
  which is sound input but cannot attest to what the r4 editorial diff touched.
  Fix: generate/provide the r4 editorial diff at the named path and re-run this
  seat against it, or explicitly rescope the pass to the whole `02-proposal.md`
  in the caller prompt.

- [minor][new] Chosen design, Validator rule 5 — "not on disk (`existsSync` —
  this is also the case-insensitive-checkout guard, A10)" — `existsSync` follows
  symlinks, so a *dangling* symlink planted at a `to` path (agents can write
  worktree files; A7) reads as "does not exist", passes rule 5, and
  `fs.renameSync` then silently replaces that dentry — rename(2) does not follow
  a final-component symlink. The claimed invariant "`to` does not exist
  anywhere" and the no-overwrite / all-or-nothing posture have a fail-open edge,
  and rule 7 does not close it because it realspaths only `to`'s parent, never
  the `to` dentry itself. Containment still holds (the rename lands inside the
  repo), so this is bounded, not exploitable escape. Fix: make rule 5's disk
  check an any-dentry test (`lstatSync` try/catch) instead of `existsSync`;
  lstat preserves A10's case guard, since a case-insensitive filesystem itself
  resolves the lookup (lstat of `readme.md` succeeds where `README.md` exists).

- [minor][new] Chosen design, Validator rule 4 — "`git cat-file -t
  <headBefore>:<from>` equals `blob` — this is the file-only rule" —
  `cat-file -t` also returns `blob` for a tracked **symlink** (mode 120000;
  its object is a blob), so the rule that claims to be the file-only rule
  admits tracked symlinks. Behavior is backstopped fail-closed by rule 7
  (realpath of a link escaping the repo is rejected; `realpathSync` on a
  dangling link throws, which the wiring routes to `invalid-reply: moves`) but
  that chain is implicit, and Test strategy #2 pins "`from` a tree" without a
  tracked-symlink case. Fix: make rule 4 assert a regular-file mode (e.g.
  `git ls-tree <headBefore> <from>` mode ∈ {100644, 100755}), or keep the check
  and add an explicit "tracked symlink `from`" case to #2 pinning the intended
  rejection.

- [minor][new] Compatibility matrix — "Node ≥ 22.12 … operators expected on the
  current patched 22.x" — the October 21, 2026 Node.js security release fixed
  CVE-2026-111497, a Brotli-decoder out-of-bounds write with RCE in every
  22.x before 22.23.4 (CVSS 4.0 9.2 per Google's scoring; PoC public), plus
  nine other CVEs. A named floor of ≥ 22.12 names vulnerable runtimes as
  compatible, and "expected on the current patched 22.x" is a hope, not a
  bound. Advisory is fixed upstream, so this is not the checklist's blocker
  case, but the design should name the security floor. Fix: state
  "Node ≥ 22.23.4" in the compatibility row and raise
  `tools/orchestration/package.json` engines to match.

## FYI (no severity)

- Git released 2026-11-10: 2.56.1 plus backports fix six CVEs (CVE-2026-111760
  recursive-clone RCE, CVE-2026-113408 protocol-v2 traversal, merge/binary-patch
  memory corruption, fast-import). None is reachable from the only git call
  this change adds (`git cat-file -t <headBefore>:<from>` on the pipeline's own
  local repo), so the "no new floor" row stands; worth a line in Rollout so
  operators patch. The no-patched-version Git for Windows advisories are
  moot here — Windows is "not exercised".
- Deferrals checked against ADR-005 per the skill: the checkpoint/reply trust
  boundary deferral to ADR-006 is consistent with the ADR's Consequences
  ("ADR-006's checkpoint-trust scope" untouched); the read-only-git deferral
  mirrors the ADR's own Deferred item verbatim; file-only pairs strengthens, not
  contradicts, ADR item 2's "tracked path" requirement. None re-opened.
- A10 kept `unverified` with a fail-closed default (rule 5/6 reject
  case-colliding endpoints on every platform) is the right security posture;
  #12's macOS-vs-Linux contrast is adequate evidence-gathering — no action.
