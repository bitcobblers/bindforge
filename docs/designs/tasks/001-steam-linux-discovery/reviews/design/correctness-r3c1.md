# correctness review r3c1 (dissent pass over b08c558 → 61907ce)

verdict: approve

Scope: the sections changed since my r3 review (`git diff` of the two proposal
blobs), then the full current document. My six r3 findings were checked first,
against the changed text; then the changed text was checked for closures that
broke something else. Inputs used: `02-proposal.md`,
`docs/tasks/T-001-steam-linux-discovery.md`, the Revision log, my r3 findings,
and read-only repo inspection (`rust-toolchain.toml`, `crates/discovery/Cargo.toml`,
`apps/discovery-cli/src/main.rs`, `crates/discovery/src/engine.rs`,
`schemas/reach-classes/v1/spec.md`) plus web verification of the re-cited A9
basis (https://doc.rust-lang.org/1.85.0/std/env/fn.home_dir.html shows
"Deprecated since 1.29.0" and "Returns the value of 'HOME' … if it is set
(including to an empty string)"; releases.rs 1.87.0 notes list "Undeprecate
`env::home_dir`").

## Closure check (my r3 findings)

1. **[blocker] library-level contradiction (§2/§3/§9/#3/#6/#8) — CLOSED, for real.**
   §3 now states one level: "`<root>` itself, always — a library is the *parent*
   of its `steamapps/` (A2), so a root is a library"; §2's root gate
   (`<candidate>/steamapps` is a directory) is what guarantees the root's own
   scan; the fixture's `"0" → ".."` joined to the VDF's directory
   (`<root>/steamapps`) resolves to `<root>` and de-dupes against the
   always-included entry; `"1" → "../secondary"` resolves to `<root>/secondary`,
   matching the fixture tree; manifests are read at `<library>/steamapps/`, so
   the root library yields 3 games and secondary 1 — four survive (#8); #3's
   "a root whose `libraryfolders.vdf` is absent still yields the root itself"
   and Risks row 2 now hold. All cross-referenced sections moved together.
2. **[major] `home_dir` deprecation on the pinned 1.85.0 floor — CLOSED.**
   §2 reads `std::env::var_os("HOME")` and never calls `home_dir()`; A9 is
   re-cited to the 1.85.0 docs page and its claims verified true there
   (deprecation banner; empty-`HOME` → `Some("")`); "removed only in 1.87.0"
   verified against the 1.87.0 release notes; §10's "no `deprecated` lint to
   work around" follows; Test #5 adds the empty/relative-home case.
3. **[major] §7 trigger vs Test #7 — CLOSED.** The trigger is now "at least one
   root was selected, **zero** games were emitted, and at least one file was
   skipped as unreadable or malformed", which makes Test #7's all-unparseable
   fixture the `Err` side and keeps the `Ok(vec![])` side (no roots; Steam with
   no games → zero skipped files). The message already counted skipped files.
   Risks row 4's "pins both sides" is now implemented. No new contradiction:
   the first paragraph's skip-and-continue list still describes the skips that
   the trigger then counts.
4. **[minor] `from_host()` blanket sentence — CLOSED.** §1 now reads "Every
   `pub` item above except `from_host()` …", pointing at Test #5 for the
   candidate list and Rollout #5 (a real-machine `discovery list` run) for the
   probe; consistent with Risks row 5.
5. **[minor] cross-root library de-dup — CLOSED.** §2's canonical path rule now
   scopes libraries "of all selected roots as one set per `discover()` call"
   and says "no library is ever scanned twice, including by two roots sharing
   one"; Test #3's per-root `libraries_in_root` assertions are unaffected.
6. **[nit] `DummyPlugin` import removal — PARTIALLY CLOSED**, see the single
   open finding below.

## Findings

- [nit][re-raised] §8 CLI wiring — "its `DiscoveryPlugin` import goes with it
  rather than being re-imported" — the revised sentence covers only one of the
  three imports `DummyPlugin` solely uses: `apps/discovery-cli/src/main.rs`
  names `DiscoveredGame` (line 3) and `DiscoveryError` (line 5) only inside
  `DummyPlugin`'s `fn discover` (line 30), and `register_plugin`'s
  `Box<dyn DiscoveryPlugin>` argument (`crates/discovery/src/engine.rs:16`)
  needs no import at the call site — so implementing the sentence as written
  and touching nothing else leaves two unused imports, which
  `cargo clippy --workspace --all-targets -- -D warnings` (A18, Test #10)
  turns into a build failure. The compiler forces the fix, so no design
  decision is at stake. Fix: widen the clause — "`DiscoveredGame` and
  `DiscoveryError`, which its `discover` signature is the only other user of,
  go with it too."

## FYI (no severity)

- A9's parenthetical "(removed only in 1.87.0)" is true (releases.rs 1.87.0:
  "Undeprecate `env::home_dir`") but the cited basis — the 1.85.0 docs page —
  supports only the deprecation and the empty-`HOME` behavior; a compatibility
  or documentation seat may want the release-notes citation appended.
- Reading `HOME` via `var_os` drops `home_dir`'s passwd fallback (unset
  `HOME` now means "no home directory" → `Ok(vec![])`); the design states this
  consistently in §2, the Compatibility matrix row, and Test #5, so it reads
  as a deliberate, documented choice.
