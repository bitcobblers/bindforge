# correctness review r1 (ratification pass c3)

verdict: approve

Scope per caller: `git diff b12a4dd..bc928d9` of `02-proposal.md` (changes since the
r1 approval), then the full current document, plus docs/tasks/T-002-steam-windows-discovery.md
and the Revision log. Ratification standard applied: report only what the revisions broke
of what was approved, or what they made false; my two prior findings re-checked first.
Spot-checked code with read-only greps: `absolute_env` still gates on `PathBuf::is_absolute`
(crates/discovery/src/steam_linux.rs:338-341) as both my prior findings described; the current
CLI roots branch (apps/discovery-cli/src/main.rs:33-38) is the one-plugin shape §8's revised
sentence ("keeps T-001's one-plugin shape") describes, so revised §8 is true and #9/#10 remain
its Linux gates.

Diff-by-diff assessment:
- §3 close ("Which HKLM views to ask, and in what order…") — agrees with §1's three-variant
  `RegistryKey`, §4's one-(key,view)-per-call seam, and §5's 32-then-64 ordering. Improvement.
- §5 candidate rule — reworded; substance identical (env defaults appended, `absolute_env`
  drop, no normalization here, canonical-path rule collapses). Dropping "its Windows result
  is a `\\?\` path" from §5 loses nothing: §7 still states it.
- §8 / #13 / *Rollout* #5 — the new mutual naming (#13 as the lone run-gate for the
  `#[cfg(windows)]` roots branch, cited at :65-67 and :72-74) matches the task's Done-when
  lines exactly and is internally consistent across the three sections. Closing the previously
  unclaimed execution gate is a strengthening, not a break.
- Revision log — both updated r1 entries match the current text (§5 view ordering pinned in
  #3; #12 type-checks vs #13 runs).
- Checklist items 2 (research-URL spot-check) and 7 (prior-round blocker/major closure): not
  applicable on a ratification scope — the diff adds no citations and the prior round had no
  blocker/major findings.

## Findings
- [minor][re-raised] §5 Root candidates — "a relative value is ignored by `absolute_env` (§1)" read with Test strategy #3 — the revision rewrote the sentence but the verification gap is unchanged and the Revision log records no decision on it: the gate's predicate is `is_absolute` (steam_linux.rs:341), host-dependent per §6, so the Linux-hosted fake can only supply POSIX-absolute values and the Windows half of the rule (a `C:\…`-shaped value kept, a Windows-relative one dropped) runs on no host. Not a break introduced by this revision; carried, still should be fixed. Fix (unchanged): one sentence stating the #3 fake supplies POSIX-absolute values and Windows-absolute coverage sits with #12/#13; or generalize the predicate over `SteamTarget` beside `install_dir_rejects_backslash`.
- [minor][re-raised] Test strategy #2 — "the doubled form `foo\\bar` as a real Windows ACF writes it (A2)" — text untouched by this diff, so it still asserts as fact the spelling A1b marks `unverified` ("the design assumes no spelling") and cites A2, which is the unescape behavior, not a source on what ACFs write. The register-side closure (revision log entry "r1: A1's backslash spelling (split out as A1b)") did not propagate to this sentence. Fix (unchanged): reword to "the doubled form a Windows writer may emit (A1b, spelling unverified)", enumerated cases unchanged.
- [nit][new] Test strategy #3 — "both views answering (one path from both → one root after the canonical-path rule, distinct paths → two roots and rows from two installs)" — the sub-outcomes named here are produced by `from_registry_and_roots`' filesystem pass (§3), not by the pure `host_candidates` list this item's header ("pure, fake registry + fake env") scopes; "rows from two installs" additionally needs two fixture trees no test item owns. Fix: scope #3 to the candidate list ("two distinct candidates, ordered") and attach the rows-from-two-installs outcome to #4's `from_registry_and_roots` harness, or drop it.

## FYI (no severity)
- §5 "Registry values arrive in mixed case and either slash form (A3, A4)": A3 supports the HKCU lowercase/forward-slash spelling and A4 the view-flag mechanics; neither row states a case or slash spelling for HKLM `InstallPath`. The claim is weaker than the pre-diff sentence and design-neutral (no normalization either way), but a compatibility seat may want the research backing checked.
