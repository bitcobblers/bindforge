# Research: Transforms as named, preconditioned policies (ADR-015)

Scope: the ADR defines the transform/policy layer — an ordered stack of named
transforms with preconditions, each declaring the taxonomy tags it touches,
refusing cleanly when it does not fit, operating only on bases and never on a
personal patch — plus a new `schemas/policy/v1/` format and a new
`crates/transforms` crate standing on `crates/core`. It does not decide the
seed preset set (ADR-016), the CLI (ADR-017), or the hub (ADR-018). Lenses
applied: **Prior art**, **Standards**, **Security landscape**, under Findings,
Prior art, and Risks & unknowns.

## Summary

Everything 015 consumes is already shipped and explicitly shaped for it:
`AxisKind::DiscreteLateral` is documented as "the only kind a polarity flip may
touch (ADR-015)", `placement_is_legal` answers a structured `PlacementCheck`
rather than a bool precisely so "ADR-015's precondition has to be able to say
why a placement is illegal", and the manifest's `docked-actions` are specified
as "never a transform target" (source: crates/core/src/action.rs; source:
crates/core/src/placement.rs; source: schemas/manifests/v1/spec.md). Shipped
remapping systems converge on the ADR's shape from the other side: XKB exposes
handedness-adjacent changes as a closed vocabulary of independent named
options (`ctrl:swapcaps`), never a mirror; Sony and Microsoft ship per-button
assignment inside *named profiles*, with fixed untouchable keys; PowerToys
warns about "orphaned" keys instead of silently mangling layouts (source:
https://man.archlinux.org/man/xkeyboard-config.7.en; source:
https://www.playstation.com/en-us/support/hardware/set-up-edge-controller/;
source: https://news.xbox.com/en-us/2023/08/03/keyboard-button-mapping-for-xbox-controllers/;
source: https://learn.microsoft.com/en-us/windows/powertoys/keyboard-manager).
The precondition-as-data posture has strong precedent — Terraform's
`precondition {condition, error_message}` blocks and CSS `@supports` both keep
conditions as reviewable data over loaded facts — while the rejected
alternative (free-form predicates) is exactly how AutoHotkey ships, i.e. a
policy from the network becomes arbitrary code (source:
https://developer.hashicorp.com/terraform/language/validate; source:
https://developer.mozilla.org/en-US/docs/Web/CSS/@supports; source:
https://www.autohotkey.com/docs/v2/). The largest gaps are in what the ADR
leaves unsaid: the precondition vocabulary's grammar, what a "cluster" is as
data (no cluster concept exists in core, and hands are inputs to the placement
predicate, never derived facts), what a mouse-button transform can
precondition on when mouse ids have no coordinates and no reach grades, and
what a transform's output *is* in a chain (source: crates/core/src/placement.rs;
source: docs/adrs/015-transforms-as-preconditioned-policies.md).

## Findings

### What 015 stands on, as shipped on `main`

- The axis-kind rule 011 "only recorded" has its consumer pre-announced in the
  enum's own doc: `DiscreteLateral` is "the only kind a polarity flip may
  touch (ADR-015)" and `ContinuousLateral` is "camera look, whose inversion is
  a game option, never a key swap" — the ADR's camera example is the enum
  comment verbatim (source: crates/core/src/action.rs).
- The shipped registry lists 39 ids: exactly four `discrete-lateral`
  (`move.steer-left|right`, `move.strafe-left|right`), one `continuous-lateral`
  (`aim.camera`), and 34 `none` — so `swap-polarity` today can legally touch
  only the two strafe ids or the two steer ids, and the registry is "the only
  id list", grown only by PR (source: schemas/action-ids/v1/registry.toml).
- The placement predicate is exactly the precondition primitive 015 names:
  `placement_is_legal(from, to, moves, bound, tables, geometry)` with clause
  (a) one common `(dx, dy)` plus distinct targets and clause (b) a per-hand
  degradation bound (`GradeBound`), returning `PlacementCheck {legal,
  shape_ok, bound, delta, move_deltas, degradations}` because "which
  placements a shipped policy chooses is ADR-015's, and this stays
  region-agnostic" (source: crates/core/src/placement.rs).
- Hands are **inputs** to that predicate, not facts read from data: "nothing in
  a table says which hand *chose* to type a key, because the left hand can
  rest on IJKL too" — so a precondition phrased "the locomotion cluster is
  contiguous and **left-handed** in this base" is not answerable from shipped
  data alone; see Risks (source: crates/core/src/placement.rs;
  source: docs/adrs/015-transforms-as-preconditioned-policies.md).
- The v2 severity scale is shipped data in a fixed order — `["home", "thumb",
  "stretch", "far", "off-reach"]`, array order *is* the scale — and the graded
  set and the ANSI-US grid agree on "exactly the 48 members of the closed
  `keyboard:` set" (source: crates/core/src/document.rs; source:
  crates/core/src/placement.rs; source: schemas/reach-classes/v2/left.toml).
- `swap-mouse-buttons()` operates outside the reach/geometry machinery: reach
  grades attach to `keyboard:*` only (a `gamepad:`/`mouse:` member in a reach
  table is a pinned refusal), and `placement_is_legal` refuses any non-
  `keyboard:*` id outright — so this transform's preconditions can only be
  vocabulary and manifest constraints, not legality (source:
  schemas/reach-classes/v2/fixtures/invalid/mouse-input.toml; source:
  crates/core/src/placement.rs; source: docs/adrs/012-input-vocabulary-and-reach-classes.md).
- The manifest constraints 015 must honour are already typed and bounded:
  `exclusive-groups` (2..=16 members per group, at most 64 groups, a member
  never in two groups), `dead-keys`, `docked-actions` ("an action the game
  hard-docks to that input, never a transform target", disjoint with dead
  keys), and `presets` — and the spec already states "a transform's
  preconditions read this document's constraints" (source:
  schemas/manifests/v1/spec.md; source: crates/core/src/manifest.rs).
- 014's chain machinery is further along than 015's Context text suggests:
  `chain_ops` already ships `load_chain` (iterative cross-file walk refusing
  `Cycle`/`ChainTooDeep` at `MAX_CHAIN_DEPTH`), `resolve` against a manifest,
  and `fork`/`diff`/`merge`; `profile/v2` deltas are typed (`set:
  Vec<(ActionId, InputId)>`, `clear`, `swap: Vec<(ActionId, ActionId)>`) with
  `kind` required at v2 and the `generated-by` provenance label capped at
  128 octets — the natural sink for a transform's output (source:
  crates/core/src/chain_ops.rs; source: crates/core/src/document.rs).
- The contract tree already names `policy` as the next format: `parse`'s doc
  says "profile, patch, **policy**, and manifest documents arriving from disk,
  from another user, or from the network are untrusted at every entry point",
  `format_majors`' doc says its list "is where a future
  `manifest`/`patch`/`policy` major gets declared when its directory is
  added", and the conformance harness comment says a new `schemas/*/v*/`
  directory is picked up "with no harness change, which is also how `patch`
  and `policy` join when their turn comes" — the integration points for
  `schemas/policy/v1/` are enumerated in the shipped code (source:
  crates/core/src/document.rs; source: crates/core/src/error.rs; source:
  crates/core/tests/conformance.rs).
- The two named refusal channels 015 inherits are visible in the code:
  a refusal is a `CoreError` variant (action.rs deliberately adds none:
  "Every refusal reuses an existing `CoreError` variant, so no new variant
  enters the error contract") while a *verdict* is structured data
  (`PlacementCheck`) — the "inapplicable transform reports and no-ops" rule
  sits in the second channel, not the first (source: crates/core/src/action.rs;
  source: crates/core/src/placement.rs).
- The tests' API probe already reads 015 back into the design: the placement
  tests call the predicate "the pure function over loaded data that ADR-015
  preconditions its transforms on", and the API probe file exists to pin the
  surface "015 preconditions its transforms on" — the surface is contractual
  before `crates/transforms` exists (source: crates/core/tests/placement.rs;
  source: crates/core/tests/zz_api_probe.rs).

### Named opt-in transforms — what shipped systems chose (Prior art lens)

- XKB's answer to "swap two keys" is a **closed vocabulary of independent,
  comma-separated named options** — `ctrl:swapcaps`, `altwin:swap_lalt_lwin`,
  `mousekey:*`-style families — applied over any layout, each one opt-in on
  its own; there is no option that mirrors a region of the keyboard
  (source: https://man.archlinux.org/man/xkeyboard-config.7.en).
- XKB ships handedness as a *named layout variant* — `us(dvorak-l)` "English
  (Dvorak, one-handed, left)" and `us(dvorak-r)` — a curated whole layout, not
  a computed mirror of `us(dvorak)`: prior art for the ADR's rejection of "one
  global mirror transform" (source: https://man.archlinux.org/man/xkeyboard-config.7.en;
  source: docs/adrs/015-transforms-as-preconditioned-policies.md).
- The DualSense Edge model is a **named profile** holding independent,
  individually-assigned decisions (button assignments, per-stick sensitivity,
  trigger input ranges), switchable mid-game by an Fn shortcut, with a Reset
  that returns assignments to defaults — and with hard system constraints the
  UI refuses rather than corrupts: "You always need to assign circle and cross
  buttons", the Fn button is unassignable, L2/R2 may swap but no other button
  may become a trigger (source:
  https://www.playstation.com/en-us/support/hardware/set-up-edge-controller/).
- Sony's own troubleshooting warns the two layers can conflict: unexpected
  inputs mean "reviewing the in-game controller settings in case they clash
  with your … profile" — i.e. vendor policy and game fact are separate layers
  whose mismatch is a known, surfaced failure, the same base-vs-manifest
  mismatch 015's preconditions exist to catch (source:
  https://www.playstation.com/en-us/support/hardware/set-up-edge-controller/).
- Microsoft's controller button mapping (Elite Series 2 / Adaptive Controller,
  later system button mapping) is likewise per-profile and per-slot, and it
  prices applicability honestly: "Keyboard input support is game dependent, so
  you may notice some games will offer full support, partial support, or no
  support" — an inapplicable-policy case acknowledged by the vendor (source:
  https://news.xbox.com/en-us/2023/08/03/keyboard-button-mapping-for-xbox-controllers/).
- PowerToys Keyboard Manager stores remaps as data rows but bounds them with a
  fixed untouchable set ("Win+L and Ctrl+Alt+Del cannot be remapped as they
  are reserved by the Windows OS"; Fn usually not remappable) and, instead of
  silently producing a mangled keyboard, **warns about "orphaning" a key** —
  prior art for "reports and never mangles" as a design obligation (source:
  https://learn.microsoft.com/en-us/windows/powertoys/keyboard-manager).
- SharpKeys is the cautionary contrast: it edits the Windows boot-time
  `Scancode Map` registry value with no preconditions at all, admits it will
  "not protect you from yourself — if you disable your DEL key and can't login
  … you'll have to reformat", and its recovery is Safe Mode plus deleting the
  registry value — the no-precondition end of the spectrum 015 designs away
  from (source: https://github.com/randyrants/sharpkeys).
- The Linux remappers show the config-data direction: xremap configures named
  remaps in YAML with app- and device-scoped blocks and documents order
  semantics ("Order is important, first match wins, so global always goes
  last"); input-remapper "maps any input to any other input" with per-device
  presets saved as files — neither carries a precondition vocabulary, so both
  leave "applies to this base?" to the author (source:
  https://github.com/xremap/xremap; source:
  https://github.com/sezanzeb/input-remapper).
- Steam Input's Action Set Layers are the gaming-industry analogue of a named
  stack above a base: layers "overlay incremental changes to the active Action
  Set" and name their parent by id — a transform naming the base it modifies
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Camera inversion really is shipped as a game option, not a rebinding:
  PCGamingWiki catalogues "invert mouse y-axis" as a standard per-game input
  option ("This only applies to games with some sort of camera control"), the
  same split 011/015 draw between game option and binding (source:
  https://www.pcgamingwiki.com/wiki/PCGamingWiki:Editing_guide/Input).

### Preconditions as data — the vocabulary question

- Terraform names the exact construct: `lifecycle { precondition { condition,
  error_message } }` — a declarative condition evaluated before the
  operation, with a **mandatory** message, refusing the operation visibly when
  false; and its docs separate preconditions from tests and policy checks,
  matching the ADR's split between transform logic and CI conformance
  (source: https://developer.hashicorp.com/terraform/language/validate).
- CSS `@supports` is the closest shape to "preconditioned transform on
  untrusted ground": conditions are a closed expression grammar over loaded
  capability data (`(display: flex)`, `selector(…)`, `and/or/not`), the rule
  is reviewed as data, and an **invalid condition invalidates the whole rule**
  rather than applying it wrongly — a fail-closed default the policy grammar
  will have to pick deliberately (source:
  https://developer.mozilla.org/en-US/docs/Web/CSS/@supports).
- RFC 6902's `test` operation shows a patch/transform language can carry a
  precondition ("The target the operation is applied to MUST be equal to the
  `value` value") without becoming a program, and RFC 6902's sequential-
  application rule ("Each operation … is applied to the target document; the
  resulting document becomes the target of the next operation") is the default
  semantics a transform *stack* inherits: later preconditions see earlier
  transforms' output (source: https://datatracker.ietf.org/doc/html/rfc6902).
- The ADR's rejected alternative — "conditions as free-form predicates" — is
  what AutoHotkey ships: remapping is a scripting language (hotkeys, hot-
  strings, `DllCall`), powerful and unauditable as a unit; the 015 comparison
  is sound on its own terms (source: https://www.autohotkey.com/docs/v2/).
- Idempotence and re-application have pinned precedent inside the repo:
  `profile/v2` fixtures already pin a `swap` pair "applied twice" and
  `set-then-clear` / "set twice, last element wins" semantics, so the algebra
  of repeated transforms is already partly contract at the patch level
  (source: schemas/profile/v2/fixtures/valid/swap-pair-applied-twice.toml;
  source: schemas/profile/v2/fixtures/valid/set-then-clear-same-action.toml;
  source: schemas/profile/v2/fixtures/valid/set-twice-last-element.toml).

### Vocabulary and sharing — standards (Standards lens)

- The de-facto common denominator for naming physical inputs cross-tool is the
  USB HID usage table chain: the Linux evdev header states "Most of the
  keys/buttons are modeled after USB HUT 1.12", and the HID Usage Tables (1.7
  current) are the source of truth those codes descend from — BindForge's
  OS-neutral `keyboard:`/`mouse:`/`gamepad:` vocabulary sits in the same
  lineage (source:
  https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h;
  source: https://usb.org/document-library/hid-usage-tables-17).
- The one genuinely cross-tool, community-shared input artifact format is SDL's
  game controller mapping DB: one line per device, plain-text `guid,name,button:…,platform:…`
  entries, community-reviewed by PR, loaded by `SDL_AddGamepadMappingsFromFile`
  — evidence that line-oriented data + a fixture/registry discipline is how
  binding data actually gets shared across tools, which is the market
  `schemas/policy/v1/` enters (source:
  https://github.com/mdqinc/SDL_GameControllerDB).
- SDL's DB also models the *maintenance* cost the ADR names for the seed set:
  its README codifies contribution rules ("Your mapping best represents the
  Xbox layout per the mapping guide … not already present in the DB") — a
  curated, reviewed collection, like 015's "curated collection to maintain"
  (source: https://github.com/mdqinc/SDL_GameControllerDB; source:
  docs/adrs/015-transforms-as-preconditioned-policies.md).
- The repo's own standards posture carries over untouched: formats are a
  documented subset of TOML v1.1.0 with a closed per-format entry set,
  golden-fixture conformance "following the JSON Schema Test Suite model", and
  no machine-checkable schema language — `policy/v1` inherits all of it
  (source: schemas/README.md; source: https://toml.io/en/v1.1.0).
- SDL's DB is device-scoped, not game-scoped, and Steam configs are game-
  scoped runtime artifacts; nothing in the surveyed field ships *game-agnostic
  named transforms that precondition on a per-game manifest* — 015's exact
  combination has no shipped prior, which prices the design as genuinely new
  (source: https://github.com/mdqinc/SDL_GameControllerDB; source:
  https://partner.steamgames.com/doc/features/steam_controller/iga_file).

### Adversarial policies — what the format must keep (Security lens)

- Policy documents are the fourth untrusted-document class in a chain that
  already reaches the network: `parse` refuses to trust any of them, the parse
  budget applies before any value is built (1 MiB checked pre-read, 16-deep
  nesting), and `data/profiles/` states the same posture for shipped files —
  `schemas/policy/v1/` needs its own invalid corpus exercising the same
  refusal family (source: crates/core/src/document.rs; source:
  crates/core/src/toml.rs; source: schemas/README.md; source:
  data/profiles/README.md).
- Composite cost grows again the way it did for chains: one chain resolution
  is already bounded at `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB) by
  stated product; a policy is an extra document per application, and if a
  transform's output is itself a stored base document, the per-application
  document count needs a named cap — TF2's exec-chain lesson (a per-file size
  cap defeated by chaining files) is the failure pattern (source:
  schemas/README.md; source: https://wiki.teamfortress.com/wiki/Scripting).
- Precondition *evaluation* is the new attack surface, but the shipped
  primitives are already pure and total over loaded data: `placement_is_legal`
  returns refusals for undefined ids rather than guessing, grades and ranks
  come only from loaded tables, and `load_geometry` grammar-checks the name
  *before* touching the filesystem — a policy that names a geometry, preset,
  or id must ride the same "refuse before asking the filesystem anything"
  rule (source: crates/core/src/placement.rs).
- A closed vocabulary is not automatically safe even when it is data: XKB's
  own options list labels `grab:break_actions` and `grab:debug` "(warning:
  security risk)" inside an otherwise pure keymap system — the vocabulary's
  *effects* need review, not just its grammar (source:
  https://man.archlinux.org/man/xkeyboard-config.7.en).
- The ordering hazard for stacks is the classic rule-correlation hazard:
  xremap pins "first match wins" ordering, RFC 6902 pins sequential mutation,
  and a transform stack that lets two transforms touch the same taxonomy tag
  needs a shipped rule for the second precondition's reference point (the
  base as loaded, or the base as already transformed this pass) — neither is
  defined yet (source: https://github.com/xremap/xremap; source:
  https://datatracker.ietf.org/doc/html/rfc6902).
- Provenance remains label-not-trust across the new layer: transform output
  entering a chain "like any other, gated by validation, not trusted by
  provenance" is already the shipped rule for `generated-by`, whose only
  grammar is a 128-octet cap — whatever the transforms stamp there must not
  become a selection key (source: crates/core/src/document.rs; source:
  docs/adrs/014-profiles-as-chains.md; source:
  docs/adrs/015-transforms-as-preconditioned-policies.md).

## Prior art

How comparable systems answered the same questions 015 poses:

| System | Unit of policy | Named? | Gated by precondition? | Untouchable set | Refuses / no-ops visibly? |
|---|---|---|---|---|---|
| XKB options (`setxkbmap -option`) | single key-swap or behaviour option, closed vocabulary | yes (option names) | no — options assume the key exists | reserved keys (server-internal) | unknown option refused by name (source: https://man.archlinux.org/man/xkeyboard-config.7.en) |
| XKB layout variants | whole curated layout (`us(dvorak-l)`) | yes | n/a (replacement, not transform) | n/a | unknown variant refused (source: https://man.archlinux.org/man/xkeyboard-config.7.en) |
| DualSense Edge profiles | named profile of per-button/per-axis decisions | yes (profile names) | hard UI rules (circle+cross mandatory, no Fn remap, trigger limits) | Fn, PS, circle/cross | UI refuses illegal assignments (source: https://www.playstation.com/en-us/support/hardware/set-up-edge-controller/) |
| Xbox Accessories button mapping | named profile, per-button mapping incl. keys | yes (3 slots) | capability-dependent (game keyboard support) | — | "full support, partial support, or no support" per game (source: https://news.xbox.com/en-us/2023/08/03/keyboard-button-mapping-for-xbox-controllers/) |
| PowerToys Keyboard Manager | remap rows (data) | no names | reserved-keys list | Win+L, Ctrl+Alt+Del, Fn, Win+G | orphaned-key warning; games unsupported warning (source: https://learn.microsoft.com/en-us/windows/powertoys/keyboard-manager) |
| SharpKeys (Scancode Map) | one key→key map, boot-time registry | no | none | triple-byte scancodes unremappable | no protection; Safe-Mode recovery documented (source: https://github.com/randyrants/sharpkeys) |
| xremap / input-remapper | config blocks with app/device conditions | no (YAML/JSON blocks) | app/device match, first-match-wins | — | config-time refusals (source: https://github.com/xremap/xremap; source: https://github.com/sezanzeb/input-remapper) |
| Steam Input | action sets + layers naming a parent set | yes (set names) | game/layer scope | — | runtime overlays, not files (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file) |
| Terraform | resource + `precondition` block | yes (named block) | yes — condition over loaded data, error message required | — | stops the operation, names the failure (source: https://developer.hashicorp.com/terraform/language/validate) |
| CSS `@supports` | rule + feature-query condition | n/a (rules) | yes — closed condition grammar over capability data | — | invalid condition ⇒ rule ignored (source: https://developer.mozilla.org/en-US/docs/Web/CSS/@supports) |
| BindForge 015 (proposed) | named transform + preconditions over loaded data | yes | yes — reach legality (012) + manifest constraints (013) | hard-docked, exclusive-group, dead keys | "reports and no-ops; it never mangles" (source: docs/adrs/015-transforms-as-preconditioned-policies.md) |

What wins by convention:

- **Independent decisions stay independently switchable.** XKB options,
  DualSense per-button assignments, and Xbox per-button mapping all make each
  swap opt-in; none ships a one-switch whole-keyboard mirror — supporting the
  ADR's "stack of independent, named decisions" over the global mirror
  (source: https://man.archlinux.org/man/xkeyboard-config.7.en; source:
  https://www.playstation.com/en-us/support/hardware/set-up-edge-controller/;
  source: https://news.xbox.com/en-us/2023/08/03/keyboard-button-mapping-for-xbox-controllers/).
- **Every mature system carries a hardcoded untouchable set** (PowerToys'
  reserved OS shortcuts, DualSense's circle/cross/Fn/trigger rules,
  SharpKeys' triple-byte refusal) — the ADR's "hard-docked or exclusive-group
  key is never a target" is the data-driven version of a universal practice
  (source: https://learn.microsoft.com/en-us/windows/powertoys/keyboard-manager;
  source: https://www.playstation.com/en-us/support/hardware/set-up-edge-controller/;
  source: docs/adrs/015-transforms-as-preconditioned-policies.md).
- **When the target does not fit, the tool says so rather than guessing** —
  PowerToys' orphan warning, Xbox' partial-support disclosure, Terraform's
  required `error_message`, CSS's drop-the-invalid-rule default — all variants
  of "reports and no-ops" (source:
  https://learn.microsoft.com/en-us/windows/powertoys/keyboard-manager;
  source: https://news.xbox.com/en-us/2023/08/03/keyboard-button-mapping-for-xbox-controllers/;
  source: https://developer.hashicorp.com/terraform/language/validate;
  source: https://developer.mozilla.org/en-US/docs/Web/CSS/@supports).

What they later regret, visibly in their own records:

- SharpKeys' FAQ is a catalogue of bricked keyboards ("I add a bunch of key
  mapping and I can't use my computer anymore? … you're mostly out of luck") —
  the cost of precondition-less remaps (source:
  https://github.com/randyrants/sharpkeys).
- PowerToys' known-issues list: game FPS impact, AltGr breakage, modifier
  remaps breaking gestures — runtime remapping fights its own layer;
  BindForge's transforms are offline data-on-data, deliberately outside that
  fight (source: https://learn.microsoft.com/en-us/windows/powertoys/keyboard-manager).
- Sony's own support has to tell users their profile may "clash" with in-game
  settings — layer mismatch left as user troubleshooting where 015 makes it a
  precondition (source:
  https://www.playstation.com/en-us/support/hardware/set-up-edge-controller/).

## Platform notes

- None of 015's own work is platform-dependent: the transforms are offline
  functions over loaded tables (`placement.rs` is "a pure function over loaded
  data"), so the platform-specific input layers (Windows boot-time Scancode
  Map vs runtime hooks; X11 xkb options vs Wayland's lack of a system
  compositor remap, pushing Linux tools to evdev/uinput; HID usage codes under
  evdev) are the codecs' problem in ADR-017, and this ADR only has to keep the
  produced bases expressible through them (source: crates/core/src/placement.rs;
  source: https://wiki.archlinux.org/title/Input_remap_utilities; source:
  https://github.com/randyrants/sharpkeys; source:
  https://learn.microsoft.com/en-us/windows/powertoys/keyboard-manager).
- Expressibility constraints already bound what a base may contain: the whole
  Windows scancode layer remaps whole keys only — no combinations, no mouse —
  while the games themselves decide whether keyboard or controller inputs are
  honoured at all; a policy that relocates a cluster is only as good as the
  game's config the codec writes (source: https://github.com/randyrants/sharpkeys;
  source: https://news.xbox.com/en-us/2023/08/03/keyboard-button-mapping-for-xbox-controllers/).
- Geometry floor: placement legality today reads only the shipped ANSI-US grid
  (`schemas/keyboard-geometry/v1/ansi-us.toml`); a relocated cluster's
  legality on ISO/JIS is new table files, not new logic, per 012 — but a
  policy shipped now is effectively ANSI-US-tested (source:
  schemas/keyboard-geometry/v1/ansi-us.toml; source:
  docs/adrs/012-input-vocabulary-and-reach-classes.md).
- The closed 48-member `keyboard:` vocabulary has no Tab/Enter/Esc/arrows/F-keys,
  so a WASD→IJKL/OKL move is inside it and a WASD→arrows move is not
  expressible at all — a vocabulary floor a seed policy author must know
  (source: schemas/reach-classes/v2/spec.md; source:
  crates/core/tests/reach_v2.rs).
- Crate shape: `crates/transforms` is the second crate in the ADR-008
  workspace; `crates/core` has no dependency on it (transforms *consume*
  core's types), so the dependency direction keeps core's zero-dependency
  posture intact, and ADR-004's mirrored test hierarchy binds the new crate's
  tests (source: docs/adrs/008-rust-for-the-product.md; source:
  docs/adrs/004-mirror-src-hierarchy-in-tests.md).

## Risks & unknowns

- **The precondition vocabulary's grammar is the whole contract and is
  unnamed.** "A fixed precondition vocabulary over loaded data" is decided;
  its entries, element encodings (space-joined pairs per house style?), and
  refusal fixtures are not — and every choice re-inherits the closed-entry-set
  discipline (source: docs/adrs/015-transforms-as-preconditioned-policies.md;
  source: schemas/manifests/v1/spec.md).
- **"The locomotion cluster is contiguous and left-handed" is not currently
  checkable.** There is no cluster concept in core (only action ids and input
  ids), and hands are explicitly inputs-to-the-predicate, never derived from
  data — contiguity needs a grid computation over base bindings, and
  "left-handed" needs either a declared hand per cluster or a derived one,
  which 012 refused to derive (source: crates/core/src/placement.rs; source:
  docs/adrs/015-transforms-as-preconditioned-policies.md).
- **Tag matching is a new grammar over the closed id set.** The examples use
  globs (`move.steer-*`, `move.strafe-*`); 011 defined no tags beyond the two-
  segment id, and `ActionId` comparison is deliberately byte-exact with no
  prefix semantics — a glob that matches no registry id must be a refusal, not
  a silent empty transform (source: crates/core/src/action.rs; source:
  docs/adrs/015-transforms-as-preconditioned-policies.md).
- **Transform output in a chain is undefined.** Transforms "produce bases" —
  but a base in 014's world is a `profile/v2` `kind = "base"` document with an
  id, a `for` pin, and delta lists. Is a transform's output materialized as a
  new profile document (what id rule? deterministic ids collide with `Cycle`
  guards), or computed at resolution time inside `resolve` (then it is not a
  shareable file)? The ADR's "personal patch … survives base regeneration"
  only works if regeneration is deterministic under `resolve`'s determinism
  surface (source: crates/core/src/chain_ops.rs; source:
  docs/adrs/014-profiles-as-chains.md; source:
  docs/adrs/015-transforms-as-preconditioned-policies.md).
- **`swap` in the deltas is action-pair-wise; `swap-mouse-buttons()` is input-
  wise.** A materialized output must express a mouse-button swap as sets on
  the actions bound to `mouse:left`/`mouse:right` — which needs those actions
  to exist in the base and manifest, another precondition class with no
  fixture yet (source: crates/core/src/document.rs; source:
  schemas/manifests/v1/spec.md).
- **Stack-order semantics unchosen.** RFC 6902 (sequential) and xremap
  (first-match-wins) are the two defaults in the field; whether a second
  transform's preconditions see the first transform's output, and what
  happens when two transforms both declare `move.steer-*`, is the algebra
  every fixture will encode — the patch-level fixtures
  (`swap-pair-applied-twice`) show the repo pins exactly this class of edge
  (source: https://datatracker.ietf.org/doc/html/rfc6902; source:
  https://github.com/xremap/xremap; source:
  schemas/profile/v2/fixtures/valid/swap-pair-applied-twice.toml).
- **Report shape.** "Reports and no-ops" matches `PlacementCheck`'s channel
  (verdict data, not `CoreError`), but there is no shipped type for a
  transform's report, and 017's CLI/hub will display it — escaping and
  truncation rules for the strings inside that report inherit from
  `CoreError`'s posture (source: crates/core/src/placement.rs; source:
  crates/core/src/error.rs; source: docs/adrs/015-transforms-as-preconditioned-policies.md).
- **Budget additions.** Policy joins the untrusted-document classes; the
  stated composite for chains is `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES`, and
  per-structure caps follow the house pattern (`MAX_PATCH_DELTA_ELEMENTS =
  256`-style). A policy needs named caps — transforms per policy, preconditions
  per transform, tags per precondition — or it becomes the chain loophole again
  (source: schemas/README.md; source: crates/core/src/document.rs).
- **The v2 invalid corpus 015 owes** follows the visible pattern: unknown
  entry, wrong-field-count rows, unknown/uppercase ids, a glob matching
  nothing, a transform naming a manifest input that is docked/dead/exclusive
  (must refuse, per manifest spec), a policy at the depth/size boundaries, and
  a version disagreeing with its `v1/` directory (source:
  schemas/manifests/v1/fixtures/; source: schemas/README.md; source:
  crates/core/tests/conformance.rs).

## Open questions

- What are the v1 `policy` entries — `format`, `version`, `id`, `for`?, a
  `transforms` array of what exact row grammar — and is the transform **kind**
  a closed set compiled into `crates/transforms` (`relocate-cluster`,
  `swap-polarity`, `swap-mouse-buttons`) with parameter rows, or an open name
  space the loader refuses unknown kinds for? The ADR's "each **off** unless
  the stack names it" says opt-in; the grammar must say closed-or-open.
- What is a *cluster* as data? Does `relocate-cluster` carry an explicit
  action list, a tag glob, or is there a shipped cluster definition per
  game/policy — and is "contiguous" checked by the grid at precondition time
  or guaranteed by the seed set?
- Is the preconditions vocabulary a closed enumeration (`placement-legal(
  to-hand, bound)`-style atoms referencing `placement_is_legal`'s parameters)
  or composed with and/or/not (CSS's route)? What is the fail mode for a
  malformed precondition — refuse the policy (Terraform) or drop it (CSS)?
- What exactly does a transform apply to: the `Vec<ResolvedBinding>`
  `resolve` computes, or a `profile/v2` base document's delta lists? Does its
  output become a stored document (id rule? `generated-by = "policy:<id>@<revision>"`?
  a pin to the policy like `for` pins manifests?) or a step inside `resolve`?
- If policy output is stored, does re-applying a policy after a manifest
  revision bump produce a *new* id (chain churn, CI re-validation matrix) or
  overwrite the old (contradicting "personal patch sits above them in the
  chain")?
- Which parameters does `swap-mouse-buttons` accept as preconditions when
  mouse ids have no geometry and no grades — manifest-side only ("the game
  binds both mouse:left and mouse:right to bound actions")? Is that a
  precondition class core can evaluate at all today?
- What bounds does the harness's zero-fixture rule force on day one, and does
  the conformance round-trip rule (serialize + re-parse equality) make policy
  documents ordered data like manifests ("row, group, and member order is
  data")?
- How do `fork`/`diff`/`merge` see transform output: is a transform-produced
  base diffable against its source base, and can `merge` reconcile a
  transform's deltas against a human patch below the personal one?
- Does ADR-016's seed set constrain the vocabulary (must v1 grammar admit
  future kinds like `remap-one` without a breaking major?), and does
  `supported_majors`' wording already fix where `policy/v2` would land?
- What does "reports" mean for CI: is an inapplicable transform in a shipped
  policy a golden-fixture class (apply policy X to base Y ⇒ no change +
  report Z), and where do those behavioural fixtures live — `schemas/policy/
  v1/fixtures` pins documents only, and placement-style verdict tests mirror
  `src` per ADR-004?
