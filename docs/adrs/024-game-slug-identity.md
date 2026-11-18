---
doc_type: adr
status: proposed
date: 2026-11-18
depends-on: [013, 022]
ledger: yes
---

# ADR 024: Give discovered games a slug the product can adopt

This ADR overrides ADR-022's slug-derivation decision (its `publisher/name`
format and `unknown/<kebab>` provisional slug) and amends the shared slug
length bound that ADR-013's contracts carry. ADR-022 predates the unit ledger,
so it has no unit to `amends`; the override is stated here in prose and the
target of the length amendment — `013-L5` — gains its `amended-by` per
`021-L4` (Rollout, item 5).

## Context

ADR-022 shipped game discovery emitting a provisional slug
`unknown/<kebab(display_name)>` and explicitly deferred manifest matching and
slug migration. Meanwhile every identifier the product actually resolves — a
manifest `id`, a `codec` or preset id, a profile `id` and each `derives`
element, a policy `id`, a geometry name, an action-id segment — rides one
shared grammar, `placement::is_slug_name` = `[a-z][a-z0-9-]*`, capped at 63
octets. The separator `/` is excluded by design: the grammar is asked before
the filesystem precisely because each of these ids doubles as a file or
directory name. So `unknown/elden-ring` can never be any product id — the
provisional namespace cannot graduate into a manifest id; a matching step
would have to re-derive one, and re-derivation is exactly the migration
ADR-022's cost column conceded.

`normalize_slug` is also loose relative to the product grammar: it admits any
Unicode alphanumeric (a store name in non-Latin script survives into the
"slug"), accepts a leading digit, yields a bare `unknown/` for an empty or
symbol-only name, applies no length bound, and defines no collision rule —
`p.t.` and `PT` normalize to the same slug with nothing to separate them.

The 63-octet bound is sourced to the RFC 1035 §2.3.1 DNS-label bound and is
spelled out in four published contracts (`schemas/manifests/v1`,
`schemas/profile/v2`, `schemas/policy/v1`, `schemas/action-ids/v1`),
fixture-pinned at 63-accepts/64-refuses in two schema trees, and duplicated
as three private `MAX_SLUG_OCTETS` constants in `crates/core`. Game display
names are the input this id must survive, and retail titles with long
subtitles exceed 63 octets once kebabbed; ADR-013 already picked 128 for the
companion free-text bound (`MAX_NATIVE_NAME_OCTETS = 128`). No shipped corpus
carries a long id today — `data/manifests/` and `data/profiles/` hold only
READMEs — so the bound and the identity shape are free to fix now and
expensive to fix later.

## Decision

We will make a discovered game's slug an id the product can adopt
byte-for-byte: the same grammar and the same shared length bound as every
product slug id, with the namespace carried in its own field instead of
embedded in the string; and we will raise the shared bound from 63 to 128
octets, buying headroom for games with especially long titles.

### Contracts

- **024-C1** The `DiscoveredGame` identity shape: `slug` is a single-segment
  id that passes the shared slug grammar and the shared length bound and is
  adoptable byte-for-byte as a manifest `id` by the future matching step; the
  namespace is a separate field whose only v1 value is `unknown` and which
  never appears inside `slug`. → Rollout, item 1.
- **024-C2** Slug derivation from a display name, as an ordered algorithm:
  case-fold ASCII; strip Latin diacritics (NFKD, drop combining marks);
  replace every remaining character outside `[a-z0-9]` with `-`; collapse
  hyphen runs; trim hyphens at both ends; prefix `g-` a leading digit; on an
  empty result fall back to the same normalization of the install directory
  name, and refuse the game when that is empty too; cap at the shared bound.
  Deterministic, std-only, no transliteration table. → Rollout, item 1.

### Laws

- **024-L1** Grammar before emit: no `DiscoveredGame` leaves a plugin or the
  discovery CLI without its `slug` checked against the shared grammar and
  bound; a violation is a named refusal and the offending value is echoed
  only through the escape-and-truncate path, never as a path. `024-B1`
  → Rollout, item 2.
- **024-L2** Collisions inside one run: distinct display names that derive
  the same slug from one plugin disambiguate with deterministic `-2`, `-3`, …
  suffixes in display-name order, re-checked against the grammar and bound.
  The same game found in two libraries keeps its duplicate slug and today's
  `(slug, install_path)` ordering — a slug names a game, not an install.
  `024-C2` → Rollout, item 3.
- **024-L3** Bound-amendment discipline: the shared bound has one stated
  home, beside the `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` family in
  `schemas/README.md`; each format spec cross-references it instead of
  restating the number. A relaxation-only change (documents valid before
  stay valid, and no shipped corpus sits near the old bound) is amended in
  place at the current schema majors; a tightening would ride a schema
  major instead. → Rollout, item 4.

### Budgets

- **024-B1** `MAX_SLUG_OCTETS = 128` — the single shared bound every
  slug-grammar id rides: a manifest `id`, `codec` or preset id, a profile
  `id` and each `derives` element, a policy `id`, a geometry name, and an
  action-id segment. Amends `013-L5` and the matching pre-ledger grammar
  sentences of `action-ids/v1`, `policy/v1`, and `profile/v2`, replacing the
  RFC 1035 §2.3.1 derivation with: one octet above the longest surveyed
  store display title, equal to the shipped sibling bound
  `MAX_NATIVE_NAME_OCTETS = 128` (`013-B4`), and well under the 255-octet
  path-segment floor of ext4, APFS, and NTFS. One named constant exported
  from `crates/core`, referenced rather than retyped at the three current
  private definitions. `024-L3` → Rollout, item 4.

### Reservations

- **024-R1** Matching and namespace resolution: the resolver that maps a
  discovered `unknown`-namespaced slug onto a real manifest id (and later a
  `publisher` namespace, once derivable), including the rename-stability
  problem this ADR defers. Owner: ADR-025 (reserved). → ADR-025.

**Unit handles.** The handles in this ADR sit at the end of the Rollout list
items: one trailing code span of unit ids governs the line's claim, an id
anywhere else governs nothing.

Rollout:

1. The identity shape and derivation land in `crates/discovery`: `slug`
   becomes the grammar-valid single-segment id, a `namespace` field joins
   `DiscoveredGame`, and `normalize_slug` is replaced by the `024-C2`
   algorithm; `apps/discovery-cli` prints the namespace beside the slug
   (table and JSON); the discovery and CLI fixtures re-pin to the prefix-free
   spelling. `024-C1 024-C2`
2. The grammar-and-bound check lands at the plugin/CLI emit boundary, in
   front of any printing or downstream handoff. `024-L1`
3. The collision-suffix rule lands in the plugin aggregation step; the
   multi-library `(slug, install_path)` ordering keeps its current tests.
   `024-L2`
4. The shared bound moves 63 → 128: one exported constant in `crates/core`
   replaces the three private copies; `schemas/manifests/v1`,
   `schemas/profile/v2`, `schemas/policy/v1`, and `schemas/action-ids/v1`
   are amended in place, with the bound's one stated home recorded in
   `schemas/README.md`; the invalid fixtures re-pin from 64 to 129 octets
   (`id-past-cap.toml` in `policy/v1` and `profile/v2`,
   `segment-past-63-octets.toml` renamed `segment-past-cap.toml` in
   `action-ids/v1`) and the boundary tests in `crates/core/tests/` re-pin to
   accept 128 and refuse 129. `024-B1 024-L3`
5. `013-L5` gains its `amended-by` annotation naming `024-B1`, per `021-L4`.
   `024-B1`
6. The deferral is recorded: the run ships no matching code, and the build
   note states that an `unknown`-namespaced slug stays provisional until the
   reserved ADR-025 resolver adopts it. `024-R1`

## Alternatives considered

**Keep provisional labels.** Leave `unknown/<kebab>` purely discovery-local
with no adoption promise, and let a future match step invent manifest ids
separately. Rejected: it makes every future discovery↔manifest join a
translation rather than a lookup, splits the corpus into two forever-disjoint
id spaces, and leaves ADR-022's migration cost open indefinitely.

**Grow the core grammar to `publisher/name`.** The most literal reading of
ADR-022's stated format. Rejected: `/` is outside the grammar on purpose —
these ids are file and directory names, and the grammar-before-filesystem
discipline (ADR-013) depends on it. Namespacing would need a storage layout
and per-segment traversal checks in four formats for a namespace nothing
can populate yet.

**Keep 63 and truncate long names with a disambiguating suffix.** Rejected:
truncation makes the slug lossy against the display name it claims to encode
and pushes prefix disambiguation onto the resolver forever, to save one
shared constant. 128 matches the shipped native-name bound, stays a
comfortable single path segment, and the corpus cost of moving now is zero.

ADR-022's rejection of AppID-derived ids stands: a discovered slug stays
under BindForge's own derivation.

## Consequences

Becomes easy: the resolver adopts a discovered slug verbatim as the manifest
`id` when it matches; one grammar, one bound, and one constant replace four
spec statements and three duplicated ones; long-subtitled titles keep a
readable full slug instead of a truncated one.

Costs: the `unknown/<slug>` spelling disappears from discovery output and its
fixtures and tests re-pin; four published contracts carry an amended bound,
and a third-party reader built against the 63-octet text will refuse a
64–128-octet id until it re-reads the spec — acceptable because no shipped
document uses more than 63 today; pre-match slugs stay unstable under store
renames, the known cost ADR-022 listed, with `024-R1`'s resolver as the exit;
and a game whose display name and install directory are both non-Latin gets
refused at discovery rather than emitted with a meaningless slug.

Not decided here: the resolver and `publisher` derivation (ADR-025,
reserved); the codec taxonomy and per-game codec selection (ADR-017);
persistence of discovery results (unchanged from ADR-022).
