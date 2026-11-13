# Research: Game manifests as the game-facts contract (ADR-013)

Scope: the ADR defines `schemas/manifests/v1/` as versioned data recording, per
game, a game id + detected config fingerprint, a codec *family id*, the action
table (game-native name ↔ taxonomy id + axis kind), constraints (exclusivity
groups, dead keys, hard-docked UI), and built-in presets — with profiles pinning
manifest id + version and CI re-validating dependents on every bump. It decides
no codec implementation (ADR-017), no seed preset set (ADR-016), and no authoring
UX. Lenses applied: **Prior art**, **Standards**, **Security landscape**, under
Findings, Prior art, and Risks & unknowns.

## Summary

The closest known system to ADR-013's manifest is Valve's Steam Input **In-Game
Actions file** and its extension, the **Action Manifest File**: a per-game,
in-revision-control, game-id-named file declaring the game's actions and action
sets under a closed vocabulary of action kinds, with `major_revision` /
`minor_revision` fields whose major exists precisely to flag breaking *action-set*
changes, plus per-surface "affects" flags that decide which players are forced to
re-adopt (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
source: https://partner.steamgames.com/doc/features/steam_controller/action_manifest_file).
No surveyed system separates *game facts* from *player policy* the way the ADR
does, and none encodes mirrorability of action pairs or key exclusivity as data —
Steam models modal-UI conflicts as parent/child *action set layers* instead, so the
constraint half of the manifest has no published equivalent to copy. The two
hardest mechanics the ADR takes on are encoding-bound rather than idea-bound: the
contract's TOML subset forbids table headers, dotted keys and inline tables, so a
three-column action table must ride a space-separated string encoding while
game-native action names routinely contain spaces (source: schemas/action-ids/v1/spec.md;
source: https://github.com/ngollan/Elite-Binds). And the manifest's "axis kind"
column restates a fact the shipped `action-ids` registry already owns per id,
which is a new drift channel inside the repo (source: schemas/action-ids/v1/registry.toml;
source: schemas/action-ids/v1/spec.md).

## Findings

### The contract and the code as they stand on `main`

- `data/manifests/` contains only a README, which already states the security
  posture ADR-013 inherits: every file there "arrived from elsewhere — a
  contributor's PR, a hub upload, a forum paste", so it "is untrusted input and
  goes through the same guards every contract document gets", and the id
  stability rule lives in `schemas/action-ids/v1/spec.md`, not in the README
  (source: data/manifests/README.md).
- `Document` has four arms (`Profile`, `ReachClasses`, `ActionIds`,
  `KeyboardGeometry`), `SUPPORTED_FORMATS` lists four names, and `format_majors`
  is the per-format half of the version gate; a manifest is one new arm, one new
  `SUPPORTED_FORMATS` entry, one new closed entry-set constant, one
  `format_majors` line, and one `serialize` branch (source: crates/core/src/document.rs).
- Adding `manifests/v1` does **not** widen `supported_majors()`: the crate already
  declares `["v1", "v2"]`, and the pinned invariant is that the declared majors
  equal the majors present under `schemas/*/v*/` — a third format at `v1` keeps
  that equality green with no change to the union. The ADR's Consequences line
  "a third format joins … the `supported_majors` list" overstates the change:
  what actually grows is the *per-format* list (source: crates/core/src/document.rs;
  source: crates/core/tests/version_gate.rs; source: docs/adrs/013-game-manifests.md).
- The conformance harness walks `schemas/*/v*/fixtures/{valid,invalid}` with no
  per-format hook, so a manifest directory is picked up automatically; for every
  format except `reach-classes` it also asserts a **round-trip**: parse, then
  re-parse `serialize(document)`, and require a semantically equal value. A
  manifest format is therefore contractually obliged to be fully
  round-trippable through `serialize`, which the reach tables were exempted from
  (source: crates/core/tests/conformance.rs).
- The zero-fixture rule fails the run if a version directory has an empty
  `fixtures/valid/` or `fixtures/invalid/`, and fixture names must be lowercase
  ASCII, `-`-joined and case-unique so the same corpus is green on a
  case-insensitive Windows filesystem (source: crates/core/tests/conformance.rs;
  source: schemas/README.md).
- The grammar is line-oriented flat key/value only: a value is "a basic string, a
  decimal integer, a boolean, or an array of those written on one line"; table
  headers, dotted keys, multi-line arrays, floats and hex integers are legal TOML
  and refused, each with an invalid fixture. That rules out inline tables and any
  nested structure for the action table, the constraint groups and the presets —
  every one of them must be a one-line array of strings (source: schemas/action-ids/v1/spec.md;
  source: schemas/README.md).
- The contract's precedent for a table-as-data is a **space-joined string**, and
  the reason it works is stated: "No member carries a space, which is what makes
  the pair encoding … split unambiguously" (`"<input-id> <class-id>"` in
  reach-classes/v2, `"<input-id> <col> <row>"` in keyboard-geometry/v1), with a
  three-field element pinned as an invalid fixture (source: schemas/reach-classes/v2/spec.md;
  source: crates/core/src/document.rs;
  source: schemas/keyboard-geometry/v1/fixtures/invalid/pair-two-fields.toml).
- Parse budgets are already exported names: `MAX_DOCUMENT_BYTES = 1024 * 1024`,
  enforced on file metadata *before* the read and by reading at most one byte more
  than the budget from a stream, and `MAX_NESTING_DEPTH = 16` (source: crates/core/src/toml.rs;
  source: schemas/README.md).
- The reference reader's escape table admits exactly `\b \f \n \r \t \" \\ \uXXXX
  \UXXXXXXXX` and refuses any other escape; TOML v1.1.0 additionally admits `\e`
  and `\xHH`, so a third-party reader built on a standard 1.1 parser accepts
  strings the contract refuses — the same fail-open direction every other subset
  refusal in the corpus pins, and it has no manifest-relevant fixture yet
  (source: crates/core/src/toml.rs; source: https://toml.io/en/v1.1.0).
- The taxonomy ids a manifest may name are loadable and checkable today:
  `load_action_registry`, `ActionRegistry::contains` and `ActionRegistry::axis_kind`
  give the ADR's "ids are validated against the loaded registry" a working API,
  and `registry.toml` is the single id list — 40 ids (4 `discrete-lateral`, 1
  `continuous-lateral`, 35 `none`) across the `move`, `aim`, `combat`,
  `interact`, `item`, `ui` and `system` namespaces
  (source: crates/core/src/action.rs; source: schemas/action-ids/v1/registry.toml;
  source: schemas/action-ids/v1/spec.md).
- Axis kind is already a property of the **id**, not of the game: `registry.toml`
  pins `discrete-lateral` to exactly the four steer/strafe ids and
  `continuous-lateral` to `aim.camera`, and the spec states the discriminator is
  "the shape of the binding, not the device". A manifest that carries an axis
  kind column therefore restates a fact the registry owns, and the contract has
  no rule today for what happens when they disagree (source: schemas/action-ids/v1/registry.toml;
  source: schemas/action-ids/v1/spec.md).
- The input vocabulary a manifest would name keys from is **closed and narrow**:
  `keyboard:` is 48 members (`a-z`, `0-9`, the fixed punctuation set, and the
  reserved word `space`),
  `mouse:` seven, `gamepad:` eighteen, and "an id outside a set is a refusal, not
  an unknown member" (source: schemas/reach-classes/v2/spec.md).
  Real games bind far more than that: TF2's bindable key-name list includes `ESC`,
  `F1`–`F12`, arrow keys, `Tab`, `Caps`, `Shift`, `Ctrl`, `Alt`, `Insert`,
  `Home`/`PgUp`/`Del`, `MOUSE1`–`MOUSE5`, `MWHEELUP`/`MWHEELDOWN`
  (source: https://wiki.teamfortress.com/wiki/Scripting).
  The vocabulary gap is a 012 decision that 013 collides with head-on, because
  "dead keys" and "hard-docked UI" are precisely about the keys a game *will not*
  honour.
- Nothing in the repo validates cross-file relationships today: the chain guards
  (`MAX_CHAIN_DEPTH = 32`, `Cycle`, `ChainTooDeep`) are about a profile's ancestor
  chain, and the harness's cross-file rule is only the per-hand table pairing.
  "Every dependent profile re-validated on each manifest bump" has no machinery
  to sit in: CI is two PR jobs (`cargo fmt`/`clippy`/`test --locked`, 15-minute
  timeout each) plus one scheduled `cargo-deny` job that is deliberately not a PR
  gate (source: crates/core/src/chain.rs; source: crates/core/tests/conformance.rs;
  source: .github/workflows/check.yml; source: .github/workflows/advisories.yml).
- The consumers are already specified downstream, so 013's vocabulary is their
  input surface: ADR-014 (depends-on 011 and 013) adds `for` — "the manifest id
  and version pin" — and `seeded-from` — "a game built-in preset, 013" — and
  ADR-015 preconditions transforms on reach legality *and* "the manifest's
  constraints (013): a hard-docked or exclusive-group key is never a target"
  (source: docs/adrs/014-profiles-as-chains.md; source: docs/adrs/015-transforms-as-preconditioned-policies.md).

### What other systems record as "game facts" (Standards + prior art)

- Steam Input requires the developer to supply an **In-Game Actions (IGA) file**
  "that defines the actions, action sets, and basic structure of their game's
  input configuration", written in Valve's KeyValues/VDF format, named
  `game_actions_X.vdf` where `X` is the game's **Steam AppID** — i.e. the game's
  identity is a store-assigned numeric id carried in the file name
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- The IGA vocabulary is closed exactly like BindForge's entry sets: "The strings
  … are the only strings in the above example the user should replace, the rest
  are reserved key words", and there are "three possible sub-objects" per action
  set — `StickPadGyro` (up to two axes), `AnalogTrigger` (one axis), `Button`
  (purely digital, "only report an on/off state")
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- That three-way classification is by **value shape**, not by mirrorability: no
  surveyed format encodes the "this id is one half of a left/right pair a
  transform may swap" idea that `discrete-lateral` carries
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: schemas/action-ids/v1/spec.md).
- The closest analogue to `continuous-lateral` is Steam's insistence that a
  camera action must be `absolute_mouse` and never `joystick_move` ("You should
  not use the `joystick_move` based input method for your camera, as mapping the
  opposite direction to get useful 1:1 data … isn't possible") — the same
  "camera is not a key you can flip" rule ADR-011 encodes as an axis kind
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: docs/adrs/011-action-taxonomy-and-core-skeleton.md).
- Steam's IGA carries a `localization` block addressed by language code, with
  action entries referencing `#token`s — i.e. the "native action name" a manifest
  must map is a localization *key*, and the display string is separate data
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Modal/hard-docked UI is modelled as a **layer**, not a constraint:
  `action_layers` entries name a `parent_set_name` ("to handle cases when you
  have in-game UI that pops up over normal gameplay like voting screens"), and
  Unity likewise models context as enable/disable-able Action Maps and one
  `InputActionMap` per context ("gameplay")
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/Actions.html).
- Valve's own anti-pattern warning matches ADR-013's core refusal: about the
  escape hatch that fires a real OS-level mouse event with an action, "You should
  never use this trick as a crutch for forcing specific fixed input bindings for
  your actions. That defeats the entire purpose of the 'actions, not inputs'
  paradigm and giving the user full control over their configuration"
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: docs/adrs/013-game-manifests.md).
- SDL's device-mapping line is the same shape as the manifest's action table —
  `"GUID,name,mapping"` with named semantic slots (`a:b0 … dpup:h0.1 leftx:a0`) —
  and it routes whole device families by a **reserved id**: "Under Windows there
  is a reserved GUID of `xinput` that covers all XInput devices", which is direct
  prior art for ADR-013 reserving a codec *family id* space without shipping
  implementations (source: https://wiki.libsdl.org/SDL2/SDL_GameControllerAddMapping).
- The community-sourced mapping database for that format is one flat text file
  vendored per game and appended to by hand
  (source: https://github.com/gabomdq/SDL_GameControllerDB/blob/master/gamecontrollerdb.txt;
  source: https://github.com/mdqinc/SDL_GameControllerDB).
- Unity's action model solves the identity problem the manifest's join depends on:
  each action has a name unique in its map *and* a unique id, "which you can use
  to reference the Action. The ID remains the same even if you rename the Action"
  (same rule for maps) — i.e. the industry's answer to native-name churn is a
  stable id carried alongside the name (source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/Actions.html).
- Unity also composes parts into one action (`AddCompositeBinding("Dpad")
  .With("Up", "<Keyboard>/w") …`) and types every action as `Value`, `Button` or
  `Pass-Through` — the nearest thing in an engine to BindForge's "one action,
  several physical endpoints" and axis-kind data
  (source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/Actions.html).
- Source-engine games record bindings as console lines in `config.cfg`
  (`bind w +forward`), with `config_default.cfg` as the shipped default executed
  when no `config.cfg` exists, plus per-class config files (`scout.cfg`, …) that
  the game auto-executes — i.e. a per-context base/patch structure living in the
  game's own format, and press/release semantics carried by the `+`/`-` prefix
  convention (source: https://wiki.teamfortress.com/wiki/Scripting).
- Source also shows why a manifest's exclusivity facts matter: opposing actions
  `+moveleft`/`+moveright` cancel each other in the stock game, and the community
  works around it with alias state machines ("null-cancelling movement")
  (source: https://wiki.teamfortress.com/wiki/Scripting).
- Elite Dangerous ships bindings as XML presets that a community maintainer wrote
  an XSD for, and that maintainer's published regrets are directly relevant:
  the format is "not a terribly designed thing with much redundancy and little
  coherence", "Documentation around file format is very sparse (a `Help.txt`
  shipping with the game that has a December 2015 timestamp) and done by
  inference", vendor-shipped presets omit `PresetName`/`MajorVersion`/`MinorVersion`
  (the game then takes the preset name from the **file name**), unbound controls
  are the sentinel `{NoDevice}`, axes carry an inversion flag and a deadzone, and
  buttons carry primary/secondary inputs, modifiers and a `ToggleOn` flag. Their
  validator "traverses binds files in the user profile and makes sure the preset
  name and version correspond to the file name, and that no duplicates are
  present to confuse the game" — the same class of check ADR-013 wants from CI
  (source: https://github.com/ngollan/Elite-Binds).
- Per-game binding files are the thing players lose when a game patches: the
  standing community advice for Elite Dangerous is to zip a copy of the bindings
  folder before every major update, because updating game files "has a habit of
  wiping out the custom key bindings that you've spent hours meddling with" —
  which is why the community maintains `.binds` tooling at all
  (source: https://www.pixelbandits.org/2022/11/how-and-why-to-back-up-key-bindings-in-elite-dangerous/;
  source: https://github.com/ngollan/Elite-Binds).

### Versioning, pinning, and drift (the ADR's central mechanic)

- Steam's equivalent of a manifest version is a pair of counters plus targeting
  flags, in the action block: `major_revision`, `minor_revision`,
  `major_revision_affects_xinput`, `major_revision_affects_mousekb`; "Major
  revisions are used for large, breaking changes to Steam Input API, such as
  adding it to an existing game or **changing of action sets**" — a breaking
  action-table change is exactly ADR-013's bump trigger
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- The flags exist because forcing an update is a player-visible cost: the flags
  narrow which populations are reset, and Valve recommends querying
  `GetDeviceBindingRevision` and prompting the player "rather than force players
  who've constructed custom configurations to rebuild them" — the same trade the
  ADR makes invisible by resolving drift in CI instead of at the player
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: https://partner.steamgames.com/doc/api/isteaminput).
- Steam also already has the base/patch adoption behavior ADR-014 builds: "if the
  user makes edits and manually selects another configuration they will no longer
  be pointed at that official configuration"
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Valve's upgrade procedure distinguishes breaking from additive edits to an
  action table — "adding/removing a new action set or adding actions that must be
  bound" is breaking and requires re-uploading every controller config and
  retiring the old one, while "adding a new optional action that does not need to
  be bound in every configuration" is a single upload with a "Use Action Block"
  override — a manual, per-artifact versioning process of the kind ADR-013
  replaces with CI
  (source: https://partner.steamgames.com/doc/features/steam_controller/workshop_uploaded_configs).
- Valve moved the manifest *into* the shipped data for drift reasons: for new
  titles it no longer recommends Workshop-hosted configurations, recommending
  bundled Action Manifest Files because they can be "checked into Perforce or
  other revision control tools", handle branch/AppID differences, and because
  "Updates are atomic rather than being done via two publishing steps"
  (source: https://partner.steamgames.com/doc/features/steam_controller/action_manifest_file).
- An Action Manifest File is a manifest in the ADR's sense one level up: it names
  the official configurations by closed device-type string and by load priority,
  with paths relative to the manifest itself, and "the first one on the list will
  be considered the highest priority" — a structure ADR-016's seed-preset choice
  will face
  (source: https://partner.steamgames.com/doc/features/steam_controller/action_manifest_file;
  source: https://partner.steamgames.com/doc/features/steam_controller/workshop_uploaded_configs).
- Valve's documented example of shipping more than one official configuration is
  a left-handed one: "You may want to create an official 'Southpaw' (left-handed)
  mode, for example" — prior art for the "built-in presets a profile may be
  `seeded-from`" half of 013
  (source: https://partner.steamgames.com/doc/features/steam_controller/workshop_uploaded_configs).
- The master action file is expected to live in version control even though the
  client re-derives its own copy from it: "The master IGA file you create will
  only ever be [used] during development (but it should still be saved,
  maintained, and stored in your version control, etc)" — the same stance as
  `data/manifests/` being reviewed, versioned data
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Chrome's manifest is the closest everyday example of the version gate the
  contract uses: `manifest.json` is required at the package root, `manifest_version`
  is an integer whose "only supported value is 3", and `version` is required
  (source: https://developer.chrome.com/docs/extensions/reference/manifest).
- Chrome also demonstrates a *consumer-side* pin the ADR does not have:
  `minimum_chrome_version` makes the package declare the oldest producer it works
  on, and adding it retroactively means users below the floor stop receiving
  updates — the failure mode of a manifest that requires a newer core
  (source: https://developer.chrome.com/docs/extensions/reference/manifest).
- Chrome's manifest fields are length-capped (`name` ≤ 75 chars, `description`
  ≤ 132, `short_name` ≤ 12) — prior art for bounding human-authored display
  strings inside a machine-validated manifest
  (source: https://developer.chrome.com/docs/extensions/reference/manifest).
- "Re-validate everything downstream of a data change" is an established CI
  practice, not an invention: Rust's crater "detects regressions … by building a
  large number of crates, running their test suites and comparing the results
  between two versions", and can "build and test without dependency updates or
  network access" (source: https://github.com/rust-lang/crater).
- Debian's next-generation CI ships it as a named workflow: Debusine lists a
  `reverse_dependencies_autopkgtest` workflow and open design work to "control
  packages tested during autopkgtest of reverse dependencies"
  (source: https://freexian-team.pages.debian.net/debusine/reference/tasks/worker/autopkgtest.html).
- Ubuntu's equivalent is documented as a maintenance step — automatic package
  testing via DEP-8, with a dedicated "Checking reverse dependencies" procedure
  for maintainers (source: https://ubuntu.com/project/docs/how-ubuntu-is-made/processes/automatic-package-testing-autopkgtest/;
  source: https://ubuntu.com/project/docs/contributors/advanced/check-reverse-dependencies/).

### Fingerprints, presets, and the format-level precedents worth copying

- "A record carries a digest of the artifact it describes" is standardised
  practice: W3C Subresource Integrity defines integrity metadata as an algorithm
  token plus a base64 digest of the resource, with a closed, strength-ordered
  algorithm set (`sha256`, `sha384`, `sha512`) and explicit algorithm agility for
  the day one is broken (source: https://www.w3.org/TR/SRI/).
- SRI also states the cost of a *small* hashed artifact: an attacker can
  "brute-force specific values in an otherwise static resource" by precomputing
  digests for likely variants, and integrity metadata sent over a non-secure
  context protects nothing — both directly relevant to a "detected config
  fingerprint" of a small per-game config file travelling through a hub
  (source: https://www.w3.org/TR/SRI/).
- Steam's config ecosystem already has a hub-of-presets analogue: Steam Input
  configurations are published artifacts identified by a file id, uploaded per
  game, and selected per device type with a load priority — the shape ADR-018's
  hub will inherit
  (source: https://partner.steamgames.com/doc/features/steam_controller/workshop_uploaded_configs).
- The golden-fixture model BindForge uses has a direct TOML-native precedent:
  `toml-test` is a "language-agnostic test suite to verify the correctness of
  TOML parsers and writers", split into valid and invalid corpora, where invalid
  tests "should try to test one thing and one thing only" and "should be named
  after the fault it is trying to expose" — the same rule `action-ids/v1/spec.md`
  states for its own fixtures (source: https://github.com/toml-lang/toml-test;
  source: schemas/action-ids/v1/spec.md).
- `toml-test` also maintains *per-version* file lists for one corpus ("not all
  tests are valid for every version of TOML; … the 1.0 tests contain a test that
  trailing commas in tables are invalid, but in 1.1 this should be considered
  valid") — a model for how a future `manifests/v2` can keep v1's corpus while
  changing what is a fault (source: https://github.com/toml-lang/toml-test).
- `toml-test` treats a nesting cap as a security recommendation, not a nicety:
  "Having a limit on table nesting is probably a good idea, as … a relatively
  short TOML document with several thousand nested tables … may use an inordinate
  amount of memory or CPU time. A limit of 128 or 256 should be more than enough"
  — BindForge's 16 is far tighter, which is consistent with a grammar that admits
  no tables at all (source: https://github.com/toml-lang/toml-test;
  source: crates/core/src/toml.rs).
- TOML v1.1.0 (the spec version the contract cites) was published 12/18/2025 and
  is the current full spec; it also fixes `.toml` as the extension and
  `application/toml` as the MIME type, and states that "Defining a key multiple
  times is invalid" and "you cannot define a table more than once"
  (source: https://toml.io/en/v1.1.0; source: schemas/README.md).

## Prior art

Formats that carry per-game input facts, by how they answered the same questions:

| System | Game identity | Action identity | Version/drift | Constraints | Presets |
|---|---|---|---|---|---|
| Steam Input IGA / Action Manifest | Steam AppID in the file name | name + `#localization` token, inside action sets, closed kind set (StickPadGyro / AnalogTrigger / Button) | `major_revision`/`minor_revision` + `major_revision_affects_xinput`/`_mousekb`; breaking vs optional upload procedure | action-set **layers** with `parent_set_name`, not key constraints | official configurations per device type, priority-ordered, "Southpaw" named as an example |
| SDL controller DB | device GUID, reserved `xinput` family id | fixed semantic slot names (`a`, `dpup`, `leftx`) | one rolling text file, hand-appended | none | none |
| Unity Input System | n/a (engine-internal) | action name **plus stable id that survives rename**, grouped in Action Maps | asset/package versioning | composite bindings; per-map enable/disable | Action Maps as switchable layouts |
| Source engine `config.cfg` | per-game cfg directory | console command names (`+forward`, `+moveright`) | none; `.vpk` content overrides cfg files | none in data — opposing actions interact in engine code only | per-class cfg files, `load_itempreset 0–3` |
| Elite Dangerous `.binds` | per-user profile directory | XML action names, primary/secondary + modifiers, `{NoDevice}` sentinel | `PresetName`/`MajorVersion`/`MinorVersion`, expected to match the file name | redundancy/coherence problems noted by the XSD author | vendor preset files, `SortOrder` |
| Chrome `manifest.json` | extension id (`key`) | n/a | `manifest_version` (only `3` accepted) + `version` + `minimum_chrome_version` | required/optional key sets | n/a |

(source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
source: https://partner.steamgames.com/doc/features/steam_controller/action_manifest_file;
source: https://partner.steamgames.com/doc/features/steam_controller/workshop_uploaded_configs;
source: https://wiki.libsdl.org/SDL2/SDL_GameControllerAddMapping;
source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/Actions.html;
source: https://wiki.teamfortress.com/wiki/Scripting;
source: https://github.com/ngollan/Elite-Binds;
source: https://developer.chrome.com/docs/extensions/reference/manifest)

What won by convention in this space:

- A **flat, human-editable text file per game**, kept in revision control beside
  the thing that consumes it (source: https://partner.steamgames.com/doc/features/steam_controller/action_manifest_file;
  source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- A **closed vocabulary** of action kinds and container names, with the human
  editable part restricted to names and tokens
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- A **store-assigned game id** carried in the file name rather than an invented
  slug (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- **Action id stable across rename**, carried next to the mutable display name
  (source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/Actions.html).

What they later regret, in their own words or by visible workaround:

- Elite Dangerous's binding format: redundancy, incoherence, near-zero
  documentation, version/name mismatch caught only by a hand-written validator
  (source: https://github.com/ngollan/Elite-Binds).
- Valve's own warning against using per-action hard-wired input events as a
  crutch, because it destroys the user's control
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Valve abandoning Workshop-hosted configs for bundled manifests after two-step
  publishing proved non-atomic across branches and AppIDs
  (source: https://partner.steamgames.com/doc/features/steam_controller/action_manifest_file).
- Community players keeping manual backups because a major update wipes their
  custom bindings — the exact failure ADR-013 prices in CI instead
  (source: https://www.pixelbandits.org/2022/11/how-and-why-to-back-up-key-bindings-in-elite-dangerous/).
- No surveyed format separates facts from policy, and none encodes exclusivity
  or docked-key facts as data — Steam pushes that into layer structure and engine
  behavior (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: https://wiki.teamfortress.com/wiki/Scripting).

## Platform notes

- Manifests are per-game data keyed to a directory name (`data/manifests/<game>/
  manifest.toml`), so the game id becomes a path component: the harness already
  treats case-insensitive contributor filesystems as a real constraint, requiring
  case-unique names, and the same rule needs saying for game directory names and
  for any future per-version subdirectory (source: data/manifests/README.md;
  source: crates/core/tests/conformance.rs; source: schemas/README.md).
- The version-directory invariant is load-bearing here too: `check_dir_version`
  refuses a document whose declared major differs from the `vN/` directory it
  loads from, which is how a `manifests/v1` file could never be read with another
  major's grammar (source: crates/core/src/document.rs).
- Where a game's config actually lives is per-OS and per-store, and the surveyed
  systems publish those paths as first-class facts (Steam `controller_config`
  paths for Windows, macOS and Linux; TF2's `tf/cfg` under the Steam install) —
  facts ADR-013 explicitly defers to ADR-017's config-path discovery, which means
  the manifest's fingerprint input has no source location yet
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: https://wiki.teamfortress.com/wiki/Scripting; source: docs/adrs/013-game-manifests.md).
- Toolchain floor for the reference implementation: `bindforge-core` depends on
  nothing, and CI runs the pinned rustup toolchain with `cargo test --workspace
  --locked`, with no database and no network assumption for the gate
  (source: crates/core/Cargo.toml; source: .github/workflows/check.yml).
- The conformance corpus is the only cross-implementation check in the repo, and
  it runs inside `cargo test`; the Windows leg of the gate is a manual contributor
  run, not a CI job, so a manifest corpus's Windows behavior is pinned only by
  the name/case rules (source: crates/core/tests/conformance.rs;
  source: crates/core/tests/gate_contract.rs; source: .github/workflows/check.yml).

## Risks & unknowns

- **Space-separated encodings vs. game-native names.** The action table has to
  ride a one-line string array because the grammar forbids tables and inline
  tables, but the surveyed games' native names are human words: Steam uses
  "Move"/"Camera" with localized display text, and Elite Dangerous XML carries
  multi-word control names. A `"Move Left aim.camera none"` element is ambiguous
  under a whitespace split, and every alternative (quoted-name-plus-space
  splitting, an explicit separator, a fixed field order with quoted first field)
  interacts with the closed escape table
  (source: schemas/action-ids/v1/spec.md;
  source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: https://github.com/ngollan/Elite-Binds).
- **Axis kind duplicated between manifest and registry.** The registry already
  owns one axis kind per id, and the spec says the arrays *are* the id set and an
  entry naming a fourth axis kind is refused; a manifest column that repeats it
  creates two answers to one fact, which is what the "presence in `fixtures/valid/`
  is the definition of supported" rule is designed to prevent elsewhere
  (source: schemas/action-ids/v1/spec.md; source: schemas/action-ids/v1/registry.toml;
  source: docs/adrs/013-game-manifests.md).
- **The input vocabulary cannot name most of what a constraint would constrain.**
  `keyboard:` is 48 members with no Tab, Enter, Esc, Shift, Ctrl, Alt, arrows or
  F-keys, and members are closed because "accepting new members widens what every
  validator accepts and therefore rides a new vocabulary major"; a manifest
  claiming a game will not honour `Tab` is unrepresentable today
  (source: schemas/reach-classes/v2/spec.md;
  source: https://wiki.teamfortress.com/wiki/Scripting).
- **Fingerprint semantics are undefined and cheaply harmful.** A "detected config
  fingerprint" needs the algorithm, what bytes are hashed, and what a mismatch
  means; SRI's own security section warns that a digest of small, guessable
  content leaks information by brute force, and that fingerprints carried over an
  unauthenticated channel are worthless — relevant the moment the hub ships
  manifests and their fingerprints together (source: https://www.w3.org/TR/SRI/;
  source: docs/adrs/013-game-manifests.md).
- **A manifest is a new untrusted-input surface with new shapes.** The existing
  budgets bound size and *nesting*, but exclusivity groups are set/part structure:
  nothing today bounds the number of groups, the size of a group, the number of
  action-table rows, or a group naming the same key twice — the same class of hole
  the chain depth bound closed for profiles (source: schemas/README.md;
  source: crates/core/src/toml.rs; source: crates/core/src/chain.rs).
- **Hostile config files are a known client-hanging vector in games.** TF2's own
  scripting docs record that looping configs "will cause the TF2 client to hang"
  and that `wait` is disabled on some servers as a countermeasure — evidence that
  game-side config semantics are powerful enough to exhaust a client, and
  `bind`/`exec` chains run arbitrary console commands from a text file
  (source: https://wiki.teamfortress.com/wiki/Scripting).
- **Anything that runs a corpus of hostile files needs sandboxing discipline**:
  crater's README warns "DO NOT RUN CRATER IN AN UNSANDBOXED ENVIRONMENT … Crater
  executes malicious code", and the repo already keeps advisory scanning off the
  PR path and pins every third-party action to a SHA
  (source: https://github.com/rust-lang/crater; source: .github/workflows/advisories.yml;
  source: .github/workflows/check.yml).
- **The CI re-validation matrix has no home and no cost model.** Two PR jobs with
  15-minute timeouts exist today; profiles-per-manifest-version is a growing
  quadratic, and the repo's precedent for expensive recurring checks is a
  scheduled job whose findings are triage input rather than a gate
  (source: .github/workflows/check.yml; source: .github/workflows/advisories.yml;
  source: docs/adrs/013-game-manifests.md).
- **Round-trip obligation is real and asymmetric.** `serialize` must reproduce any
  valid manifest semantically, and the reach-table exemption shows the harness
  will carry per-format behavior differences — a manifest with several string
  encodings is the format where round-trip failure is most likely
  (source: crates/core/tests/conformance.rs; source: crates/core/src/document.rs).
- **Grammar drift risk against TOML 1.1.** The contract cites a subset of TOML
  v1.1.0, v1.1.0 adds escapes (`\e`, `\xHH`) and syntax (multi-line arrays with
  trailing commas, inline tables, dotted keys) that the contract refuses, and
  `toml-test` maintains separate test lists per TOML version — so "legal TOML,
  refused by us" cases need pinning per format, per version
  (source: https://toml.io/en/v1.1.0; source: https://github.com/toml-lang/toml-test;
  source: schemas/README.md).
- **The taxonomy is small against real games.** 40 ids exist, ARPG-scoped;
  ADR-011 says ids are a compatibility promise grown only through manifest review,
  so the first real manifest lands as a batch of registry additions plus a
  manifest — the review order between the two is unstated
  (source: schemas/action-ids/v1/registry.toml; source: docs/adrs/011-action-taxonomy-and-core-skeleton.md;
  source: schemas/action-ids/v1/spec.md).
- **`manifest id` has no grammar.** ADR-009's superseded example writes
  `for = "remnant2@0.1"`, a single lowercase segment with a dotted version, while
  the taxonomy's id grammar is exactly two lowercase dot-separated segments capped
  at 63 octets per segment, and format majors are `vN` — three incompatible
  spelling conventions for the one string every profile pins
  (source: docs/adrs/009-canonical-profile-and-manifest-model.md;
  source: schemas/action-ids/v1/spec.md; source: schemas/README.md).
- **Unbounded string values in a document that will be displayed.** Chrome caps
  name/description lengths; the contract caps only total document size, and every
  refusal quoting an untrusted value must escape and truncate — a manifest full of
  long native names stresses that rule in a new way
  (source: https://developer.chrome.com/docs/extensions/reference/manifest;
  source: schemas/README.md; source: crates/core/tests/conformance.rs).

## Open questions

- What is the exact encoding of one action-table row inside a one-line string
  array, and how is a game-native name containing spaces, quotes or `#` written —
  and does the native name need its own grammar (length cap, allowed characters)
  like the input and action ids have?
- Does the manifest restate axis kind at all, or does it name only taxonomy ids
  and let `ActionRegistry::axis_kind` answer? If it restates it, which side wins
  on disagreement, and is that a parse refusal or a validation warning?
- Are "dead keys" and "exclusivity group members" `keyboard:*`/`mouse:*`/
  `gamepad:*` ids from the closed v2 vocabulary? If yes, does ADR-013 force the
  input vocabulary to grow (a vocabulary major, per `reach-classes/v2/spec.md`),
  or are constraints expressed some other way?
- What exactly does "hard-docked UI constraint" mean as data — a set of actions
  that share one control, a control the game rebinds nowhere, or a
  Steam-style parent/child layer where a set of actions is only available in a
  given context?
- What is the manifest id grammar and the manifest data-version grammar, and how
  does the `for = "<id>@<version>"` pin in ADR-014 spell both? Should the id be a
  store id (Steam AppID precedent) plus a BindForge slug, and should the data
  version be semver-style or a `major_revision`/`minor_revision` pair like
  Steam's?
- What is hashed for the "detected config fingerprint" — the whole config file, a
  normalized subset, or the action table alone — with which algorithm, and what
  must a consumer do on mismatch: refuse, warn, or fall back to a recorded
  version?
- Which budgets does the manifest format add to the shared parse path (rows,
  groups, group members, name lengths), and do they become new named constants
  like `MAX_CHAIN_DEPTH`?
- Who owns the "built-in presets" section semantically: does a preset record the
  game's shipped layout as data (a base a profile can be `seeded-from` by id), and
  what makes a preset id stable when the game renames its presets between patches
  (Unity's rename-surviving id is the model — is a stable preset id required)?
- What is the rule for a manifest bump that is additive (one new action row) —
  same version, new version, or a minor revision — and does CI gate on it or
  report it, given the repo's precedent of keeping expensive/external checks off
  the PR path?
- Does the manifest record the *shipped default binding* of each action, and if
  so does ADR-015's "transforms apply only to bases" get its base from the
  manifest rather than from a preset?
- Which fixtures does `manifests/v1` owe beyond the standard set — at minimum an
  unknown-entry refusal, a version refusal, an unregistered taxonomy id
  refusal, an unknown input id refusal, and a duplicate-membership refusal — and
  which of those are parse-level versus registry-validation-level?
