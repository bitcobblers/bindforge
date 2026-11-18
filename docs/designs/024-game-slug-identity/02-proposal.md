# Design: Give discovered games a slug the product can adopt (ADR-024)

## Context

Source: `docs/adrs/024-game-slug-identity.md`, built on
`docs/designs/024-game-slug-identity/01-research.md`. Discovery today emits a
provisional slug `unknown/<kebab(display_name)>` that can never be a product id,
because every product identifier rides one shared slug grammar (`[a-z][a-z0-9-]*`)
that excludes `/` by design. This design makes a discovered game's `slug` a
single-segment id that already passes that grammar and the shared length bound,
and moves the namespace into its own `DiscoveredGame` field. It raises the
shared bound from 63 to 128 octets with one stated home and one exported
constant, amending `013-L5` in the ledger. No matching or resolver code ships
here; that is `024-R1`'s reservation to ADR-025.

## Goals / Non-goals

- Goals:
  - Replace `normalize_slug` with the ordered `024-C2` derivation (canonical
    statement: Chosen design, "Derivation"), producing a grammar-valid,
    bound-valid, adoptable-byte-for-byte slug or a named refusal.
  - Carry the namespace as a separate closed field (`unknown` is its only v1
    value) in `DiscoveredGame`, table, and JSON output.
  - Enforce the grammar and the bound before any `DiscoveredGame` leaves a
    plugin or the CLI, with refusals echoed only through core's
    escape-and-truncate renderer.
  - Implement the deterministic per-run collision-suffix rule (`-2`, `-3`, …).
  - Move the shared bound to 128 as one exported constant in `crates/core`,
    amend the four published specs in place to cross-reference it, and re-pin
    every fixture and boundary test.
  - Record the ledger amendment (`013-L5` gains `amended-by 024-B1`).
- Non-goals:
  - No resolver, no `publisher` namespace derivation, no rename-stability
    machinery (ADR-025, reserved).
  - No change to the slug grammar itself (no Windows reserved-name refusal, no
    `publisher/name` form).
  - No transliteration beyond single-base-letter Latin folding; no hash fallback.
  - No persistence or performance-budget work beyond what today's code already
    counts.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | The product has exactly one slug grammar predicate, `[a-z][a-z0-9-]*`, today the crate-private `placement::is_slug_name`, and it carries no length bound internally | verified | crates/core/src/placement.rs:67-84 |
| A2 | The 63 bound is duplicated in four private constants — `MAX_SLUG_OCTETS` at policy.rs:37, manifest.rs:63, document.rs:220 and `MAX_SEGMENT_OCTETS` at action.rs:38 — not the "three private copies" the ADR names | verified | crates/core/src/policy.rs:37; crates/core/src/manifest.rs:63; crates/core/src/document.rs:220; crates/core/src/action.rs:38 |
| A3 | The 63 number is restated in exactly four published contracts, in action-ids both in the bound sentence and in a refusal example naming a 64-octet segment | verified | schemas/manifests/v1/spec.md:54,56; schemas/profile/v2/spec.md:38,39,71; schemas/policy/v1/spec.md:40,92; schemas/action-ids/v1/spec.md:54,58 |
| A4 | `crates/discovery` is the only product crate not depending on `bindforge-core`; its deps are `serde` plus `winreg` under `cfg(windows)` | verified | crates/discovery/Cargo.toml:6-13 |
| A5 | Rust `std` exposes no Unicode normalization API; the toolchain is pinned to channel 1.85.0 | verified | https://doc.rust-lang.org/std/primitive.str.html#method.to_lowercase; rust-toolchain.toml:6 |
| A6 | In a 143-title Steam store survey, the longest slug the `024-C2` algorithm derives was 64 octets, nothing in the sample exceeded 128, and one distinct-name collision pair (`Ys IX`) is real; the survey run is verified, the population claim is not | verified (survey) / unverified (population) | 01-research.md Findings; https://store.steampowered.com/api/storesearch/ |
| A7 | No shipped corpus document carries an id near 63 octets | verified | `data/` holds only data/manifests/README.md and data/profiles/README.md |
| A8 | `MAX_NATIVE_NAME_OCTETS = 128` is shipped and already a public export | verified | crates/core/src/manifest.rs:57; crates/core/src/lib.rs:108-111 |
| A9 | `schemas/README.md` states the reference implementation exports "all eleven constants", and `lib.rs` re-exports exactly eleven | verified | schemas/README.md:127-129; crates/core/src/lib.rs:98-117 |
| A10 | An absent or empty ACF `name` already falls back to `installdir` before the slug is derived | verified | crates/discovery/src/steam_common.rs:248-251 |
| A11 | `schemas/manifests/v1/fixtures/invalid/` has no past-cap `id` fixture (only `id-uppercase` and a native-name `name-past-cap`), so the manifest `id` bound is pinned only by a Rust test today | verified | schemas/manifests/v1/fixtures/invalid/ listing |
| A12 | Windows reserved device names (`con`, `prn`, `aux`, `nul`, `com1`–`com9`, `lpt1`–`lpt9`) are legal strings under the slug grammar | verified | https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file |
| A13 | The escape-and-truncate renderer is core-private (`one_line`) and there is no equivalent helper in `discovery` | verified | crates/core/src/error.rs:166-186 |
| A14 | A 128-octet lowercase-ASCII slug is one legal path segment on every platform this product names (255-per-component class) | unverified | 01-research.md Platform notes (linux limits.h; Microsoft naming docs; owncloud/Wikipedia per-component surveys) |
| A15 | The std-only fold table reproduces the "NFKD, drop combining marks" outcome for every code point in the covered blocks whose canonical decomposition base is an ASCII Latin letter | unverified | not checked code-point-by-code-point against UnicodeData.txt; normalization semantics per https://www.unicode.org/reports/tr15/ |
| A16 | 128 is "one octet above the longest surveyed store display title" (024-B1's stated rationale) | unverified | the verified survey (A6) found a 64-octet slug, not a 127-octet title; 01-research.md Findings |

## Options considered

**Where the grammar-and-bound check is enforced.** (a) `crates/discovery` adds a
`bindforge-core` dependency and core exports the single grammar predicate and
bound; every emit site calls the same code. (b) Duplicate a private grammar
check inside `discovery` — rejected: it re-creates exactly the constant-drift
failure `024-B1`'s one-constant rule exists to prevent, and would put a second
grammar in the tree that A1 says does not exist today. (c) Move `DiscoveredGame`
into core behind a validating constructor — rejected: it moves a discovery type
into the depends-on-nothing root and teaches core about games; the check belongs
at the emit boundary, not in a relocated type. Chosen: (a); the one new edge
(`discovery → bindforge-core`) points at the root, keeping core's zero-dependency
status (A4).

**Honouring "NFKD, drop combining marks" under "std-only".** The two clauses
cannot both be literally true (A5). (a) Add `unicode-normalization` — rejected:
`024-C2` pins "std-only" and the dependency graph is deliberately minimal.
(b) Drop the diacritic clause — rejected: accented titles would fold their
accents to `-` ("Pokémon" → `pok-mon`), the readability loss the clause exists
to prevent. (c) Implement a bounded std-only fold table: every precomposed letter
in Latin-1 Supplement and Latin Extended-A whose Unicode canonical decomposition
base is a single ASCII Latin letter maps to its lower-case ASCII base; nothing
maps to more than one character (no `ß`→`ss`, no `æ`→`ae`), and every unmapped
character outside `[a-z0-9]` becomes `-`, which is also what NFKD-with-dropped-
marks yields for every non-Latin base. Chosen: (c); it is the smallest reading
under which both clauses hold, at the cost of exotic blocks (e.g. Latin Extended
Additional) folding accents to `-` (A15; pinned by Test strategy #1).

**Collision suffix vs the length bound.** `024-L2` re-checks suffixed slugs
against the bound but does not say what happens at the bound. (a) WordPress-style
base truncation before appending the suffix — rejected: truncation is the lossy,
against-the-display-name move ADR-024 rejected as the long-title answer, and
keeping it even at the 128 edge invites the same disambiguation debt the ADR
refused to push onto the resolver. (b) Reserve headroom (derive bases only to
124) — rejected: it silently shortens every near-cap slug to pay for a collision
the corpus has never shown (A6). (c) Try suffixes `-2`, `-3`, …; accept the first
candidate that is unclaimed and still within the bound; if none fits, refuse the
game through the same named-refusal path used for an un-derivable name. Chosen:
(c) — refusal is already the ADR's answer for a name that cannot be slugged,
and the surveyed corpus makes the case hypothetical (A6).

**Windows reserved names.** (a) Refuse the reserved-device set (A12) inside the
shared grammar now, fixing slugs, manifest ids, geometry names and profile ids at
once — rejected: it amends a published grammar across four contracts with no
mandate in any ADR in the read set, and this ADR changes the bound and the
identity shape, not the character class. (b) Defer with a named owner. Chosen:
(b) — see Deferred.

ADR-level alternatives (keep `unknown/<kebab>`; grow the grammar to
`publisher/name`; keep 63 and truncate) were decided in ADR-024 and are not
re-opened here.

## Chosen design

### Shared grammar and bound — `crates/core`

- `placement.rs`: add `pub const MAX_SLUG_OCTETS: usize = 128;` beside
  `is_slug_name` and make that `pub`; the constant's doc comment carries no
  rationale of its own, only the README bullet's text (Specs, fixtures, ledger).
- `load_geometry` checks the name against `MAX_SLUG_OCTETS` beside the
  `is_slug_name` check, before the join: `024-B1` enumerates a geometry name
  among the bound's riders, so the bound applies wherever the grammar does. No
  scope narrowing is permitted — neither the README bullet nor the constant's
  doc comment may state that the bound "stops where a format's spec stops" or
  otherwise drop a rider; each states the rider set exactly as `024-B1`
  enumerates it. The today-pinned case that a past-bound geometry name *loads*
  is re-pinned to the refusal (Test strategy #14).
- `lib.rs`: `pub use placement::{is_slug_name, MAX_SLUG_OCTETS};` — this is the
  twelfth exported constant, and `lib.rs` stays the only export list.
- `policy.rs`, `manifest.rs` and `document.rs`: delete their private
  `MAX_SLUG_OCTETS`; `action.rs`: delete its private `MAX_SEGMENT_OCTETS` (A2).
  All four sites reference the exported constant. Where a refusal message embeds
  the number (e.g. `action.rs` `SEGMENT_SHAPE_RULE` "1-63 octets"), the string is
  built with `format!` from the constant so a future amendment has one source.
- `error.rs`: `one_line` becomes `pub fn escape_and_truncate(raw: &str) -> String`
  and is re-exported from `lib.rs`, so `024-L1`'s echo discipline has one
  implementation (A13).

### Identity shape — `crates/discovery/src/game.rs`

- New `pub enum Namespace { Unknown }` with `#[serde(rename_all = "kebab-case")]`
  and a `Display` impl; `DiscoveredGame` gains `pub namespace: Namespace` between
  `slug` and `display_name`. The JSON row (compact serde of `DiscoveredGame`)
  gains `"namespace":"unknown"` — a deliberate divergence from `Platform`, which
  carries no serde attribute and pins as `"SteamLinux"`
  (`apps/discovery-cli/tests/cli_tests.rs:186-189`), so this row's closed-enum
  spelling is mixed by decision, and the value to build is `"unknown"`; the
  shape change rides the re-pinned CLI fixtures (see Test strategy #5, #8).

### Derivation — `crates/discovery/src/slug.rs`

`normalize_slug` is deleted (A4's public-surface note; guarded by Test
strategy #12). In its place, the canonical `024-C2` algorithm:

- `pub fn derive_slug(name: &str) -> Option<String>` — std-only, in order: fold
  precomposed Latin via a private `fold_latin_base(char) -> Option<char>` match
  table covering Latin-1 Supplement and Latin Extended-A single-base-letter
  decompositions, both cases, each entry emitting its lower-case ASCII base
  (`Å`→`a`, `É`→`e`; A15; no multi-character transliteration); then ASCII
  case-fold (`A-Z`→`a-z`); then map every remaining character outside
  `[a-z0-9]` to `-`; collapse `-` runs; trim `-` at both ends; prefix a leading
  digit with `g-`; return `None` on an empty result (output is pure ASCII, so
  octets equal chars).
- Length is **not** truncated anywhere: a derived (or suffixed) slug longer than
  `bindforge_core::MAX_SLUG_OCTETS` is a named refusal, not a cut string.
- Callers fall back per the ADR: derive from the display name; if `None`, derive
  the same way from the install directory name; if still `None`, refuse (this
  formalises the existing fallback, A10, and replaces today's silent bare
  `unknown/`).

### Emission, refusal and collisions — `crates/discovery`

- `Cargo.toml` gains `bindforge-core = { workspace = true }`.
- `steam_common::discovered_game` becomes fallible: it derives the slug (with the
  install-directory fallback), checks the final value against
  `bindforge_core::is_slug_name` and the bound before constructing the row
  (defence in depth — the derivation is grammar-correct by construction except
  for length), and returns `None` on refusal so the game is never added.
  Plugin-side refusals are count-only: they increment the existing `skipped`
  counter (whose message counts files, so a refused row counts there as a
  skipped row) and are never echoed per row — that counter's "unreadable or
  malformed" diagnostic fires only when a plugin emitted nothing at all
  (`steam_common.rs:101`), and `024-L1`'s named per-game refusal is the
  CLI-boundary line of the next section alone.
- `steam_common` gains a collision pass, run once per plugin over all rows that
  plugin's `discover()` collected (after all its libraries are scanned), before
  the `Vec` is returned: iterate distinct display names in byte order, keeping an
  assigned-slug map; the first name to claim a base slug keeps it, a later
  distinct name that derives a claimed slug takes the first free candidate
  `base-2`, `base-3`, … (skipping candidates already claimed, which keeps a
  suffixed slug from colliding with a different name's base slug); an assigned
  slug with no in-bound candidate is refused (removed and counted into
  `skipped`); rows sharing a display name share its assigned slug, so the same
  game in two libraries keeps the duplicate slug and today's
  `(slug, install_path)` ordering, which the pass re-applies before returning
  (`024-L2`).
- The `unknown` value is the only thing `Namespace` can hold in v1; the collision
  pass never touches it.

### CLI boundary and output — `apps/discovery-cli`

- Before rendering (table or JSON), every row from any `dyn DiscoveryPlugin` —
  including future third-party plugins — passes the same `is_slug_name` + bound
  check; an invalid row is refused: dropped, with one stderr line
  `error: refused game with invalid slug: <escape_and_truncate(value)>` and no
  path (`024-L1`); valid rows still print.
- `COLUMNS` becomes `["Slug", "Namespace", "Display Name", "Install Path", "Platform"]`
  (namespace beside the slug per Rollout item 1); `row_cells` adds
  `game.namespace.to_string()`; widths stay `chars().count()`-measured.

### Specs, fixtures, ledger

- `schemas/README.md` gains the bound's one stated home, and with it the single
  rationale text that `placement.rs`'s doc comment copies verbatim: a bullet in
  the exported budget family, `MAX_SLUG_OCTETS = 128`, the shared bound for every
  slug-grammar id; sibling of `MAX_NATIVE_NAME_OCTETS` (A8), path-component class
  per A14's assumption, the RFC 1035 §2.3.1 derivation explicitly superseded, and
  ADR-024's surveyed-title clause replaced rather than repeated (it is
  unverified, A16); the "all eleven constants" sentence becomes twelve (A9).
- `schemas/manifests/v1`, `schemas/profile/v2`, `schemas/policy/v1`,
  `schemas/action-ids/v1` `spec.md` sentences (A3) are amended in place: the
  number and the RFC 1035 derivation become a cross-reference to the README
  bullet, and the action-ids refusal example loses its literal length — "a
  64-octet segment" becomes "a segment past the shared bound" and names the
  renamed `segment-past-cap.toml`. The action-id segment keeps its stricter shape
  rules (letter first, letter or digit last) and borrows only the bound number.
  Acceptance, pinned by Test strategy #13: no bound sentence in the four files
  contains a literal `63` or `128`; `schemas/action-ids/v1/spec.md` contains no
  `RFC 1035` string anywhere; each bound sentence names `MAX_SLUG_OCTETS` and
  points at `schemas/README.md`. The README bullet's claim that the four
  cross-reference it must be true of the shipped corpus.
- Fixtures: `id-past-cap.toml` in `schemas/policy/v1/` and `schemas/profile/v2/`
  re-pin from 64 to 129 octets (literal and prose);
  `segment-past-63-octets.toml` in `schemas/action-ids/v1/` becomes
  `segment-past-cap.toml` with a 129-octet segment; and `schemas/manifests/v1/`
  gains the missing `id-past-cap.toml` invalid fixture (A11), since the manifest
  `id` is the exact id a discovered slug must be adoptable as.
- Ledger: the `013-L5` unit line in `docs/adrs/013-game-manifests.md` gains its
  `amended-by 024-B1` annotation per `021-L4`; `013`'s prose and `013-R1` stay
  untouched as history. The annotation is gate-pinned, not convention:
  `crates/core/tests/ledger_annotations.rs` (Test strategy #13) fails the
  workspace suite until the string is on the unit line. Commits are the
  runner's, per repo convention.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Third-party readers built against the 63-octet spec text refuse 64–128-octet ids | Low (no shipped document sits near 63, A7) | Medium | Relaxation-only amendment in place at current majors (024-L3); README states the bound once so a re-read is one hop; the new `id-past-cap.toml` fixtures pin the boundary for conformance suites |
| A real title derives past 128 octets and is refused | Very low (A6 sample max is 64) | Low | Named refusal through the shared skip path; ADR-025 sees the refusal counts before any bound change |
| The std-only fold table diverges from true NFKD for code points outside the two covered blocks | Medium (exotic-block names exist) | Low | Every unmapped character becomes `-`, which is also the NFKD-with-dropped-marks outcome for non-Latin bases (A15); the covered boundary is pinned by Test strategy #1 |
| Downstream consumers of the removed public `normalize_slug` | Low | Low | The only in-workspace consumer is `apps/discovery-cli`, re-pinned in the same run; Test strategy #12 pins the new public surface |
| Duplicate slugs across plugins, runs or hosts persist in a future corpus | Medium (by design, 024-L2) | Low | `(slug, install_path)` ordering unchanged; the resolver that must reconcile them is ADR-025's named job |
| A game named like a Windows device (`con`) yields an id that cannot be a directory there | Low | Medium | Grammar already blocks traversal and separators; reserved-name refusal deferred with an owner (Deferred); the case is pinned as known current behaviour by Test strategy #11 |
| Collision-suffix probe loop repeats bound checks many times | Low | Low | Candidates are probed against one claimed-slug set per plugin run; probe count is bounded by distinct-name count, no measurable cost today (01-research.md Risks: no baseline exists; none claimed) |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux (ext4) | Yes | 128-octet lowercase-ASCII slug is one legal ≤255 component (A14); CI gate runs the full suite here |
| Windows (NTFS) | Compile-verified, with a named gap | Grammar excludes `:`/`.` and trailing space/period; reserved device names remain reachable (A12) and are deferred (Deferred); its CI verification level is Test strategy #11's |
| macOS (APFS) | Yes, no CI leg | 255-per-component class (A14); no product code path is macOS-specific, and the runtime verification level is Test strategy #11's |
| Third-party schema readers | Yes, with a known behaviour change | Relaxation-only (024-L3): a document valid under the 63-octet text stays valid under 128, so an old-text reader refuses only 64–128-octet ids — of which none ship at merge (A7) — and the divergence is confined to future ids no known consumer produces |
| `tools/orchestration` (Node) | Not affected | The pipeline-internal `nnn-slug` run-identity grammar (`SLUG_RE`, `tools/orchestration/src/pipeline/types.ts:287`) is a distinct grammar that admits a leading digit and rides no shared product slug bound; nothing there reads the changed surface; `npm ci && npm run check` unchanged |
| Rust toolchain (pinned 1.85.0) | Yes | std-only derivation needs no new toolchain feature (A5); one new workspace dependency edge (`discovery → bindforge-core`) |

## Test strategy

1. Unit: `derive_slug` table — the survey cases are reproduced exactly (the 64-octet
   `warhammer-40-000-darktide-…` slug; `g-7-days-to-die`, `g-8bitboy`,
   `g-8-bit-armies`; `nier-automata` from `NieR:Automata™`); diacritic folding
   (`é`→`e`, `ü`→`u`, `ñ`→`n`, and upper-case `Å`→`a`, `É`→`e`); no transliteration (`ø`, `æ`, `ß` each become
   `-`); Latin Extended Additional (`ḗ`) folds to `-`; non-Latin-only input
   returns `None`.
2. Unit: bound and grammar — a 128-octet derived slug is accepted and a 129-octet
   derivation is a named refusal (never truncated); `is_slug_name` refuses empty,
   uppercase, leading digit, `/`, `.` and `..` shapes against the shared
   constant.
3. Unit: fallback and refusal — an empty display name falls back to the install
   directory name through the same algorithm; both empty means the game is not
   emitted and the skip counter increments.
4. Unit: collision rule — the `Ys IX: Monstrum Nox` / `Ys IX -Monstrum NOX-` pair
   yields the base slug for the first name in byte order and `-2` for the second;
   two rows with one display name keep the same slug; a distinct name whose base
   equals an already-claimed suffixed slug takes the next free candidate; a name
   with no in-bound candidate is refused and counted.
5. Unit: `Namespace` serde round-trips as `"unknown"`, its `Display` renders
   `unknown`, and a `DiscoveredGame` serialises with the new `namespace` field
   between `slug` and `display_name` (JSON shape pin).
6. Unit: core boundary re-pins — `crates/core/tests/{manifest.rs, policy.rs,
   chain_ops.rs, action.rs}` accept 128 and refuse 129, `action_ids.rs` re-pins
   its required-fixture literal to `segment-past-cap.toml`, and numeric refusal
   messages interpolate the exported constant.
7. Unit: `escape_and_truncate` renders control characters as `\uXXXX` escapes and
   truncates long values with `…`, so a refusal echo can never carry a path or a
   second line.
8. Integration: the CLI table renders five columns with `Namespace` beside `Slug`,
   and `--json` rows carry `"namespace":"unknown"`; the eight discovery/CLI test
   files carrying `unknown/` re-pin to the prefix-free spelling, and the test
   that asserts absence of `unknown/` on paths is inverted to assert slugs contain
   no `/`.
9. Integration: a plugin emitting a row whose slug violates the grammar or bound
   gets that row dropped by the CLI with one named stderr refusal containing the
   escaped echo, while valid rows still render. The check is an extractable
   named function in `main.rs` over a `Vec<DiscoveredGame>` (rows in, kept rows
   out, refusals collected) so `apps/discovery-cli` asserts it directly in an
   in-binary test module — the scratch-Steam-root subprocess tests cannot reach
   this filter, because `steam_common` refuses bad rows earlier; the filter's
   absence of tests is a Test-strategy failure, not a coverage footnote.
10. Integration: schema conformance corpora — the re-pinned `id-past-cap.toml`
    (policy/v1, profile/v2, new in manifests/v1) and `segment-past-cap.toml`
    (action-ids/v1) are refused at 129 octets, and every valid fixture in all
    four trees still parses under the 128 bound.
11. Platform: the full suite runs on the Linux CI runner only — Windows is
    compile-verified by the existing `cargo check --target x86_64-pc-windows-msvc`
    leg, which runs no test binary, and macOS has no CI leg — so no test asserts a
    platform-specific outcome; derivation, collision and refusal results are
    platform-independent. The `con`-style reserved-name derivation is pinned as
    known current behaviour there, pending the deferred refusal.
12. API probe: a new `crates/discovery/tests/zz_api_probe.rs` (core's pattern)
    references the crate's public surface — `derive_slug`, `Namespace`,
    `DiscoveredGame` fields — so the public-surface change is guarded.
13. Doc-ledger and spec-literal pins: `crates/core/tests/ledger_annotations.rs`
    asserts three claims against the corpus (repo root resolved from
    `CARGO_MANIFEST_DIR`): (a) the `013-L5` unit line in
    `docs/adrs/013-game-manifests.md` carries `amended-by 024-B1`; (b) no bound
    sentence in the four amended `spec.md` files contains a literal `63` or
    `128`, and `schemas/action-ids/v1/spec.md` carries no `RFC 1035` string;
    (c) the `schemas/README.md` bound bullet states `MAX_SLUG_OCTETS = 128` and
    its rider set names a keyboard geometry name — the one stated home and its
    riders are pinned against drift.
14. Unit: geometry-name bound — `load_geometry` refuses a geometry name past
    `MAX_SLUG_OCTETS` beside the grammar check, and
    `crates/core/tests/placement.rs` re-pins the case that today asserts a
    past-bound geometry name *loads*.

## Rollout

Merged to `main` as one branch; the runner owns the commits, in dependency order.
Commit granularity: items 1+2 are one commit (`024-L3`) and items 4+5 another (a
row's field set and the CLI's rendering of it are one change); the rest are
branch-level, with item 7's gates run at branch scope. No data migration: the
shipped corpus holds no ids to rewrite (A7), and `namespace` defaults to
`unknown` for every v1 row.

1. Core: export `MAX_SLUG_OCTETS = 128` and `is_slug_name` from `placement.rs`
   via `lib.rs`; delete the four private copies (A2); export
   `escape_and_truncate`; `load_geometry` gains the bound check (Test
   strategy #14); re-pin the four boundary test files, the `placement.rs`
   geometry case, and the `action_ids.rs` fixture literal (Test strategy #6).
2. Specs and fixtures: README gets the bound's one stated home and the
   twelve-constants sentence; the four `spec.md` files cross-reference it, the
   action-ids refusal example included; fixtures re-pin and the new
   `schemas/manifests/v1/fixtures/invalid/id-past-cap.toml` lands.
3. Ledger: `013-L5` gains `amended-by 024-B1` per `021-L4`, gate-pinned by
   `crates/core/tests/ledger_annotations.rs` (Test strategy #13); `013`'s prose
   and `013-R1` untouched.
4. Discovery: the `bindforge-core` edge; `slug.rs` rewritten to the canonical
   derivation; `Namespace` field; construction-path refusals and the per-plugin
   collision pass; the eight re-pinned test files; the new API probe.
5. CLI: the `Namespace` column, the JSON shape, and the emit-boundary validation
   with escaped refusal echoes.
6. Build note (`03-build-notes.md`): record that an `unknown`-namespaced slug
   stays provisional until the reserved ADR-025 resolver adopts it, and that the
   run ships no matching code (`024-R1`).
7. Gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
   -- -D warnings`, `cargo test --workspace --locked`, and `npm ci && npm run
   check` in `tools/orchestration`, per `.github/workflows/check.yml`.

## Deferred / out of scope

- A resolver that maps an `unknown`-namespaced slug onto a real manifest id, and
  rename stability across store renames — ADR-025 (reserved, `024-R1`).
- `publisher` namespace: its derivation, grammar, and any namespace value beyond
  `unknown` — ADR-025.
- Reconciling duplicate `unknown` slugs across plugins, runs or hosts — ADR-025
  (it is resolver-input, not discovery-input).
- Refusing Windows reserved device names at the shared grammar (would fix slugs,
  manifest ids, profile ids and geometry names together) — no follow-up yet;
  candidate grammar-amendment ADR.
- Transliteration or fallback beyond single-base-letter Latin folding
  (`ß`→`ss`, CJK romanisation, Punycode-style encodings) — none; ADR-024 decides
  refusal for un-derivable names.
- A versioned machine-readable contract for the discovery CLI's JSON output —
  no follow-up yet; today the re-pinned CLI fixtures are the pin.
- Codec taxonomy and per-game codec selection — ADR-017.
- Persistence of discovery results — ADR-022, unchanged.

## Revision log

- Initial: check placement (core dependency + exported predicate); std-only fold
  table for the NFKD clause; suffix candidates stay in bound (refusal otherwise);
  reserved names deferred; manifest `id-past-cap.toml` added; `013-L5` note only.
- Derivation (with Options, Test strategy #1): the Latin table emits lower-case
  ASCII bases and runs before the ASCII case-fold, so accented caps survive.
- Shared grammar and bound / Specs, fixtures, ledger (with Test strategy #6,
  Rollout 1): one rationale text, at the README bullet; the action-ids refusal
  example names the renamed fixture, and `action_ids.rs` is its re-pin site.
- Emission, refusal and collisions: plugin-side refusals are count-only, so
  024-L1's named per-game refusal is the CLI-boundary line alone.
- Identity shape: no `Platform` serde precedent exists; the kebab-case
  `"unknown"` is a stated divergence from the pinned `"SteamLinux"`.
- Compatibility matrix: third-party schema readers and `tools/orchestration`
  commit to definite statuses; Windows and macOS claim their real verification.
- Test strategy #11 / Rollout: the gate is the Linux suite plus the existing
  Windows compile-check; items 1+2 and 4+5 are commit-atomic, the rest
  branch-level.
- Escalation repair (after code r1c4, correctness blockers): the ledger
  annotation and the spec-literal discipline gain gate teeth —
  `crates/core/tests/ledger_annotations.rs` (Test strategy #13); the
  geometry-name narrowing is reversed into the `024-B1` rider it contradicted
  (`load_geometry` bound check, Test #14); Test #9 mandates the in-binary CLI
  filter seam its assertion needs.

## As-built

Recorded 2026-11-18 after code review closed the run, against the shipped
branch (`e786de8f...HEAD`). These record what was built; they amend no design
decision above. Full working detail is in `03-build-notes.md` (Deviations).

1. **Slug refusal messages split by fault kind, bound messages lead with the
   number.** Affected: Chosen design → "Shared grammar and bound —
   `crates/core`" (the `format!`-from-the-constant clause) and Test strategy #6.
   As built, every slug-grammar check site (`action.rs` via
   `segment_within_bound` + `segment_bound_rule()`, `manifest.rs::check_slug`,
   `document.rs::check_profile_slug` and `check_pin`, `policy.rs::load_policy`,
   `placement.rs::load_geometry`) enforces shape and bound as two named faults,
   and the bound message interpolates `MAX_SLUG_OCTETS` and leads, with the
   offender echoed last. Reason: `escape_and_truncate` cuts an echoed value at
   80 characters, and a bound offender is by definition longer than the bound
   the message must name — an offender-leading message pushes the number past
   the cut. Consequences: `SEGMENT_SHAPE_RULE` keeps no length number at all;
   the action-id bound rule is a `format!` fn rather than a number-bearing
   const; and `check_pin`'s new bound arm emits its `InvalidValue` directly
   instead of wrapping in the local "is not a manifest pin" frame. The
   accepted/rejected sets are unchanged at every site.
2. **Two new `bindforge-core` dependency edges, not one.** Affected: Options
   considered → "Where the grammar-and-bound check is enforced" (the "one new
   edge" clause) and the Compatibility matrix's toolchain row.
   `apps/discovery-cli` also gains `bindforge-core = { workspace = true }`,
   because Test strategy #9's emit-boundary filter lives in the CLI binary and
   calls the predicate and the renderer directly. `crates/discovery` also
   gains a `serde_json` dev-dependency (its emission tests decode the JSON
   rendering). Core's zero-dependency status holds.
3. **The action-ids fixture rename landed as delete + add.** Affected: Chosen
   design → "Specs, fixtures, ledger". `segment-past-63-octets.toml` is
   deleted and `segment-past-cap.toml` (a 129-octet segment) added beside it;
   the superseded filename is itself pinned out-of-corpus by
   `slug_bound_corpus`, so an in-place edit is impossible.
4. **The `013-L5` annotation landed as a continuation line, by the operator's
   escalation repair `c4c849b8`.** Affected: Chosen design → "Specs, fixtures,
   ledger" and Rollout 3 (the annotation was expected on the unit line, by a
   runner-owned commit). The gate-pinned `ledger_annotations` test is green on
   it; no stage ever wrote `docs/adrs/**`.
5. **CLI emit-boundary filter shape.** Affected: Test strategy #9. Shipped as
   `fn refuse_invalid_slugs(&mut Vec<DiscoveredGame>) -> Vec<String>` —
   retaining in place and returning the exact stderr lines — where the design
   phrased the seam as rows in / kept rows out / refusals collected. Same
   named, in-binary-tested seam; refusal wording matches the design verbatim.
6. **The fold table omits `Ŀ`/`ŀ` (U+013F/U+0140).** Affected: Options
   considered → "Honouring NFKD…" (c) and Chosen design → "Derivation". These
   are the only letters in the two covered blocks whose canonical
   decomposition base is a single ASCII Latin letter (`L`/`l` plus a combining
   dot) that the shipped `fold_latin_base` maps to `-` rather than `l`. Every
   other code point in both blocks matches the design's coverage claim; the
   omission is unpinned, and titles containing these letters derive the same
   way the design prescribes for every character outside the table.
