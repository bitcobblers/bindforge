# Code review — security seat, round 1 pass c3 (ratification)

- Change under review: `git diff 752fd5abc9e4e88b57a98ef1c23c31daa3983c13..HEAD -- ':(exclude)docs/designs'`
- Design: `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md`
- Pass type: ratification. Scope: only the three post-approval commits
  (`77e24c72` §7 guard clause, `752096e9` revert of the unauthorized
  `Platform` serde rename, `c9fb37b7` CLI test expectation). Question asked:
  did they break anything this seat approved at r1?
- Inputs honored: the diff, the proposal, `docs/tasks/T-001-steam-linux-discovery.md`,
  the runner's green gate record, this seat's own r1 findings. No other
  `reviews/` content opened.

## Changes reviewed

1. `crates/discovery/src/game.rs:5-9` — `#[serde(rename_all = "kebab-case")]`
   and its doc note removed from `Platform`. The serialized variant spelling
   returns to the serde default `SteamLinux`; `Display` (game.rs:11-18) keeps
   printing `steam-linux` and `Plugin::name()` keeps returning `"steam-linux"`.
   Grep over `crates`/`apps` shows no in-tree consumer parses the serialized
   variant name expecting kebab-case; only the CLI JSON test (below) asserts
   it. This restores the proposal's Context pledge ("no change to
   `DiscoveredGame`, `Platform` ...") and §8 ("the JSON shape are unchanged"),
   so the revert moves *toward* the design, not away from it. No security
   property is engaged: variant spelling in JSON is neither a secret nor an
   input the plugin re-consumes. Any ADR-022 kebab-vs-Pascal serialization
   tension predates this branch and is the compatibility seat's ground, not a
   regression introduced here.
2. `apps/discovery-cli/tests/cli_tests.rs:124` — test now asserts
   `obj["platform"] == "SteamLinux"`, matching the reverted shape; the JSON
   shape assertions (five fields, null `codec_id`) are untouched. Security-
   neutral test change; the green `cargo test --workspace --locked` leg of the
   gate record covers it.
3. `crates/discovery/src/steam_linux.rs:73-89` — §7 error guard gains the
   explicit first clause: `!self.roots.is_empty() && games.is_empty() &&
   skipped > 0` (was `games.is_empty() && skipped > 0`), with a comment
   restating §7 verbatim. This is behavior-neutral: every `*skipped += 1` sits
   inside `libraries_in_root_with_skips` (steam_linux.rs:148-175) or
   `scan_library` (steam_linux.rs:200-231), both reachable only under the same
   `!self.roots.is_empty()` condition that already gates the scan block
   (steam_linux.rs:61-71), so `skipped > 0` already implied non-empty roots.
   No previously-erroring scan can now silently return `Ok(vec![])`, and the
   error message — the selected roots plus the skip count, no file contents —
   is byte-identical to the r1-approved text, staying on the §7 stderr channel
   the proposal reserves for it. Test strategy #7's both-sides pins survive
   (steam_linux_tests.rs:454 nonexistent-root → `Ok(vec![])`;
   steam_linux_tests.rs:465 all-broken root → `Err`).

## Security checklist walk (references/security.md)

1. External input validated at boundary — unchanged by this diff: `keyvalues`
   caps and `usable_install_dir` (steam_linux.rs:275-282, incl. the NUL-byte
   refusal) untouched. N/A-to-delta.
2. Path handling — unchanged; no new path sources, canonical path rule intact.
3. Injection — unchanged; no shell/regex surface added.
4. Secrets/PII — §7 message wording unchanged from r1 approval; still names
   roots and a count only, on stderr; test comments leak nothing.
5. New dependencies — none in this diff; `Cargo.lock` posture unaffected.
6. Filesystem writes — none added.
7. Clean rejection of hostile input — §7 guard still fires exactly on its
   three clauses (now spelled in code); green gate record confirms the
   failure-policy tests pass.

## Diff to my r1 approved state

Nothing this diff touches weakens an approved security property. The one open
finding from this seat (minor) is carried forward: `read_text_file`
(steam_linux.rs:287-316) is untouched by these commits, so the pre-read
`metadata`-length cap, its TOCTOU window, and the missing `is_file()` gate on
the fixed `libraryfolders.vdf` leg (reachable at steam_linux.rs:170 for
zero-length-metadata special files) stand exactly as approved at r1, at the
same minor severity. The r1 fix suggestion (open the handle, verify
`metadata()?.is_file()`, `Read::take(MAX + 1)`, treat a limit-hit as an
oversize skip) still applies.

## Verdict

approve — zero blocker/major findings; later fixes are design-conforming
reverts or behavior-neutral hardening, and the green gate record
(`cargo test --workspace --locked`, clippy `-D warnings`, fmt, npm check, all
exit 0) covers every changed leg.
