# Complexity review r1 (pass c0, full panel)

Scope: the whole of `02-proposal.md`. Inputs: the proposal, `docs/adrs/017-keybinding-extraction-codecs.md`,
the proposal's Revision log, and read-only inspection of the code the proposal cites.
No previous findings for this seat.

verdict: approve

## Checklist walk

1. **Reuse before new parts** — checked `tools/orchestration` (Node pipeline CLI: nothing reusable for
   codecs) and `crates/`. The capped-read posture in `src/read.rs` is *not* reusable from
   `crates/discovery`: `read_text_file` is private (`crates/discovery/src/steam_common.rs:332`) and
   `mod steam_common;` is a private module (`crates/discovery/src/lib.rs:6`), and the repo's precedent is
   per-crate restatement of a budget (`keyvalues.rs:16-21`). No reuse finding. `escape_and_truncate`
   (`crates/core/src/error.rs:190`), `MAX_SLUG_OCTETS`/`MAX_DISCOVERY_SLUG_OCTETS`
   (`crates/core/src/placement.rs:83,95`) and `MAX_NATIVE_NAME_OCTETS` (`crates/core/src/manifest.rs:58`)
   all exist and the design reuses them rather than minting new bounds. The env-lookup-closure shape it
   mirrors is real (`steam_linux.rs:92`, `steam_common.rs:385`).
2. **Every moving part → a stated Goal / ADR unit** — the crate, trait, `Detection`, registry,
   `GameContext`, resolvers, budgets, contract, family, CLI face and statuses all carry unit handles
   (`017-C1`–`C5`, `L1`–`L5`, `B1`–`B3`, `S1`). Two parts have no consumer inside the run: the
   `013-R2` fingerprint paragraph (finding 2 below is *not* about it — see the value note) and
   `GameContext.platform` (finding 5). The dead `appids()` pass and the published
   `family_ids`/`is_known_family` are Revision-log decisions with recorded bases; not re-raised.
3. **30–50 % smaller design** — one section has one: §Discovery plumbing (finding 1). The JSON choice,
   which is what drives the two harness carve-outs and the README amendment, is essentially forced: the
   documented subset refuses table headers and dotted keys (`schemas/README.md:61-69`) and `017-C4` says
   `--json` prints the snapshot document verbatim. I found no smaller shape for the contract, the family
   or the CLI face.
4. **New dependencies** — no new external crate: `crates/*` glob exists (`Cargo.toml:12`),
   `bindforge-core` is a workspace dep, `serde_json` is already in `Cargo.lock:317`. One imprecision in
   how the run pins that claim (finding 4).
5. **Defer half so the first slice stays thin** — Rollout 1–7 are ordered ADR-thin and each lands
   separately; nothing in the second half blocks the first.
6. **Knobs with no consumer / one-implementation abstractions** — `TreeKind`'s three kinds are all
   returned; the two passes are `017-C5`-mandated; the `env` closure has test consumers. Two items:
   `platform` (finding 5) and the contract-side `user` enumeration (finding 2).
7. **Value** — see the closing paragraph.

## Findings

- [minor][new] §Discovery plumbing (`crates/discovery`) — "`src/steam_common.rs`: `SteamCore::scan()` returns `Vec<ScanRecord>`; `discover()` becomes `Ok(self.scan()?.into_iter().map(|r| r.game).collect())`" —
  the chosen seam threads per-row root ownership through the code path that produces `list`'s byte-pinned
  rows, and that single choice is what generates the run's only High-impact compatibility risk ("The
  `steam_common` refactor changes what `list` prints"), Test strategy 1, Rollout 1's re-pin caveat and a
  new cross-crate public type that `crates/discovery/tests/zz_api_probe.rs` does not yet name. A smaller
  shape reaches the same goals: leave `discover()` exactly as it is (`crates/discovery/src/steam_common.rs:90-130`)
  and add one read-only inherent accessor per Steam plugin returning the two sets the scan already
  computes — the canonicalized roots held on `SteamCore` and the libraries from the existing
  `libraries_in_root_with_skips` (`steam_common.rs:137`, reachable only through the new accessor because
  `steam_common` is a private module) — and let `GameContext` carry `roots: Vec<PathBuf>` /
  `libraries: Vec<PathBuf>` with `user_trees` de-duplicating resolved trees by path. That is closer to
  `017-C2`'s own words ("every steam root the scan resolved — main root and library roots separately"),
  which the shipped single-owning-root context does not carry, and it deletes `ScanRecord`, the
  ownership-preserving rewrite of the shared `canonical_paths` dedupe, the two wrapper methods, Test 1
  and the `list`-output risk row. Cost: `user_trees` may see the same tree through two anchors, handled
  by the existing path de-dup and `MAX_USER_DIRS_PER_GAME`. Fix: replace the `ScanRecord`/`scan_records()`
  bullet pair with the accessor + set-valued `GameContext`, and cite `017-C2`'s "main root and library
  roots separately" as satisfied literally rather than by ownership inference.
- [minor][new] §`bindings/v1` Bounds — "`user` one of the three spellings above" —
  the contract validates a *launcher-context vocabulary* (1–20 digits | `steamuser` | `host`), i.e. the
  resolvers' internal knowledge becomes a format rule, so `017-R2`'s Windows contexts — whose user
  directory is not `steamuser` (A17) — require a `bindings` major rather than a new resolver, and the
  loader carries a rule no Goal asks for. Test strategy 10's "a non-digit `user` under a userdata-shaped
  label" makes the coupling concrete: the document carries no tree-kind key, so the loader can only
  enforce a pattern union invented here. Fix: state `user` as an opaque label under the bounds every
  other string already rides (`1`–`MAX_NATIVE_NAME_OCTETS`, no control characters, no leading/trailing
  space) and keep the three spellings as prose about the resolvers, deleting the enumeration and that
  one invalid fixture.
- [minor][new] §Read discipline and budgets — "`MAX_USER_DIRS_PER_GAME = 16` … `MAX_LOCATE_DEPTH = 8` … design-time" —
  these are the run's only two budgets with no ADR unit (the proposal says as much), and nothing in the
  document establishes the value: the layouts they must cover are `unverified` in the register (A15, A17),
  no surveyed layout depth or per-machine SteamID count is recorded, and the locate walk's reach is
  bounded purely by a number. The property that cannot be established from the text is that `8` is at
  least the depth of the deepest layout the family claims to resolve — which is the whole reason the walk
  is generic instead of template-shaped. Fix: one sentence naming the deepest surveyed layout path
  (native, Proton, userdata) beside each constant, and a locate-walk test at `surveyed_depth + 1`; keep
  the constants, ground them.
- [minor][new] §Test strategy 16 and Rollout 9 — "and the workspace `Cargo.lock` is unchanged" / "any `Cargo.lock` change is a finding" —
  as worded this pin cannot pass: adding `crates/codecs` to the workspace necessarily changes
  `Cargo.lock` (a new `[[package]] bindforge-codecs` entry plus new dependency edges from it and from
  `apps/discovery-cli`), and `serde_json` is not in `[workspace.dependencies]` (`Cargo.toml:20-24`), so the
  new crate declares it directly. The substantive claim (A12: no new external crate, no version change)
  is true and I confirmed `serde_json` is already resolved (`Cargo.lock:317`). Fix: restate both places
  as "the only `Cargo.lock` diff is the new member's own entry and its edges — no new external package and
  no version change", so the gate asserts the property that was meant.
- [nit][new] §`GameContext` — "`pub platform: discovery::game::Platform,`" —
  no rule in the design reads it: the three tree templates are anchored on `root`/`library`/env, and the
  Windows `claim-refused` answer comes from `from_record`'s digit-appid gate plus no located trees, not
  from the platform. Fix: name the clause that consumes it, or drop the field until `017-R2` needs it
  (`017-C2` asks for the row's identity parts, not its platform).
- [nit][new] §The CLI face — "the shared table renderer, generalized from `COLUMNS.len()` to a `&[String]` header so `list`'s five columns and their pinned widths are unchanged" next to item 6's "`discovery list` is untouched" —
  the two sentences describe editing the renderer `list` renders through (`apps/discovery-cli/src/main.rs:377-420`,
  where `Row`/`Widths` are fixed-size arrays keyed to `COLUMNS`) while claiming `list` is untouched. The
  generalization is the smaller option over a second width algorithm; the text just has to stop claiming
  the file is untouched. Fix: say "list's rendering code is refactored to slice widths with byte-identical
  output, pinned by `cli_output_polish_tests`/`cli_json_mode_tests`" instead.

## FYI (no severity)

- §Config-path resolvers: `SteamUserdata` and `ProtonPrefix` templates both carry `ctx.appid`, while
  `NativeHost` is the host's whole config root; a grammar-driven walk from a root that is shared by every
  application on the machine can locate another title's `Saved/Config` INI and report it under this game's
  lookup key, and the single-game `native-linux` fixture would not show it. That is correctness's and
  security's call, not this seat's — flagging so it is not lost.
- Contract-tree amendment 3 defines the hashed bytes as "the located files' octets concatenated in the
  snapshot's sorted `provenance.files` order, with no separator and no path or length octets" — a
  concatenation with no boundaries is collision-prone across file seams, and nothing in this run computes
  it to find out. Correctness's.
- §Corpus boundary and Test strategy 15 say the sweep "skips exactly `schemas/bindings/`" while the rule's
  home text is "skips `data/catalog/` and `schemas/catalog/` and nothing else"
  (`schemas/README.md:85-88`); amendment 1 needs to state the enlarged set in the README sentence too.
  Correctness/compatibility's.
- The aggregate budget sentence's worst case (512 MiB read for one game) is a performance-seat number;
  this seat only confirms the constants feeding it are all consumed.
- The `013-R2` fingerprint paragraph and the published `family_ids`/`is_known_family` ship with no
  in-run consumer. Both discharges are ADR-017-reserved reservations (`docs/adrs/013-game-manifests.md:155-159`)
  and both are Revision-log decisions, so they are not findings here — noted only so the ledger seat can
  see the cost is one spec paragraph and two functions.

## Value check

Proportionate. The design's size is largely dictated by ADR-017: five contracts, five laws, three
budgets and one seed family, each of which shows up exactly once as a part. The genuinely self-added
machinery is small — two extra budgets, two harness carve-out widenings, one JSON format the TOML subset
leaves no alternative to, and one dead-but-`017-C5`-mandated appid pass. The one place where the design
is bigger than what it needs is the discovery plumbing (finding 1), and it is also the only place where it
edits a byte-pinned path; the remaining findings are precision and grounding, not scale. Nothing here
blocks implementation.

## Process note

One unscoped repo-wide grep (`JSON_FIXTURE_FORMAT|SOURCE_KEY_TREE|…`) returned path and text matches from
this run's `reviews/design/` and `transcripts/` trees before I re-ran the search scoped to `crates/`,
`apps/`, `schemas/` and `Cargo.lock`. No finding above was derived from that text: each cites the proposal,
ADR-017, or code in `crates/`, `apps/`, `schemas/` and `Cargo.lock`, all read directly. Shell access beyond
`git diff*` was denied to this session, so no cargo/fmt/npm leg was run; every code claim I relied on was
read from source rather than executed, and the two claims that would have needed a build (`serde_json`
already resolved, `crates/*` glob) are cited to `Cargo.lock:317` and `Cargo.toml:12`.
