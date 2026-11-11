# security review r6

verdict: approve

Scope: the editorial diff `docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r6.diff`,
read against `02-proposal.md` and `docs/adrs/005-runner-executed-file-moves.md`.
Confirmation pass: no prior security findings of my own to re-check.

## Findings

None. Every hunk in the diff is language-only; no security-relevant clause was
altered, weakened, or made false.

Checklist walk over the diff's touched text:

1. **Trust boundaries / untrusted input** — the diff's only substantive edit to
   untrusted-input handling is the Wiring-into-retry parenthetical, "a failure —
   a returned issue or a throw from a rule — takes the existing schema-failure
   branch". The em-dashes replace commas in the original "a failure, a returned
   issue or a throw from a rule, takes": both throw paths and returned-issue
   paths still route to `invalid-reply: moves`, so a hostile reply cannot escape
   validation as an unhandled exception. Unchanged.
2. **Parsing untrusted data (size limits, schema validation)** — the Reply-field
   hunk only re-wraps the code span. Joined text is byte-identical to the split
   version: `from: z.string().min(1).max(200)` / `to: z.string().min(1).max(200)`
   and `.default([]).max(64)`; the wrap removal introduced no space inside
   `z.string().min(1)`, so the endpoint-length and pair-count DoS caps still read
   as caps. Unchanged.
3. **Path handling** — rules 1, 2, 5, 6, 7 (`\`/NUL/control/`:` denial,
   no absolute path, no `.`/`..`, `STATE_FILE`/`LOCK_FILE`/case-folded `.git`
   segments, `lstatSync` dentry check, case-folded disjointness, realpath
   containment) are not in any hunk; not touched by the reworded Sweep-interaction
   and Prompts headings, whose bold-period shift changes no semantics.
4. **Secrets / PII** — nothing in the diff changes what is logged or leaves the
   machine; the validator message stays `invalid-reply: moves — <issue>`.
5. **Dependencies** — the diff adds no dependency. The unchanged
   Compatibility-matrix row "Node ≥ 22.23.4 … the 22.x line containing the
   October 2026 security release (Brotli RCE CVE-2026-111497 et al.)" is accurate:
   the Node.js October 21, 2026 security release fixes CVE-2026-111497 (Brotli
   decoder out-of-bounds write → RCE, CVSS 9.2, reported exploited in the wild) in
   22.23.4 (https://nodejs.org/en/blog/vulnerability/october-2026-security-releases).
   No known unfixed advisory applies to the floor named, so no blocker. The only
   git call the change adds is `git ls-tree` (rule 4), which is not an advisories
   surface the way the A6 config-execution sinks are; those stay deferred to the
   follow-up read-only-git ADR, consistent with ADR-005's own deferral paragraph.
6. **Least privilege** — the diff preserves "the runner — still the only
   committer" and the no-agent-git-write position; no privilege is added.
7. **Fail closed** — the Apply hunk's "the same `moves-apply-failed` outcome and
   preserved worktree as a rename error" keeps the guard refusal inside the
   environmental failure class (fresh session, worktree preserved, nothing
   half-applied), and `moves-rollback-failed: <error>` still escalates to the
   operator instead of retrying on a damaged worktree. Test #11's rewording
   ("a pair that can no longer be applied between validation and apply") still
   pins no-commit + byte-identical worktree. Rollout #2's "optional, defaulting
   to `[]`" still keeps old checkpoints parsing unchanged.

Revision log: read; the r4 Node-floor entry and the r4 `cat-file`/`existsSync`
replacement are recorded closures, not re-opened here. Deferred items (directory
moves, case-only renames, OS/sandbox protection, checkpoint trust → ADR-006,
read-only git) do not contradict ADR-005's Decision or its deferral paragraph;
ADR-005 item 2 requires only that `from` be a tracked path and `to` not exist, so
the file-only deferral is consistent with the source.

## FYI (no severity)
- The diff's last hunk appends an unresolved annotation inside the proposal
  itself — "Editor notes: Deferred / out of scope — … is ambiguous; author to
  clarify." Flagging an open question inside a shipped artifact is an editorial
  hygiene matter for the editorial seat, not a security one; the underlying
  deferred line ("Directory moves as single pairs — this ADR-005 if a real change
  needs them") does not create or remove a path-handling rule.
- "as that of a rename error" → "as a rename error" is grammatically looser than
  the original but preserves the comparison; if an editor retightens it, keep
  `moves-apply-failed` as the named outcome class so the fail-closed reading
  stays unambiguous.
