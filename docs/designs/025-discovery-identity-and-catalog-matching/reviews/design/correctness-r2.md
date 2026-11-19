# correctness review r2

verdict: approve

Scope: the editorial-r2 diff only (`reviews/design/editorial-r2.diff`), read
against `02-proposal.md` (post-apply state), `docs/adrs/025-discovery-identity-and-catalog-matching.md`,
and the proposal's Revision log. Single editorial confirmation pass; this seat
carried no open findings into it.

## Diff fidelity

The diff's 24 removed / 24 added lines match the editorial commit
(`3501eab7`, `02-proposal.md`: 24/24) one-for-one, so nothing the language pass
touched lies outside the reviewed hunks. All twelve hunks were checked
character-by-character for semantic drift; every one is a word-order,
verb-insertion, line-reflow, capitalization, spelling, or code-span fix:

- Context: "exempt from the shared slug grammar and **from** `024-B1`, **and**
  rides its own bound" — same exemption set and same bound as ADR-025 `025-C1`
  ("exempt from the shared slug grammar and the `024-B1` bound, capped instead
  by `025-B1`").
- Options (c): "both within `MAX_NATIVE_NAME_OCTETS` and **on** a `char`
  boundary" — matches Chosen design ("clamp … at a `char` boundary") and Test
  strategy #2's "longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`".
- Bound: "**is** explicitly *not* a slug-grammar bound" — grammar only; the
  bullet's content (cap = 128, not a grammar bound, never a path component, plus
  the `MAX_NATIVE_NAME_OCTETS` borrowing) is unchanged, so `024-B1`/`024-L3`
  non-drift claim still holds.
- Match store load and the two named refusals: line reflow only.
- `probe()`: "every returned `SocketAddr`**'s** `IpAddr::is_loopback()`" — same
  post-resolution address check; reads as `addr.ip().is_loopback()`, a real std
  API, and the loopback property is still established by address, not name.
- Step 6: the broken nested inline-code tuple is re-spelled as
  (`slug`, `game_id`, friendly name of the first sharing row in
  `(slug, install_path)` order) — the same three fields as the store's
  `{ slug, game_id, friendly_name }`, so `025-C3`/`025-L2` semantics are intact.
- JSON clause: one added comma; the key set and the `025-L5` clause are unchanged.
- Boundary tests: "Match store" → "match store" (capitalization of a section
  reference only).
- Test #4: "defence" → "defense"; the document's spelling register is otherwise
  uniformly US (`behavior`, `normaliz*`), and no `defence`/`behaviour`/
  `normalis*` remains — the fix removes the last inconsistency rather than
  introducing one.
- Test #10: "a body over 65 536 **octets**" (was "bytes") — now agrees with the
  response-parse bullet ("at most `65_536` body octets") and with the header-block
  case in the same test line. Terminology alignment only; the refusal name and
  the cap are unchanged.

No assumption-register row, interface name, bound, refusal string, law id, or
rollout handle is altered by any hunk, so nothing this seat previously approved
is now false.

## Verification run

- Named symbols the diff-adjacent text leans on all exist as spelled:
  `bindforge_core::escape_and_truncate` (crates/core/src/error.rs:190, re-exported
  at crates/core/src/lib.rs:110), `MAX_NATIVE_NAME_OCTETS`
  (crates/core/src/manifest.rs:58, re-exported lib.rs:114), `MAX_SLUG_OCTETS` and
  `is_slug_name` (crates/core/src/placement.rs:83,95), `steam_common::absolute_env`
  (steam_common.rs:424), `steam_linux::host_candidates` (steam_linux.rs:90),
  `refuse_invalid_slugs` (apps/discovery-cli/src/main.rs:115),
  `resolve_collisions` (steam_common.rs:278), `Namespace` (game.rs:31),
  `derive_slug` (slug.rs:76).
- Spelling-register greps above.
- Deferred items and the Revision log are untouched by the diff; no deferral was
  re-opened, and no finding here contradicts a Revision-log rebuttal.

## Checklist disposition

1. Assumption register — no register row in the diff; not applicable to this scope.
2. Research-URL spot-check — no source citation in the diff; not applicable.
3. Internal contradictions — checked each hunk against the Chosen design, Test
   strategy, and the ADR: none introduced.
4. Missing edge cases — no behavior described by the diff changed.
5. Interface names/shapes — `slug`/`game_id`/`friendly_name`, the two refusal
   strings, and `CatalogError::{Unreachable, TransportUnavailable}` are
   unchanged by the diff.
6. Does the design decide what the ADR asked — unaffected by a language pass.
7. Prior-round blocker/major closure — this seat entered r2 with no open finding;
   nothing regressed in the changed text.

## Findings

None.

## FYI (no severity)

- Step 6 (Scan flow) now spells the record tuple with prose inside parentheses
  — "(`slug`, `game_id`, friendly name of the first sharing row in
  `(slug, install_path)` order)". Correct, but a presentation-style seat may
  prefer `slug`, `game_id`, and the first sharing row's `friendly_name`.
- `probe()`'s "every returned `SocketAddr`'s `IpAddr::is_loopback()`" is a
  doubled possessive; the reading is unambiguous but a style seat may re-flow it.
