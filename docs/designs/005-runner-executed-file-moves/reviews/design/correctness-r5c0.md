# correctness review r5 (ratification c0)

verdict: approve

Scope: the blob diff 2168fac..967d5e8 of 02-proposal.md (the r4 revisions: rules 4-5
rewritten from `git cat-file -t`/`existsSync` to `git ls-tree` mode check /
`lstatSync` dentry check; A10 reworded; Node engines raised to ≥22.23.4; test
strategy #1/#2/#12 and the Revision log updated to match), plus the full current
document read against docs/adrs/005-runner-executed-file-moves.md and the
correctness checklist. Prior correctness seat held no findings, so nothing
previously cited is re-checked for regression beyond the changed text.

## Findings

- [minor][new] Chosen design — Validator, rule 4 — "`git ls-tree <headBefore> -- <from>` mode 100644/100755 — this is the file-only rule; trees and mode-120000 symlinks fail it" — the command as written does not list the entry itself when `from` is a tree: git-ls-tree's own man page states `<path>` is "a list of patterns to match" and documents `-d` ("show only the named tree entry itself, not its children") precisely because the default form lists a matched directory's *children*. So `git ls-tree <head> -- src` for directory `src` emits 100644/100755 lines for `src`'s files — an implementation that literally checks "the mode of the ls-tree output" accepts a directory, which is exactly the file-only rule rule 4 exists to enforce. (The glob-pattern property compounds it: rule 1 does not ban `*`, `?`, `[`, so a `from` containing metacharacters can pattern-match a different tracked entry.) The symlink half of the rewrite is exactly right (tracked symlinks show mode 120000, which `cat-file -t` had called `blob`). Not a blocker: Test strategy #2 pins "`from` a tree or tracked symlink", so the pin catches a naive implementation — but the rule text should specify the command that makes the test pass by construction. Fix: in rule 4, write the check as `git ls-tree -d <headBefore> -- <from>` (single line, the entry itself; modes 100644/100755 pass, 040000 and 120000 fail), or equivalently state that the validator requires exactly one output line whose path field equals `from` before applying the mode test; the Compatibility row's "only git call this change adds" wording is unaffected.

## Ratification checks that passed

- Rule 5's `lstatSync` dentry check: correct and strictly stronger than the old `existsSync` — lstat stats a dangling symlink's dentry where existsSync follows it and reports nothing, and `rename(2)` silently replaces an existing destination dentry; the planted-dangling-link hole the Revision log records is genuinely closed, and #2's "dangling symlink included" case pins it. Consistent with A10's rewording, rule 7's "to's parent (rule 5)" reference, the Risks row's "rule 5's disk check", and #12's macOS/Linux contrast.
- A10: still honestly `unverified`, mechanism basis updated in step with rule 5; no claim promoted to fact.
- Compatibility Node row: the new basis checks out externally — CVE-2026-111497 (Brotli decoder OOB write / RCE) was disclosed 2026-10-21, fixed in Node 22.23.4 in the October 2026 security releases, and is on CISA KEV with a patch due today; package.json:30-32 currently reads `>=22.12.0`, matching "raised from".
- Compatibility git row now says `git ls-tree` (was `cat-file`) — agrees with rule 4 after the rewrite.
- Failure-classification, retry wiring, rollback, and sweep-interaction text survived the rewrap untouched; Revision log entries match the revised text (validator rules 4-5, engines raise, git-floor claim).

## FYI (no severity)

- The engines raise to ≥22.23.4 is a real, security-motivated change but is scope beyond ADR-005 items 1-4 and is not pinned by any rollout step or test; whether it belongs in this run's branch is a complexity/editorial call, not a correctness one. The factual claim behind it is accurate.
