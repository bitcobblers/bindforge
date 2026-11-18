# Research: ADR-024 — give discovered games a slug the product can adopt

Lenses applied: **Prior art**, **Standards**, **Security landscape**. Internal
claims cite repo paths and line ranges; external claims cite URLs. Store-name
figures come from a survey run against Valve's own store-search endpoint on
2026-11-18 (URL in Findings).

## Summary

ADR-024 replaces ADR-022's provisional `unknown/<kebab(display_name)>` with a
single-segment slug that already satisfies the shared product slug grammar and
length bound, moves the namespace into its own `DiscoveredGame` field, and
raises the shared bound from 63 to 128 octets, amending `013-L5` (source:
docs/adrs/024-game-slug-identity.md). The repo today has exactly one grammar
predicate (`placement::is_slug_name`, crate-private), three private
`MAX_SLUG_OCTETS = 63` constants plus a fourth 63 constant under another name,
and four published spec sentences that restate the number (source:
crates/core/src/placement.rs:67-84). Discovery is the only product crate that
does not depend on `bindforge-core`, so "grammar before emit" (024-L1) forces a
new dependency edge or a new public predicate in core (source:
crates/discovery/Cargo.toml:6-13). A survey of 143 live Steam store titles
confirms both premises: one derives to 64 octets — refused by today's bound —
and two distinct store entries derive to the same slug, so 024-L2's suffix rule
has real input (source: https://store.steampowered.com/api/storesearch/).
The open decisions for the design are where the emit-boundary check lives, how
NFKD folding is honoured on a std-only crate, how the collision suffix and the
length cap interact, and whether Windows-reserved names are refused at the
grammar.

## Findings

### What the tree says today

- One grammar, crate-private: `[a-z][a-z0-9-]*`, byte-based, no length bound
  inside the predicate — every caller applies its own cap, and the doc comment
  says it is checked "before it is joined onto a directory, which is what keeps
  a `..` or a separator from ever asking the filesystem for a path" (source:
  crates/core/src/placement.rs:67-84).
- The bound is duplicated four times in code, not three: `MAX_SLUG_OCTETS = 63`
  at crates/core/src/policy.rs:37, crates/core/src/manifest.rs:63 and
  crates/core/src/document.rs:220, plus `MAX_SEGMENT_OCTETS = 63` at
  crates/core/src/action.rs:38 — the action-id segment bound, which 024-B1
  lists inside the shared bound but Rollout item 4 does not name among "the
  three private copies" (source: crates/core/src/action.rs:35-44;
  docs/adrs/024-game-slug-identity.md:97-107, 133-141).
- Action-id segments share the *number*, not the *grammar*: a segment opens
  with a lowercase letter, holds `a-z`, `0-9`, `-`, "closes with a letter or a
  digit", and is at most 63 octets — a stricter trailing rule than the slug
  grammar, and the spec's 63 is stated there too (source:
  schemas/action-ids/v1/spec.md:50-59).
- The 63 number is restated in exactly four published contracts, matching the
  ADR's claim: `schemas/manifests/v1/spec.md:54` (`id`) and `:56` (`codec`);
  `schemas/profile/v2/spec.md:38`, `:39` (`derives` elements), `:71`;
  `schemas/policy/v1/spec.md:40` and `:92`; `schemas/action-ids/v1/spec.md:54`
  (source: those files).
- Boundary pins that must re-pin 63/64 → 128/129 are in four Rust test files,
  not two: crates/core/tests/manifest.rs:780-812 (`id`, `codec`, preset),
  crates/core/tests/policy.rs:611-639, crates/core/tests/chain_ops.rs:693-726
  (`id` and `derives` element), crates/core/tests/action.rs:174-229 (segment)
  (source: those files).
- Corpus gap the ADR's fixture list inherits: `schemas/manifests/v1/`
  `fixtures/invalid/` has `id-uppercase.toml` but **no past-cap id fixture**, so
  the manifest `id` — the very id a discovered slug must be adoptable
  byte-for-byte as — has no conformance fixture for the bound; only the Rust
  test pins it (source: schemas/manifests/v1/fixtures/invalid/;
  crates/core/tests/manifest.rs:780-812; docs/adrs/024-game-slug-identity.md:135-141).
- The fixture bodies carry the old number in prose as well as in the 64-octet
  literals, so a re-pin is a text edit, not only a length edit:
  `schemas/policy/v1/fixtures/invalid/id-past-cap.toml:1-3` ("64 octets — one
  past the 63-octet slug cap") and
  `schemas/action-ids/v1/fixtures/invalid/segment-past-63-octets.toml:1-3`
  (source: those files).
- The exported-constant family has a stated count: `schemas/README.md:127-129`
  says "The reference implementation exports all eleven constants as the public
  names for these limits", and `crates/core/src/lib.rs:98-117` re-exports
  exactly eleven. Exporting `MAX_SLUG_OCTETS` makes twelve, and 024-L3 wants the
  bound's "one stated home" recorded in this same file (source:
  schemas/README.md:99-129; crates/core/src/lib.rs:98-117).
- Derivation today is the loose one the ADR describes: full-Unicode
  `to_lowercase`, keeps any `char::is_alphanumeric`, collapses and trims
  hyphens, no length cap, no collision rule, and always prefixes `unknown/` —
  so a symbol-only name yields bare `unknown/` (source:
  crates/discovery/src/slug.rs:1-35).
- `normalize_slug` is public API of the `discovery` crate and its spelling is
  pinned by five unit tests; `discovery` has no public-surface probe test (core
  has `crates/core/tests/zz_api_probe.rs`), so replacing it is an unguarded
  public-surface change (source: crates/discovery/tests/slug_tests.rs:1-32;
  crates/core/tests/zz_api_probe.rs:140-141, 398-410).
- Nothing checks the slug before it leaves the plugin: `discovered_game` sets
  `slug: normalize_slug(&manifest.display_name)` with no validation, rows are
  ordered by `(slug, install_path)`, and the CLI prints four columns and no
  bound check before printing (source: crates/discovery/src/steam_common.rs:215-234,
  :116; apps/discovery-cli/src/main.rs:31-32, 86-87, 120-127).
- The escape-and-truncate path 024-L1 names exists only in core's error
  rendering (control characters → `\uXXXX`, plus truncation, per the contract in
  `schemas/README.md:131-136`); the `discovery` crate has no equivalent helper,
  so the law's echo discipline has no home there yet (source:
  crates/core/src/error.rs:181; crates/core/src/document.rs:1846;
  schemas/README.md:131-136).
- The fallback source already exists: an empty or absent ACF `name` falls back
  to `installdir` before the slug is derived, so 024-C2's "fall back to the same
  normalization of the install directory name" formalises shipped behaviour, and
  the new refusal case replaces today's silent `unknown/` (source:
  crates/discovery/src/steam_common.rs:248-256).
- Re-pin surface for the disappearing `unknown/` spelling: eight test files
  across two crates carry it — `crates/discovery/tests/{slug_tests.rs:5-31,
  steam_linux_tests.rs:342-390, steam_windows_tests.rs:486-632,
  plugin_tests.rs:15-30, game_tests.rs:7-14}`,
  `apps/discovery-cli/tests/{cli_tests.rs:111-175, cli_dual_plugin_tests.rs:86-89,
  cli_output_polish_tests.rs:173-222}` — including one test that asserts the
  *absence* of `unknown/` on a path (source: those files).
- The zero-cost claim for the bound is verifiable: `data/` holds only
  `data/manifests/README.md` and `data/profiles/README.md` — no shipped
  document sits near 63 (source: data/manifests/README.md, data/profiles/README.md).
- The ADR's stated rationale for 128 is a *surveyed title* claim plus the
  sibling constant; the sibling is real and already 128 (`MAX_NATIVE_NAME_OCTETS
  = 128`, `013-B4`) (source: crates/core/src/manifest.rs:57;
  schemas/README.md:99-106).
- Survey evidence (Valve store-search API, English/US locale, 143 unique titles
  across 15 queries, run 2026-11-18; source:
  https://store.steampowered.com/api/storesearch/?term=deluxe%20edition&l=english&cc=US
  and the same endpoint with `term=` `collector's edition`, `legend of`, `goty`,
  `anniversary`, `7 days`, `8-bit`, `3d print`, `S.T.A.L.K.E.R.`, `ドラゴン`,
  `Assassin's Creed`, `Fate`, `NieR`, `Sonic`, `Ys`), applying 024-C2 as written:
  - one title derives past the current cap: `Warhammer 40,000: Darktide -
    Sisters Novitiate Class Deluxe Edition` → 64-octet slug
    `warhammer-40-000-darktide-sisters-novitiate-class-deluxe-edition`; nothing
    in the sample derived past 128 octets, which is consistent with the ADR's
    "128 is comfortable, 63 is not" but does **not** establish the ADR's
    stronger "one octet above the longest surveyed store display title".
  - a real same-run collision: `Ys IX: Monstrum Nox` and `Ys IX -Monstrum NOX-`
    are two distinct store entries whose 024-C2 slugs are byte-identical
    (`ys-ix-monstrum-nox`) — 024-L2's suffix rule has live input even inside one
    franchise.
  - the `g-` leading-digit rule fires often in the sample: `7 Days to Die` →
    `g-7-days-to-die`, `8BitBoy™` → `g-8bitboy`, `8-Bit Armies` →
    `g-8-bit-armies`.
  - non-ASCII in display names is routine and folds away (`™`, `’`, `—`):
    `NieR:Automata™` → `nier-automata`; no name in the sample derived to the
    empty string, so the "both name and install dir non-Latin → refuse" case is
    untested by the survey and remains a judgement call.
- Identity-model context: Steam's own key for a game is an immutable numeric App
  ID, and ADR-022 rejected adopting it, so a BindForge slug stays name-derived by
  choice (source: https://partner.steamgames.com/doc/store/application;
  docs/adrs/022-game-discovery.md:59).

### Standards

- The 63 bound's stated derivation is the RFC 1035 §2.3.1 DNS-label limit, and
  the repo's own specs say so ("the DNS-label bound RFC 1035 §2.3.1 publishes,
  restricted to lowercase and taken exactly") — moving to 128 deliberately
  leaves that anchor, which is why the ADR has to substitute a new rationale
  (source: schemas/action-ids/v1/spec.md:52-55;
  https://www.rfc-editor.org/rfc/rfc1035.html#section-2.3.1).
- Kubernetes carries the same tension and resolves it the other way: RFC 1123
  label names are "at most 63 characters", and "while RFC 1123 technically
  allows labels to start with digits, the current Kubernetes implementation
  requires both RFC 1035 and RFC 1123 labels to start with an alphabetic
  character" — i.e. comparable corpora either refuse a digit-leading name or
  transform it; 024-C2's `g-` prefix is the transform option, and 128 is well
  outside the 63 these systems hold to (source:
  https://kubernetes.io/docs/concepts/overview/working-with-objects/names/).
- IDNA (UTS #46) is the standard worked example of exactly this mapping —
  display strings onto DNS labels — and it solves it with a mapping/case-fold
  stage plus Punycode so that any script fits the 63-octet label; BindForge's
  024-C2 instead discards non-Latin and refuses on emptiness, which is the
  stricter, lossier choice and should be stated as such (source:
  https://www.unicode.org/reports/tr46/).
- NFKD is defined by UAX #15; `unicode-normalization` is the Rust implementation
  of that form (its own docs describe the UAX #15 iterators and the NFKD
  stream-safe rule). Rust `std` exposes case mapping (`str::to_lowercase`) but
  **no** normalisation of any kind, on any toolchain — so "NFKD, drop combining
  marks" and "std-only" cannot both be literally true (source:
  https://www.unicode.org/reports/tr15/;
  https://docs.rs/unicode-normalization/latest/unicode_normalization/;
  https://doc.rust-lang.org/std/primitive.str.html#method.to_lowercase;
  rust-toolchain.toml:6 pins channel `1.85.0`).
- Django's `slugify` is the reference open-source slug algorithm and it diverges
  from 024-C2 in two defensible ways: it *deletes* characters outside
  `[a-z0-9_-]` rather than mapping them to `-`, and it applies **no** length cap
  (source: https://github.com/django/django/blob/main/django/utils/text.py,
  `slugify`).
- WordPress' `wp_unique_post_slug()` implements 024-L2's rule and exposes its
  hard part: it loops `-2`, `-3`, … and *truncates the base slug* to
  `200 - (strlen(suffix) + 1)` before appending, because the suffix must fit the
  column bound — the same interaction 024-L2 leaves implicit at 128 octets
  (source: https://developer.wordpress.org/reference/functions/wp_unique_post_slug/).

### Security landscape

- Grammar-before-filesystem is the existing, correct defence for ids that become
  path components, and it is what makes a slug like `../x` impossible; the slug
  grammar also excludes `.` and `:`, the two characters Windows reserves (source:
  crates/core/src/placement.rs:67-84;
  https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file).
- The residual hole in that grammar is the Windows reserved-device set:
  `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9` are legal
  `[a-z][a-z0-9-]*` strings, and a game whose name folds to one yields an id that
  cannot be a directory on Windows — the class Git defends against with
  `core.protectNTFS` (default `true` on Windows) and that CVE-2019-1353 covers
  ("when running Git in WSL while accessing a working directory on a regular
  Windows drive, none of the NTFS protections were active") (source:
  https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file;
  https://git-scm.com/docs/git-config;
  https://nvd.nist.gov/vuln/detail/CVE-2019-1353).
- Windows is a supported contributor platform for this product and its file
  namespace is case-insensitive by guidance ("do not assume case sensitivity"),
  which the lowercase-only slug grammar already satisfies; the same guidance
  forbids names ending in a space or period — unreachable inside the slug
  grammar, so hyphen trimming is enough (source:
  https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file).
- Documents and ids arriving from elsewhere are untrusted at every entry point,
  and the contract already requires that a refusal echo be escaped and truncated
  and never carry a local path — 024-L1's "echo only through the
  escape-and-truncate path" is that rule applied to a new boundary, and
  untrusted-artifact naming is a recognised path-traversal class (source:
  schemas/README.md:83-93, 131-136;
  https://owasp.org/www-community/attacks/Path_Traversal).
- Normalisation-collapsed collisions are the reason a collision rule must exist
  at all: case-folding plus diacritic folding makes `p.t.`/`PT`-style pairs real,
  and the industry response is uniqueness enforcement at write time with a
  deterministic suffix (WordPress) or a secondary stable id plus a mutable slug
  (IGDB exposes both a numeric `id` and a name-derived `slug` on its records)
  (source: https://developer.wordpress.org/reference/functions/wp_unique_post_slug/;
  https://api-docs.igdb.com/).

## Prior art

- **Steam (Valve)** — primary key is an immutable numeric App ID, explicitly
  "a unique ID called an App ID"; store display names are attributes, freely
  renamed (source: https://partner.steamgames.com/doc/store/application).
  ADR-022 rejected using it, and ADR-024 re-affirms that (source:
  docs/adrs/022-game-discovery.md:59; docs/adrs/024-game-slug-identity.md:169-170).
- **IGDB** — game records carry a numeric `id` and a name-derived `slug`; the
  slug is the shareable handle, the number is the anchor, so slug churn is
  survivable by construction (source: https://api-docs.igdb.com/).
- **WordPress** — name-derived `post_name` slugs with an enforced uniqueness
  suffix loop (`-2`, `-3`, …) and truncation to fit the stored width, and, for
  renames, `wp_check_for_changed_slugs()` records the previous slug in
  `_wp_old_slug` so old links keep resolving: rename stability is solved with an
  old-slug ledger, not by making the slug stable (source:
  https://developer.wordpress.org/reference/functions/wp_unique_post_slug/;
  https://github.com/WordPress/WordPress/blob/master/wp-includes/post.php).
- **FriendlyId (Rails)** — the `:history` add-on keeps every previous slug in a
  `sluggable`-keyed table and overrides record lookup to resolve old slugs, the
  same ledger answer to the rename problem ADR-024 defers to ADR-025 (source:
  https://github.com/norman/friendly_id/blob/master/lib/friendly_id/history.rb).
- **Django** — `slugify`: NFKD → ASCII-encode with `ignore` → drop
  non-`[\w\s-]` → collapse `[-\s]+` to `-` → strip `-_`; no length bound, and
  non-Latin scripts vanish to the empty string rather than being transliterated
  (source: https://github.com/django/django/blob/main/django/utils/text.py).
- **Kubernetes** — RFC 1123 label / subdomain naming for objects that become DNS
  names: 63-octet labels, lowercase alphanumerics and `-`, must start and end
  alphanumerically, digit-start refused (source:
  https://kubernetes.io/docs/concepts/overview/working-with-objects/names/).
- **IDNA / UTS #46** — the standards answer for "human name → single path or
  label segment that any system can carry": map, case-fold, normalise, then
  Punycode anything non-ASCII so it still fits 63 (source:
  https://www.unicode.org/reports/tr46/).
- **Git** — the reserved-name/NTFS hazard is treated as a security property of
  checkout, default-on on Windows (`core.protectNTFS`), i.e. tooling refuses
  hostile names at the write boundary rather than sanitising them (source:
  https://git-scm.com/docs/git-config;
  https://nvd.nist.gov/vuln/detail/CVE-2019-1353).
- **Nothing in this repo has solved it yet** — `unknown/<kebab>` is the only
  shipped scheme and it was explicitly provisional with migration deferred
  (source: docs/adrs/022-game-discovery.md:23, 53-55, 81;
  crates/discovery/src/slug.rs:34).

## Platform notes

- Linux: per-component name limit `NAME_MAX = 255`, total `PATH_MAX = 4096`
  (source: https://github.com/torvalds/linux/blob/master/include/uapi/linux/limits.h).
  A 128-octet ASCII slug leaves half a component of headroom for the suffix.
- Windows: per-component "commonly 255 characters"; `MAX_PATH` 260 unless the
  long-path opt-in is enabled; reserved device names and the no-trailing-space-or-period
  rule; case-insensitive by guidance (source:
  https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file;
  https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation).
- APFS/HFS+ and NTFS/exFAT/FAT are all in the 255-per-component class (NTFS in
  UTF-16 characters, APFS up to 255 characters), so a lowercase-ASCII 128-octet
  slug is one legal segment on every platform this product names — the "255-octet
  path-segment floor" the ADR claims holds, though these figures are per-component,
  not per-path (source:
  https://doc.owncloud.com/server/11.1/admin_manual/troubleshooting/path_filename_length.html;
  https://en.wikipedia.org/wiki/Filename).
- Toolchain: pinned Rust channel `1.85.0`, crates on edition 2024;
  `crates/core` is "the depends-on-nothing root", `bindforge-transforms` depends
  on core and nothing else, and `discovery` depends on `serde` (plus `winreg`
  under `cfg(windows)`) — so a core→discovery grammar API is a new edge in a
  deliberately minimal graph (source: rust-toolchain.toml:6;
  crates/core/Cargo.toml:9; crates/transforms/Cargo.toml:10-13;
  crates/discovery/Cargo.toml:6-13).
- Discovery inputs are already UTF-8-gated: ACF/VDF files over 1 MiB or not
  valid UTF-8 are skipped, so derivation never sees arbitrary bytes; `installdir`
  is separately validated as a single safe path component, and on Windows a
  `\`-bearing `installdir` is refused (source:
  crates/discovery/src/steam_common.rs:276-308, :258-274).
- The CLI's table width is measured in `chars().count()`, so a fifth `Namespace`
  column is a rendering and test change, not an encoding one; JSON output is a
  compact serialisation of `DiscoveredGame`, so a new `namespace` field changes
  the JSON shape consumers see (source: apps/discovery-cli/src/main.rs:75-87,
  129-139; crates/discovery/src/game.rs:20-27).

## Risks & unknowns

- **std-only vs NFKD.** 024-C2 mandates NFKD + combining-mark removal and
  simultaneously says "std-only, no transliteration table"; `std` has no
  normalisation API (source: docs.rs `unicode-normalization`, doc.rust-lang.org
  `str::to_lowercase`). Every way out is a decision the ADR has not made: add a
  dependency, hand-roll a Latin-1/Latin-Extended-A decomposition table (which is
  arguably a transliteration table), or drop the diacritic clause.
- **Suffix vs cap interaction is undefined.** 024-L2 says a suffixed slug is
  "re-checked against the grammar and bound"; 024-B1 caps at 128 and the suffix
  is `-2`, `-3`, … A 126/127/128-octet slug cannot take a suffix and stay in
  bound, so the rule is silent exactly where titles are longest — and the
  surveyed evidence shows those are real titles (source:
  docs/adrs/024-game-slug-identity.md:81-86, 97-107; WordPress truncates instead
  (source: https://developer.wordpress.org/reference/functions/wp_unique_post_slug/)).
- **Collision scope is per plugin, per run.** Distinct titles that fold together
  are disambiguated "in display-name order" inside one plugin; two plugins, or
  one plugin across two runs/hosts, can still emit the same slug for different
  games, and today's `(slug, install_path)` ordering deliberately keeps
  duplicates. What the future resolver does with a corpus of colliding
  `unknown` slugs is unowned until ADR-025 (source:
  docs/adrs/024-game-slug-identity.md:81-86, 111-114;
  crates/discovery/src/steam_common.rs:116).
- **Windows reserved names** are reachable through derivation and remain
  reachable after this ADR (source:
  https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file;
  crates/core/src/placement.rs:74-84). No ADR in the read set has taken this on.
- **Cross-implementation reach of a relaxation.** Four published specs carry
  63 today; readers built against that text refuse 64–128 octet ids, and the
  repo's stated versioning rule is that minor bumps are additive only — 024-L3's
  claim that a relaxation rides the current major is a contract claim about
  "documents valid before stay valid", and it is the third-party reader, not the
  document set, that takes the break (source: schemas/README.md:37-57;
  docs/adrs/024-game-slug-identity.md:87-93, 179-183).
- **Ledger bookkeeping.** `013-L5` must gain `amended-by 024-B1` (021-L4) — and
  the same 63 sentence also appears in `013-R1`'s prose and in
  `docs/adrs/013-game-manifests.md:151`, which history says should not be
  rewritten (source: docs/adrs/021-decision-unit-ledger.md:89-92;
  docs/adrs/013-game-manifests.md:148-152).
- **Public-surface risk in `discovery`.** Replacing a `pub fn` and adding a field
  to a serialised struct has no API-probe test in that crate, unlike core
  (source: crates/discovery/tests/slug_tests.rs:1;
  crates/core/tests/zz_api_probe.rs).
- **No published performance baseline** exists for discovery-style scanning or
  for per-id grammar checks, and none is derivable from the read set; the work is
  O(name length) per game plus a collision map, so the risk is the collision
  loop's repeated bound checks rather than anything measurable today — treat any
  budget as the design's own invention.

## Open questions

1. How is "NFKD, drop combining marks" honoured under `std`-only: dependency,
   bounded hand-rolled Latin decomposition table, or drop the clause and let
   accented names lose their accents to `-` (the current helper already does
   that only by accident of `char::is_alphanumeric`)?
2. Where does the shared check live for a consumer outside core: export
   `MAX_SLUG_OCTETS` and a public `is_slug_name` (or a combined
   `check_slug(&str) -> Result`) from `bindforge-core` and add that dependency
   edge to `crates/discovery`, or put the emit-boundary check behind a core-side
   `DiscoveredGame` constructor?
3. Does `MAX_SEGMENT_OCTETS` in `crates/core/src/action.rs:38` fold into the
   exported constant, and does `schemas/action-ids/v1/spec.md`'s segment sentence
   get re-pointed at the shared bound even though segment grammar ends with a
   letter or digit while the slug grammar does not?
4. What is the exact stated home and wording in `schemas/README.md` — including
   the "eleven constants" count and whether the slug bound belongs in the
   adversarial-budget section at all, since its rationale is path-component
   legality, not hostile-input cost?
5. When a slug is already at the bound, does a collision suffix truncate the
   base, refuse the game, or is the bound reserved so slugs derive to ≤125? And
   is the suffix loop's "display-name order" defined for byte-equal display names?
6. Add the missing `id-past-cap`-shaped fixture to `schemas/manifests/v1/`
   `fixtures/invalid/` in this run (the format whose id adopts a slug), or keep
   the corpus as shipped and carry the bound in Rust tests only?
7. Does the `namespace` field's JSON/table spelling get pinned like
   `Platform`'s (`unknown`, kebab, serde derive), and is the JSON shape change
   recorded anywhere a consumer can see it, given the CLI's JSON is today's only
   machine-readable output?
8. Is a non-Latin-only game (name and install dir both folding to empty)
   refused, or emitted with a hash/`g-`-style fallback? The ADR accepts refusal;
   the survey found no such case in an English-locale store, so the population is
   unknown — is a counted refusal the product answer, and who reports it to the
   player?
9. Are Windows reserved device names refused at the grammar (fixing it once for
   manifests, geometry names, profile ids and slugs) or deferred with an owner?
10. Does 024-L1's refusal travel as a `DiscoveryError` variant, and does the
    echo of the offending value need a new escape-and-truncate helper in
    `discovery`, or does it reuse core's rendering by becoming a core-visible
    type?
11. Is `013-L5`'s `amended-by` annotation written into
    `docs/adrs/013-game-manifests.md` (a completed ADR, edited only by the
    runner), and does the manifest `id` spec line's `013-L5` handle gain a
    `024-B1` handle alongside it?
