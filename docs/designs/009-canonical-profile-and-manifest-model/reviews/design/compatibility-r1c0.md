# compatibility review r1 (panel pass c0, full document)

verdict: request-changes

Inputs read: `02-proposal.md` (incl. its Revision log), `docs/adrs/009-canonical-profile-and-manifest-model.md`,
and the shipped implementation files the proposal cites as bases (`crates/core/tests/conformance.rs`,
`crates/core/src/toml.rs`, `crates/core/src/document.rs`, `crates/core/tests/common/mod.rs`,
`schemas/README.md`, `schemas/reach-classes/v1/left.toml`). No review report or transcript opened.
Previous compatibility findings: none (first pass).

## Checklist coverage

1. **Compatibility matrix** — every row carries a status, one row is `unknown` → finding 1.
2. **OS input APIs** — pass. The design targets none of X11/Wayland/Raw Input/HID and justifies the
   abstraction: Non-goals "Any input-event API (injection, capture)" and the matrix row "A game's own
   config files (any OS, Proton prefix) | n/a | this design reads and writes only `schemas/` and `data/`
   documents". Naming is string-level (`keyboard:`/`mouse:`/`gamepad:`), so no OS input surface is coupled.
3. **Node version floor** — N/A (Rust repo). Floor analogue respected: pinned Rust 1.85.0 / edition 2024,
   and the design adds no dependency (A9, A10, matrix row "Rust toolchain 1.85.0 (pinned) | yes").
4. **Native modules** — pass. Pure Rust, `bindforge-core` stays depends-on-nothing,
   `bindforge-transforms` depends on the path crate only; no prebuild/node-gyp/CI matrix story needed.
   The serde deviation from the ADR's "(serde-based)" is disclosed with basis and *reduces* dependency risk.
5. **Keyboard reality** — layout/IME/geometry deferrals are consistent with the ADR's accepted costs
   (ANSI-US only, physical-position naming later); non-ASCII game-native action names are representable
   (values are basic strings; the shipped reader decodes non-ASCII, `quoted()` escapes only control
   chars). Gaps that remain are findings 4 and 6.
6. **File paths** — `&Path`-based load APIs and the walked-index rule are right; findings 3 and 5.
7. **Platform test per supported platform** — pass. Test strategy #15 names Linux CI and a Windows
   contributor machine, matching the two `yes` platform rows; macOS is `no (v1)` and has no platform leg,
   correctly. Rollout 7 gates the announcement on the Windows leg.

## Findings

- [major][new] Compatibility matrix — "Hub upload path (wire budget, server-side chain pre-resolution) | unknown" —
  a row whose status is `unknown` is not a support record: nothing downstream can decide it, and the
  checklist requires a status. This is a matrix-vocabulary problem, not a request to design the hub
  (the ADR-014 deferral itself is consistent with the ADR's "Not decided here: … the hub service and
  voting"). Fix: change that status cell to `no (v1) — out of scope` and keep the existing note
  ("ADR-014 owns it (A23); the client-side guards and the id-index rule are the model's contribution").
  One cell; no new mechanism.

- [major][new] Chosen design → Encoding rules → Round-trip split (with Goals and Test strategy #1 and #14) —
  "pure lookup data (`taxonomy`, `reach-classes`, `placement`) is asserted by load, using the exemption
  `reach-classes/v1/spec.md` already states" — that exemption is not a rule the harness applies by format
  policy: it is a hardcoded format-name equality test, `crates/core/tests/conformance.rs:63`
  (`if dir.format == "reach-classes"`). Every other format directory's `fixtures/valid/` gets the full
  parse → `serialize` → re-parse → equality check. So the design's two load-asserted formats require a
  `conformance.rs` change, which contradicts Goals "machine-checkable by the existing harness with no
  harness change" and Test strategy #14 "adding the five new `schemas/*/v*/` directories requires no change
  to `crates/core/tests/conformance.rs`". As written, the three claims cannot all hold; A8's basis
  (`conformance.rs:322-335`) supports directory *discovery* only, not the exemption.
  Fix: make all six formats round-trip (add `serialize` arms in Rollout 2's `Document` extension) and
  reserve the load-assert exemption for `reach-classes`, whose loader reads a per-hand *directory pair*,
  not a single document. That replaces "taxonomy, reach-classes, placement are asserted by load"
  (Test strategy #1) and the quoted sentence in Round-trip split. (If the exemption is widened instead,
  Goals' "no harness change" and Test strategy #14 must name that harness edit; the round-trip choice is
  the smaller change and keeps the harness frozen.)

- [major][new] Chosen design → Profiles are chains §3 + Manifests §5 — "its halves validated against the
  game-id grammar" and "content `data/manifests/<game>/manifest.toml` (fixture-name rule for `<game>`)" —
  the game-id grammar is the only gate between an untrusted `for` pin and a filesystem path component,
  and it is never defined. Encoding rules enumerate exactly two id shapes (taxonomy ids, input ids) and
  the quoted-key id list omits game ids; the rule being leaned on exists only as a test helper,
  `crates/core/tests/common/mod.rs:352` (`is_fixture_name`, which strips a `.toml` suffix), so no shipped
  `crates/core` code can enforce it and `BadPin` has no specified predicate. This is the same
  path-traversal class the design closes for profile ids under A18 ("Ids never appear in a path"), left
  open on the manifest side. Fix: add the game-id shape to the canonical Encoding rules as
  `[a-z0-9-]+` (lowercase, dots and `@` excluded — the fixture-name *stem* rule), and state that manifest
  resolution looks the pin's game half up in a walked index keyed on each manifest's declared `game`
  entry, replacing path construction from `<game>`.

- [minor][new] Chosen design → Input vocabulary — "it arrives as a *new* prefix (`pos:<part>`), leaving
  `keyboard:` meaning 'US-ANSI glyph' permanently" — the canonical grammar closes the namespace set
  (`<namespace>` ∈ `keyboard|mouse|gamepad`), so minting `pos` is a widening of a closed set in every
  format that carries input ids (`profile`, `patch`, `manifest`'s `default:`/`dead-keys`, `policy`), not
  only in the reach tables. Deferred says only that it "needs a `reach-classes/v2`", and the ADR's
  "the id shape reserves it" is true of the shape, not of the version gate — the matrix row "Non-US symbol
  layouts … the reserved `pos:` prefix is the later answer" leaves implementers to guess whether a v1
  reader accepts `pos:a`. Fix: one sentence in Input vocabulary (canonical): `pos` joins the `<namespace>`
  set only in a v2 of every format that carries input ids, and a v1 document naming one is `UnknownInput`.

- [minor][new] Chosen design → Profiles are chains §3 — "File names follow the shipped fixture-name rule
  (lowercase ASCII, `-`-joined)" — the shipped rule as implemented requires a `.toml` suffix
  (`is_fixture_name` returns false for anything not ending in `.toml`), while profile content is
  `data/profiles/**/*.bindprofile` (this section's own words, and ADR-009 Decision 3). The named rule
  therefore cannot be satisfied by the files it is cited to govern, and the Windows matrix row leans on
  that rule for its case-uniqueness claim. Fix: cite the stem rule — "lowercase ASCII, `-`-joined stem" —
  and state the extension separately (`.bindprofile` for profile/patch content, `.toml` for `schemas/`
  fixtures).

- [minor][new] Chosen design → Input vocabulary + Files created by this design — "`schemas/README.md` lists
  the keyboard names v1 admits" — the design makes README the single canonical enumeration for the v1
  keyboard surface, and both the authoring bar ("both hands graded over the full admitted keyboard name
  set") and Test strategy #3 ("every admitted keyboard name is graded in both hands") iterate it, but the
  README inventory row reads "format list, the canonical encoding rules, `version`/`revision` split" and
  Rollout 1 says only "the canonical encoding rules and the input-vocabulary list" — and no such list
  exists in `schemas/README.md` today (no key-name enumeration present). The set that the US-ANSI
  support claim is tested against has no named source. Fix: add the admitted keyboard-name list to that
  row's contents and name it as the enumeration Test strategy #3 iterates.

## FYI (no severity)

- The manifest's `dead-keys` entry means "inputs the game ignores", not the keyboard-layout dead key
  (diacritic composition). The keyboard-reality reader will hit that collision; naming is a
  clarity/proposal-consistency concern, and layout-level dead keys are correctly inside the deferred
  non-US-layout work.
- `action-continuous-lateral` is named in the §5 Entries text and Test strategy #9, but the §5 example
  fragment shows only `action-discrete-lateral` — a citation seat may want the example to show both.
- The design says "US-ANSI"; ADR-009 says "US-QWERTY glyph" with "ANSI geometry". Same intent, two
  phrases; worth one alignment by the language seat.
- "emphasis is the reading of that input's reach class for the hand that owns it" — the design gives no
  rule for which hand owns an action; that is a correctness seat question, not a compatibility one.
- Platform support as claimed is narrow by design (Linux CI + Windows contributor machine, ANSI-US, no
  gamepad, no config I/O) and each limit is stated in the matrix; from compatibility, nothing here
  forecloses the ADR-012 codec work or the hub.
