# correctness review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`), full-panel pass c0, round 1. Inputs used: the
proposal, `docs/adrs/024-game-slug-identity.md`, the proposal's Revision log (single r1
entry, no rejected findings), the cited product sources
(`crates/core/src/{placement,action,document,manifest,policy,error,lib}.rs`,
`crates/core/tests/*`, `crates/discovery/{Cargo.toml,src/slug.rs,src/game.rs,src/steam_common.rs}`,
`apps/discovery-cli/src/main.rs` and its tests, `schemas/*/spec.md`, `schemas/README.md`,
fixture listings, `Cargo.toml`, `rust-toolchain.toml`), plus webfetch of the four URLs
named in the assumption register. No other run record was opened or used: one repo-wide
content grep incidentally listed paths under other designs' `reviews/` and `transcripts/`
directories; none was read, and nothing from them informs any finding below.

## Findings

- [major][new] Chosen design → Derivation — "fold precomposed Latin via a private
  `fold_latin_base(char) -> Option<char>` match table covering Latin-1 Supplement and Latin
  Extended-A single-base-letter decompositions, both cases" — the steps are stated "in
  order" with ASCII case-fold first and the Latin fold second, and the Options section says
  each covered letter "maps to that base", i.e. its Unicode canonical-decomposition base.
  For every uppercase precomposed letter that base is ASCII *uppercase* (`Å` → `A`, `É` →
  `E`, `İ` → `I`), and by the time the table runs the ASCII case-fold is already behind it,
  so the next clause — "map every remaining character outside `[a-z0-9]` to `-`" — dashes
  it out. `Åpen` then derives `pen`, and an all-caps accented title loses every accented
  letter: exactly the accented-fold-to-`-` loss the Options section names as the reason
  option (b) was rejected ("\"Pokémon\" → `pok-mon`"). Test strategy #1 cannot catch it:
  its diacritic cases are `é`, `ü`, `ñ`, all lowercase. Fix: one clause — the table maps
  each covered letter to its *lower-case* ASCII base (or the ASCII case-fold is re-applied
  after the diacritic fold) — and add an upper-case accented case (`É` → `e`, `Å` → `a`) to
  Test strategy #1.

- [major][new] Chosen design → Specs, fixtures, ledger — "the four `spec.md` sentences (A3)
  are amended in place: the number and the RFC 1035 derivation are replaced by a
  cross-reference" — A3's action-ids citation is `schemas/action-ids/v1/spec.md:54` only,
  and line 58 of the same sentence block is left out: "`move..steer`, `steer-left`,
  `move.steer.left`, and a 64-octet segment are each a refusal, and each has a fixture under
  `fixtures/invalid/` naming it". Under `MAX_SLUG_OCTETS = 128` a 64-octet segment is legal,
  so the amended spec ships a published refusal example that contradicts the bound two lines
  above it, and its promised fixture is the very file this run renames to
  `segment-past-cap.toml` with a 129-octet segment. Fix: extend the amendment list (and A3's
  citation) to that example clause — replace "a 64-octet segment" with "a segment past the
  shared bound" — so the sentence names the renamed fixture instead of a stale length.

- [minor][new] Test strategy #6 — "`crates/core/tests/{manifest.rs, policy.rs, chain_ops.rs,
  action.rs}` accept 128 and refuse 129" — those four are exactly the files that pin the
  63/64 boundary, but the file that pins the *name* of the action-ids invalid fixture is a
  fifth one: `crates/core/tests/action_ids.rs:124` lists `"segment-past-63-octets.toml"` in
  its required-fixture set. Neither #6 nor #10 names it, so the fixture rename this design
  orders has no named re-pin site. Fix: add `action_ids.rs` to #6's file list (and to Rollout
  item 1's re-pin list) with the new literal `segment-past-cap.toml`.

- [minor][new] Chosen design → Emission, refusal and collisions — "refusals increment the
  existing `skipped` counter, and that diagnostic's wording widens from \"unreadable or
  malformed\" to name un-derivable names as well" — that diagnostic is
  `crates/discovery/src/steam_common.rs:101`, whose guard is
  `!self.roots.is_empty() && games.is_empty() && skipped > 0`: it fires only when the plugin
  emitted *nothing*. A run that refuses one game and emits the rest prints nothing at all,
  so the widened wording never appears, and 024-L1's "a violation is a named refusal" is
  user-invisible for the Steam plugins — their rows are refused inside the plugin and never
  reach the CLI's named `error: refused game with invalid slug:` line. (Related: the counter
  reports "N file(s) skipped", while a refused row is a game read from a file that parsed
  fine.) Fix: emit one named line per construction-path refusal through `escape_and_truncate`
  (mirroring the CLI's line, no path), or state plainly that plugin-side refusals are
  count-only and that 024-L1's named refusal is the CLI-boundary line alone, so the law's
  trace is honest.

- [minor][new] Chosen design → Identity shape — "`pub enum Namespace { Unknown }` with
  `#[serde(rename_all = \"kebab-case\")]` and a `Display` impl, mirroring `Platform`'s
  pattern" — `Platform` (`crates/discovery/src/game.rs:5-18`) carries no serde attribute, and
  `apps/discovery-cli/tests/cli_tests.rs:186-189` pins `obj["platform"] == "SteamLinux"`
  with the comment that "the r1 major `#[serde(rename_all = \"kebab-case\")]` was reverted as
  unauthorized". So the pattern being mirrored does not exist, and the resulting JSON row
  mixes `"namespace":"unknown"` with `"platform":"SteamLinux"`. Fix: drop "mirroring
  `Platform`'s pattern" and state the deliberate divergence (and that the closed-enum JSON
  spelling in this row is now mixed), so a builder follows the pinned `"unknown"` rather than
  the cited precedent.

- [nit][new] Chosen design → Shared grammar and bound vs Specs, fixtures, ledger — "surveyed-title
  rationale per ADR-024, with A16's caveat" (the `placement.rs` doc comment) versus "with the
  substituted rationale (sibling of `MAX_NATIVE_NAME_OCTETS`, A8; path-component class per
  A14's assumption; supersession of the RFC 1035 §2.3.1 derivation stated explicitly)" (the
  `schemas/README.md` bullet) — the same constant gets two different published rationales,
  and the README is 024-L3's one stated home, so its wording is the contract while the code
  comment cites a clause the README drops. Fix: write the rationale once and reuse it in both
  places, stating explicitly whether 024-B1's "one octet above the longest surveyed store
  display title" clause is carried (with A16's caveat) or replaced.

- [nit][new] Rollout — "Same commit as the constant bump so code and spec text cannot drift
  (024-L3)" — the design requires commit-level atomicity for constant-plus-specs, but then
  sequences item 4 (discovery, including "the eight re-pinned test files", four of which live
  in `apps/discovery-cli/tests/`) ahead of item 5 (the CLI's `Namespace` column and JSON
  shape), so no intermediate commit satisfies `cargo test --workspace --locked` on its own.
  Fix: one clause — items 4 and 5 land as one commit, or state that the gates are branch-level
  and only the item 1+2 pair is commit-atomic.

## Checklist record

1. Assumption register — A1, A2, A3, A4, A5, A7, A8, A9, A10, A11, A12, A13 re-checked
   against the cited sources; all hold. A2's correction of the ADR's "three private copies"
   is right: `MAX_SLUG_OCTETS` at `policy.rs:37`, `manifest.rs:63`, `document.rs:220` plus
   `MAX_SEGMENT_OCTETS` at `action.rs:38`. A9's count is right: `lib.rs` exports exactly
   eleven constants. A11's fixture listing is right. A6's collision pair is real: Valve's
   `storesearch` returns both `Ys IX: Monstrum Nox` (1351630) and `Ys IX -Monstrum NOX-`
   (1732330), which derive to one slug. A14, A15, A16 are marked unverified and the body
   respects that. Not a finding: A5's cited page documents case mapping rather than the
   absence of a normalisation API; the claim is true and that page's method list confirms it
   (no NFD/NFKD entry point; the closest names are `to_casefold_unnormalized` /
   `eq_ignore_case_unnormalized`, absent from the pinned 1.85.0).
2. URL spot-check — `doc.rust-lang.org/std/primitive.str.html`,
   `store.steampowered.com/api/storesearch/`, `learn.microsoft.com/.../naming-a-file`,
   `unicode.org/reports/tr15/`: all live, none contradicts. A12's reserved-device set is
   confirmed by the Microsoft page and is grammar-legal, as claimed.
3. Internal contradictions — findings above (Derivation ordering; two rationales for one
   constant; the false `Platform` precedent). Otherwise components agree: grammar-correct-by-
   construction holds for `derive_slug`'s output shape; the collision pass, the
   `(slug, install_path)` sort at `steam_common.rs:116` and the CLI's append-then-render flow
   line up with 024-L2.
4. Missing edge cases — uppercase accented bases (major); empty/non-Latin name and the
   install-dir fallback are covered and match `steam_common.rs:248-251`; cross-plugin and
   cross-run duplicates are deferred consistently.
5. Interfaces — one grammar predicate, one bound constant, one renderer, all named once; the
   `namespace` JSON value is pinned in three places. The `Platform` precedent finding sits
   here.
6. Decides what ADR-024 asked — yes: Rollout items 1–6 all have a home (identity shape,
   derivation, emit-boundary check, collision rule, 63→128 with one exported constant and the
   four specs amended, `013-L5` gains `amended-by 024-B1` — format matches `021-L4` and
   `template.md:53`, and `013-L5` exists at `docs/adrs/013-game-manifests.md:114`). C1, C2,
   L1, L2, L3, B1, R1 all resolved; "cap at the shared bound" is decided as refusal, not
   truncation, consistent with the ADR's rejection of truncation.
7. Revision rounds — not applicable: r1 is the first pass and the Revision log records no
   rejected finding, so nothing here is re-raised.

## FYI (no severity)

- `document.rs:1506` (`is_tag_prefix`) also reads `MAX_SLUG_OCTETS`, so the shared raise
  silently widens the `policy/v1` tag-pattern prefix from 63 to 128 octets; the spec states
  that prefix's character class but no number (`schemas/policy/v1/spec.md:66-76`), and 024-B1
  does not enumerate tag prefixes. Worth the compatibility seat deciding whether the README's
  "every slug-grammar id" sentence should name the tag prefix as a bound consumer too.
- Refused games are counted in a counter whose message counts *files* ("`skipped`"); if the
  wording is widened, the noun needs it as well.
- The JSON row's field count goes 5 → 6 (`apps/discovery-cli/tests/cli_tests.rs:183` pins
  five) and no `#[serde(default)]` is stated for `namespace`; any consumer that parses the
  CLI's JSON must accept the new key.
- A14's per-component bound is the right frame for a slug, but the Microsoft page's 260-char
  `MAX_PATH` is a whole-path limit; the slug lands beside `<root>/steamapps/common/<installdir>`
  and as `data/manifests/<id>/`, which is a path-total question, not a component one.
