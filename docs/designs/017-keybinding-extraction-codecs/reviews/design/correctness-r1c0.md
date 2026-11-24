# correctness review r1 (pass c0, full panel)

verdict: request-changes

Scope: whole document. Inputs: `02-proposal.md`, `docs/adrs/017-keybinding-extraction-codecs.md`,
the Revision log, and the code/contract files the assumption register cites. No other run
record was read.

## Basis checked (no finding)

Verified against the cited code/contract text, and supporting their rows as written: A1
(`plugin.rs:4-7`, `game.rs:115-126`), A2 (`steam_common.rs:90-131`, incl. the `BTreeSet`
canonicalize-and-dedupe in `canonical_paths`), A3 (`steam_linux.rs:92-128`), A4
(`keyvalues.rs:21` = 1 MiB, `steam_common.rs:329-361` pre-allocation length check + strict
`String::from_utf8` skip), A5 (`input.rs:70-121`: 26 letters + 10 digits + 11 punctuation +
`space` = 48, 7 mouse, 18 gamepad, and "modifiers and named keys are outside the closed set"
at :83), A6 (`manifests/v1/spec.md:56` codec bound, `:121` `013-B4` =
`MAX_NATIVE_NAME_OCTETS = 128`, README:145-160 puts `MAX_SLUG_OCTETS` at `024-B1` — the ADR's
`013-B4` citation for the codec bound is indeed a slip), A7 (`spec.md:157-160`), A8
(`spec.md:123-129`), A9 (`boundary_tests.rs:104-130`, one `SOURCE_KEY_TREE` const feeding both
skip roots), A10 (`common/mod.rs:426-445`, `conformance.rs:47-54`), A11/A13
(`main.rs:34-60,88-109,129-142`), A12 (both `Cargo.toml`s, `Cargo.toml:12` globs), A14
(`ADR-008` clause 2 is the workspace list, clause 3 is "Formats are the contract" — the row's
correction holds), A21 (`data/` holds exactly the four files named), A22
(`check.yml:25-59,61-81`). `bindforge_core::MAX_SLUG_OCTETS`, `escape_and_truncate`,
`MAX_DISCOVERY_SLUG_OCTETS` and `Source::spells_lookup_key` all exist as the design names them;
`supported_majors()` is the `[v1,v2]` union, so a third `v1` format needs no core change.

Webfetch spot-checks (anti-hallucination gate): the UE 4.27 Configuration Files page (A15)
supports `Saved/Config/[Platform]/[Category].ini`, `[/Script/Engine.PlayerInput]`,
`Bindings=(Name=…,Command=…)`, bottom-most-wins, and the `+`/`-`/`.` operators; PCGamingWiki
Glossary:Game_data (A17/A18) supports `<SteamLibrary>/steamapps/compatdata/<AppID>/pfx/` and
`~/.steam/steam/userdata/<user-id>/<AppID>/`. The IGA-file page (A18's first basis) is live but
is about Steam Input controller configs; it supports the userdata location only tangentially —
acceptable only because A18 is honestly marked `unverified`.

## Findings

- [major][new] Config-path resolvers — "`NativeHost`: `$XDG_CONFIG_HOME` when set and absolute,
  else `$HOME/.config`, with `user` the constant `host`" — this tree is not game-scoped, unlike
  `SteamUserdata` (`<ctx.root>/userdata/<dir>/<ctx.appid>`) and `ProtonPrefix`
  (`…/compatdata/<ctx.appid>/pfx/…`), which carry `<ctx.appid>` in the template. `detect` then
  "walk[s] each `UserTree` depth-first … collecting `*.ini` files whose path ends
  `Saved/Config/<one directory>/<name>.ini`" to `MAX_LOCATE_DEPTH = 8`, so from the shared host
  config root it locates **every** Unreal title's user config, and reports the first 32 (byte
  order) as this game's. Failure scenario: two UE titles installed, both writing
  `~/.config/<ProjectName>/Saved/Config/Linux/GameUserSettings.ini`; `discovery bindings
  steam/<appidA>` emits title B's actions and inputs under title A's lookup key. That violates
  `017-L3` ("A snapshot states only what a located file states") and falsifies §The first
  family's claim of "grammar-driven location inside a tree the context owns" — the context owns
  nothing about `$HOME/.config`. No fixture exposes it: `native-linux` carries one game.
  Fix: make the native tree game-scoped or drop it this run — either scope `NativeHost` to
  `<config>/<one directory>` whose name the record can justify (naming the text it replaces:
  the sentence "no UE project name is guessed"), or return no `NativeHost` tree until `017-R4`
  supplies project/appid routing and move the native context to Deferred, with
  `017-S1`'s "native … contexts" then recorded as a divergence in the Revision log.

- [major][new] `GameContext` (`017-C2`) — "pub root: PathBuf, // the owning root: `userdata/`
  lives here (A18)" — ADR-017 `017-C2` asks for "every steam root the scan resolved — main root
  and library roots separately, because `userdata/` lives only under the main root", and this
  context carries exactly one root: the library's owner ("the root itself, or the root whose
  `libraryfolders.vdf` named it"). The two are not the same object whenever more than one full
  install is in play, which A3 states as fact; then `SteamUserdata` is resolved against the
  wrong root and a game that does have `userdata/` answers
  `family-confirmed-files-absent`/no-claim. Two further defects in the same sentence: it states
  as fact what A18 — marked `unverified` — states about the *main* root, and this divergence
  from the source's wording is not among §Context's declared "Three clauses of the source …
  implemented differently than worded" nor in the Revision log. Fix: resolve `userdata/`
  against every root the scan resolved (union of `<root>/userdata/<dir>/<appid>` that exist,
  byte order) and delete the "userdata/ lives here" comment; or keep the single owning root and
  add the Revision-log entry recording the narrowing of `017-C2` plus its unit id to Rollout 8's
  ledger hand-off.

- [major][new] Config-path resolvers with The codec face and Determinism — "`env` is a lookup
  closure, mirroring `steam_linux::host_candidates` and `matches::store_path`, so every rule
  here is unit-tested without touching the process environment" — the closure has nowhere to
  come from in production: `detect` is pinned by `017-C1` as `fn detect(&self, ctx: &GameContext)
  -> Option<Detection>`, so the family must call `user_trees(ctx, &|k| std::env::var(k).ok())`
  itself and read `XDG_CONFIG_HOME`/`HOME` from the process. That is precisely what `017-L4`
  forbids — "nothing about wall-clock time, environment beyond the context, or registry
  iteration order can reach an artifact" — and §Determinism restates the law with that clause
  quietly missing ("Nothing reads wall-clock time, and nothing about registry iteration order or
  directory-return order reaches an artifact"), so the same `GameContext` and file contents
  yield different snapshots under different `XDG_CONFIG_HOME`. Test strategy #12 cannot expose
  it: it shuffles "the registry and the `UserTree` vector", never the environment. Fix: carry
  the resolved host config root in `GameContext` (`Option<PathBuf>`, filled by the CLI from the
  same env closure `store_path` is given at `main.rs:113`) and drop the `env` parameter from
  `user_trees` — that replaces the process-env read rather than adding a knob; or record the
  narrowing of `017-L4`'s environment clause in the Revision log and add the env-varied case to
  #12.

- [major][new] Test strategy #16 with Rollout 9 — "the workspace `Cargo.lock` is unchanged" and
  "any `Cargo.lock` change is a finding" — adding `crates/codecs` to the `crates/*` glob adds a
  `[[package]] name = "bindforge-codecs"` entry to the root lockfile: every current member is
  recorded there (`bindforge-core`, `bindforge-registry`, `bindforge-transforms`, `discovery`,
  `discovery-cli`), and `cargo test --workspace --locked` fails when the lockfile is stale. So
  the pin is unsatisfiable as written and the rollout instructs the reviewer to treat a
  mandatory change as a defect. §`crates/codecs` and the Compatibility matrix state it correctly
  as "no `Cargo.lock` **version** change"; the two absolute statements contradict them. Fix:
  restate #16 as "the lockfile's external package set and every resolved version are unchanged;
  the only delta is the `bindforge-codecs` entry and its path edges", and narrow Rollout 9's
  clause to "any change to an external dependency or its version is a finding".

- [minor][new] Read discipline and budgets — "one extraction opens at most
  `MAX_USER_DIRS_PER_GAME × MAX_CANDIDATE_FILES` files, so its worst-case read cost is
  `16 × 32 × MAX_BINDING_FILE_OCTETS` (512 MiB)" — `MAX_USER_DIRS_PER_GAME` bounds only
  "`userdata/` directories considered per game", while `user_trees` returns three template
  families: up to 16 `SteamUserdata` trees **plus** one `ProtonPrefix` plus one `NativeHost`,
  each with its own `MAX_CANDIDATE_FILES` located set (§The first family: "capped at
  `MAX_CANDIDATE_FILES` per tree"). The product the sentence ships to `schemas/README.md` is
  understated by two trees (512 files stated against 576 actual). Fix: name
  `MAX_USER_TREES_PER_GAME = 18` (or `MAX_USER_DIRS_PER_GAME + 2`), bound the resolvers with it,
  and state the aggregate as `MAX_USER_TREES_PER_GAME × MAX_CANDIDATE_FILES` = 576 MiB.

- [minor][new] The first family — "Accepted lines: `[Section]` headers, `Key=Value`,
  `+Key=Value`, `-Key=Value`, and `.Key=Value` as UE's documented line-continuation" — the
  source A15 cites documents four operators, not three: `+` adds if absent, `-` removes on exact
  match, `.` "adds a new property … like `+` except it will potentially add a duplicate line"
  (not a line-continuation, which is what Test strategy #7 then pins as "`.Key=` continuation
  joins"), and `!` "removes a property; but you do not have to have an exact match, just the
  name". A `!Bindings=(…)` line is therefore "out-of-grammar" and silently skipped, so the
  snapshot keeps reporting a binding the file removed — the silent drift this section says it
  refuses ("drift becomes loud rather than a silently shorter snapshot"), in the other
  direction. Fix: add `!Key=` to the accepted set as a removal from the working set (or name it
  `UnparsableEntry` alongside the broken-tuple case) and restate `.` per the cited doc as an add
  that may duplicate; re-word #7 accordingly.

- [minor][new] The codec face (`017-C1`) — "fn extract(&self, claim: &Detection) ->
  Result<Vec<BindingSnapshot>, ExtractionRefusal>" — `017-C1` words the same method as
  "`extract(&Detection) -> Result<BindingSnapshot, ExtractionRefusal>`" (singular). The `Vec` is
  the right resolution of the ADR's own `017-C1`/`017-C3` tension and the section argues it, but
  it is a fourth reading that differs from the source's wording, while §Context claims exactly
  three such readings and the Revision log has no entry for it. Fix: one Revision-log line —
  "`017-C1`'s `extract` returns one document per located user tree, not one document" — and add
  the unit id to Rollout 8's `amended-by` hand-off.

- [minor][new] The CLI face with Determinism (`017-L4`) — "More than one record matching (one
  game in two libraries) is not an error: each is processed and rendered" — the `NativeHost`
  tree depends on nothing in the record, so a game installed in two libraries locates the *same*
  tree twice and `--json` prints the same document twice; and since §`bindings/v1` restricts
  `user` to "a SteamID, `steamuser`, `host`", no field distinguishes them, so the law's "across
  documents, byte order by `user`" has an unordered tie and the artifact states one fact twice.
  Fix: in CLI step 4, render documents deduplicated by (`user`, `provenance.files`) before
  sorting, and say so in §Determinism so the ordering law has no tie.

- [nit][new] Test strategy #15 — "the boundary sweep skips exactly `schemas/bindings/`" —
  contract-tree amendment 4 makes the skip set `SOURCE_KEY_TREES: [&str; 2] = ["catalog",
  "bindings"]`, so "exactly" is wrong as written. Fix: "skips exactly `data/catalog/`,
  `schemas/catalog/` and `schemas/bindings/`".

- [nit][new] Read discipline (never-write) — "every open is `FileOptions::new().read(true)` and
  nothing else" — the type is `std::fs::OpenOptions`; `FileOptions` is not a name the crate can
  write. Fix: name `std::fs::OpenOptions::new()`.

## FYI (no severity)

- A22 and Rollout 9 both list the cargo trio plus the Node leg; `check.yml` also runs a third
  job, `cargo deny check advisories` (ADR-027 Rollout 3), which reads the root `Cargo.lock`.
  Nothing here trips it — no new external package — but the gate list in the proposal is
  incomplete against the file it cites.
- `scan_records()` is deliberately an inherent seam, so the bindings face cannot build
  `Vec<Box<dyn DiscoveryPlugin>>` the way `list` does (main.rs:129-142); step 1's "Build the
  Steam plugins exactly as `list` does" means a second, concretely-typed copy of that selection
  and its `cfg(windows)` pair. Ownership question for the complexity seat.
- `schemas/README.md`'s opening format-directory enumeration already omits `catalog`; amendment
  1 mirrors that precedent, so `bindings` will be unlisted there too. Whether to fix the index
  or keep the precedent is a compatibility-seat call.
