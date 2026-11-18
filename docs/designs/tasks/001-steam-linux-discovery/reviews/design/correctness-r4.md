# correctness review r4 (editorial confirmation pass, scope: reviews/design/editorial-r4.diff)

verdict: approve

## Scope and method

Ratification-style pass: read only the named diff, `02-proposal.md`,
`docs/tasks/T-001-steam-linux-discovery.md` and the proposal's Revision log; no other run record was
opened. Previous-round correctness findings: none (editorial phase is one language pass per round),
so the test applied is: does any changed line break something previously approved, or make a cited
section false? Read-only checks run: `git diff HEAD` on the proposal (empty — the file on disk is the
post-diff state), grep confirmation that each hunk's `+` side is what the proposal now carries, and
webfetch spot-checks of two assumption bases touched by the diff's hunks (A9's
https://doc.rust-lang.org/1.85.0/std/env/fn.home_dir.html and A2/A3's
https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs).

## Checklist walk

1. Assumption register — the only register row the diff touches is A9, changed solely by inserting a
   comma after "so". The cited basis supports the row's operative claims: the 1.85.0 page shows
   `home_dir` deprecated and, on Unix, "Returns the value of the 'HOME' environment variable if it is
   set (including to an empty string)", which is what backs the design's `var_os("HOME")` choice and
   the unset/empty ⇒ `Ok(vec![])` rule (§2, §7). No `verified` row was reworded into something its
   basis no longer supports.
2. URL spot-check — the two bases checked resolve and support the claims. A2 ("`<library>/steamapps/
   common/<installdir>`") is `resolve_app_dir` verbatim; A3's "keeps only blocks whose key parses as a
   number, each carrying `path`" is `filter(|(key, _)| key.parse::<u32>().is_ok())` then `get("path")`;
   and the same file's comment "values in `libraryfolders.vdf` may be stale" backs the *Options*
   rejection that the diff's Context hunk ("settles each such question") still relies on.
3. Internal contradictions — none introduced. §2's "`from_host()` is a thin wrapper over
   `from_roots()`" matches the code sketch in §1 (`from_host` / `from_roots` constructors, plugin holds
   roots because `discover(&self)` takes no arguments, A17) and §8's `is_empty()`-based dispatch. The
   replaced "wrapper over it" was not pinning anything else.
4. Edge cases — no edge-case text was added, removed, or reworded; the failure-policy, path-validation
   and de-duping rules the diff's context lines carry are byte-identical in their substance.
5. Interfaces — no name or shape changed: `SteamLinuxPlugin`, `from_roots`, `host_candidates`,
   `libraries_in_root`, `parse_manifest`, `--steam-root` appear once each in the same shapes; the
   Americanization "behaviour" → "behavior" removed the only spelling drift of that word and matches
   the document's dominant spelling.
6. Source-document coverage — unchanged by the diff; the task's four "Done when" clauses are still
   answered by *Goals*, §1–§8, *Test strategy* and *Rollout* #5.
7. Revision-round closure — the diff is the r4 editorial pass itself; nothing previously closed was
   reopened. The Revision log's entries still describe the document as it stands.

## Findings

None. Every hunk is additive punctuation (serial commas in Context and *Test strategy* #10, the
"A9 … so," comma), a like-for-like spelling normalization, a tense change ("say more than `new()`
does" → "would") that preserves the clippy `new_without_default` argument, a referent-expanding
"each one" → "each such question" consistent with the *Deferred* list, and an antecedent-clarifying
split of §2's sentence that now names `from_roots()` — a referent consistent with §1 and §8. No
changed line breaks an approved section or makes a cited section false.

## FYI (no severity)

- A9's parenthetical "`std::env::home_dir()` is `#[deprecated]` (removed only in 1.87.0)" cites the
  1.85.0 page, which shows the deprecation dating from 1.29.0 and says nothing about the 1.87.0
  removal; the operative claim (deprecated on the pinned 1.85 floor, so `var_os("HOME")` is used)
  holds, and the parenthetical sits on a context line of this diff, not in its changed text — flag it
  to a language/consistency seat only if a future round already touches that row.
