# Code review — correctness seat, round 1 dissent pass c1

- Run: 001-steam-linux-discovery (task flow)
- Pass: c1 (dissent; last revision written to answer correctness r1c0 findings)
- Scope: `git diff 752fd5abc9e4e88b57a98ef1c23c31daa3983c13..HEAD -- ':(exclude)docs/designs'`
  — one hunk: `crates/discovery/src/steam_linux.rs` (§7 error guard + message wording,
  commit `77e24c72`), read against the full current `02-proposal.md`,
  `docs/tasks/T-001-steam-linux-discovery.md`, and the runner's gate record (green:
  `npm run check`, `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace --locked`).
- Inputs honored: no review reports or transcripts opened; `03-build-notes.md` is not a
  permitted input for this seat, so its declaration posture is recorded as unverified.

## Previous-round findings — status

1. **major — `Platform` serde `rename_all = "kebab-case"` (crates/discovery/src/game.rs:8)
   UNADDRESSED.** `git show main:crates/discovery/src/game.rs` has no `#[serde(...)]`
   attribute on `Platform`; the branch still adds it, and the dissent diff (the only
   fix in range, `77e24c72`) does not touch `game.rs`, `cli_tests.rs`, or the proposal.
   The current proposal is likewise unamended: Context (02-proposal.md:14) still says
   "no change to `DiscoveredGame`, `Platform`, …", Non-goals (:35) still forbid any change
   to `DiscoveredGame`, and §8 (:262) still claims "the table header and JSON shape are
   unchanged"; the Revision log (:412-437) carries only r0 entries and no authorization
   of a serialization change. `apps/discovery-cli/tests/cli_tests.rs:121` still hard-codes
   `assert_eq!(obj["platform"], "steam-linux")`, so the coupled test still passes only
   because of the forbidden attribute: the serialized `DiscoveredGame.platform` value
   differs from `main`'s serde default (`"SteamLinux"` → `"steam-linux"`), a user-visible
   JSON change the proposal guaranteed not to happen. Severity rationale unchanged from
   r1c0 (major, not blocker: no live consumer regression — `DummyPlugin` emitted nothing —
   and ADR-022:34 plus the `Display` impl already use kebab-case, so it is a defensible
   alignment if declared and authorized; whether `03-build-notes.md` declares it cannot be
   checked from the permitted inputs, so the checklist item-3 blocker path is not asserted).
   Fix (unchanged): revert the serde attribute and assert `"SteamLinux"` in the CLI test
   to keep T-001 purely additive, OR amend proposal §8/Context/Non-goals to declare and
   authorize the kebab-case serialization and record the deviation in build notes —
   a proposal amendment plus build-notes declaration satisfies it; silence does not.
2. **nit — `tools/orchestration/src/pipeline/flows.ts:92` 12-min light stage timeout and
   agent transcript denies: RESOLVED.** The 12-minute light-flow `stageTimeoutMs` is the
   documented value (docs/workflows.md:341 records `stageTimeoutMs` = 12 min for the
   light flow), and commit `b2adcc9c` ("restore the ADR-016 transcripts fence and the
   documented 12-min light stage timeout") states the harness posture as a restoration.
   Finding withdrawn.
3. **nit — interior-NUL refusal in `usable_install_dir`
   (crates/discovery/src/steam_linux.rs:275-282): OPEN (not gate-blocking).** Unchanged
   in this diff; the added comment (:272-274) now argues the refusal by §5's `..` rationale.
   Still a small extension beyond §5's stated rejection set; harmless, leave as-is or fold
   into §5 later.

## This pass's changed code (steam_linux.rs:73-89)

- The guard `!self.roots.is_empty() && games.is_empty() && skipped > 0` spells §7's three
  clauses (proposal :244) verbatim and is behavior-identical to the base two-clause guard
  (`skipped > 0` is unreachable with zero roots: the counter only increments inside the
  per-root scan, :61-71). Correct.
- The message rename "file(s) skipped" → "entries skipped" is honest: `skipped` also
  counts a failed `read_dir` of a library's `steamapps/` (:201-207) and unreadable
  directory entries (:209-212), not only files. §7's "count of skipped files" and the
  trigger's "at least one file was skipped" (proposal :244-246) predate this diff and
  already disagreed with the counter's scope; this change does not make that divergence
  true — it only names it (FYI, no severity: one-line proposal fix would align §7's
  "file(s)" wording with the counted "entries", since §7's own skip list already includes
  non-files like "an unreadable library").
- Test strategy #7's test (steam_linux_tests.rs:463-492) pins only "names the selected
  root" and "no file contents", neither of which the rename disturbs; the green gate
  (`cargo test --workspace --locked`, exit 0) confirms.
- No new correctness defect introduced by this hunk.

## Checklist

1. Components/interfaces: previously verified (r1c0); this diff adds none. Pass.
2. Assumption register: no assumption silently changed by this diff. Pass.
3. Deviations declared in `03-build-notes.md`: **evidence missing** — not a permitted
   input for this seat; the finding-1 deviation is at minimum undeclared in the proposal
   itself. Not escalated to blocker only for the reasons in finding 1.
4. Scope creep: finding 1 stands; nothing new crept in this diff.
5. `03-build-notes.md` claims: not verifiable from permitted inputs (recorded).
6. Error paths fail closed: §7 path verified above; `Ok(vec![])` for no-roots re-confirmed
   at steam_linux_tests.rs:451-458. Pass.
7. Previous blocker/major addressed: **no** — finding 1 open.

## Verdict

**request-changes** — one open major (finding 1), unaddressed by the fix in range.
