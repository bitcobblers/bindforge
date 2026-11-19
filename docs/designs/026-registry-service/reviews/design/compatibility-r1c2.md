# Compatibility review r1 (dissent pass c2)

verdict: approve

Scope: `git diff 9ff2ec78 07e8b3f8` on `02-proposal.md`, then the full current
document. Inputs: the proposal, `docs/adrs/026-registry-service.md` (with
`025-C1`/`025-B1` for the lookup-key rule), the Revision log, and this seat's
r1 findings. Read-only checks actually run (`cargo` legs; no write leg used):
`cargo test -p bindforge-core --locked --test conformance --test version_gate`
→ 9 + 11 passed; `cargo test -p discovery --locked --test boundary_tests` → 4
passed. `cargo fmt`/`clippy` and the npm legs were not needed for any claim
here. Supporting reads: `crates/discovery/src/game.rs:66-89`,
`apps/discovery-cli/src/matches.rs:40-105,222-234`,
`apps/discovery-cli/src/catalog.rs:172-196,228-271` and its `unreachable` sites,
`crates/core/tests/common/mod.rs:73-150,419-435`,
`crates/core/tests/conformance.rs:67-102,330-376,404-426`,
`crates/core/tests/version_gate.rs:359-404`,
`crates/core/tests/slug_bound_corpus.rs:155-219`,
`crates/core/tests/action_ids.rs:325-354`,
`crates/discovery/tests/boundary_tests.rs` (whole file),
`crates/discovery/Cargo.toml`.

## Status of this seat's r1 findings

1. **major — dot ban narrowing the admitted lookup-key vocabulary** — **closed
   in substance.** §The serving face step 1 now takes `spells_lookup_key` +
   `025-B1` as the rule, admits a dotted external id, names `is_slug_name`'s dot
   ban as `game_id`-only, and confines the extra refusals to path safety; the
   corpus refusal list, the cache-load clause and #3/#4 all follow the same
   rule-reference, and #3 now pins `local/Steam.SE` as admitted. One character
   class of the same defect survives in the new list — reported below as a minor
   residual, not a re-raise of the major.
2. **minor — the self-cancelling `fixture_file_names_…` guard** — **closed, as
   proposed.** Verified against the shipped harness: `fixtures()` filters nothing
   by suffix (`common/mod.rs:141-150`), so the un-guarded name test does see
   `.json` files, `is_fixture_name` hard-codes `.toml` (`common/mod.rs:421-425`),
   and its one call site is `conformance.rs:348` — the design's
   "the one call site is in the fixture-name test below" is accurate, and the
   directory-aware predicate is the minimum change that keeps the
   Windows case-uniqueness claim (#19) covered.
3. **minor — row 1 labeled "Linux & other Unix" against a `No` exotic-Unix row**
   — **closed.** Row 1 is `Linux`, the footer names (Linux CI, macOS, Windows)
   and adds "and no other Unix is". Every row now carries exactly one status;
   no empty cells, no `unknown` (checklist 1 clean).
4. **minor — re-scoping `the_shipped_corpus_carries_no_source_key_spelling`**
   — **closed, as proposed.** Verified against the shipped sweep
   (`boundary_tests.rs:110-156`): roots are `data/` (recursive) plus every
   `schemas/<format>/<major>/fixtures/valid`, `checked > 0` at the end, and the
   skip-only-catalog formulation is expressible there; the two neighbouring
   tests in that file use only `crates/discovery/tests/fixtures/steam/`, so
   they are unaffected. The Options rewrite is now accurate too: the rejected
   "not in `schemas/`" location dodges only the conformance walk.
5. **nit — #13's unnamed relocation seam** — **closed.** #13 names
   `XDG_DATA_HOME`/`APPDATA` in the child environment for #17 and states there
   is no path knob; both names match the shipped `data_dir`
   (`matches.rs:91-99`), which the design's "relative = unset" already covers.

Also verified in changed text, no finding: the five copied-invariant test
citations (#7/#14-15/#8/#13/#10-11) match the named suites; A10's
majors-equality claim holds (`version_gate.rs:392-404`,
`action_ids.rs:332-349` compare major-name sets, so `catalog/v1` is absorbed);
`the_walk_picks_up_every_version_directory_the_contract_tree_ships`
(`conformance.rs:404-426`) and `slug_bound_corpus.rs`'s fixed
`SLUG_BEARING_TREES` list are contains-checks, so a new `schemas/catalog/v1/`
costs them no edit — the design's harness file list is complete for the walks
that exist; and the `discovery` dependency's `winreg` edge is Windows-target-only
exactly as §Deferred and §`apps/registry` state
(`crates/discovery/Cargo.toml:38-39`).

## Findings

- [minor][re-raised] §The serving face step 1 (carried into §Catalog document
  and its spec, §Budgets' cache-load clause, and #1/#3/#4) — "Path safety is the
  only extra refusal, at its own scope: after the first `/`, one non-empty
  component with no further `/` or `\`, none equal to `.` or `..`, and no control
  byte, **space**, or invalid-UTF-8 sequence" — the space ban is not path safety,
  and the paragraph's own parenthetical says keys "never [are] joined onto
  paths", so it refuses a spelling no shipped rule refuses while the sentence
  above it claims "that shipped pair is the registry's whole key rule".
  `Source::spells_lookup_key` (`crates/discovery/src/game.rs:84-89`) admits any
  non-empty external id after a closed-set token, the single composer
  `Source::lookup_key` (game.rs:66-68) imposes nothing more, and the shipped
  store's only slug checks are cap + `spells_lookup_key`
  (`apps/discovery-cli/src/matches.rs:222-234`); the shipped client sends such a
  key on the wire (`percent_encode` escapes the space as `%20`, catalog.rs:174-185,
  A19). A space-bearing `025-C1` key — the `local/<dir>` external id, whose
  derivation `025-C1` states only as "the kebab'd install directory name", with
  no grammar — is therefore refused at all four seams this rule governs: candidate
  query (`404`), admin upsert (refused by name), corpus load, and cache load (one
  such slug refuses the whole cache document). The document is also internally
  split on the point: #3 admits the non-kebab'd `local/Steam.SE` (lenient reading
  of the derivation) while this clause refuses a space (strict reading), so T-005
  cannot tell which vocabulary it is pinning. The blast radius is far smaller
  than r1's dot case — no shipped emitter produces a space today — hence minor.
  Fix: delete `space` from the banned set, leaving "no control byte or
  invalid-UTF-8 sequence", so the extra refusals are exactly the named path-safety
  and line-integrity classes; no other text changes, since every dependent seam
  already points at this one clause. If a space ban is wanted on some other
  ground, keep it but say that in this paragraph and name it as a narrowing of
  the `025-C1`/`026-C3` key vocabulary, which no shipped spelling rule shares.

## Checklist

1. **Compatibility matrix** — every row carries a status (`Yes`/`Yes`/`Yes`/
   `No`); no `unknown`, no empty cell; footer names exactly the three `Yes` rows
   the tasks owe behavior to. Pass.
2. **OS input APIs (X11/Wayland/Raw Input/HID)** — N/A: the design ships no
   input surface; its only OS-facing code is `std::net` and the data-dir rule.
3. **Node version floor** — N/A: nothing in `tools/orchestration` changes.
4. **Native modules** — pure Rust; the one platform-scoped transitive
   (`winreg`, `cfg(windows)`) is named and priced to the deferred lift. Pass.
5. **Keyboard/name reality** — the residual above: the key vocabulary is the one
   place this design admits real-world non-alphanumeric names, and the space class
   is refused without a rule behind it. Non-ASCII is correctly admitted.
6. **File paths / per-OS data dirs** — `<data>` rule follows the shipped
   `StoreLayout`/`data_dir` closure (XDG_DATA_HOME → HOME, APPDATA, relative =
   unset), paths joined not concatenated, matrix names both documents. Pass.
7. **Platform test per supported platform** — #19 names Linux (cargo gates),
   Windows and macOS (the #13 env-closure seam plus the un-guarded case-unique
   fixture-name test, now genuinely un-guarded). Pass.

## FYI (no severity)

- For the performance/security seats, from the same §Budgets rewrite:
  `MAX_REQUEST_BODY_OCTETS` = `MAX_CATALOG_DOCUMENT_OCTETS` = 16,777,216 with
  `MAX_CONCURRENT_CONNECTIONS` = 64 bounds inbound buffering at ~1 GiB, and — per
  the §Concurrency change — every accepted upsert re-serializes and tmp-and-renames
  a document that may now be 16 MiB **inside the one model write lock** that
  serving reads need. The two decisions were made in separate sections and never
  priced together.
- For correctness: §Admin face and auth anchors the loopback judgment to "the
  test `probe()` applies to each resolved address (`catalog.rs:248`)", and that
  `is_loopback` half is deleted by `026-L1` in the same rollout (§CLI amendment:
  "`probe()` deletes the `is_loopback()` non-loopback-resolution refusal"). The
  citation is accurate against today's code and the text disclaims copying, so it
  is only a pointer that will point at removed code after `T-008`.
- §Test strategy #6 pins the stale-transition through `--refresh-interval 1s`
  while nothing in the design pins that flag's value grammar (bare seconds vs. a
  suffix), which is the part `T-007` must guess.
