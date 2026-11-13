# Research: Input vocabulary and reach classes (ADR-012)

Scope: the ADR asks for (1) a per-hand reach-class taxonomy and the key→class
mapping, (2) an OS-neutral physical-input vocabulary (`keyboard:*`, `mouse:*`,
`gamepad:*`), and (3) a placement-legality predicate over the v2 tables,
shipped as `schemas/reach-classes/v2/` without touching v1. Lenses applied:
Prior art, Standards, Platform notes.

## Summary

Every mature key-naming registry we examined names physical keys, and every
one of them names them from a US-ANSI reference geometry — USB HID calls the
key `Keyboard q and Q`, macOS calls it `kVK_ANSI_Q` "according to the key
position on an ANSI-standard US keyboard", the Web calls it `KeyQ` — so
ADR-012's glyph-based `keyboard:<key>` under ANSI geometry follows the
industry's common denominator rather than inventing a fifth scheme. That
common denominator is USB HID Usage Page 0x07: Linux evdev and SDL scancodes
both declare themselves modeled on it, and W3C's `code` values follow its
naming where possible. None of these registries carries any comfort/reach
information, so the five-class per-hand grading is genuinely new data
BindForge authors itself; the only published grading tradition is the
touch-typing home-row method (left hand `ASDF`, right hand `JKL;`), which
grades keys by posture from a home position — exactly the "posture, not
distance" basis the ADR states. Placement legality has strong prior art in
game practice — WASD, ESDF, IJKL, numpad clusters are all shape-preserving
relocations — but no tool encodes legality as data; the ADR's predicate has
no published equivalent to reuse.

## Findings

### The contract as it stands on `main`

- The shipped v1 format pins only layout and load rules; its own spec says the
  class *content* — which classes exist and what they mean — "is owned by
  ADR-012", and the shipped class ids `["near", "mid", "extended"]` are
  labelled placeholders pending that taxonomy (source: schemas/reach-classes/v1/spec.md;
  source: schemas/reach-classes/v1/left.toml).
- The v1 entry set is closed (`classes`, `format`, `hand`, `version`): any
  other entry is refused, and a document declaring a major outside
  `supported_majors()` (or no major at all) is rejected with a structured
  error naming the supported majors — never defaulted (source: crates/core/src/document.rs;
  source: schemas/README.md).
- `crates/core`'s `supported_majors()` currently returns exactly `["v1"]`, and
  a test pins that the declared majors equal the majors present under
  `schemas/*/v*/`, so adding `schemas/reach-classes/v2/` plus a `MAJOR_V2`
  entry is what keeps that equality test green instead of breaking it
  (source: crates/core/src/document.rs; source: crates/core/tests/version_gate.rs).
- The loader already separates per-hand tables and expresses the
  file-name/hand agreement rule as a pure function; `ReachClass` carries only
  an id today, with "ADR-012 owns the taxonomy the ids name and any further
  fields" written into its doc comment (source: crates/core/src/reach.rs).
- The contract's versioning rule ("one directory per major", "minor bumps are
  additive only", fixtures define "supported", harness walk picks up new
  formats with no harness change) makes a v2 major the only in-contract route
  for adding `keys`, matching the ADR's Decision One (source: schemas/README.md).
- The formats' fixture-conformance model is explicitly modeled on the JSON
  Schema Test Suite — golden valid/invalid corpora every implementer runs —
  so v2 must ship its own fixtures in the same shape (source: schemas/README.md;
  source: https://github.com/json-schema-org/JSON-Schema-Test-Suite).
- The surrounding series constrains this ADR: 011 fixed the action taxonomy
  (`move.steer-*` etc., `discrete-lateral` axis kind) and explicitly left
  "input naming and reach tables (012)" undecided; 015 preconditions transforms
  on "the placement legality predicate (012)" without 012 choosing placements
  (source: docs/adrs/011-action-taxonomy-and-core-skeleton.md;
  source: docs/adrs/015-transforms-as-preconditioned-policies.md).
- The grammar subset the tables must stay inside is a documented subset of
  TOML v1.1.0; toml.io currently publishes v1.1.0 as a full spec alongside
  v1.0.0, so the contract's cited spec version exists, but implementers
  building on parsers that ship only 1.0.0 are still within the contract
  because the subset is stated in the contract itself (source: https://toml.io/en/;
  source: schemas/reach-classes/v1/spec.md).

### The input-vocabulary half: naming registries already exist

- USB HID Usage Page 0x07 is the de-facto root registry: usage names are
  glyph-based (`Keyboard q and Q`), and its own note says that because
  keyboards vary by language, vendors must keep the same usage for the same
  *position* and just print different keycaps — the canonical example is the
  US `Y` usage staying in place on German keyboards where the legend is `Z`
  (source: https://www.usb.org/sites/default/files/hut1_3.pdf).
- The Web platform's physical-key vocabulary is `KeyboardEvent.code`, defined
  as based on the key's physical location, not varying with locale, named
  from the US key in that position "which follows the naming convention found
  in [USB-HID]", with values like `KeyQ`, `Semicolon`, `Quote`; the spec's
  rationale sentence names "WASD keys in games" as the motivating case
  (source: https://www.w3.org/TR/uievents-code/).
- Linux evdev `KEY_*` codes declare themselves "modeled after USB HUT 1.12"
  and use the same glyph naming (`KEY_Q`, `KEY_SEMICOLON`, `KEY_APOSTROPHE`)
  (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h).
- SDL3 scancodes are "the physical representation of a key on the keyboard,
  independent of language and keyboard mapping", with values taken from the
  USB usage page, and name punctuation by US glyph (`SDL_SCANCODE_SEMICOLON`,
  `SDL_SCANCODE_APOSTROPHE`) (source: https://raw.githubusercontent.com/libsdl-org/SDL/main/include/SDL3/SDL_scancode.h).
- macOS Carbon virtual keycodes identify physical keys, and Apple's header
  says the `ANSI`-named constants "are labeled according to the key position
  on an ANSI-standard US keyboard", warning that other layouts put those
  legends on different physical keys — the exact caveat ADR-012 pins as
  "ANSI geometry in v1" (source: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.6.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h).
- Windows virtual-key codes split the problem differently: layout-independent
  keys get fixed codes (`VK_RETURN`, `VK_SHIFT`) but punctuation is region-
  dependent and opaquely named (`VK_OEM_1` = `;:` on US ANSI, `VK_OEM_7` =
  `'"`), showing why a portable vocabulary must carry its geometry assumption
  explicitly (source: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
- Mouse vocabulary in practice is five named buttons plus a wheel that is an
  event, not a button: Pointer Events number `button` 0 left, 1 middle, 2
  right, 3 "X1 (back)", 4 "X2 (forward)", and wheel input is a separate
  `WheelEvent`; HID caps a mouse at Button1–Button5. ADR-012's
  `mouse:left|middle|right|side-1|side-2` plus `wheel-*` maps cleanly onto
  that convention with neutral names replacing X1/X2 (source: https://www.w3.org/TR/pointerevents4/;
  source: https://www.usb.org/sites/default/files/hut1_3.pdf).
- Gamepad naming has no global id registry; the Web approach is a "Standard
  Gamepad" *layout* matched by relative control positions and HID usage
  conventions, and Windows carries named `VK_GAMEPAD_*` codes (A/B/X/Y,
  shoulders, triggers, dpad, thumbstick directions) — naming by role/position,
  never by vendor (source: https://www.w3.org/TR/gamepad/;
  source: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
- The ADR's choice to name mouse/gamepad inputs but not reach-grade them
  matches how these registries already treat non-keyboard devices: neither
  the Pointer Events button tables nor the Gamepad API assigns any
  key-reach semantics (source: https://www.w3.org/TR/pointerevents4/;
  source: https://www.w3.org/TR/gamepad/).

### The reach-class half: grading precedents

- The home-row method — fingers rest on `ASDF` (left) and `JKL;` (right),
  each finger reaches assigned keys from that anchor and returns — is the
  published basis for per-hand comfort, and it encodes the ADR's asymmetry
  fact for free: the right pinky's home position *is* `;` (with `'` one
  stretch away), while the left pinky's home is `a` with nothing analogous
  on its right side (source: https://en.wikipedia.org/wiki/Touch_typing).
- Keyboard-layout ergonomics work grades effort by finger movement and
  posture, not centimetres: Colemak's published comparison counts finger
  travel, same-hand row jumps, and home-row-only words, and claims most
  typing falls "on the strongest and fastest fingers" — evidence that
  posture-graded classes are the established unit, supporting the ADR's
  rejection of per-key pixel distance (source: https://colemak.com/).
- Keyboard layout history shows grading per hand is not symmetric either:
  touch-typing instruction assigns different key territories per hand and
  index fingers carry the most keys (source: https://en.wikipedia.org/wiki/Touch_typing).
- No naming registry examined (HID, evdev, Windows VK, macOS kVK, SDL, W3C
  `code`) carries any comfort/reach data — reach grading is unclaimed ground,
  which is consistent with the ADR treating the taxonomy as BindForge's own
  contract data (source: https://www.usb.org/sites/default/files/hut1_3.pdf;
  source: https://www.w3.org/TR/uievents-code/).

### The placement-legality half: prior art is folklore, not data

- Movement clusters are shape-preserving relocations with decades of game
  practice: WASD is the standard PC default (popularized via Quake/QuakeWorld,
  first mainstream default in Half-Life, possibly first used in Telengard,
  1982); ESDF, DCAS, IJKL, numpad and arrow-key clusters are each justified
  by reach arguments — ESDF keeps the left hand home-row-aligned and uses
  the `F` bump, IJKL is described as "an ergonomic inverted T" for the right
  hand, left-handed mousers are pointed to the numpad (source: https://en.wikipedia.org/wiki/Arrow_keys).
- The WASD layout's own comfort rationale is the ADR's predicate in prose:
  WASD is preferred because the left thumb keeps the space bar and the left
  pinky keeps Ctrl/Shift "as opposed to the arrow keys which lack other keys
  in proximity" — i.e., relocation legality is about preserving the
  *reachability context* of the cluster, not just its glyph shape (source: https://en.wikipedia.org/wiki/Arrow_keys).
- WQSE-style variants ("index and ring fingers' natural positions when the
  middle finger is on W are Q and E rather than A and D") show that players
  already argue in terms of per-finger reach classes, without any data
  format encoding it (source: https://en.wikipedia.org/wiki/Arrow_keys).
- Remapping conventions in browsers (IJKL because arrow keys scroll) show
  legality drivers can be non-ergonomic (platform conflicts); whether such a
  reason belongs in the predicate or only in 015's policy layer is unresolved
  upstream of this ADR (source: https://en.wikipedia.org/wiki/Arrow_keys).

## Prior art

Registry formats for physical inputs, by naming scheme:

- USB HID Usage Page 0x07 (`Keyboard q and Q`, Boot-keyboard minimum, Button
  page Button1–Button5 for mice) — the common denominator: evdev and SDL both
  declare derivation from it, W3C follows its naming (source: https://www.usb.org/sites/default/files/hut1_3.pdf;
  source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h;
  source: https://raw.githubusercontent.com/libsdl-org/SDL/main/include/SDL3/SDL_scancode.h;
  source: https://www.w3.org/TR/uievents-code/).
- OS layers: Windows VK codes (region-dependent `VK_OEM_*`), macOS
  `kVK_ANSI_*` (US-ANSI position naming), Linux evdev `KEY_*` / `BTN_SIDE`
  /`BTN_EXTRA` / `BTN_SOUTH..BTN_WEST` for pads (source: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes;
  source: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.6.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h;
  source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h).
- Web layer: `KeyboardEvent.code` (US-ANSI glyph names, locale-invariant),
  Pointer Events `button`/`buttons` numbering, Gamepad API "standard" layout
  mapping (source: https://www.w3.org/TR/uievents-code/;
  source: https://www.w3.org/TR/pointerevents4/; source: https://www.w3.org/TR/gamepad/).
- Nothing in this lineage names inputs by neutral *role* (`keyboard:w` is
  written `KeyQ`-style or `0x14`-style but never geometry-independent), so
  ADR-012's reserved-for-later physical-position naming has no published
  predecessor to copy — a gap the open questions should probe.
- Major-versioned file formats with golden fixtures: BindForge's own harness
  model is taken from the JSON Schema Test Suite; the "incompatible change ⇒
  new major, additive change ⇒ minor" rule this ADR follows is the semver
  convention applied to on-disk documents (source: https://github.com/json-schema-org/JSON-Schema-Test-Suite;
  source: https://semver.org/; source: schemas/README.md).
- Reach grading: touch-typing finger assignments and layout-optimization
  counts (finger travel, same-hand jumps, home-row words) are the only
  published grading systems found; none ships as machine-readable key→grade
  data (source: https://en.wikipedia.org/wiki/Touch_typing; source: https://colemak.com/).
- Cluster relocation: WASD/ESDF/DCAS/IJKL/HJKL/QAOP/numpad variants, with
  ergonomic rationales, all undocumented as data (source: https://en.wikipedia.org/wiki/Arrow_keys).

## Platform notes

- Geometry assumptions are load-bearing on every OS: the US `\|` key and the
  UK `#~` key are two separate HID keys but share one code on every platform
  because they never co-occur, and ANSI vs ISO moves the backslash key —
  evidence that "US glyph under ANSI geometry" must be stated in the format,
  not inferred (source: https://www.w3.org/TR/uievents-code/;
  source: https://raw.githubusercontent.com/libsdl-org/SDL/main/include/SDL3/SDL_scancode.h).
- Linux evdev names side buttons by role (`BTN_SIDE`, `BTN_EXTRA`, aliasing
  `BTN_FORWARD`/`BTN_BACK`), Windows by number (`XBUTTON1/2`), Web by
  direction ("X1 (back)", "X2 (forward)") — three vocabularies for
  `mouse:side-1|side-2`, confirming the ADR's need for its own neutral id
  (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h;
  source: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes;
  source: https://www.w3.org/TR/pointerevents4/).
- Gamepad button identity is even looser across platforms: evdev uses
  position words (`BTN_SOUTH/EAST/NORTH/WEST`), Windows uses Xbox letters
  (`VK_GAMEPAD_A`…), the Web uses canonical indices under a "standard"
  layout flag — `gamepad:*` ids will face letter-vs-position naming debates
  even though 012 does not reach-grade them (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h;
  source: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes;
  source: https://www.w3.org/TR/gamepad/).
- The reference reader is Rust over a hand-pinned TOML subset: a third-party
  reader built on a standard TOML parser "accepts more than the contract
  does, and must reject the rest" (table headers, dotted keys, multi-line
  arrays, floats, hex ints all have invalid fixtures), so the v2 spec must
  pin the same closed grammar (source: schemas/README.md;
  source: schemas/reach-classes/v1/spec.md).
- v2 documents live beside v1 in the same harness walk (`schemas/*/v*/`), and
  fixture naming rules (lowercase ASCII words joined by `-`, case-unique)
  bind the new fixtures (source: schemas/README.md; source: crates/core/tests/version_gate.rs).

## Risks & unknowns

- Glyph naming inherits the US-ANSI assumption the registries already
  regret loudly (HID's Y/Z note, Apple's layout warning, Windows `VK_OEM_*`
  opacity); a v1 of the vocabulary that is glyph-keyed means reach grades for
  AZERTY/QWERTZ users are *wrong, not merely unmapped* — e.g. AZERTY players
  bind ZQSD, which are different physical keys than WASD (source: https://www.usb.org/sites/default/files/hut1_3.pdf;
  source: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.6.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h;
  source: https://en.wikipedia.org/wiki/Arrow_keys).
- No external benchmark validates a five-class key→grade table; the only
  published gradings (touch-typing territories, Colemak's counts) use
  different units, so the table's correctness will rest on fixtures and
  review, not on a citable standard (source: https://en.wikipedia.org/wiki/Touch_typing;
  source: https://colemak.com/).
- The ADR gives no rule for keys reachable by two fingers or both thumbs
  (space, enter, shift); the home-row method itself treats thumb territory
  separately from the eight fingers, and `thumb` is a class name for a
  *finger*, not a posture — a per-key grading rule for multi-finger keys has
  to be invented (source: https://en.wikipedia.org/wiki/Touch_typing).
- "Class degraded beyond a declared bound" needs an order on class ids; the
  v1 format's `classes` array is order-carrying but the contract attaches no
  meaning to order, so the predicate's bound needs a stated source of
  ordinality (source: schemas/reach-classes/v1/spec.md).
- Two majors living at once means third-party v1 readers see placeholder ids
  until they migrate — a documented cost, but it makes the v2 spec text the
  first stable public statement of the taxonomy (source: docs/adrs/012-input-vocabulary-and-reach-classes.md;
  source: schemas/reach-classes/v1/left.toml).
- The `keys` array entries `"<input-id> <class-id>"` split on a space, so
  input-id syntax must forbid spaces — and punctuation glyphs in ids (`;`,
  `'`, `` ` ``) interact with the closed string grammar's escape rules; no
  prior registry in the surveyed set puts a raw glyph inside a space-joined
  pair, so this encoding has no prior art to copy (source: docs/adrs/012-input-vocabulary-and-reach-classes.md;
  source: schemas/reach-classes/v1/spec.md).

## Open questions

- What is the exact `keyboard:<key>` id set for v1: lowercase US glyphs for
  letters/digits, and how are `;` `'` `` ` `` `\` `[` `]` `,` `.` `/`
  spelled in the id — literal glyph, or a name — given the grammar's bare
  string entries and the space-separated pair encoding?
- What is the per-key grading rule when several fingers (or either thumb)
  reach a key — best finger, designated finger, or hand-level class?
- Does the legality predicate's "declared bound" read class ids as ordinal
  by table position, or must v2 declare an explicit grade number per class?
- Does the predicate consider the cluster's neighbours (space/Ctrl/Shift
  proximity, per the WASD rationale) or only the cluster's own keys — the
  ADR's clause (b) covers only the actions' own classes, but the folklore
  reasons about the surrounding reach context (source: https://en.wikipedia.org/wiki/Arrow_keys)?
- Which region set counts as a legal target for a movement cluster — does
  the numpad count (left-handed-mouser prior art), and is the arrow-key
  cluster a region? (source: https://en.wikipedia.org/wiki/Arrow_keys)
- How should 012's spec describe the deferred physical-position naming: just
  the id shape reserved, or also a per-layout geometry-file sketch so adding
  non-US geometries stays "new table files, not new logic"?
