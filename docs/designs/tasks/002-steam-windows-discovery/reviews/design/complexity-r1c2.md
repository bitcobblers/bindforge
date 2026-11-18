# complexity review r1 — dissent pass c2

verdict: approve

Scope: the diff b12a4dd..bc928d9 (§3 sentence, §5 env/normalization prose, §8 comment + gate naming, Test strategy #3 and #13, Rollout #5, Revision log) plus the full current document. Inputs: 02-proposal.md, docs/tasks/T-002-steam-windows-discovery.md, the Revision log, my r1 findings.

## Status of my r1 findings

- **[major — CLOSED, as fixed.]** §8's roots branch now names its lone gate exactly as asked: §8 says "the Windows branch is type-checked by #12 and run only by #13's `--steam-root` check, named as its lone gate in *Rollout* #5"; #13 enumerates "`discovery list --steam-root <non-default install>`" and "that the roots run yields that install's `steam-windows` rows — the only gate for §8's roots branch"; *Rollout* #5 names #13 for "§8's `#[cfg(windows)]` roots branch (:65-67)". Task :65-67 and :72-74 quoted by Rollout #5 match docs/tasks/T-002-steam-windows-discovery.md:65-67 and :72-74. No other gate runs the `#[cfg(windows)]` branch (#12 type-checks only; Rollout #3's #10 covers the probe branch on Linux), so "only gate" is accurate.
- **[minor — half closed; the residual is the one finding below.]** #3's "both keys present" expectation is now the five-entry ordered list consistent with §5's always-append rule ("HKCU `SteamPath`, then both HKLM views' `InstallPath` in §5's order, then the two env candidates") — fixed. The both-views consequences are now enumerated, but written as expectations the named pure test cannot produce (finding 1).
- **[nit — CLOSED.]** §3 now reads "Which HKLM views to ask, and in what order, is `host_candidates`' policy" — no first/branch reading left, no license for a short-circuit against #8's "exactly the three `RegistryKey` variants".

## Findings

- [minor][new] Test strategy #3 — "both views answering (one path from both → one root after the canonical-path rule, distinct paths → two roots and rows from two installs)" — the enumeration's other cases state expectations at the level of the named gate (a candidate list from a pure `host_candidates`: §5 "no filesystem access", and #3's own "pure, fake registry + fake env"), but this new case states its expectation as downstream results — "one root after the canonical-path rule" and "rows from two installs" are outputs of `canonical_paths`/`discover()`, which a pure `host_candidates` call cannot produce. Fixture reality compounds it: the only scannable install tree is `crates/discovery/tests/fixtures/steam/minimal` (with `all-broken` unscannable by design), so "rows from two installs" has no fixture behind it, and #5's "no new fixture tree is introduced" is the design's stated fixture posture. The revision itself shows the right level — it trimmed the registry==env case's analogous downstream clause ("but collapses to one scanned root after the canonical-path rule") to the pure form ("not deduped by `host_candidates` (§5)") in the same edit — and no existing Linux test pins the root-level two-equal-roots collapse (the dedup test at steam_linux_tests.rs:162 is library-level inside one root; no test passes `from_roots` two roots), so as written the collapse consequence my r1 minor asked to pin could end up asserted by nobody. Not a blocker: the intended behavior is unambiguous and backed by the BTreeSet in canonical_paths (crates/discovery/src/steam_linux.rs:322-334). Fix: state the case's expectation at #3's level — equal values from both views → both appended in §5's order; distinct values → both entries appended in §5's order — and pin the downstream consequence in #4, where the fake registry points both HKLM variants at the same `minimal` fixture root and the four games (not eight) come back; drop "rows from two installs" or replace it with the distinct-values candidate list rather than sanctioning a second install fixture #5 refuses.

## Checks run (read-only)

- `git diff b12a4dd..bc928d9` — every changed claim traced; Revision log r1 entries match the changed text.
- apps/discovery-cli/src/main.rs:33-38 — current CLI registers exactly one plugin with the roots branch as §8's "T-001's one-plugin shape" states; §8's replacement shape keeps the Linux roots branch `#[cfg(not(windows))]` one-plugin.
- apps/discovery-cli/tests/cli_tests.rs — exactly three tests (`test_cli_list_format`, `test_cli_list_json_format`, `test_cli_list_json_empty_for_missing_steam_root`); "exactly the four fixture rows" assertions at :37/:71/:91 back #9's untouched-passes claim.
- crates/discovery/src/steam_linux.rs:38-43 (`from_host` env-closure shape), :322-334 (canonical_paths canonicalize→require-steamapps→BTreeSet dedup), :336-342 (`absolute_env` set-and-absolute) — §5's changed sentences ("ignored by `absolute_env` (§1)"; canonical-path rule as the reconciler) match the code and §1's move list.
- docs/tasks/T-002-steam-windows-discovery.md:65-67, :72-74 — Rollout #5's line citations are correct.

## Checklist posture (complexity)

No new package/module/interface/dependency introduced by this revision; nothing here grows the design. The r1 proportionality verdict stands: `SteamCore<T>`/`SteamTarget` and the `&dyn Fn` seam trace to the task's hard reuse requirement (:22-35) and seam requirement (:45-51); the abstraction's two implementations and one registry consumer are the minimum the task's rules ("`SteamWindowsPlugin`… sharing the scan core", Linux "must not change") can be met with. The single finding is a test-enumeration level mismatch, not added machinery.

## FYI (no severity)

- §5 — "Registry values arrive in mixed case and either slash form (A3, A4)" — the softening from "lowercase with forward slashes or backslash-delimited" is directionally right after A1b's dead-citation lesson, but A3 documents only HKCU's lowercase/forward-slash spelling and A4's basis (the redirector docs) says nothing about either value's case or slash form; the claim as stated still rests on register entries that don't carry it. The design behavior is spelling-agnostic either way, so it's a sourcing-strictness question for the sources/correctness seats, not a design defect.
