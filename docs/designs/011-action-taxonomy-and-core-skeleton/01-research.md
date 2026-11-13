# Research: Action taxonomy and the `crates/core` skeleton (ADR-011)

## Summary

ADR-011 asks for two things at once: a shared action-identity layer (dotted
lowercase ids plus an orthogonal axis-kind attribute, no placement semantics)
and the one-time bootstrap of the spine — `crates/` workspace, `schemas/`
conformance directory, `data/` placeholders — that ADRs 012–015 only extend
(source: docs/adrs/011-action-taxonomy-and-core-skeleton.md). The identity
idea is well-proven in adjacent tooling: Steam Input, Unity's Input System,
and SDL's controller DB all separate named semantic actions from physical
inputs, and the Xbox Accessibility Guidelines recommend remapping *actions*
rather than controls — which is exactly the join the taxonomy enables. The
valid/invalid golden-fixture harness the ADR mandates is the same pattern the
JSON Schema Test Suite and `toml-test` use, and both document concrete
harness-interface details worth copying. The strongest caution from prior
registries (Linux evdev, DNS) is that a published id set is a one-way door:
the interesting undecided parts are the id grammar boundary, the extension
and deprecation procedure, and reconciliation with the already-populated
`crates/core`/`schemas/` tree on this branch.

## Findings

- The ADR fixes the dependency root: 012 explicitly declares
  `depends-on: [011]`, and 011 is the only installment that creates the
  workspace, `schemas/` harness, and `data/` placeholders; 012–015 add
  schema/fixture pairs only (source: docs/adrs/011-action-taxonomy-and-core-skeleton.md, docs/adrs/012-input-vocabulary-and-reach-classes.md).
- "Actions, not inputs" is industry-established: Valve's In-Game Actions
  (IGA) file makes developers declare each game's actions as uniquely-named
  string keys grouped into action sets, and Steam's config ecosystem binds
  physical devices to those names — the same profile↔game matching shape as
  BindForge's translation problem (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Valve had to solve the id-stability problem for a live action namespace:
  breaking changes to the action set require `major_revision`/`minor_revision`
  flags so players' saved configurations are migrated rather than silently
  broken — a precedent for treating the taxonomy as a versioned compatibility
  surface (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Unity's Input System separates logical meaning from device controls the
  same way, and where string names proved mutable it added a second, opaque
  identity: every action keeps a GUID that "remains the same even if you
  rename the Action" — evidence that a name-as-id scheme needs an explicit
  rename/deprecation policy, since Unity sidestepped that need with a
  separate id (source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.7/manual/Actions.html).
- Accessibility guidance independently validates identity-before-placement:
  XAG 107 tells developers to let players "remap game actions instead of
  individual game controls", so a shared action namespace is the expected
  substrate for handedness/accessibility profiles (source: https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/107).
- The golden-fixture conformance harness the ADR mandates matches the
  JSON Schema Test Suite model: language-agnostic test corpora, cloned by
  each implementer, where "the conversion of the … objects into tests within
  a specific language … is left to the implementation author" — BindForge's
  `schemas/*/v*/fixtures/{valid,invalid}/` + CI walker is the Rust-local
  instantiation of that pattern (source: https://github.com/json-schema-org/JSON-Schema-Test-Suite).
- `toml-test` is the closest structural template and documents two harness
  rules worth adopting: invalid fixtures should "test one thing and one
  thing only" and be named after the fault they expose, and deep nesting is
  a known DoS vector where "having a limit on table nesting is probably a
  good idea" — corroborating the branch's existing `MAX_NESTING_DEPTH`
  guard as fixture-worthy behavior (source: https://github.com/toml-lang/toml-test).
- Existing registry precedents show the taxonomy's "ids in the wild cannot
  churn" cost is real and is handled by written discipline: Linux evdev's
  `input-event-codes.h` freezes legacy LED codes as "grandfathered for
  backwards compatibility with userspace (via evdev)" and commands "a new
  KEY_FOO define MUST be added" instead of repurposing names — the same
  extend-don't-mutate regime ADR-011 assigns to manifest review (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h).
- A dotted-lowercase-with-hyphens id grammar has a proven, published
  grammar to borrow: DNS labels (RFC 1035 §2.3.1) restrict labels to
  letters/digits/hyphen (letter-first, let-dig last), declare case
  insignificant, and cap labels at 63 octets — a ready-made set of
  accept/reject rules for the id spec's invalid fixtures (source: https://www.rfc-editor.org/rfc/rfc1035).
- Growth-by-review of a shared name DB is proven at community scale:
  SDL_GameControllerDB is "a community sourced database" whose contribution
  rules require confirming a mapping "improves, and does not degrade" the
  experience before PR/issue review merges it — precedent for the taxonomy's
  manifest-review gate (source: https://github.com/gabomdq/SDL_GameControllerDB/blob/master/README.md).
- If id evolution ever needs breaking change, SemVer's published procedure
  for deprecation (announce in a minor, remove in the next major) is the
  standard vocabulary for the taxonomy's version story (source: https://semver.org/).

## Prior art

**Action namespaces in input tooling (what names/formats won):**

- Valve Steam Input — developer-authored IGA file (`"ActionName" "\#Title"`
  under Button / AnalogTrigger / StickPadGyro blocks in VDF); unique string
  action names per action set; analog-vs-digital separated by *block*,
  echoing ADR-011's axis-kind attribute; revision flags carry breaking
  changes to live configurations (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Unity Input System — `InputActionAsset` → named `InputActionMap`s → named
  `InputAction`s, each also carrying a rename-stable GUID; JSON
  (de)serialization of action assets makes the action list a shareable file
  (source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.7/manual/Actions.html).
- SDL GameController API + SDL_GameControllerDB — won by convention with
  short lowercase *abstract* names for logical positions (`a:b1`,
  `leftshoulder:b4`, `leftx:a0`), deliberately Xbox-centric, with the
  community DB as the growth mechanism; shows a small fixed vocabulary can
  carry a whole ecosystem, at the price of forcing non-Xbox layouts into it
  (source: https://github.com/gabomdq/SDL_GameControllerDB/blob/master/README.md, https://wiki.libsdl.org/SDL2/SDL_GameControllerAddMapping).
- Linux evdev `input-event-codes.h` — the de-facto cross-tool common
  denominator on Linux for *physical* codes (`KEY_W`, `BTN_LEFT`, `BTN_SOUTH`);
  it is code-named constants, not semantic actions, which is exactly the
  layer BindForge's taxonomy sits above (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h).
- USB HID Usage Tables (USB-IF) — the hardware-side naming standard; evdev
  states its key codes are "modeled after USB HUT 1.12", so both physical
  registries ADR-012 will lean on already descend from one document (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h, https://usb.org/sites/default/files/hut1_21.pdf).
- RetroArch overrides/remaps — per-core/per-directory/per-game `.rmp` remap
  files that "save only settings that differ from the preceding settings
  files", applied base→override in a fixed order: prior art that 014's
  profile chains can reference for delta-only layering, though none of it
  carries a shared action taxonomy — the gap BindForge fills (source: https://docs.libretro.com/guides/overrides/).
- Conformance harnesses — JSON Schema Test Suite (same golden-fixture model
  the repo's `schemas/README.md` already names) and `toml-test` (valid vs
  invalid corpora, one-fault-per-invalid-test naming, version-conditional
  test lists) are the two formats worth imitating (source: https://github.com/json-schema-org/JSON-Schema-Test-Suite, https://github.com/toml-lang/toml-test).

## Platform notes

- The repo's workspace already follows the Cargo Book's recommended shape —
  "Keep all member packages in a flat directory (commonly `crates/`) and
  use a glob pattern for the `members` field" — and a virtual manifest must
  set `resolver` explicitly, which `Cargo.toml` does with `resolver = "3"`
  (source: https://doc.rust-lang.org/cargo/reference/workspaces.html).
- `edition = "2024"` in the workspace package pins the floor: Rust 2024
  shipped in 1.85.0, matching `rust-version = "1.85"` and the
  `rust-toolchain.toml` channel (source: https://doc.rust-lang.org/edition-guide/rust-2024/index.html).
- `rust-toolchain.toml` is a rustup-supported pin ("suitable to check in to
  source control"; rustup installs the named channel automatically), so the
  CI gate needs no extra toolchain setup — the `check.yml` rust job relies
  on exactly this (source: https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file).
- The conformance harness runs inside `cargo test --workspace` on every PR
  (fmt / clippy -D warnings / test, `--locked`), so 011's "CI runs the
  fixtures" requirement is already wired at the gate level; if third-party
  readers must run the same fixtures from other languages, `toml-test`'s
  stdin/exit-code harness contract is the proven cross-language interface
  (source: https://github.com/toml-lang/toml-test).
- The formats are TOML; `toml-test` documents that v1.0 and v1.1 differ on
  specific constructs and ships per-version file lists — relevant because
  the branch's `schemas/README.md` narrows to a documented subset of TOML
  v1.1.0 (source: https://github.com/toml-lang/toml-test).

## Risks & unknowns

- **Greenfield claim vs. branch state.** ADR-011 says "the product is
  greenfield: the repo holds only the orchestration harness"
  (source: docs/adrs/011-action-taxonomy-and-core-skeleton.md, Context),
  but this branch already carries a `crates/core` with public surface for
  chain resolution, reach tables, and a TOML parse budget, plus
  `schemas/profile/v1` and `schemas/reach-classes/v1` with fixtures
  (source: crates/core/src/lib.rs, schemas/ tree). These look like
  superseded-ADR-009-era scaffolding; `schemas/README.md` still schedules
  work "with ADR-009". Whether 011 builds on, trims, or resets that tree
  is undetermined and changes the stage's size.
- **The ~40 ids are not enumerated anywhere** in the ADR (source: docs/adrs/011-action-taxonomy-and-core-skeleton.md); the id spec's fixtures are unbuildable until the set is written down.
- **No deprecation/removal procedure for ids** is defined. Valve needed
  revision flags for exactly this situation once ids were in the wild
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file); Unity avoided the problem with GUIDs
  (source: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.7/manual/Actions.html). "Extension through manifest review" covers additions but not withdrawal.
- **Axis kind is a taxonomy-local concept** — no external standard grades a
  game action as `discrete-lateral`/`continuous-lateral`/`none` — so its
  contract value rests entirely on BindForge's own fixtures; misclassification
  (e.g. is `move.steer-*` discrete or continuous?) silently changes what
  015's polarity flips may touch.
- **Id collision with reserved namespaces:** the ADR names `system.*` as a
  bucket but doesn't say how unknown/unprefixed ids are treated by readers;
  registry precedents freeze such decisions in writing precisely because
  they are one-way (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h).
- **`data/` directories don't exist yet** (no `data/` in the tree), and git
  tracks no empty directories — the "created empty with a README" step needs
  the README to exist in the commit, as the root `Cargo.toml` comment already
  reasons for `apps/` (source: Cargo.toml, repo tree).

## Open questions

1. What is the initial ~40-id list, and in which file does it live — a
   `schemas/action-ids/v1/spec.md` table, or a data file the harness parses?
2. Pin the id grammar: is the DNS-style rule (segment starts letter, ends
   letter/digit, interior `[a-z0-9-]`, ≤63 octets — RFC 1035 §2.3.1) adopted
   wholesale, and are uppercase or leading-digit segments *invalid fixtures*
   or merely canonicalization cases? (source: https://www.rfc-editor.org/rfc/rfc1035)
3. What happens to the existing `crates/core` surface (`resolve_chain`,
   `load_reach_tables`, parse budget) that ADR-012/014/015 territory already
   shipped into it? Keep, gate behind modules, or reset to a true skeleton
   per "no behaviour beyond this ADR"?
4. Is the conformance harness Rust-only (a `cargo test` fixture walker, as
   today) or also runnable by third parties (a `toml-test`-style external
   runner contract), given ADR-008's "third-party readers" requirement?
5. How is axis kind pinned in fixtures — one valid/invalid pair per id, or
   a registry table file that the harness diffs?
6. What is the deprecation path for an id found wrong after publication —
   reserved tombstone, aliasing, or documented removal at a major version
   bump (semver-style)? (source: https://semver.org/)
7. Should the stale `schemas/README.md` (which references ADR-009 and says
   `manifest`/`policy` arrive "with ADR-009") be corrected in 011's stage,
   since it is the contract document third-party readers start from?
8. What goes in the `data/manifests/` and `data/profiles/` READMEs beyond
   "populated by 013/014" — do they state the id-stability promise to
   contributors, mirroring SDL_GameControllerDB's contribution rules
   (source: https://github.com/gabomdq/SDL_GameControllerDB/blob/master/README.md)?
