# Research: Extracting installed games' key bindings through codec families (ADR-017)

Citation convention: internal claims cite a repo-relative path with line
anchors, resolved against the run's base checkout; external claims cite a URL.
Lenses applied (of `references/lenses.md`): **Prior art**, **Platform notes**,
**Security landscape**.

## Summary

ADR-017 asks for four things — a codec-family trait and closed registry, a
launcher-context-aware config-path resolver, a `bindings/v1` snapshot contract,
and a read-only `discovery bindings` CLI face — and the checkout contains none
of them: `crates/discovery` exposes only `DiscoveryPlugin::{name, discover}`
with rows whose `codec_id` is always `None`, and the CLI has exactly one
subcommand (source: crates/discovery/src/plugin.rs#L4-L7; apps/discovery-cli/src/main.rs#L52-L60; crates/discovery/src/game.rs#L115-L126).
The hard part is not INI parsing but *tree naming*: today's scan flattens every
Steam root's libraries into one set and keeps no notion of a "main root", while
`userdata/`, `steamapps/compatdata/` and the native host layout each live under
a different one (source: crates/discovery/src/steam_common.rs#L90-L107,137-L152).
The second hard part is the entry shape: the ADR-012 input vocabulary is a
closed set of 48 keyboard glyphs, 7 mouse members and 18 gamepad members with
"modifiers and named keys … outside the closed set", while real binding files
bind `ALT`, `TAB`, `CAPSLOCK`, arrow keys, `MouseX` and `Gamepad_RightX`
(source: crates/core/src/input.rs#L70-L121).
External formats also show that the chosen first family spans two storage eras —
`[/Script/Engine.PlayerInput] Bindings=(…)` INI versus UE5.3+ Enhanced Input,
which persists remaps as a binary SaveGame the INI codec cannot see — and that
Steam itself already solves this problem by AppID-keyed VDF documents under the
main root (source: https://dev.epicgames.com/documentation/unreal-engine/configuration-files?application_version=4.27;
https://partner.steamgames.com/doc/features/steam_controller/iga_file).
The main decisions this research hands to the designer are the codec-id bound,
whether `bindings/v1` is TOML or JSON, whether a snapshot may carry a discovery
lookup key against `025-L5`, entry ordering, and how `013-R2`/`013-R3` — both
reserved to ADR-017 and unmentioned by it — are discharged.

## Findings

### What the checkout already provides

- The discovery row is exactly the row `017-C2` must extend, and the codec slot
  already exists and is always empty: `DiscoveredGame { slug, friendly_name,
  display_name, install_path, platform, codec_id: Option<String> }`, with
  `slug` built only through `Source::lookup_key` as `<source>/<external-id>`
  and `Source` a closed three-token set (`steam`, `epic`, `local`) of which only
  `steam` has a shipped emitter (source: crates/discovery/src/game.rs#L30-L126;
  docs/adrs/025-discovery-identity-and-catalog-matching.md#L75-L90).
- `DiscoveryPlugin::discover` takes **no arguments**, so the roots live on the
  plugin struct (a documented constraint from T-001 A17); handing a codec a
  `GameContext` therefore means either a new trait method or a context built at
  the call site, not a change to the existing signature
  (source: crates/discovery/src/plugin.rs#L4-L7; crates/discovery/src/steam_linux.rs#L45-L75).
- **The scan cannot name the main root today.** `SteamCore` holds
  `roots: Vec<PathBuf>`, unions the libraries of all roots, canonicalizes and
  dedupes them, then scans each — after which nothing records which root was
  the main one or which library came from which root. `017-C2`'s "main root and
  library roots separately" is new plumbing through the shared core, not a
  getter over existing state (source: crates/discovery/src/steam_common.rs#L62-L107,133-L152).
- Root selection is an **ordered candidate list**, not a resolved main root:
  `$XDG_DATA_HOME/Steam` (default `$HOME/.local/share/Steam`),
  `.steam/steam`, `.steam/root`, `.steam/debian-installation`, three Flatpak
  roots under `~/.var/app/com.valvesoftware.Steam`, and two Snap roots; every
  candidate is kept, not just the first that exists
  (source: crates/discovery/src/steam_linux.rs#L92-L128).
- The repo's read discipline already matches `017-L1`/`017-B1`: a metadata
  length check against a 1 MiB cap applied *before* allocation
  (`keyvalues::MAX_DOCUMENT_BYTES = 1024 * 1024`, `MAX_NESTING_DEPTH = 16`, and
  `read_text_file` refusing an over-cap file by counting a skip), the same
  posture core exports as `MAX_DOCUMENT_BYTES` (source: crates/discovery/src/keyvalues.rs#L21-L24,77-L81;
  crates/discovery/src/steam_common.rs#L332-L337; crates/core/src/toml.rs#L36-L58;
  schemas/README.md#L100-L113). `017-B1`'s number is therefore the *same* 1 MiB
  the constant family already publishes, and schemas/README is where a new
  budget is announced (source: schemas/README.md#L118-L177).
- A VDF/KeyValues parser already exists in-repo (`crates/discovery/src/keyvalues.rs`
  with quoted-value unescaping and depth caps), so a future Valve-lineage codec
  family needs no new parser — a fact the registry's "closed static set" should
  not spend design effort re-deriving (source: crates/discovery/src/keyvalues.rs#L10-L24).

### The CLI face and its pinned shapes

- `discovery` has exactly one subcommand, `List { --steam-root }`, plus global
  `--json` and `--catalog-url`; the table's five columns are pinned and the JSON
  keys are byte-pinned from `DiscoveredGame`'s serde shape (keys `slug`,
  `display_name`, `install_path`, `platform` spelled PascalCase `SteamLinux`,
  `codec_id`), a split the repo deliberately preserves (source: apps/discovery-cli/src/main.rs#L34-L70; docs/tasks/T-003-polish-discovery-cli-output.md#L32-L46).
- Failure voice today is exactly one: a single `error: ` line plus
  `ExitCode::FAILURE`. `017-C4`'s two named statuses (claim-refused,
  family-confirmed-files-absent) each exiting non-zero means *distinguishing*
  exit codes must be introduced and pinned; nothing in the CLI distinguishes
  them today (source: apps/discovery-cli/src/main.rs#L79-L83).
- **A scan is not a local operation.** `list` refuses before constructing any
  plugin if no catalog endpoint is configured, probes that endpoint, then loads
  the per-user match store, enumerates, and queries candidates for every
  unresolved slug; `025-L1`'s connectivity gate (as amended by `026-L1`) admits
  a remote endpoint and, in the code's own words, sends every queried source key
  to it in clear text. Because `017-C4` says `bindings` "runs the same scan
  `list` runs", extraction inherits that mandatory network probe and that
  disclosure — which sits uneasily beside ADR-017's Consequences claim that
  privacy "stays local by construction" (source: apps/discovery-cli/src/main.rs#L42-L47,87-L123;
  docs/adrs/025-discovery-identity-and-catalog-matching.md#L108-L117; docs/adrs/017-keybinding-extraction-codecs.md#L232-L236).
- The per-user data directory convention already exists and is sandbox-aware:
  `$XDG_DATA_HOME` when set and absolute, else `$HOME/.local/share`, joined to
  `bindforge/…` (today `bindforge/matches.json`). A snapshot cache or provenance
  side-file, if any, should ride it rather than invent one
  (source: apps/discovery-cli/src/matches.rs#L5-L8,47-L103).
- `data/` holds only `profiles/README.md`, `manifests/README.md`,
  `catalog/README.md` and `catalog/v1/catalog.json`; there is **no appid→family
  table anywhere**, while `017-C5` runs "an appid-table claim pass" and
  `017-R4` defers machine-readable routing to future `data/` content. In this
  run the appid table can only live in code inside each codec — worth stating
  explicitly in the proposal so the two units do not read as contradicting
  (source: data/ tree listing; docs/adrs/017-keybinding-extraction-codecs.md#L109-L113,161-L162).

### Contract fit with the formats tree

- The manifest's `codec` entry is governed as **slug grammar plus the shared
  `MAX_SLUG_OCTETS` (128)**; ADR-017's `017-C1` instead cites "`013-B4` bounds",
  and `013-B4` is `MAX_NATIVE_NAME_OCTETS = 128` — a different constant with a
  different meaning. The two numbers agree at 128 today, but the citation is the
  wrong home: `013-B4` was never the codec-id bound (`013-R1` reserved the id
  space at "slug grammar, at most 63 octets", relaxed to 128 by `024-B1`)
  (source: schemas/manifests/v1/spec.md#L56; crates/core/src/manifest.rs#L58;
  crates/core/src/placement.rs#L67-L95; schemas/README.md#L145-L160;
  docs/adrs/013-game-manifests.md#L151-L153).
- Two ADR-013 reservations name ADR-017 as owner and ADR-017 never mentions
  them: **`013-R3`** ("`codec` and preset ids are not registry-checked — no
  implementation set exists to check them against") becomes dischargeable the
  moment this run creates the first closed implementation set, and **`013-R2`**
  (fingerprint semantics: which bytes are hashed, what a mismatch means) is the
  natural content of `017-C3`'s provenance block — the manifest already carries
  an optional `sha256-<base64>` fingerprint field with a closed one-algorithm set
  (source: docs/adrs/013-game-manifests.md#L150-L160; schemas/manifests/v1/spec.md#L137,157;
  docs/adrs/017-keybinding-extraction-codecs.md#L92-L99).
- `017-C3` says a snapshot records "the game's lookup key". `025-L5` is the
  corpus-boundary law: the curated game id is "the only identifier of this ADR
  that enters a shipped artifact or a path component; source keys and friendly
  names stay in discovery output and the local store", and boundary fixtures
  assert no source key reaches a manifest or profile. Whether a `bindings/v1`
  document is a "shipped artifact" is the designer's call, and it decides whether
  `steam/<appid>` is legal content in `schemas/bindings/v1/fixtures/` — the
  hygiene sweep currently skips that spelling only under `data/catalog/` and
  `schemas/catalog/` (source: docs/adrs/025-discovery-identity-and-catalog-matching.md#L134-L138,182-L186;
  schemas/README.md#L85-L88; docs/adrs/017-keybinding-extraction-codecs.md#L92-L99).
- **Format family question.** Every format in `schemas/` is TOML except one that
  is named as the exception (`catalog`, added by `026-C3`, with the reason
  recorded in schemas/README so "Formats are TOML" stays true). `017-C3`'s
  `--json` mode prints "the snapshot document verbatim", which reads as JSON
  being the document form; if so, `bindings` becomes the second named JSON
  format and the README sentence must be amended the same way `catalog` was
  (source: schemas/README.md#L61-L88; docs/adrs/017-keybinding-extraction-codecs.md#L100-L108).
- Determinism has a hole the ADR does not close: `017-L4` pins byte-identical
  output but no ordering for `entries`, and the discovery scan's own ordering is
  a `(slug, install_path)` sort with an explicit "no collision pass" posture. A
  snapshot needs a stated entry ordering (and a stated order for the per-SteamID
  grouping) or two runs over the same files can differ
  (source: docs/adrs/017-keybinding-extraction-codecs.md#L132-L135; crates/discovery/src/steam_common.rs#L100-L107,128).
- Naming drift against ADR-008: the planned workspace named this responsibility
  `crates/adapters` ("codec families, config path discovery, write policy") and
  `crates/scanner` ("installed-game detection, config fingerprinting"); ADR-017
  creates `crates/codecs` and folds detection into it, and the shipped layout is
  already `crates/discovery` instead of `scanner`. The proposal should either
  retire `adapters`/`scanner` in prose or record the divergence
  (source: docs/adrs/008-rust-for-the-product.md#L36-L47; docs/adrs/017-keybinding-extraction-codecs.md#L64-L71).
- ADR-017 justifies keeping codecs out of core by "ADR-008 clause 3, the
  `transforms` precedent", but clause 3 is "Formats are the contract" (schemas/
  is the contract, core is the reference implementation); the crate-split
  precedent is clause 2's workspace list plus ADR-009's "`crates/transforms`
  owns the policy machinery". The conclusion is sound; the citation pointer is
  off by a clause and the designer may want it stated correctly
  (source: docs/adrs/008-rust-for-the-product.md#L38-L57; docs/adrs/009-canonical-profile-and-manifest-model.md#L59-L62,142-L148;
  docs/adrs/017-keybinding-extraction-codecs.md#L64-L71).
- ADR-022's own cost list anticipated exactly this run: "No codec selection yet;
  that requires a common codec taxonomy first", and its `codec_id: Option<String>`
  was declared "blank for v1". ADR-017 is that taxonomy's first installment, and
  `017-L5` keeps `list` spelling `None` — so `codec_id` will be populated on the
  bindings face only, which the CLI's byte-pinned JSON shape for `list` makes
  testable rather than ambiguous (source: docs/adrs/022-game-discovery.md#L36-L48,78;
  docs/adrs/017-keybinding-extraction-codecs.md#L20-L21,136-L139).

### The input vocabulary versus what files actually contain

- The closed vocabulary: `keyboard:<single ANSI-US glyph>` plus the reserved word
  `space` — 48 graded members counting letters, digits and the eleven
  punctuation glyphs — `mouse:` five buttons + two wheel directions, `gamepad:`
  the 16 W3C standard-layout names plus two stick presses; the code comment says
  outright that "modifiers and named keys are outside the closed set", and there
  is no chord or modifier syntax in the id grammar at all
  (source: crates/core/src/input.rs#L60-L121).
- Real binding stores name exactly those excluded things. A shipped Source-family
  config binds `F3`, `CAPSLOCK`, `TAB`, `ALT`, `PGUP`, `DOWNARROW`, `UPARROW` and
  `\`; a shipped Unreal `DefaultInput.ini` maps `Key=Left`, `Key=Right`,
  `Key=MouseX`, `Key=MouseY`, `Key=Gamepad_RightX/Y`
  (source: https://github.com/acx0/cs2-cfg/blob/master/csgo-autoexec.cfg;
  https://github.com/martiancraft/Unreal-Engine-4-Plugins/blob/master/Plugins%20Example%20Project/Config/DefaultInput.ini).
- Therefore `017-C3`'s "inputs are the ADR-012 vocabulary spellings" plus
  `017-L3`'s no-inference law means a first extraction will hit inputs the
  contract cannot spell. The design needs a *named* handling for
  out-of-vocabulary inputs (per-entry refusal, a named status, or an explicit
  `unmapped` spelling) — silence here will either drop bindings invisibly,
  contradicting `017-L3`, or refuse whole snapshots, contradicting the ADR's
  goal of first manifests. This is the single largest content risk of the run
  (source: docs/adrs/017-keybinding-extraction-codecs.md#L92-L99,128-L131; crates/core/src/input.rs#L70-L121).
- The Source family's *direction* is the mirror image of the snapshot entry:
  a Source config binds a **key to a command string**, and the command is often
  compound or indirect — `bind "i" "toggle cl_teamid_overhead_mode 1 2"` and
  `alias`-defined names referenced by other binds; `<game>/cfg/` holds shared
  defaults while per-user overrides live under `userdata/<steamid>/<appid>/`.
  ADR-017 defers Source to `017-R3`, but the `{action, inputs}` pair shape is
  fixed *by this run*, and formats only grow through new majors — so the shape
  must be chosen so that key→command, aliases and compound commands remain
  expressible without a `bindings/v2`
  (source: https://developer.valvesoftware.com/wiki/Server.cfg;
  https://github.com/acx0/cs2-cfg/blob/master/csgo-autoexec.cfg;
  https://www.pcgamingwiki.com/wiki/Glossary:Game_data;
  docs/adrs/017-keybinding-extraction-codecs.md#L159-L160,209-L214; schemas/README.md#L37-L57).
- Unreal's own duplicate semantics are order-dependent: `.Bindings=(Name="Q",Command="…")`
  adds a line without deduping and "the bottom-most binding takes effect". A
  faithful extractor must fold or preserve order deterministically, and must
  decide which file wins when the same property appears in several located files
  — the engine's answer is a *load hierarchy* (base → platform → user), which is
  exactly the tree `017-C2` hands over (source: https://dev.epicgames.com/documentation/unreal-engine/configuration-files?application_version=4.27).

## Prior art

- **Valve already operates a codec family at catalog scale, routed by AppID.**
  Developers ship an In-Game Actions file, `game_actions_<appid>.vdf`, placed in
  `<steam>/controller_config/` (Windows `C:\Program Files (x86)\Steam\controller_config`,
  Linux `~/.steam/steam/controller_config`); the document is KeyValues/VDF, wraps
  `actions` + `localization`, groups actions into action sets and the three
  container kinds `StickPadGyro` / `AnalogTrigger` / `Button`, and carries
  `major_revision` / `minor_revision` plus `major_revision_affects_mousekb`
  flags precisely so that a breaking change to the action vocabulary can be
  pushed at players who already built configurations. Two transferable lessons:
  an action-name namespace needs a revision mechanism the moment it is shared
  (the manifest's `revision` + `013-C4` pin is BindForge's answer, so the
  snapshot needs none of its own), and Valve's split of *action kinds* by
  analog shape is the same distinction ADR-009 put in the taxonomy's "axis kind"
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- **Valve's per-user binding data lives where `017-C2` says it must be looked
  for**: locally saved input configs are at
  `<Steam>/UserData/<userid>/241100/remote/controller_config/<appid>/<savename>.vdf`
  — main-root `userdata/`, keyed by a local SteamID directory and then by
  AppID. This is the strongest available confirmation that (a) `userdata/` is
  main-root-only and (b) the "grouped per local SteamID" shape of `017-C1`'s
  `Detection` matches how the platform itself partitions the data
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  https://www.pcgamingwiki.com/wiki/Store:Steam).
- **Unreal's user-facing lineage** is the `Saved/Config/<Platform>/` INI tree
  whose `Input` category carries `[/Script/Engine.PlayerInput] Bindings=`, with
  the engine's documented `+`/`-`/`.` line-merge operators and a documented
  per-category file hierarchy (base `Default*.ini`, platform, then user override).
  Community guides confirm the packaged-game final file lands in
  `Saved/Config/<Platform>/` and that the editor writes to
  `Saved/Config/WindowsEditor/`, i.e. the platform directory name is *not* one
  constant across editor and packaged builds — a resolver detail the `unreal-ini`
  family must pin rather than infer (source: https://dev.epicgames.com/documentation/unreal-engine/configuration-files?application_version=4.27;
  https://hyaniner.com/en/blog/config-ini-of-ue-5.5-20250218;
  https://ikrima.dev/ue4guide/application-os-generic-platform/config-files).
- **Unreal has since moved the same fact to a different medium.** UE 5.3
  introduced `UEnhancedInputUserSettings` as the store for user key mappings, and
  it persists as a saved-game object under `Saved/SaveGames/` (tutorials show the
  `EnhancedInputUserSettings` file there, and note remapping persistence behaved
  differently between editor and packaged builds). For a codec whose declared
  grammar is "the `Saved/Config` INI layout", an Enhanced-Input-era title is a
  family-confirmed game whose binding files genuinely are not in the INI tree at
  all — precisely `017-L2`'s named status, and a case where that status is
  *correct but misleading* (source: https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/EnhancedInput/UEnhancedInputUserSettings;
  https://medium.com/@xersendo/ue-enhanced-input-system-in-game-remapping-before-and-after-ue-5-3-03986abff066;
  https://www.youtube.com/watch?v=iUj7VUBVHKM).
- **Source-family storage** is a per-line console command (`// comments`,
  `;`-separated commands, `bind`/`alias`/`exec`) with a shared `<game>/cfg/`
  directory and per-user overrides under `userdata/<steamid>/<appid>/local/cfg/`
  — a grammar friendlier than INI but inverted relative to the snapshot entry
  (source: https://developer.valvesoftware.com/wiki/Server.cfg;
  https://github.com/acx0/cs2-cfg/blob/master/csgo-autoexec.cfg).
- **A cross-tool naming convention exists and BindForge already depends on it**:
  PCGamingWiki's game-data glossary is the community's table of "where this
  store's per-user data lives" (`<Steam-folder>\userdata\<user-id>\<app-id>\`,
  `$XDG_DATA_HOME/Steam/` on Linux, Wine-registry caveats, "treat backslashes as
  forward slashes"). It is the closest thing to a shared registry of the layout
  knowledge `017-R4` wants to make machine-readable, and it is prose, not data
  (source: https://www.pcgamingwiki.com/wiki/Glossary:Game_data;
  https://www.pcgamingwiki.com/wiki/Store:Steam).
- No established open-source standard for *exported native bindings* surfaced in
  this pass; the formats that exist are all engine- or launcher-private
  (Bindings=/`bind`/IGA VDF), which is the empirical case for `bindings/v1` as a
  contract rather than an internal CLI shape
  (source: docs/adrs/017-keybinding-extraction-codecs.md#L216-L220).

## Platform notes

- **Three trees, one game.** A steam-published title may keep user bindings under
  `<main root>/userdata/<steamid>/<appid>/`, a native Linux install keeps them
  under the host's user directories, and a Proton launch keeps them under
  `<library>/steamapps/compatdata/<appid>/pfx/drive_c/users/steamuser/…` at the
  Windows paths the game believes. Valve documents the AppID-keyed `userdata`
  layout; the community documents both that and the per-library `compatdata`
  location, and `libraryfolders.vdf` is documented as the library table on Linux
  at `<steam>/steamapps/libraryfolders.vdf` — the file the repo already reads
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  https://www.pcgamingwiki.com/wiki/Store:Steam;
  https://www.gamingonlinux.com/guides/view/how-to-find-where-steam-stores-proton-game-files;
  crates/discovery/src/steam_common.rs#L137-L152).
- **The Proton prefix's user directory is a constant, not a variable**: "the user
  is always named `steamuser` — Proton never uses your real username", so the
  Windows → prefix mapping table can harden `{APPDATA}` →
  `users/steamuser/AppData/Roaming` rather than glob `drive_c/users/*`. That same
  write-up's stated pain point — everything in the path is predictable except
  the AppID — is the argument for `017-C5`'s appid-table claim pass
  (source: https://checkpoint64.com/blog/steam-deck-save-file-location;
  https://www.gamingonlinux.com/guides/view/how-to-find-where-steam-stores-proton-game-files).
- **Prefixes live under the library that holds the game, and Flatpak/Snap Steam
  relocate the whole tree**: `~/.var/app/com.valvesoftware.Steam/data/Steam/steamapps/compatdata/<appid>/`.
  The repo already enumerates Flatpak and Snap *roots* for scanning installs, so
  the resolver inherits those roots, but a candidate-path template must be
  expressed relative to the owning root rather than to `$HOME`
  (source: https://www.gamingonlinux.com/guides/view/how-to-find-where-steam-stores-proton-game-files;
  crates/discovery/src/steam_linux.rs#L112-L125).
- **Prefix trees are held together by symlinks**, and that is where they break:
  Proton's own `migrate_user_paths` creates links such as
  `drive_c/users/steamuser/Local Settings/Application Data → ../AppData/Local`
  and `My Documents`, and real failures on this exact structure are reported as
  symlink-loop and cross-device (`Errno 5` / `File exists`) breakage on NTFS/exFAT
  libraries. Any candidate-locating walk inside a prefix therefore needs a
  stated symlink policy (follow or not), because a follow policy can leave the
  prefix and a loop can hang a `bindings` run
  (source: https://forum.endeavouros.com/t/multiple-steam-proton-games-no-longer-launching/16692;
  https://forum.manjaro.org/t/steam-proton-not-working-on-ntfs-drive/84912;
  https://discussion.fedoraproject.org/t/steam-fails-to-launch-games-with-proton-because-of-python-error/77376).
- **Encoding is not uniform, even inside one family.** UE loads external text via
  `appLoadFileToString`/`appBufferToString`, which recognizes a UTF-16 BOM in
  either endianness and behaves platform-dependently when the BOM is absent; UE
  text files are stored as UTF-16 unless every character fits one byte, and a
  staging bug report shows `DefaultGame.ini` written UTF-16LE while its siblings
  were UTF-8. Today's shared reader produces a strict-UTF-8 `String`, so a codec
  must either decode BOM-sniffed UTF-16 or refuse by name
  (source: https://dev.epicgames.com/documentation/unreal-engine/character-encoding-in-unreal-engine;
  https://forums.unrealengine.com/t/file-encoding-of-ini-files-changes-during-staging/2514754;
  crates/discovery/src/steam_common.rs#L332-L340).
- Toolchain note: ADR-008 already names "filesystem-level work (game scanning,
  INI codecs, Proton prefixes)" as a reason the product is Rust, and accepts
  "lifetimes in the codec layer" as a named cost — the crate the ADR is about is
  the one that cost was reserved for (source: docs/adrs/008-rust-for-the-product.md#L19-L21,88-L93).
- Sandbox note: the repo's own Steam root probing already understands Flatpak and
  Snap Steam, and the CLI's data directory honours `XDG_DATA_HOME` as "already the
  sandbox redirect"; a BindForge process running *inside* a sandbox is the case
  where reading another app's tree fails outright, and no code today tests it
  (source: crates/discovery/src/steam_linux.rs#L112-L125; apps/discovery-cli/src/matches.rs#L5-L8).

## Risks & unknowns

- **Vocabulary gap (highest).** Real bindings name modifiers, F-keys, arrows and
  analog axes the closed input grammar refuses; see the Findings bullet above.
  Every downstream consumer inherits the choice, and ADR-012's own text defers
  physical-position naming for non-US layouts, so the gap widens across keyboards
  (source: crates/core/src/input.rs#L70-L121; docs/adrs/012-input-vocabulary-and-reach-classes.md#L57-L62,89-L91).
- **Contract-vs-Ambiguity: `bindings/v1` entry shape is decided once.** Because
  formats grow only through new majors, fixing `{action, inputs}` now without
  room for key→command direction, aliases, chords, or axis-with-scale (`Scale=-1.0`)
  is a future breaking major rather than an additive minor
  (source: schemas/README.md#L37-L57; https://dev.epicgames.com/documentation/unreal-engine/configuration-files?application_version=4.27).
- **Privacy claim vs the scan the face reuses.** `017-C4` reuses `list`'s scan,
  which fails closed without a reachable catalog and sends queried source keys to
  whatever endpoint is configured; the ADR's privacy consequence should either
  acknowledge that or the face should be allowed to skip the catalog leg
  (source: docs/adrs/017-keybinding-extraction-codecs.md#L100-L108,232-L236; apps/discovery-cli/src/main.rs#L87-L109).
- **Local paths in refusals.** `013-L5` and the formats contract keep local paths
  out of messages; `017-C4`'s probe mode exists to print located candidate file
  paths, and the bindings face's provenance names files. The "never echo a path"
  discipline is written for documents arriving from elsewhere, not for a local
  operator tool; the design should say which rule governs here so clippy-era test
  expectations are not guessed (source: schemas/README.md#L179-L185; docs/adrs/013-game-manifests.md#L114-L116).
- **Security surface is local untrusted files, not the network.** The threat is
  a game- or prefix-owned file that is malformed, enormous, UTF-16, symlinked out
  of the tree, or larger than the entry budgets. The existing controls (pre-read
  size cap, nesting cap, closed entry sets, bounded single-line refusals) plus
  `017-L1`'s never-write law cover the first three; the symlink case is unhandled
  anywhere in the checkout (source: schemas/README.md#L100-L125,179-L185; crates/discovery/src/steam_common.rs#L332-L340).
- **Aggregate read cost.** With `017-B1` = 1 MiB and `017-B3` = 32 candidates per
  game per SteamID, the per-game worst case is ~32 MiB of reads plus an unbounded
  SteamID multiplier; the formats contract states its aggregates in exactly this
  product form (`MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` = 32 MiB), so the proposal
  owes one such sentence and a decision on whether the SteamID count itself is
  capped — `017-B3` caps files per SteamID but nothing bounds SteamIDs per machine
  (source: docs/adrs/017-keybinding-extraction-codecs.md#L143-L150; schemas/README.md#L126-L144).
- **Proton layout drift.** ADR-017 already concedes "Proton layout assumptions
  track Valve's releases"; the symlink machinery above is exactly the part that
  moves, and `017-S1`'s verification pass is against two titles on one machine,
  so the fixtures must encode the layout claim, not the machine
  (source: docs/adrs/017-keybinding-extraction-codecs.md#L230-L232;
  https://forum.endeavouros.com/t/multiple-steam-proton-games-no-longer-launching/16692).
- **Unknown: no published latency/throughput baselines** exist for one-shot
  binding extraction (the per-event 1000 Hz budget framing in the lenses does not
  apply to this ADR — extraction is offline, not in the input path), so the only
  budgets worth pinning are byte and count budgets of the kind `017-B1..B3`
  already are (source: /home/mike/repos/bindforge-017-keybinding-v2/.opencode/skills/research/references/lenses.md#L15-L18).
- **Unknown: `017-S1`'s "two locally installed Unreal-lineage titles" are not
  named**, and the run cannot install games; whether the verification pass is a
  committed fixture tree, an operator-run manual step, or a skipped item recorded
  in build notes is unclaimed by the Rollout.

## Open questions

1. Which constant bounds a codec family id — the manifest spec's shared
   `MAX_SLUG_OCTETS` (128, slug grammar) or the ADR's cited `013-B4`
   (`MAX_NATIVE_NAME_OCTETS`)? If the ADR's citation is a slip, the proposal
   should cite `024-B1` and the spec line, and `017-C1` should be written to
   match `schemas/manifests/v1/spec.md` (source: docs/adrs/017-keybinding-extraction-codecs.md#L75-L84; schemas/manifests/v1/spec.md#L56).
2. Is `bindings/v1` TOML or JSON, and if JSON, does the proposal amend
   schemas/README's "one format is JSON, and it is named here" passage the way
   `catalog` did? `--json` printing "the document verbatim" strongly implies
   JSON (source: schemas/README.md#L61-L88; docs/adrs/017-keybinding-extraction-codecs.md#L100-L108).
3. May a snapshot carry the `<source>/<external-id>` lookup key and a SteamID,
   or does `025-L5` confine those to discovery output and the local store? What
   do the `schemas/bindings/v1/fixtures/valid/` files then key a snapshot by, and
   does the corpus hygiene sweep need a second carve-out?
   (source: docs/adrs/025-discovery-identity-and-catalog-matching.md#L134-L138; schemas/README.md#L85-L88).
4. How are inputs that the ADR-012 closed set cannot spell handled — per-entry
   named refusal, a per-snapshot partial status, or an explicit unrepresentable
   spelling — and which unit in the ledger owns it? Nothing in `017-C3`/`017-L3`
   covers it (source: crates/core/src/input.rs#L70-L121; docs/adrs/017-keybinding-extraction-codecs.md#L92-L99,128-L131).
5. How does the first family express an *ordered/duplicate* binding store —
   Unreal's bottom-most-wins `.Bindings` folds into `{action, inputs}` how, and
   is the losing duplicate recorded or dropped under `017-L3`?
   (source: https://dev.epicgames.com/documentation/unreal-engine/configuration-files?application_version=4.27)
6. What entry ordering and per-SteamID grouping order makes `017-L4`'s
   byte-identical claim testable? (source: docs/adrs/017-keybinding-extraction-codecs.md#L132-L135)
7. Where does the appid→family table physically live in this run, given
   `017-C5`'s appid pass and `017-R4`'s deferral of routing *data* — a hard-coded
   list in `crates/codecs`, and does that list belong to the registry or to each
   codec (source: docs/adrs/017-keybinding-extraction-codecs.md#L109-L113,159-L162)?
8. How are `013-R2` (fingerprint semantics) and `013-R3` (implementation-set
   checks on `codec` ids) discharged or explicitly re-reserved by this run, given
   both name ADR-017 as owner and this run creates the first implementation set?
   (source: docs/adrs/013-game-manifests.md#L150-L160)
9. Does `discovery bindings` run the catalog gate and candidate queries, or is it
   allowed to enumerate offline? If it runs them, does the run record that a
   bindings query discloses the AppID to the configured registry
   (source: apps/discovery-cli/src/main.rs#L87-L123; docs/adrs/017-keybinding-extraction-codecs.md#L232-L236)?
10. Which distinct exit codes name claim-refused versus family-confirmed-files-
    absent, and does `--json` mode print a machine-readable status for each
    (the CLI currently has one failure voice and one code)
    (source: apps/discovery-cli/src/main.rs#L79-L83; docs/adrs/017-keybinding-extraction-codecs.md#L100-L127)?
11. What is the symlink policy inside a Proton prefix, and does `017-B3`'s
    candidate search follow links at all (source: https://forum.endeavouros.com/t/multiple-steam-proton-games-no-longer-launching/16692)?
12. Does the `unreal-ini` codec accept BOM'd UTF-16 INI files, and is refusing one
    a `ExtractionRefusal` variant name the CLI prints
    (source: https://dev.epicgames.com/documentation/unreal-engine/character-encoding-in-unreal-engine)?
13. Does `bindings` populate `DiscoveredGame.codec_id` on its own rows (the ADR's
    opening says the field leaves `None` "only on the face this ADR adds") or
    report the claiming codec purely through snapshot provenance — and if the
    former, how is `017-L5`'s pinned `None` in `list` output kept provable
    (source: docs/adrs/017-keybinding-extraction-codecs.md#L20-L21,136-L139)?
14. Does this run retire or reconcile ADR-008's `crates/adapters` /
    `crates/scanner` names, and is the "ADR-008 clause 3" citation corrected to
    the clause that actually carries the crate-split precedent
    (source: docs/adrs/008-rust-for-the-product.md#L36-L57)?
15. Is an Enhanced-Input-era Unreal title a claim with files-not-found
    (`017-L2`) or a non-claim, and does the family id record the *lineage* or the
    *storage grammar* — the answer sets the precedent for every future family
    split (source: https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/EnhancedInput/UEnhancedInputUserSettings;
    docs/adrs/017-keybinding-extraction-codecs.md#L124-L127).
