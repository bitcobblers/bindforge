# security review r5c0

ratification pass on the r5-approved text; scope: blob diff 2168fac..967d5e8, then full current document.

verdict: approve

## Checklist walk (security)

- Trust boundary (moves list is model/agent-supplied, treated untrusted): unchanged by the diff; validate-all-then-apply, fail-closed routing (`invalid-reply: moves`), and the rules 4–7 apply-time guard — all controls I approved — survive the rewording of the Wiring and Apply paragraphs intact.
- Rule 5 `existsSync` → `lstatSync` try/catch (A10 basis updated to match): a strict tightening — a planted dangling symlink is now a refusing dentry where `existsSync` followed it and reported "absent", letting `rename(2)` silently replace it. Strictly more conservative; nothing I approved is weakened.
- Rule 4 `git cat-file -t` → `git ls-tree` mode check: rejects mode-120000 tracked symlinks and trees, which the old blob-equality check admitted (`cat-file -t HEAD:<symlink>` returns `blob`). Direction is tightening; but see finding 1 on the lookup's exactness.
- Compatibility row, Node engines raised ≥22.12.0 → ≥22.23.4 citing "the October 2026 security release (Brotli RCE CVE-2026-111497 et al.)": checked against primary sources. Node.js published security releases 2026-10-21 shipping v22.23.4 / v24.22.1 / v26.12.1; CVE-2026-111497 is the Brotli-decoder out-of-bounds write with RCE potential, added to CISA KEV 2026-10-27 on evidence of active exploitation, and 22.23.4 is the first 22.x release containing the fix. The claim is accurate, the raise is security-positive, and no unfixed advisory against the 22.23.4 floor is known as of today. No new runtime dependency; `git ls-tree` is ancient plumbing, so the "no new git floor" note stays true.
- Deferred section vs. ADR-005: the read-only-git deferral (item 5 to a follow-up ADR, items 1–4 stand) matches the ADR's Deferred clause verbatim in substance; no contradiction to re-open.
- My r5 nit on A6's "four read-only-named commands" opener: text unchanged, no regression; ADR-005 itself names the four commands and Options 1–2 still use A6 soundly for threat grounding. Per ratification scope, not re-reported as a finding.

## Findings

- [minor][new] Chosen design → Validator → rule 4 — "`from` is a tracked regular file at `headBefore` (`git ls-tree <headBefore> -- <from>` mode 100644/100755 — this is the file-only rule; trees and mode-120000 symlinks fail it)" — the r4 swap replaced an exact lookup (`cat-file -t <headBefore>:<from>`, whose rev syntax names one exact path) with a pathspec: git's own manual states ls-tree's `<path>` arguments "aren't really raw pathnames, but rather a list of patterns to match". Pathspecs carry glob magic (`*`, `?`, `[`) and directory-prefix matching by default, and rule 1 does not exclude those characters. So an implementation reading the rule as written — output non-empty and a mode 100644/100755 line — does not enforce the claimed rule: `ls-tree HEAD -- foo` with `foo` a tracked *directory* lists its child blobs at 100644 (nothing named `foo` is tracked), which would pass rule 4's mode check and then let `renameSync` relocate the whole directory tree, a move class the document explicitly rejects (Non-goals, Options 4); and a `from` containing glob characters can match multiple entries, so the mode check can pass on a different entry while the rename moves a literal symlink named `from` — the tracked-symlink hole the r4 revision set out to close. Test strategy #2 pins the simple tree/symlink cases, so only the multi-match/glob shape and the spec's own wording are exposed. Fix: in rule 4, pin the lookup to one exact entry: `git ls-tree <headBefore> -- ':(literal)<from>'` and require an output line whose path field equals `from` exactly; empty output or no exact-path line fails. (`:(literal)` pathspec magic has existed since git 1.9, so the "no new git floor" row still holds.)

## FYI (no severity)

- The engines raise is asserted against `tools/orchestration/package.json:30-32`, outside this seat's read inputs; confirming CI images and operator machines actually run ≥22.23.4 (the KEV-listed CVE's remediation depends on the runtime that executes the runner, not the engines field) belongs to the build/compatibility seat.
- The Revision log's Validator entry names its round only inline ("r4 replaced ..."); every other entry carries a leading round label. Editorial, for the proposal-editing pass.
